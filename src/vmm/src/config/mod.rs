use linux_loader::cmdline::Cmdline;
use std::convert::{TryFrom, TryInto};
use std::path::PathBuf;

mod builder;

const KERNEL_CMDLINE_CAPACITY: usize = 4096;
// Default command line, public for tests
#[doc(hidden)]
pub const KERNEL_CMDLINE_DEFAULT: &str = "console=ttyS0 i8042.nokbd reboot=k panic=1 pci=off";

// Max size for an interface name
pub const IFACE_NAME_MAX_LEN: usize = 16;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Kernel configuration file check error")]
    KernelConfig(String),

    #[error("Tap configuration error")]
    TapConfig(String),
}

#[derive(Debug, PartialEq)]
pub struct NetConfig {
    pub tap_name: String,
}

/// VMM configuration.
#[derive(Debug, Default)]
pub struct VMMConfig {
    /// Linux kernel path and its commandline
    pub kernel: KernelConfig,

    /// Initramfs path
    pub initramfs: Option<PathBuf>,

    /// Number of virtual CPUs assigned to the guest
    pub cpus: u8,

    /// Memory amount (in MBytes) assigned to the guest
    pub memory: u32,

    /// A level of verbosity, and can be used multiple times
    pub verbose: i32,

    /// Stdout console file path
    pub console: Option<String>,

    /// Define a TAP interface name used to give network to the guest
    pub tap: Option<NetConfig>,
}

/// Store the current state of the kernel & its command line
/// arguments
#[derive(Clone, Debug, PartialEq)]
pub struct KernelConfig {
    /// Path to the kernel binary
    pub kernel_path: PathBuf,

    /// Command line arguments for kernel binary run
    pub cmdline: Cmdline,
}

impl TryFrom<String> for KernelConfig {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let path = PathBuf::from(value);
        if !path.exists() {
            return Err(Error::KernelConfig("File does not exist".to_string()));
        }

        let kernel = KernelConfig {
            kernel_path: path.clone(),
            ..Default::default()
        };

        Ok(kernel)
    }
}

impl Default for KernelConfig {
    fn default() -> Self {
        KernelConfig {
            kernel_path: PathBuf::default(),
            // We define the highest capacity of CMD line so we don't have overflow problems
            cmdline: KernelConfig::default_cmdline(),
        }
    }
}

impl KernelConfig {
    pub fn new(path: String, cfg_cmdline: Option<String>) -> Result<Self, Error> {
        let mut cmdline = Cmdline::new(KERNEL_CMDLINE_CAPACITY);
        cmdline
            .insert_str(cfg_cmdline.unwrap_or(KERNEL_CMDLINE_DEFAULT.to_string()))
            .map_err(|_| Error::KernelConfig("Capacity error on kernel cmdline".to_string()))?;

        let mut kernel: KernelConfig = path.try_into()?;
        kernel.cmdline = cmdline;

        Ok(kernel)
    }

    pub fn default_cmdline() -> Cmdline {
        let mut cmd = Cmdline::new(KERNEL_CMDLINE_CAPACITY);

        // Safe `unwrap` as sufficient capacity
        cmd.insert_str(KERNEL_CMDLINE_DEFAULT).unwrap();
        cmd
    }
}

impl TryFrom<String> for NetConfig {
    type Error = Error;

    fn try_from(tap_str: String) -> Result<Self, Error> {
        if tap_str.len() >= IFACE_NAME_MAX_LEN {
            return Err(Error::TapConfig(format!(
                "TAP name {} is too long",
                tap_str
            )));
        }

        Ok(NetConfig { tap_name: tap_str })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Error as ConfigError;
    use crate::config::KernelConfig;
    use std::convert::TryInto;

    type Error = crate::Error;

    #[test]
    fn test_fail_string_long_netconfig() {
        let invalid_if_name = String::from("this_string_is_way_too_long_for_tap_name");
        let net_result: Result<NetConfig, ConfigError> = NetConfig::try_from(invalid_if_name);

        assert!(net_result.is_err())
    }

    #[test]
    fn test_success_try_from_kernelconfig() {
        let valid_path = String::from("./Cargo.toml");
        let kernel: Result<KernelConfig, crate::config::Error> = valid_path.try_into();
        assert!(kernel.is_ok())
    }

    #[test]
    fn test_sucess_new_kernelconfig() {
        let valid_path = String::from("./Cargo.toml");
        let kernel = KernelConfig::new(valid_path.clone(), None);
        assert!(kernel.is_ok());
        {
            let kernel = kernel.unwrap();
            assert_eq!(kernel.kernel_path.to_str().unwrap(), valid_path);
        }
    }

    #[test]
    fn test_fail_new_kernelconfig() {
        // This is an invalid file
        let valid_path = String::from("./Cargo.tomle");
        let kernel = KernelConfig::new(valid_path.clone(), None);
        assert!(kernel.is_err());
    }

    #[test]
    fn test_sucess_new_with_cmd_kernelconfig() {
        // As we know Cargo.toml exists, we ensure a OK result
        let valid_path = String::from("./Cargo.toml");
        let cmdline = String::from(KERNEL_CMDLINE_DEFAULT);
        let kernel = KernelConfig::new(valid_path.clone(), Some(cmdline.clone()));
        assert!(kernel.is_ok());

        {
            let kernel = kernel.unwrap();
            assert_eq!(kernel.cmdline.as_str(), cmdline);
            assert_eq!(kernel.kernel_path.to_str().unwrap(), valid_path);
        }
    }

    #[test]
    fn test_success_try_from_string_netconfig() {
        let origin = String::from("str");

        let target: Result<NetConfig, ConfigError> = origin.clone().try_into();
        assert_eq!(false, target.is_err());
        assert_eq!(NetConfig { tap_name: origin }, target.unwrap());
    }
}
