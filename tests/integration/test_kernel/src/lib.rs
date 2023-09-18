#![no_std]

use core::unreachable;

pub mod prelude {
    pub use super::{exit_qemu, serial, QemuExitCode};
    pub use core::panic::PanicInfo;
    pub use kernel::bootloader_api::{entry_point, BootInfo};
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    use kernel::x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }

    unreachable!()
}

pub fn serial() -> uart_16550::SerialPort {
    let mut port = unsafe { uart_16550::SerialPort::new(0x3F8) };
    port.init();
    port
}
