use vmm::VMM;

#[derive(Debug)]
pub enum Error {
    VmmNew(vmm::Error),

    VmmConfigure(vmm::Error),

    VmmRun(vmm::Error),
}

fn main() -> Result<(), Error> {
    // Create a new VMM
    let mut vmm = VMM::new().map_err(Error::VmmNew)?;

    // Configure the VMM:
    // * Number of virtual CPUs
    // * Memory size (in MB)
    // * Path to a Linux kernel
    vmm.configure(1, 1024, "/path/to/kernel/vmlinux.bin")
        .map_err(Error::VmmConfigure)?;

    // Run the VMM
    vmm.run();

    Ok(())
}
