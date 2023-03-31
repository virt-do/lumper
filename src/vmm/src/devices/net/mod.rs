pub mod interface;

pub(crate) mod bindings;
pub(crate) mod tap;

use std::{
    borrow::{Borrow, BorrowMut},
    cmp,
    error::Error,
    fmt::{self, Debug, Display},
    os::fd::{AsRawFd, RawFd},
    sync::atomic::Ordering,
};

use virtio_device::{VirtioConfig, VirtioDeviceActions, VirtioDeviceType, VirtioMmioDevice};

use virtio_bindings::bindings::virtio_net::{
    self, VIRTIO_NET_F_CSUM, VIRTIO_NET_F_GUEST_CSUM, VIRTIO_NET_F_GUEST_TSO4,
    VIRTIO_NET_F_GUEST_TSO6, VIRTIO_NET_F_GUEST_UFO, VIRTIO_NET_F_HOST_TSO4,
    VIRTIO_NET_F_HOST_TSO6, VIRTIO_NET_F_HOST_UFO,
};
use virtio_queue::{Queue, QueueOwnedT, QueueT};
use vm_device::{MutVirtioMmioDevice, VirtioMmioOffset};
use vm_memory::{Bytes, GuestAddress, GuestAddressSpace};
use vmm_sys_util::eventfd::EventFd;

use interface::Interface;

// TODO: Make this configurable.
const VIRTIO_FEATURES: u64 = (1 << bindings::VIRTIO_F_VERSION_1)
    | (1 << VIRTIO_NET_F_CSUM)
    | (1 << VIRTIO_NET_F_GUEST_CSUM)
    | (1 << VIRTIO_NET_F_HOST_TSO4)
    | (1 << VIRTIO_NET_F_HOST_TSO6)
    | (1 << VIRTIO_NET_F_HOST_UFO)
    | (1 << VIRTIO_NET_F_GUEST_TSO4)
    | (1 << VIRTIO_NET_F_GUEST_TSO6)
    | (1 << VIRTIO_NET_F_GUEST_UFO);

const MAX_BUFFER_SIZE: usize = 65565;

#[derive(Debug)]

pub enum VirtioNetError {
    InvalidIfname,
    VirtioQueueError(virtio_queue::Error),
    IoCtlError(std::io::Error),
    IoError(std::io::Error),
    MemoryError(vm_memory::GuestMemoryError),
    QueueError(virtio_queue::Error),
}
impl Error for VirtioNetError {}
impl Display for VirtioNetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "virtio net error")
    }
}

pub type Result<T> = std::result::Result<T, VirtioNetError>;

pub struct VirtioNet<M: GuestAddressSpace + Clone + Send, I: Interface> {
    pub device_config: VirtioConfig<Queue>,
    pub guest_irq_fd: EventFd,
    pub address_space: M,
    pub interface: I,
}

impl<M: GuestAddressSpace + Clone + Send, I: Interface> VirtioNet<M, I> {
    pub fn new(memory: M, irq_fd: EventFd, if_name: &str) -> Result<Self> {
        Ok(Self {
            device_config: VirtioConfig::new(
                VIRTIO_FEATURES,
                vec![
                    Queue::new(256).map_err(VirtioNetError::QueueError)?,
                    Queue::new(256).map_err(VirtioNetError::QueueError)?,
                ],
                // Not used in the current implementation.
                Self::config_vec(virtio_net::virtio_net_config {
                    ..Default::default()
                }),
            ),
            address_space: memory,
            guest_irq_fd: irq_fd,
            interface: I::open_named(if_name)?,
        })
    }

    fn config_vec(config: virtio_net::virtio_net_config) -> Vec<u8> {
        let mut config_vec = Vec::new();
        config_vec.extend_from_slice(&config.mac);
        config_vec.extend_from_slice(&config.status.to_le_bytes());
        config_vec.extend_from_slice(&config.max_virtqueue_pairs.to_le_bytes());
        config_vec.extend_from_slice(&config.mtu.to_le_bytes());
        config_vec.extend_from_slice(&config.speed.to_le_bytes());
        config_vec.extend_from_slice(&config.duplex.to_le_bytes());
        config_vec
    }

    fn is_reading_register(&self, offset: &VirtioMmioOffset) -> bool {
        if let VirtioMmioOffset::DeviceSpecific(offset) = offset {
            !(*offset as usize) < self.device_config.config_space.len() * 8
        } else {
            true
        }
    }

    fn write_frame_to_guest(
        &mut self,
        original_buffer: &mut [u8; MAX_BUFFER_SIZE],
        size: usize,
    ) -> Result<bool> {
        let mem = self.address_space.memory();
        let mut chain = match &mut self.device_config.queues[0]
            .iter(&*mem)
            .map_err(VirtioNetError::QueueError)?
            .next()
        {
            Some(c) => c.to_owned(),
            _ => return Ok(false),
        };

        let mut count = 0;
        let buffer = &mut original_buffer[..size];

        while let Some(desc) = chain.next() {
            let left = buffer.len() - count;

            if left == 0 {
                break;
            }

            let len = cmp::min(left, desc.len() as usize);
            chain
                .memory()
                .write_slice(&buffer[count..count + len], desc.addr())
                .map_err(VirtioNetError::MemoryError)?;

            count += len;
        }

        if count != buffer.len() {
            // The frame was too large for the chain.
            println!("rx frame too large");
        }

        self.device_config.queues[0]
            .add_used(&*mem, chain.head_index(), count as u32)
            .map_err(VirtioNetError::QueueError)?;

        Ok(true)
    }

