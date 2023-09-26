#![feature(abi_x86_interrupt)]

use crate::println;
use pic8259::ChainedPics;
use spin::{Mutex, Once};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

pub static PICS: Once<Mutex<ChainedPics>> = Once::new();

// hardware interrupts PICs (slots 32-47)
// Safety - ensure that the PICs does not overlap
const PIC_1_OFFSET: u8 = 32;
const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}
impl From<InterruptIndex> for u8 {
    fn from(value: InterruptIndex) -> Self {
        value as u8
    }
}
impl From<InterruptIndex> for usize {
    fn from(value: InterruptIndex) -> Self {
        value as usize
    }
}

/// Initialize Programmable Interrupt Controllers (PICs).
///
/// This functions setups the standard PIC 1 and PIC 2 controllers to properly handle hardware
/// interrupts.
///
/// # Panics
/// This function will panic if it is called more than once.
pub fn init_pic() {
    PICS.call_once(|| {
        // # Safety
        // we ensure that the PICs does not overlap and the offsets are correct
        let mut chained_pics = unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) };
        unsafe {
            chained_pics.initialize();
        }

        Mutex::new(chained_pics)
    });
}

pub fn set_handler<T: InterruptHandler>(idt: &mut InterruptDescriptorTable, _: T) {
    if !PICS.is_completed() {
        panic!("PICs must be initialized before setting up interrupt handlers");
    }

    idt[usize::from(T::INDEX)].set_handler_fn(unsafe {
        core::mem::transmute::<
            unsafe extern "x86-interrupt" fn(InterruptStackFrame),
            extern "x86-interrupt" fn(InterruptStackFrame),
        >(T::handle)
    });
}

pub unsafe trait InterruptHandler {
    const INDEX: InterruptIndex;

    unsafe extern "x86-interrupt" fn handle(_: InterruptStackFrame);
}

#[macro_export]
macro_rules! interrupt_handler {
    (name = $name:ident, index = $index:expr, handler = fn$($tokens:tt)*) => {
        struct $name;

        unsafe impl $crate::hardware_interrupt::InterruptHandler for $name {
            const INDEX: $crate::hardware_interrupt::InterruptIndex = $index;

            unsafe extern "x86-interrupt" fn handle(stack_frame: InterruptStackFrame) {
                #[inline(always)]
                fn inner$($tokens)*
                inner(stack_frame);

                $crate::hardware_interrupt::PICS.get().unwrap().lock().notify_end_of_interrupt($index.into());
            }
        }
    }
}
