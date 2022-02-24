use crate::config;
use crate::config::VMMConfig;
use std::convert::TryInto;
use std::path::PathBuf;

impl VMMConfig {
    /// Create the builder to generate a vmm config
    pub fn builder(num_vcpus: u8, mem_size_mb: u32, kernel_path: &str) -> VMMConfigBuilder {
        VMMConfigBuilder::new(num_vcpus, mem_size_mb, kernel_path)
    }
}

/// See VMNConfig for explanation about these options
#[derive(Debug, Default)]
pub struct VMMConfigBuilder {
    kernel: PathBuf,
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
    // TODO: Maybe add a management of errors (e.g. checking kernel_path exists here)
    pub fn new(num_vcpus: u8, mem_size_mb: u32, kernel_path: &str) -> Self {
        VMMConfigBuilder {
            cpus: num_vcpus,
            memory: mem_size_mb,
            kernel: PathBuf::from(kernel_path),
            ..Default::default()
        }
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
