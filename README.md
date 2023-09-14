# CosmOS
A simple operating system written in Rust.

# Table of Contents
- [CosmOS](#cosmos)
- [Setup](#setup)
  - [QEMU](#qemu)
- [Run](#run)
- [OS dev resources](#os-dev-resources)
  - [General](#general)
  - [Bootloader](#bootloader)

# Setup

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
- Install QEMU from [here](https://qemu.weilnetz.de/w64/)
- Add toolchain 
    ```
    rustup component add rust-src --toolchain nightly-x86_64-pc-windows-msvc
     ```
- Install llvm 
    ```
    rustup component add llvm-tools-preview 
    ```
- Add qemu to PATH (example: `C:\Program Files\qemu`)  

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