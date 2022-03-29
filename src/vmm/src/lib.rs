// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

#![cfg(target_arch = "x86_64")]

extern crate libc;

extern crate linux_loader;
extern crate vm_memory;
extern crate vm_superio;

use std::fs::File;
use std::io;
use std::io::stdout;
use std::os::unix::io::AsRawFd;
use std::os::unix::prelude::RawFd;
use std::sync::{Arc, Mutex};
use std::thread;

use kvm_bindings::{kvm_userspace_memory_region, KVM_MAX_CPUID_ENTRIES};
use kvm_ioctls::{Kvm, VmFd};
use linux_loader::loader::{self, KernelLoaderResult};
use vm_memory::{Address, GuestAddress, GuestMemory, GuestMemoryMmap, GuestMemoryRegion};
use vmm_sys_util::terminal::Terminal;
mod cpu;
use cpu::{cpuid, mptable, Vcpu};
mod devices;
use devices::serial::LumperSerial;

mod epoll_context;
use epoll_context::{EpollContext, EPOLL_EVENTS_LEN};

mod kernel;

pub mod config;
use crate::config::VMMConfig;

#[derive(Debug, thiserror::Error)]
/// VMM errors.
pub enum Error {
    #[error("Failed to write boot parameters to guest memory: {0}")]
    BootConfigure(linux_loader::configurator::Error),

    #[error("Error configuring the kernel command line: {0}")]
    Cmdline(linux_loader::cmdline::Error),

    #[error("Failed to load kernel: {0}")]
    KernelLoad(loader::Error),

    #[error("Invalid E820 configuration")]
    E820Configuration,

    #[error("Highmem start address is past the guest memory end")]
    HimemStartPastMemEnd,

    #[error("IO Error: {0}")]
    IO(io::Error),

    #[error("Error issuing an ioctl to KVM")]
    KvmIoctl(kvm_ioctls::Error),

    #[error("vCPU errors")]
    Vcpu(cpu::Error),

    #[error("Memory error")]
    Memory(vm_memory::Error),

    #[error("Serial creation error")]
    SerialCreation(io::Error),

    #[error("IRQ registration error")]
    IrqRegister(io::Error),

    #[error("epoll creation error")]
    EpollError(io::Error),

    #[error("STDIN read error")]
    StdinRead(kvm_ioctls::Error),

    #[error("STDIN write error")]
    StdinWrite(vm_superio::serial::Error<io::Error>),

    #[error("Terminal configuration error")]
    TerminalConfigure(kvm_ioctls::Error),

    #[error("Console configuration error")]
    ConsoleError(io::Error),

    #[error("TAP interface could not be found or not specified")]
    TapError,

    #[error("Parse config failed")]
    ConfigError(config::Error),
}

/// Dedicated [`Result`](https://doc.rust-lang.org/std/result/) type.
pub type Result<T> = std::result::Result<T, Error>;
pub struct NetConfig {
    tap_name: String,
}

pub struct VMM {
    vm_fd: VmFd,
    kvm: Kvm,
    guest_memory: GuestMemoryMmap,
    vcpus: Vec<Vcpu>,

    serial: Arc<Mutex<LumperSerial>>,
    epoll: EpollContext,
}

impl VMM {
    /// Create a new VMM.
    pub fn new() -> Result<Self> {
        // Open /dev/kvm and get a file descriptor to it.
        let kvm = Kvm::new().map_err(Error::KvmIoctl)?;

        // Create a KVM VM object.
        // KVM returns a file descriptor to the VM object.
        let vm_fd = kvm.create_vm().map_err(Error::KvmIoctl)?;

        let epoll = EpollContext::new().map_err(Error::EpollError)?;
        epoll.add_stdin().map_err(Error::EpollError)?;

        let vmm = VMM {
            vm_fd,
            kvm,
            guest_memory: GuestMemoryMmap::default(),
            vcpus: vec![],
            serial: Arc::new(Mutex::new(
                LumperSerial::new(Box::new(stdout())).map_err(Error::SerialCreation)?,
            )),
            epoll,
        };

        Ok(vmm)
    }

    pub fn configure_memory(&mut self, mem_size_mb: u32) -> Result<()> {
        // Convert memory size from MBytes to bytes.
        let mem_size = ((mem_size_mb as u64) << 20) as usize;

        // Create one single memory region, from zero to mem_size.
        let mem_regions = vec![(GuestAddress(0), mem_size)];

        // Allocate the guest memory from the memory region.
        let guest_memory = GuestMemoryMmap::from_ranges(&mem_regions).map_err(Error::Memory)?;

        // For each memory region in guest_memory:
        // 1. Create a KVM memory region mapping the memory region guest physical address to the host virtual address.
        // 2. Register the KVM memory region with KVM. EPTs are created then.
        for (index, region) in guest_memory.iter().enumerate() {
            let kvm_memory_region = kvm_userspace_memory_region {
                slot: index as u32,
                guest_phys_addr: region.start_addr().raw_value(),
                memory_size: region.len() as u64,
                // It's safe to unwrap because the guest address is valid.
                userspace_addr: guest_memory.get_host_address(region.start_addr()).unwrap() as u64,
                flags: 0,
            };

            // Register the KVM memory region with KVM.
            unsafe { self.vm_fd.set_user_memory_region(kvm_memory_region) }
                .map_err(Error::KvmIoctl)?;
        }

        self.guest_memory = guest_memory;

        Ok(())
    }

