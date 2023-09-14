#![no_std]
#![no_main]

use bootloader_api::{entry_point, BootInfo};
use core::panic::PanicInfo;

mod logger;
mod vga;

entry_point!(main);

fn main(boot_info: &'static mut BootInfo) -> ! {
    let framebuffer = boot_info.framebuffer.as_mut().unwrap();
    let vga = vga::Writer::new(framebuffer);

    logger::init_global(vga);

    // dupa driven development
    println!("dupa");

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    loop {}
}
