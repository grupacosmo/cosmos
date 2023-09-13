#![no_std]
#![no_main]

use bootloader_api::{entry_point, info::PixelFormat, BootInfo};
use core::{panic::PanicInfo, ptr};

entry_point!(main);

fn main(boot_info: &'static mut BootInfo) -> ! {
    let framebuffer = boot_info.framebuffer.as_mut().unwrap();
    let mut framebuffer_info = framebuffer.info();
    let buf = boot_info.framebuffer.as_mut().unwrap().buffer_mut();

    // clear bios logs
    for b in buf.iter_mut() {
        *b = 0x00;
    }

    let mut write_pixel = |x: usize, y: usize, intensity: u8| {
        let pixel_offset = y * framebuffer_info.stride + x;
        let color = match framebuffer_info.pixel_format {
            PixelFormat::Rgb => [intensity, intensity, intensity / 2, 0],
            PixelFormat::Bgr => [intensity / 2, intensity, intensity, 0],
            PixelFormat::U8 => [if intensity > 200 { 0xf } else { 0 }, 0, 0, 0],
            other => {
                // set a supported (but invalid) pixel format before panicking to avoid a double
                // panic; it might not be readable though
                framebuffer_info.pixel_format = PixelFormat::Rgb;
                panic!("pixel format {:?} not supported in logger", other)
            }
        };
        let bytes_per_pixel = framebuffer_info.bytes_per_pixel;
        let byte_offset = pixel_offset * bytes_per_pixel;
        buf[byte_offset..(byte_offset + bytes_per_pixel)]
            .copy_from_slice(&color[..bytes_per_pixel]);
        let _ = unsafe { ptr::read_volatile(&buf[byte_offset]) };
    };

    for i in 0..10 {
        for j in 0..10 {
            write_pixel(20 + i, 5 + j, 0xB9);
        }
    }

    for i in 0..10 {
        for j in 0..10 {
            write_pixel(40 + i, 5 + j, 0xB9);
        }
    }

    for i in 0..50 {
        for j in 0..10 {
            write_pixel(10 + i, 40 + j, 0xB9);
        }
    }

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
