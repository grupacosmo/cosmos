# CosmOS
A simple operating system written in Rust.

**TOC**
- [CosmOS](#cosmos)
- [Setup](#setup)
  - [Linux](#linux)
    - [Arch](#arch)
    - [Ubuntu](#ubuntu)
  - [Windows](#windows)
- [Run](#run)

# Setup

1. Install QEMU (see below)

## Linux

### Arch
```shell
pacman -S qemu-desktop
```

### Ubuntu
```shell
apt install qemu-system-x86
```

## Windows
Install QEMU from [here](https://www.qemu.org/download/#windows)

# Run
This will build the kernel, create an image and launch it with qemu.
```
cargo run
```