#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use test_kernel::prelude::*;

entry_point!(main);

fn main(boot_info: &'static mut BootInfo) -> ! {
    kernel::init(boot_info);

    #[allow(unconditional_recursion)]
    fn stack_overflow() {
        stack_overflow();
    }
    stack_overflow();

    exit_qemu(QemuExitCode::Failed)
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use core::fmt::Write;

    let mut buf = heapless::String::<512>::new();
    write!(buf, "{info}").unwrap();
    if buf.contains("double fault") {
        exit_qemu(QemuExitCode::Success);
    } else {
        writeln!(serial(), "{info}").unwrap();
        exit_qemu(QemuExitCode::Failed);
    }
}
