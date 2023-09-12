#![no_std]
#![no_main]

use core::{panic::PanicInfo, slice};

mod color {
    pub const BLACK: u8 = 0x0;
    pub const GREEN: u8 = 0x2;
    pub const RED: u8 = 0x4;
    pub const BROWN: u8 = 0x6;
    pub const CYAN: u8 = 0xb;
    pub const WHITE: u8 = 0xf;
}

const VGA_BUFFER_ADDRESS: usize = 0xb8000;
const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 20;
const VGA_BUFFER_LEN: usize = VGA_WIDTH * VGA_HEIGHT * 2;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let vga_buffer =
        unsafe { slice::from_raw_parts_mut(VGA_BUFFER_ADDRESS as *mut u8, VGA_BUFFER_LEN) };

    draw_hello(vga_buffer);

    draw_tree(2, vga_buffer);

    loop {}
}

fn draw_hello(vga_buffer: &mut [u8]) {
    for (i, &byte) in b"Hello World!".iter().enumerate() {
        vga_buffer[i * 2] = byte;
        vga_buffer[i * 2 + 1] = if i % 2 == 0 { color::CYAN } else { color::WHITE };
    }
}

fn draw_tree(h_offset: usize, vga_buffer: &mut [u8]) {
    const H: usize = 7;
    const HALF_H: usize = H/2;

    for i in 1..H {
        let offset = H - i;
        let pos_y = (i + h_offset);
        let start_x_offset = offset;

        for w in 1..i*2 {
            let pos_x = start_x_offset + w;
            let index = xy_to_vga_buffer_offset(pos_x, pos_y);
            let ball = (w ^ i) % 3 != 0;
            vga_buffer[index] = if ball { b' ' } else { b'*' };
            vga_buffer[index + 1] = full_color(color::RED, color::GREEN);
        }
    }

    let trunk_buffer_offset =
        xy_to_vga_buffer_offset(H, H + h_offset);
    vga_buffer[trunk_buffer_offset] = b' ';
    vga_buffer[trunk_buffer_offset + 1] = full_color(color::WHITE, color::BROWN);
}

fn xy_to_vga_buffer_offset(x: usize, y: usize) -> usize {
    y * VGA_WIDTH * 2 + x * 2
}

fn full_color(fg: u8, bg: u8) -> u8 {
    fg | bg << 4
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