    pub fn process_tap(&mut self) -> Result<()> {
        {
            let buffer = &mut [0u8; MAX_BUFFER_SIZE];

            loop {
                let read_size = match self.interface.read(buffer) {
                    Ok(size) => size,
                    Err(_) => {
                        break;
                    }
                };

                let mem = self.address_space.memory().borrow_mut().clone();

                if !self.write_frame_to_guest(buffer, read_size)?
                    && !self.device_config.queues[0]
                        .enable_notification(&*mem.clone())
                        .map_err(VirtioNetError::QueueError)?
                {
                    break;
                }
            }
        }

        if self.device_config.queues[0]
            .needs_notification(&*self.address_space.memory())
            .map_err(VirtioNetError::QueueError)?
        {
            // TODO: Figure out why we need to do that
            self.device_config
                .interrupt_status
                .store(1, Ordering::SeqCst);

            // Error should be recoverable as is, so we just log it.
            self.guest_irq_fd.write(1).unwrap_or_else(|e| {
                println!("Failed to signal irq: {:?}", e);
            });
        }

        Ok(())
    }
}

impl<M: GuestAddressSpace + Clone + Send, I: Interface> AsRawFd for VirtioNet<M, I> {
    fn as_raw_fd(&self) -> RawFd {
        self.interface.as_raw_fd()
    }
}

impl<M: GuestAddressSpace + Clone + Send, I: Interface> VirtioDeviceType for VirtioNet<M, I> {
    fn device_type(&self) -> u32 {
        bindings::VIRTIO_NET_DEVICE_ID
    }
}

impl<M: GuestAddressSpace + Clone + Send, I: Interface> VirtioMmioDevice for VirtioNet<M, I> {
    // Please note that this method can be improved error handling wise.
    // We are limited in how we can handle errors here, as we are not allowed to return a Result.
    fn queue_notify(&mut self, val: u32) {
        if val == 0 {
            return;
        }

        let mem = self.address_space.memory().clone();
        let irq = &mut self.guest_irq_fd;
        let queue = &mut self.device_config.queues[1];

        loop {
            match queue.disable_notification(&*mem) {
                Ok(_) => {}
                Err(e) => {
                    println!("Failed to disable notification: {:?}", e);
                    break;
                }
            }

            // Consume entries from the available ring.
            // Never fails since we know the memory is valid.
            while let Some(chain) = queue.iter(&*mem).unwrap().next() {
                let mut data_buffer: Vec<u8> = Vec::new();
                chain.clone().for_each(|desc| {
                    let initial_buffer_len = data_buffer.len();

                    data_buffer.resize(data_buffer.len() + desc.len() as usize, 0);

                    // Safe as we just allocated the buffer and mem is valid.
                    // If it actually fails, it is probably unrecoverable anyway.
                    mem.read_slice(&mut data_buffer[initial_buffer_len..], desc.addr())
                        .unwrap();
                });

                if (data_buffer.len() as usize) < bindings::VIRTIO_HDR_LEN {
                    println!("invalid net packet");
                    return;
                }

                match self.interface.write(&data_buffer) {
                    Ok(_) => {
                        queue
                            .add_used(&*mem, chain.head_index(), 0x100)
                            // Try continuing even if we failed to add the used buffer.
                            .unwrap_or_else(|e| {
                                println!("Failed to add used buffer: {:?}", e);
                            });

                        if queue.needs_notification(&*mem).unwrap_or_default() {
                            irq.write(1).unwrap_or_else(|e| {
                                println!("Failed to signal irq: {:?}", e);
                            });
                        }
                    }
                    Err(e) => {
                        println!("Failed to write to tap: {:?}", e);
                    }
                }
            }

            if !queue.enable_notification(&*mem).unwrap_or_default() {
                break;
            }
        }
    }
}

impl<M: GuestAddressSpace + Clone + Send, I: Interface> Borrow<VirtioConfig<virtio_queue::Queue>>
    for VirtioNet<M, I>
{
    fn borrow(&self) -> &VirtioConfig<virtio_queue::Queue> {
        &self.device_config
    }
}

impl<M: GuestAddressSpace + Clone + Send, I: Interface> BorrowMut<VirtioConfig<virtio_queue::Queue>>
    for VirtioNet<M, I>
{
    fn borrow_mut(&mut self) -> &mut VirtioConfig<virtio_queue::Queue> {
        &mut self.device_config
    }
}

impl<M: GuestAddressSpace + Clone + Send, I: Interface> VirtioDeviceActions for VirtioNet<M, I> {
    type E = VirtioNetError;

    fn activate(&mut self) -> Result<()> {
        self.interface
            .activate(VIRTIO_FEATURES, bindings::VIRTIO_HDR_LEN)?;

        Ok(())
    }
    fn reset(&mut self) -> std::result::Result<(), Self::E> {
        println!("virtio net reset");
        Ok(())
    }
}

impl<M: GuestAddressSpace + Clone + Send, I: Interface> MutVirtioMmioDevice for VirtioNet<M, I> {
    fn virtio_mmio_read(&mut self, _base: GuestAddress, offset: VirtioMmioOffset, data: &mut [u8]) {
        if self.is_reading_register(&offset) {
            self.read(u64::from(offset), data);
        }
    }

    fn virtio_mmio_write(&mut self, _base: GuestAddress, offset: VirtioMmioOffset, data: &[u8]) {
        if self.is_reading_register(&offset) {
            self.write(u64::from(offset), data);
        }
    }
}
