#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use bootloader_api::{entry_point, BootInfo};
use core::panic::PanicInfo;
use kernel::{println, BOOTLOADER_CONFIG};

entry_point!(main, config = &BOOTLOADER_CONFIG);

fn main(boot_info: &'static mut BootInfo) -> ! {
    kernel::init(boot_info);

    println!("it works");

    kernel::halt_loop();
}
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    kernel::halt_loop();
}
