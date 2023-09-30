#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use kernel::{
    memory::MEMORY_MANAGER,
    x86_64::{structures::paging::PageTableFlags, VirtAddr},
    BOOTLOADER_CONFIG,
};
use test_kernel::prelude::*;

entry_point!(main, config = &BOOTLOADER_CONFIG);

fn main(boot_info: &'static mut BootInfo) -> ! {
    kernel::init(boot_info);

    let memory_namager = MEMORY_MANAGER.get().unwrap();

    let region_start = 0x4242_4242_0000_u64;

    memory_namager
        .lock()
        .allocate_frames_for_memory_region(
            region_start,
            0x4000,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
        )
        .expect("failed to allocate pages");

    let virt = VirtAddr::new(region_start);
    let phys = memory_namager.lock().translate_address(virt);

    assert_ne!(phys, None);

    let ptr = region_start as *mut u8;
    unsafe { ptr.write_volatile(0x42) };
    let value = unsafe { ptr.read() };

    assert_eq!(value, 0x42);

    exit_qemu(QemuExitCode::Success)
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use core::fmt::Write;
    writeln!(serial(), "{info}").unwrap();
    exit_qemu(QemuExitCode::Failed);
}
