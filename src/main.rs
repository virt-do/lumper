use std::u32;

use clap::Parser;
use vmm::VMM;

#[derive(Parser)]
#[clap(version = "0.1", author = "Polytech Montpellier - DevOps")]
struct VMMOpts {
    /// Linux kernel path
    #[clap(short, long)]
    kernel: String,

    /// Initramfs path
    #[clap(short, long)]
    initramfs: Option<String>,

    /// Number of virtual CPUs assigned to the guest
    #[clap(short, long, default_value = "1")]
    cpus: u8,

    /// Memory amount (in MBytes) assigned to the guest
    #[clap(short, long, default_value = "512")]
    memory: u32,

    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, action=clap::ArgAction::Count )]
    verbose: u8,

    /// Stdout console file path
    #[clap(long)]
    console: Option<String>,

    /// Interface name
    #[clap(long)]
    net: Option<String>,

    /// no-console
    #[clap(long)]
    no_console: bool,
}

#[derive(Debug)]
pub enum Error {
    VmmNew(vmm::Error),

    VmmConfigure(vmm::Error),

    VmmRun(vmm::Error),
}

fn main() -> Result<(), Error> {
    let opts: VMMOpts = VMMOpts::parse();

    // Create a new VMM
    let mut vmm = VMM::new().map_err(Error::VmmNew)?;

    // Configure the VMM:
    // * Number of virtual CPUs
    // * Memory size (in MB)
    // * Path to a Linux kernel
    // * Optional path to console file
    vmm.configure(
        opts.cpus,
        opts.memory,
        &opts.kernel,
        opts.console,
        opts.no_console,
        opts.initramfs,
        opts.net,
    )
    .map_err(Error::VmmConfigure)?;

    // To use Writer with serial device :
    // * Create mpsc channel :
    // let (tx, rx) = std::sync::mpsc::channel();
    // * Create a new Writer
    // let writer = Writer::new(tx);
    // * Add the Writer when configuring the VMM
    // * Use the rx receiver to read the data

    // Run the VMM
    vmm.run(opts.no_console).map_err(Error::VmmRun)?;

    Ok(())
}
