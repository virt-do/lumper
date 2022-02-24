use std::convert::TryFrom;
use std::path::PathBuf;

mod builder;

#[derive(Debug, PartialEq)]
pub struct NetConfig {
    pub tap_name: String,
}

/// VMM configuration.
#[derive(Debug, Default)]
pub struct VMMConfig {
    /// Linux kernel path
    pub kernel: PathBuf,

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

impl TryFrom<Option<String>> for NetConfig {
    // TODO: Add management to check if the tap name is valid for instance
    type Error = crate::Error;

    fn try_from(tap_str: Option<String>) -> Result<Self, Self::Error> {
        let tap_name = match tap_str {
            Some(tap) => Ok(tap),
            None => Err(Self::Error::TapError),
        }?;

        Ok(NetConfig { tap_name: tap_name })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;

    type Error = crate::Error;

    #[test]
    fn test_success_try_from_string_netconfig() {
        let origin = Some(String::from("str"));

        let target: Result<NetConfig, Error> = origin.clone().try_into();
        assert_eq!(false, target.is_err());
        assert_eq!(
            NetConfig {
                tap_name: origin.unwrap(),
            },
            target.unwrap()
        );
    }

    #[test]
    fn test_fail_try_from_string_netconfig() {
        let target: Result<NetConfig, Error> = None.try_into();
        assert_eq!(true, target.is_err());
        assert!(matches!(target.unwrap_err(), Error::TapError))
    }
}
