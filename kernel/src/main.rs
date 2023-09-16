#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use bootloader_api::{entry_point, BootInfo};
use core::panic::PanicInfo;
use kernel::{interrupt, logger, println, vga};

entry_point!(main);

fn main(boot_info: &'static mut BootInfo) -> ! {
    interrupt::init();

    let framebuffer = boot_info.framebuffer.as_mut().unwrap();
    let vga = vga::Writer::new(framebuffer);

    logger::init_global(vga);

    // FIXME: cleanup main once we have a test harness
    for num in 0..50 {
        println!("line {}", num)
    }
    println!("end");

    fn stack_overflow() {
        stack_overflow();
    }
    stack_overflow();

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    loop {}
}
