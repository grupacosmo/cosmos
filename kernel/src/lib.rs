#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use bootloader_api::{config::Mapping, BootInfo, BootloaderConfig};

pub mod interrupt;
pub mod logger;
pub mod memory;
pub mod vga;

pub use bootloader_api;
pub use x86_64;
use x86_64::VirtAddr;

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

pub fn init(boot_info: &'static mut BootInfo) {
    interrupt::init();

    let framebuffer = boot_info.framebuffer.as_mut().unwrap();
    let vga = vga::Writer::new(framebuffer);

    logger::init_global(vga);

    let Some(physical_memory_offset) = boot_info
        .physical_memory_offset
        .into_option()
        .map(VirtAddr::new)
    else {
        panic!("physical_memory_offset in not present, make sure that `map-physical-memory` BootloaderConfig option is enabled");
    };

    memory::init_global(physical_memory_offset, &boot_info.memory_regions);
    interrupt::enable_interrupts();
}

pub fn halt_loop() -> ! {
    loop {
        // halts until next interrupt
        x86_64::instructions::hlt();
    }
}
