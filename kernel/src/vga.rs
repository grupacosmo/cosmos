use core::ptr;

use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const GREEN: Color = Color::new(0x00, 0xFF, 0x00);

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Converts from RGB to grayscale using NTSC formula
    pub fn greyscale(self) -> u8 {
        let (r, g, b) = (f32::from(self.r), f32::from(self.g), f32::from(self.b));
        (0.299 * r + 0.587 * g + 0.114 * b) as u8
    }
}

#[derive(Debug)]
pub struct Writer<'a> {
    buffer: &'a mut [u8],
    info: FrameBufferInfo,
}

impl<'a> Writer<'a> {
    pub fn new(framebuffer: &'a mut FrameBuffer) -> Self {
        Self {
            info: framebuffer.info(),
            buffer: framebuffer.buffer_mut(),
        }
    }

    pub fn width(&self) -> usize {
        self.info.width
    }

    pub fn height(&self) -> usize {
        self.info.height
    }

    pub fn clear(&mut self) {
        self.buffer.fill(0x00);
    }

    pub fn write_pixel(&mut self, x: usize, y: usize, color: Color) {
        let offset = x + y * self.info.stride;
        let color = match self.info.pixel_format {
            PixelFormat::Rgb => [color.r, color.g, color.b, 0],
            PixelFormat::Bgr => [color.b, color.g, color.r, 0],
            PixelFormat::U8 => [color.greyscale(), 0, 0, 0],
            unknown => {
                // set a supported (but invalid) pixel format before panicking to avoid a double
                // panic; it might not be readable though
                self.info.pixel_format = PixelFormat::Rgb;
                panic!("pixel format {:?} not supported in vga writer", unknown)
            }
        };
        let bytes_per_pixel = self.info.bytes_per_pixel;
        let byte_offset = offset * bytes_per_pixel;
        self.buffer[byte_offset..(byte_offset + bytes_per_pixel)]
            .copy_from_slice(&color[..bytes_per_pixel]);
        let _ = unsafe { ptr::read_volatile(&self.buffer[byte_offset]) };
    }

    pub fn shift_up(&mut self, npixels: usize) {
        let offset = self.info.bytes_per_pixel * self.width() * npixels;
        self.buffer.copy_within(offset.., 0);
    }
}
