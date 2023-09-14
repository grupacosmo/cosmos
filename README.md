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
## QEMU
### Linux

#### Arch
```shell
pacman -S qemu-desktop
```

#### Ubuntu
```shell
apt install qemu-system-x86
```

### Windows
1. Install QEMU from [here](https://qemu.weilnetz.de/w64/)
2. Add qemu installation directory to PATH

# Run
This will build the kernel, create an image and launch it with qemu.
```
cargo run
```