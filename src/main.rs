use std::{io::Read, os::unix::net::UnixListener, path::Path, thread::sleep, u32};

use clap::Parser;
use vmm::{devices::Writer, VMM};

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

    #[clap(long)]
    socket: bool,
}

#[derive(Debug)]
pub enum Error {
    VmmNew(vmm::Error),

    VmmConfigure(vmm::Error),

    VmmRun(vmm::Error),
}

fn main() -> Result<(), Error> {
    let opts: VMMOpts = VMMOpts::parse();

    let console = opts.console.unwrap();
    if opts.socket {
        let path = Path::new(console.as_str());
        if std::fs::metadata(path).is_ok() {
            std::fs::remove_file(path).unwrap();
        }

        println!("Socket path: {}", path.to_str().unwrap());

        let unix_listener = UnixListener::bind(path).unwrap();

        std::thread::spawn(move || {
            // read from socket
            let (mut stream, _) = unix_listener.accept().unwrap();
            let mut buffer = [0; 1024];
            loop {
                let n = stream.read(&mut buffer).unwrap();
                if n == 0 {
                    break;
                }
                let s = String::from_utf8_lossy(&buffer[0..n]).to_string();
                print!("{}", s);
            }
        });
    }

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
        Some(console),
        opts.no_console,
        opts.initramfs,
        opts.net,
        opts.socket,
    )
    .map_err(Error::VmmConfigure)?;

    // Run the VMM
    vmm.run(opts.no_console).map_err(Error::VmmRun)?;

    Ok(())
}
