#![no_std]
#![no_main]

use bootloader_api::{entry_point, BootInfo};
use core::panic::PanicInfo;

mod vga;

entry_point!(main);

fn main(boot_info: &'static mut BootInfo) -> ! {
    let mut framebuffer = boot_info.framebuffer.as_mut().unwrap();
    let mut vga = vga::Writer::new(&mut framebuffer);

    vga.clear();

    for i in 0..10 {
        for j in 0..10 {
            vga.write_pixel(20 + i, 5 + j, vga::Color::GREEN);
        }
    }

    for i in 0..10 {
        for j in 0..10 {
            vga.write_pixel(40 + i, 5 + j, vga::Color::GREEN);
        }
    }

    for i in 0..50 {
        for j in 0..10 {
            vga.write_pixel(10 + i, 40 + j, vga::Color::GREEN);
        }
    }

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
