// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause
use crate::config::IFACE_NAME_MAX_LEN;
use crate::devices::bindings::ifreq;
use crate::devices::{Error, Result};
use libc::{c_char, c_int, c_uint, c_ulong, IFF_NO_PI, IFF_TAP, IFF_VNET_HDR};
use std::fs::File;
use std::io::{Error as IoError, Read, Result as IoResult, Write};
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::os::unix::prelude::RawFd;
use std::panic::catch_unwind;
use vmm_sys_util::ioctl::{ioctl_with_mut_ref, ioctl_with_ref, ioctl_with_val};
use vmm_sys_util::{ioctl_expr, ioctl_ioc_nr, ioctl_iow_nr};

const TAP_FILE: *const c_char = b"/dev/net/tun\0".as_ptr() as *const c_char;

// See if_tun.h
// https://elixir.bootlin.com/linux/v4.17/source/include/uapi/linux/if_tun.h#L34
// 84 is the ascii code for "T", see if_tun.h too
const TUNTAP: ::std::os::raw::c_uint = 84;
ioctl_iow_nr!(TUNSETIFF, TUNTAP, 202, ::std::os::raw::c_int);
ioctl_iow_nr!(TUNSETOFFLOAD, TUNTAP, 208, ::std::os::raw::c_uint);
ioctl_iow_nr!(TUNSETVNETHDRSZ, TUNTAP, 216, ::std::os::raw::c_int);

/// Virtual Tunnel struct used create a TUN/TAP device
#[derive(Debug)]
pub struct Tap {
    file: File,
    pub(crate) if_name: [u8; IFACE_NAME_MAX_LEN],
}

/// Take if_name and return a null terminated C string with our interface
/// name inside
/// if_name cannot be bigger than max_len, as it's already checked in
/// config builder
fn terminated_if_name(if_name: &str) -> Result<[u8; IFACE_NAME_MAX_LEN]> {
    let bytes_name = if_name.as_bytes();

    if bytes_name.len() > IFACE_NAME_MAX_LEN {
        return Err(Error::InvalidTapLength(format!("{}", if_name)));
    }
    // Create an empty array (\0 -> NULL)
    let mut terminated_name = [b'\0'; IFACE_NAME_MAX_LEN];
    terminated_name[..bytes_name.len()].copy_from_slice(bytes_name);
    Ok(terminated_name)
}

impl Tap {
    /// We create a TAP device and we need tap device name to do that
    pub fn open_named(if_name: &str) -> Result<Self> {
        let fd = unsafe {
            // Open will either open an existing file or create one
            // O_CLOEXEC: Close the socket when an exec is done on file
            libc::open(TAP_FILE, libc::O_RDWR | libc::O_NONBLOCK | libc::O_CLOEXEC)
        };

        // Failed to open file
        if fd < 0 {
            return Err(Error::OpenTun(IoError::last_os_error()));
        }

        let tuntap = unsafe { File::from_raw_fd(fd) };

        let terminated_name = terminated_if_name(if_name);
        // This part is something I clearly don't understand, we put flags & stuff
        // to what?
        let mut req = ifreq::default();
        // We have only a single mut ref at once, so should be safe (i don't know)
        let ifrn_name = unsafe { req.ifr_ifrn.ifrn_name.as_mut() };
        ifrn_name.copy_from_slice(terminated_name?.as_ref());

        let ifru_flags = unsafe { req.ifr_ifru.ifru_flags.as_mut() };
        *ifru_flags = (IFF_TAP | IFF_NO_PI | IFF_VNET_HDR) as i16;

        let ret = unsafe { ioctl_with_mut_ref(&tuntap, TUNSETIFF(), &mut req) };
        if ret < 0 {
            return Err(Error::OpenTun(IoError::last_os_error()));
        }

        // Safe since only the name is accessed, and it's cloned out
        Ok(Tap {
            file: tuntap,
            if_name: unsafe { *req.ifr_ifrn.ifrn_name.as_ref() },
        })
    }

    /// Set offload flags for the tap interface
    // offload flags = flags to know what to do when tap is cleaned up?
    pub fn set_offload(&self, flags: c_uint) -> Result<()> {
        let ret = unsafe { ioctl_with_val(&self.file, TUNSETOFFLOAD(), c_ulong::from(flags)) };

        if ret < 0 {
            return Err(Error::IoctlError(IoError::last_os_error()));
        }

        Ok(())
    }

    /// Size update of vnet header
    pub fn set_vnet_hdr_size(&self, size: c_int) -> Result<()> {
        let ret = unsafe { ioctl_with_ref(&self.file, TUNSETVNETHDRSZ(), &size) };

        if ret < 0 {
            return Err(Error::IoctlError(IoError::last_os_error()));
        }

        Ok(())
    }
}

impl Read for Tap {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        self.file.read(buf)
    }
}

impl Write for Tap {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> IoResult<()> {
        Ok(())
    }
}

impl AsRawFd for Tap {
    fn as_raw_fd(&self) -> RawFd {
        self.file.as_raw_fd()
    }
}
