#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use bootloader_api::BootInfo;

pub mod interrupt;
pub mod logger;
pub mod vga;

pub use bootloader_api;
pub use x86_64;

pub fn init(boot_info: &'static mut BootInfo) {
    interrupt::init();

    let framebuffer = boot_info.framebuffer.as_mut().unwrap();
    let vga = vga::Writer::new(framebuffer);

    logger::init_global(vga);
    interrupt::enable_interrupts();
}

pub fn halt_loop() -> ! {
    loop {
        // halts until next interrupt
        x86_64::instructions::hlt();
    }
}
