use crate::config;
use crate::config::{KernelConfig, NetConfig, VMMConfig};
use std::convert::TryInto;
use std::path::PathBuf;

impl VMMConfig {
    /// Create the builder to generate a vmm config
    pub fn builder(
        num_vcpus: u8,
        mem_size_mb: u32,
        kernel: KernelConfig,
    ) -> Result<VMMConfigBuilder, config::Error> {
        Ok(VMMConfigBuilder::new(num_vcpus, mem_size_mb, kernel)?)
    }
}

/// See VMNConfig for explanation about these options
#[derive(Debug, Default)]
pub struct VMMConfigBuilder {
    kernel: KernelConfig,
    initramfs: Option<PathBuf>,
    cpus: u8,
    memory: u32,
    verbose: i32,
    console: Option<String>,
    tap: Option<config::NetConfig>,
}

impl VMMConfigBuilder {
    /// This method should be called when config is done, it generates the needed config
    pub fn build(self) -> VMMConfig {
        VMMConfig {
            kernel: self.kernel,
            initramfs: self.initramfs,
            cpus: self.cpus,
            memory: self.memory,
            verbose: self.verbose,
            console: self.console,
            tap: self.tap,
        }
    }
}

impl VMMConfigBuilder {
    pub fn new(
        num_vcpus: u8,
        mem_size_mb: u32,
        kernel: KernelConfig,
    ) -> Result<Self, config::Error> {
        let builder = VMMConfigBuilder {
            cpus: num_vcpus,
            memory: mem_size_mb,
            kernel: kernel,
            ..Default::default()
        };

        Ok(builder)
    }

    pub fn verbose(mut self, lvl: i32) -> Self {
        self.verbose = lvl;
        self
    }

    pub fn console(mut self, console: Option<String>) -> Self {
        self.console = console;
        self
    }

    pub fn initramfs(mut self, initramfs_path: Option<String>) -> Self {
        self.initramfs = match initramfs_path {
            Some(initramfs) => Some(PathBuf::from(initramfs)),
            None => None,
        };
        self
    }

    pub fn tap(mut self, tap_name: Option<String>) -> Result<Self, config::Error> {
        self.tap = match tap_name {
            Some(tap) => Some(tap.try_into()?),
            None => None,
        };
        Ok(self)
    }
}
