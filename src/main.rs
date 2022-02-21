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
        VMMConfig::builder()
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
