use std::{error::Error, fmt::Display};

pub mod interface;

#[derive(Debug)]
#[allow(dead_code)]
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
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "virtio net error")
    }
}

#[allow(dead_code)]
pub type Result<T> = std::result::Result<T, VirtioNetError>;