    pub fn configure_io(&mut self) -> Result<()> {
        // First, create the irqchip.
        // On `x86_64`, this _must_ be created _before_ the vCPUs.
        // It sets up the virtual IOAPIC, virtual PIC, and sets up the future vCPUs for local APIC.
        // When in doubt, look in the kernel for `KVM_CREATE_IRQCHIP`.
        // https://elixir.bootlin.com/linux/latest/source/arch/x86/kvm/x86.c
        self.vm_fd.create_irq_chip().map_err(Error::KvmIoctl)?;

        self.vm_fd
            .register_irqfd(
                &self
                    .serial
                    .lock()
                    .unwrap()
                    .eventfd()
                    .map_err(Error::IrqRegister)?,
                4,
            )
            .map_err(Error::KvmIoctl)?;

        Ok(())
    }

    pub fn configure_console(&mut self, console_path: Option<String>) -> Result<()> {
        if let Some(console_path) = console_path {
            // We create the file if it does not exist, else we open
            let file = File::create(&console_path).map_err(Error::ConsoleError)?;

            let mut serial = self.serial.lock().unwrap();
            *serial = LumperSerial::new(Box::new(file)).map_err(Error::SerialCreation)?;
        }

        Ok(())
    }

    pub fn configure_vcpus(
        &mut self,
        num_vcpus: u8,
        kernel_load: KernelLoaderResult,
    ) -> Result<()> {
        mptable::setup_mptable(&self.guest_memory, num_vcpus)
            .map_err(|e| Error::Vcpu(cpu::Error::Mptable(e)))?;

        let base_cpuid = self
            .kvm
            .get_supported_cpuid(KVM_MAX_CPUID_ENTRIES)
            .map_err(Error::KvmIoctl)?;

        for index in 0..num_vcpus {
            let vcpu = Vcpu::new(&self.vm_fd, index.into(), Arc::clone(&self.serial))
                .map_err(Error::Vcpu)?;

            // Set CPUID.
            let mut vcpu_cpuid = base_cpuid.clone();
            cpuid::filter_cpuid(
                &self.kvm,
                index as usize,
                num_vcpus as usize,
                &mut vcpu_cpuid,
            );
            vcpu.configure_cpuid(&vcpu_cpuid).map_err(Error::Vcpu)?;

            // Configure MSRs (model specific registers).
            vcpu.configure_msrs().map_err(Error::Vcpu)?;

            // Configure regs, sregs and fpu.
            vcpu.configure_regs(kernel_load.kernel_load)
                .map_err(Error::Vcpu)?;
            vcpu.configure_sregs(&self.guest_memory)
                .map_err(Error::Vcpu)?;
            vcpu.configure_fpu().map_err(Error::Vcpu)?;

            // Configure LAPICs.
            vcpu.configure_lapic().map_err(Error::Vcpu)?;

            self.vcpus.push(vcpu);
        }

        Ok(())
    }

    // Run all virtual CPUs.
    pub fn run(&mut self) -> Result<()> {
        for mut vcpu in self.vcpus.drain(..) {
            println!("Starting vCPU {:?}", vcpu.index);
            let _ = thread::Builder::new().spawn(move || loop {
                vcpu.run();
            });
        }

        let stdin = io::stdin();
        let stdin_lock = stdin.lock();
        stdin_lock
            .set_raw_mode()
            .map_err(Error::TerminalConfigure)?;
        let mut events = vec![epoll::Event::new(epoll::Events::empty(), 0); EPOLL_EVENTS_LEN];
        let epoll_fd = self.epoll.as_raw_fd();

        // Let's start the STDIN polling thread.
        loop {
            let num_events =
                epoll::wait(epoll_fd, -1, &mut events[..]).map_err(Error::EpollError)?;

            for event in events.iter().take(num_events) {
                let event_data = event.data as RawFd;

                if let libc::STDIN_FILENO = event_data {
                    let mut out = [0u8; 64];

                    let count = stdin_lock.read_raw(&mut out).map_err(Error::StdinRead)?;

                    self.serial
                        .lock()
                        .unwrap()
                        .serial
                        .enqueue_raw_bytes(&out[..count])
                        .map_err(Error::StdinWrite)?;
                }
            }
        }
    }

    pub fn configure(&mut self, cfg: VMMConfig) -> Result<()> {
        self.configure_console(cfg.console)?;
        self.configure_memory(cfg.memory)?;
        let kernel_load = kernel::kernel_setup(&self.guest_memory, cfg.kernel)?;
        self.configure_io()?;
        self.configure_vcpus(cfg.cpus, kernel_load)?;

        Ok(())
    }
}
