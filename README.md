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
- [OS dev resources](#os-dev-resources)
  - [General](#general)
  - [Bootloader](#bootloader)

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

# OS dev resources
**General**
- [Writing an OS in Rust](https://os.phil-opp.com/)
- [OSDev Wiki](https://wiki.osdev.org/Main_Page)
- [GitHub rust-osdev/bootloader](https://github.com/rust-osdev/bootloader)

**Bootloader**
- [Writing a boot loader in Assembly and C](https://www.codeproject.com/Articles/664165/Writing-a-boot-loader-in-Assembly-and-C-Part)
- [How to develop your own Boot Loader](https://www.codeproject.com/Articles/36907/How-to-develop-your-own-Boot-Loader)