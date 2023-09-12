#![no_std]
#![no_main]

use core::{panic::PanicInfo, slice};

mod color {
    pub const CYAN: u8 = 0xb;
}

const VGA_BUFFER_ADDRESS: usize = 0xb8000;
const VGA_BUFFER_LEN: usize = 20 * 80;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let vga_buffer =
        unsafe { slice::from_raw_parts_mut(VGA_BUFFER_ADDRESS as *mut u8, VGA_BUFFER_LEN) };

    for (i, &byte) in b"Hello World!".iter().enumerate() {
        vga_buffer[i * 2] = byte;
        vga_buffer[i * 2 + 1] = color::CYAN;
    }

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
