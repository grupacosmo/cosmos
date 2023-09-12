# CosmOS
A simple operating system written in Rust.

**TOC**
- [Setup](#setup)
  - [Linux](#linux)
    - [Arch](#arch)
    - [Ubuntu](#ubuntu)
  - [Windows](#windows)
- [Run](#run)

# Setup

1. Install bootimage
    ```
    cargo install bootimage
    ```
2. Install QEMU (see below)

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
This will build the kernel, link it and launch qemu.
```
cargo run
```