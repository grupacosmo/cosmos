use crate::vga;
use core::fmt;
use noto_sans_mono_bitmap::{FontWeight, RasterHeight};
use spin::{once::Once, Mutex};

const FONT_WEIGHT: FontWeight = FontWeight::Regular;
const RASTER_HEIGHT: RasterHeight = RasterHeight::Size16;
const RASTER_WIDTH: usize = noto_sans_mono_bitmap::get_raster_width(FONT_WEIGHT, RASTER_HEIGHT);

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::logger::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    LOGGER.wait().lock().write_fmt(args).unwrap();
}

static LOGGER: Once<Mutex<Logger<'static>>> = Once::new();

/// # Panics
/// The function will panic if it is called more than once.
pub fn init_global(writer: vga::Writer<'static>) {
    let mut logger = Logger::new(writer);
    logger.clear();
    LOGGER.call_once(|| Mutex::new(logger));
}

#[derive(Debug)]
pub struct Logger<'a> {
    writer: vga::Writer<'a>,
    x: usize,
    y: usize,
}

impl<'a> Logger<'a> {
    pub fn new(writer: vga::Writer<'a>) -> Self {
        Self { writer, x: 0, y: 0 }
    }

    pub fn newline(&mut self) {
        self.y += RASTER_HEIGHT.val();
        self.x = 0;
    }

    pub fn clear(&mut self) {
        self.x = 0;
        self.y = 0;
        self.writer.clear();
    }

    pub fn write_char(&mut self, c: char) {
        if c == '\n' {
            self.newline();
            return;
        }

        if c == '\r' {
            self.x += 0;
            return;
        }

        if self.x + RASTER_WIDTH >= self.writer.width() {
            self.newline();
        }

        if self.y + RASTER_HEIGHT.val() >= self.writer.height() {
            self.clear()
        }

        let rc = noto_sans_mono_bitmap::get_raster(c, FONT_WEIGHT, RASTER_HEIGHT).unwrap();
        for (i, row) in rc.raster().iter().enumerate() {
            for (j, &pixel) in row.iter().enumerate() {
                let color = vga::Color::new(0, pixel, 0);
                self.writer.write_pixel(self.x + j, self.y + i, color);
            }
        }
        self.x += RASTER_WIDTH
    }
}

impl fmt::Write for Logger<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c)
        }
        Ok(())
    }
}
