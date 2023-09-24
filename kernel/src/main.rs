#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use bootloader_api::{entry_point, BootInfo};
use core::panic::PanicInfo;
use kernel::println;

entry_point!(main);

fn main(boot_info: &'static mut BootInfo) -> ! {
    kernel::init(boot_info);

    println!("it works");

    loop {
        // halts until next interrupt
        x86_64::instructions::hlt();
    }
}
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    loop {
        x86_64::instructions::hlt();
    }
}
