use std::convert::TryFrom;
use std::path::PathBuf;

mod builder;

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

}

