use crate::config;
use crate::config::{KernelConfig, VMMConfig};
use std::convert::TryInto;

impl VMMConfig {
    /// Create the builder to generate a vmm config
    pub fn builder(
        num_vcpus: u8,
        mem_size_mb: u32,
        kernel_path: String,
    ) -> Result<VMMConfigBuilder, crate::Error> {
        Ok(VMMConfigBuilder::new(num_vcpus, mem_size_mb, kernel_path)
            .map_err(crate::Error::ConfigError)?)
    }
}

/// See VMNConfig for explanation about these options
#[derive(Debug, Default)]
pub struct VMMConfigBuilder {
    kernel: KernelConfig,
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
        kernel_path: String,
    ) -> Result<Self, config::Error> {
        let builder = VMMConfigBuilder {
            cpus: num_vcpus,
            memory: mem_size_mb,
            kernel: kernel_path.try_into()?,
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

    pub fn tap(mut self, tap_name: Option<String>) -> Self {
        self.tap = match tap_name.try_into() {
            Ok(cfg) => Some(cfg),
            _ => None,
        };
        self
    }
}
