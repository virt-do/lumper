// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

#![cfg(target_arch = "x86_64")]

use std::fs::File;
use std::path::PathBuf;
use std::result;

use linux_loader::bootparam::boot_params;
use linux_loader::cmdline::Cmdline;
use linux_loader::configurator::{linux::LinuxBootConfigurator, BootConfigurator, BootParams};
use linux_loader::loader::{elf::Elf, load_cmdline, KernelLoader, KernelLoaderResult};
use vm_memory::{GuestAddress, GuestMemoryMmap};

use crate::{Error, Result};

// x86_64 boot constants. See https://www.kernel.org/doc/Documentation/x86/boot.txt for the full
// documentation.
// Header field: `boot_flag`. Must contain 0xaa55. This is the closest thing old Linux kernels
// have to a magic number.
const KERNEL_BOOT_FLAG_MAGIC: u16 = 0xaa55;
// Header field: `header`. Must contain the magic number `HdrS` (0x5372_6448).
const KERNEL_HDR_MAGIC: u32 = 0x5372_6448;
// Header field: `type_of_loader`. Unless using a pre-registered bootloader (which we aren't), this
// field must be set to 0xff.
const KERNEL_LOADER_OTHER: u8 = 0xff;
// Header field: `kernel_alignment`. Alignment unit required by a relocatable kernel.
const KERNEL_MIN_ALIGNMENT_BYTES: u32 = 0x0100_0000;

// RAM memory type.
// TODO: this should be bindgen'ed and exported by linux-loader.
// See https://github.com/rust-vmm/linux-loader/issues/51
const E820_RAM: u32 = 1;
const E820_RESERVED: u32 = 2;

/// Address of the zeropage, where Linux kernel boot parameters are written.
pub(crate) const ZEROPG_START: u64 = 0x7000;

const HIMEM_START: u64 = 0x0010_0000; // 1 MB

/// Address where the kernel command line is written.
const CMDLINE_START: u64 = 0x0002_0000;
// Default command line
pub const DEFAULT_CMDLINE: &str = "console=ttyS0 i8042.nokbd reboot=k panic=1 pci=off";

fn add_e820_entry(
    params: &mut boot_params,
    addr: u64,
    size: u64,
    mem_type: u32,
) -> result::Result<(), Error> {
    if params.e820_entries >= params.e820_table.len() as u8 {
        return Err(Error::E820Configuration);
    }

    params.e820_table[params.e820_entries as usize].addr = addr;
    params.e820_table[params.e820_entries as usize].size = size;
    params.e820_table[params.e820_entries as usize].type_ = mem_type;
    params.e820_entries += 1;

    Ok(())
}

/// Build boot parameters for ELF kernels following the Linux boot protocol.
///
/// # Arguments
///
/// * `guest_memory` - guest memory
/// * `himem_start` - address where high memory starts.
/// * `mmio_gap_start` - address where the MMIO gap starts.
/// * `mmio_gap_end` - address where the MMIO gap ends.
pub fn build_bootparams(
    allocator: &vm_allocator::AddressAllocator,
) -> std::result::Result<boot_params, Error> {
    let mut params = boot_params::default();

    params.hdr.boot_flag = KERNEL_BOOT_FLAG_MAGIC;
    params.hdr.header = KERNEL_HDR_MAGIC;
    params.hdr.kernel_alignment = KERNEL_MIN_ALIGNMENT_BYTES;
    params.hdr.type_of_loader = KERNEL_LOADER_OTHER;

    allocator
        .allocated_slots()
        .iter()
        .for_each(|slot| match slot.node_state() {
            vm_allocator::NodeState::Ram => {
                add_e820_entry(&mut params, slot.key().start(), slot.key().len(), E820_RAM)
                    .unwrap();
            }
            vm_allocator::NodeState::ReservedAllocated => {
                add_e820_entry(
                    &mut params,
                    slot.key().start(),
                    slot.key().len(),
                    E820_RESERVED,
                )
                .unwrap();
            }
            _ => {}
        });

    Ok(params)
}

/// Set guest kernel up.
///
/// # Arguments
///
/// * `kernel_cfg` - [`KernelConfig`](struct.KernelConfig.html) struct containing kernel
///                  configurations.
pub fn kernel_setup(
    guest_memory: &GuestMemoryMmap,
    kernel_path: PathBuf,
    cmdline: &Cmdline,
    allocator: &vm_allocator::AddressAllocator,
) -> Result<KernelLoaderResult> {
    let mut kernel_image = File::open(kernel_path).map_err(Error::IO)?;
    let zero_page_addr = GuestAddress(ZEROPG_START);

    // Load the kernel into guest memory.
    let kernel_load = Elf::load(
        guest_memory,
        None,
        &mut kernel_image,
        Some(GuestAddress(HIMEM_START)),
    )
    .map_err(Error::KernelLoad)?;

    // Generate boot parameters.
    let mut bootparams = build_bootparams(allocator)?;

    let cmdline_str = cmdline
        .as_cstring()
        .map_err(Error::Cmdline)?
        .into_string()
        .map_err(Error::IntoStringError)?;

    let cmdline_size = cmdline_str.len() as u32;

    // Add the kernel command line to the boot parameters.
    bootparams.hdr.cmd_line_ptr = CMDLINE_START as u32;
    bootparams.hdr.cmdline_size = cmdline_size + 1;

    // Shrink the command line to the actual size.

    let mut shrinked_cmdline =
        linux_loader::cmdline::Cmdline::new(cmdline_size as usize + 1).map_err(Error::Cmdline)?;
    shrinked_cmdline
        .insert_str(&cmdline_str)
        .map_err(Error::Cmdline)?;

    // Load the kernel command line into guest memory.
    load_cmdline(
        guest_memory,
        GuestAddress(CMDLINE_START),
        // Safe because the command line is valid.
        cmdline,
    )
    .map_err(Error::KernelLoad)?;

    // Write the boot parameters in the zeropage.
    LinuxBootConfigurator::write_bootparams::<GuestMemoryMmap>(
        &BootParams::new::<boot_params>(&bootparams, zero_page_addr),
        guest_memory,
    )
    .map_err(Error::BootConfigure)?;

    Ok(kernel_load)
}
