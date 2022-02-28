// SPDX-License-Identifier: Apache-2.0

use std::io::Error as IoError;

mod bindings;
pub(crate) mod serial;
pub mod tap;

/// Custom defined [`std::result::Result`]
pub type Result<T> = std::result::Result<T, Error>;

/// Error related to MMIO / devices
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Cannot create a new Mmio Range")]
    Bus(vm_device::bus::Error),
    #[error("Cannot get next MMioConfig, memory overflow")]
    Overflow,

    #[error("Failed to open /dev/net/tun0")]
    OpenTun(IoError),

    #[error("Failed to communicate with device")]
    IoctlError(IoError),

    #[error("TAP interface name {0} is too long")]
    InvalidTapLength(String),
}
