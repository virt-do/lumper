use std::u32;

use clap::Parser;
use vmm::config::VMMConfig;
use vmm::VMM;

#[derive(Parser)]
#[clap(version = "0.1", author = "Polytech Montpellier - DevOps")]
struct VMMOpts {
    /// Linux kernel path
    #[clap(short, long)]
    kernel: String,

    /// Number of virtual CPUs assigned to the guest
    #[clap(short, long, default_value = "1")]
    cpus: u8,

    /// Memory amount (in MBytes) assigned to the guest
    #[clap(short, long, default_value = "512")]
    memory: u32,

    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,

    /// Stdout console file path
    #[clap(long)]
    console: Option<String>,

    /// Define a TAP interface name used to give network to the guest
    #[clap(short, long)]
    tap: Option<String>,
}

#[derive(Debug)]
pub enum Error {
    VmmNew(vmm::Error),

    VmmConfigure(vmm::Error),

    VmmRun(vmm::Error),
}

impl From<VMMOpts> for VMMConfig {
    fn from(opts: VMMOpts) -> Self {
        VMMConfig::builder(opts.cpus, opts.memory, &opts.kernel)
            .tap(opts.tap)
            .console(opts.console)
            .verbose(opts.verbose)
            .build()
    }
}

fn main() -> Result<(), Error> {
    let opts: VMMOpts = VMMOpts::parse();
    let cfg: VMMConfig = VMMConfig::from(opts);

    // Create a new VMM
    let mut vmm = VMM::new().map_err(Error::VmmNew)?;

    // Configure the VMM:
    // * Number of virtual CPUs
    // * Memory size (in MB)
    // * Path to a Linux kernel
    // * Optional path to console file
    vmm.configure(cfg).map_err(Error::VmmConfigure)?;

    // Run the VMM
    vmm.run().map_err(Error::VmmRun)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::VMMOpts;
    use std::path::PathBuf;
    use vmm::config::{NetConfig, VMMConfig};

    // Test whether the configuration is properly parsed from clap options
    // to VMMConfig format
    #[test]
    fn test_parse_config_success() {
        let tap = Some(String::from("tap0"));
        let console = Some(String::from("console.log"));
        let kernel = String::from("kernel_file");

        let opts: VMMOpts = VMMOpts {
            kernel: kernel.clone(),
            cpus: 1,
            memory: 256,
            verbose: 0,
            console: console.clone(),
            tap: tap.clone(),
        };
        let cfg = VMMConfig::from(opts);

        let net_config = Some(NetConfig::try_from(tap.clone()).unwrap());

        // We hard code values as we don't want to implement Copy & Clone to
        // VMMOpts struct just for this test
        assert_eq!(PathBuf::from(kernel), cfg.kernel);
        assert_eq!(1, cfg.cpus);
        assert_eq!(256, cfg.memory);
        assert_eq!(0, cfg.verbose);
        assert_eq!(console, cfg.console);
        assert_eq!(tap.unwrap(), net_config.unwrap().tap_name);
    }
}
