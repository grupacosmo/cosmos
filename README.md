# CosmOS
A simple operating system written in Rust.

# Table of Contents
- [CosmOS](#cosmos)
- [Setup](#setup)
  - [QEMU](#qemu)
- [Run](#run)

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