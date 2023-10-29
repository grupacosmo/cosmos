#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use kernel::BOOTLOADER_CONFIG;
use test_kernel::prelude::*;

entry_point!(main, config = &BOOTLOADER_CONFIG);

fn main(boot_info: &'static mut BootInfo) -> ! {
    kernel::init(boot_info);
    exit_qemu(QemuExitCode::Success)
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use core::fmt::Write;
    writeln!(serial(), "{info}").unwrap();
    exit_qemu(QemuExitCode::Failed);
}
