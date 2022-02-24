# Lumper

<img src="https://img.shields.io/github/workflow/status/virt-do/lumper/lumper%20build%20and%20unit%20tests?style=for-the-badge" />

`lumper` is an experimental Virtual Machine Manager written in Rust. The project aims to provide a performant, intuitive CLI allowing users to run & manage a virtual machine.

**Project is experimental and should not be used in any production systems.**

## Quick start

### Prerequisites

Make sure you have a compiled linux kernel with a initramfs configured before you start.

If you don't have one, follow these steps :

- Make a basic rootfs :

```bash
./rootfs/mkrootfs.sh
```

- Build a linux kernel :

```bash
./kernel/mkkernel.sh
```

Press enter when `Built-in initramfs compression mode` is asked.

### Build

```bash
cd lumper
cargo build --release
```

### Run

```bash
./target/release/lumper --kernel <your_kernel_path>/arch/x86/boot/compressed/vmlinux.bin
```

You should see this in your console :

```bash
/ #
```

To see all the available arguments :

```bash
./target/release/lumper --help
```

## How to contribute ?

If you are interested in contributing to the Lumper project, please take a look
at the [CONTRIBUTING.md](CONTRIBUTING.md) guide.