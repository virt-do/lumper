use std::{
    io::{Read, Write},
    os::fd::AsRawFd,
};

use super::Result;

pub trait Interface: Read + Write + AsRawFd + Send + Sync {
    fn activate(&self, virtio_flags: u64, virtio_header_size: usize) -> Result<()>;
    fn open_named(if_name: &str) -> Result<Self>
    where
        Self: Sized;
}
