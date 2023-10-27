use crate::{print, println};
use core::fmt::Debug;
use pc_keyboard::{HandleControl, Keyboard};
use pic8259::ChainedPics;
use spin::once::Once;
use spin::Mutex;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::{
    instructions,
    instructions::port::Port,
    instructions::tables,
    registers::segmentation::{Segment as _, CS, SS},
    structures::{
        gdt::{self, GlobalDescriptorTable},
        idt::{InterruptDescriptorTable, InterruptStackFrame},
        tss::TaskStateSegment,
    },
    VirtAddr,
};

type SupportedKeyboard = Keyboard<pc_keyboard::layouts::Us104Key, pc_keyboard::ScancodeSet1>;

static IDT: Once<InterruptDescriptorTable> = Once::new();
static TSS: Once<TaskStateSegment> = Once::new();
static GDT: Once<GlobalDescriptorTable> = Once::new();
static SEGMENT_SELECTORS: Once<SegmentSelectors> = Once::new();
static PICS: Once<Mutex<ChainedPics>> = Once::new();
static KEYBOARD: Once<Mutex<SupportedKeyboard>> = Once::new();

const DOUBLE_FAULT_IST_INDEX: u16 = 0;

// hardware interrupts PICs (slots 32-47)
// Safety - ensure that the PICs does not overlap
const PIC_1_OFFSET: u8 = 32;
const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

const PS2_CONTROLLER_PORT: u16 = 0x60;

/// Initialize interrupt handlers.
///
/// # Panics
/// This function will panic if it is called more than once.
pub fn init() {
    init_gdt();
    init_idt();
    init_pic();
}

/// Enable interrupts if they are not enabled.
pub fn enable_interrupts() {
    if !instructions::interrupts::are_enabled() {
        instructions::interrupts::enable();
    }
}

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

#[derive(Debug)]
struct SegmentSelectors {
    code_selector: gdt::SegmentSelector,
    tss_selector: gdt::SegmentSelector,
}

/// Initialize the Interrupt Descriptor Table (IDT).
///
/// IDT stores function pointers to interrupt handlers.
/// Handlers can have their dedicated stacks configured in Interrupt Stack Table (IST).
/// This is useful when catching exceptions caused by kernel's stack overflow to avoid memory
/// corruption.
///
/// Refer to [`InterruptDescriptorTable`] for more information.
///
/// # Panics
/// This function will panic if it is called more than once.
fn init_idt() {
    let idt = IDT.call_once(|| {
        let mut idt = InterruptDescriptorTable::new();

        // # exceptions
        idt.alignment_check.set_handler_fn(alignment_check_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.bound_range_exceeded
            .set_handler_fn(bound_range_exceeded_handler);
        idt.debug.set_handler_fn(debug_handler);
        idt.invalid_tss.set_handler_fn(invalid_tss_handler);
        idt.stack_segment_fault
            .set_handler_fn(stack_segment_fault_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.virtualization.set_handler_fn(virtualization_handler);
        idt.vmm_communication_exception
            .set_handler_fn(vmm_communication_exception_handler);
        idt.general_protection_fault
            .set_handler_fn(general_protection_fault_handler);
        // # Safety
        // `DOUBLE_FAULT_IST_INDEX` has a corresponding entry in IST and is not used by any other
        // interrupt handler
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX);
        }

        idt[InterruptIndex::Timer.into()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.into()].set_handler_fn(keyboard_interrupt_handler);

        idt
    });
    idt.load();
}

/// Initialize the Global Descriptor Table (GDT.
///
/// GDT contains the _segments_ of the program. Each segment describes a different purpose.
/// The TSS segment's interrupt stack table holds pointers to stacks dedicated for interrupt
/// handlers.
///
/// Refer to [`GlobalDescriptorTable`] for more information.
///
/// # Panics
/// This function will panic if it is called more than once.
fn init_gdt() {
    let tss = TSS.call_once(|| {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[usize::from(DOUBLE_FAULT_IST_INDEX)] = {
            const STACK_SIZE: usize = 4096 * 5;
            // FIXME: use dynamic memory allocation instead of statically allocated stack
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            // # Safety
            // This code can be called only once so there will be no data races
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            // Return end address as stacks are filled from the end to the beginning
            stack_start + STACK_SIZE
        };
        tss
    });

    let gdt = GDT.call_once(|| {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(gdt::Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(gdt::Descriptor::tss_segment(tss));
        SEGMENT_SELECTORS.call_once(|| SegmentSelectors {
            code_selector,
            tss_selector,
        });
        gdt
    });

    gdt.load();

    let selectors = SEGMENT_SELECTORS.get().unwrap();

    // # Safety
    // Above we ensure that the code and TSS selectors point to valid entries
    unsafe {
        // At this point the SS (stack segment) register contains selector with index 2,
        // which happens to be the index of TSS. Set it to 0 to avoid issues.
        // You can call `println!("{:?}", (SS::get_reg(), selectors));` to see this for yourself.
        SS::set_reg(gdt::SegmentSelector::NULL);
        CS::set_reg(selectors.code_selector);
        tables::load_tss(selectors.tss_selector);
    }
}

/// Initialize Programmable Interrupt Controllers (PICs).
///
/// This functions setups the standard PIC 1 and PIC 2 controllers to properly handle hardware
/// interrupts.
///
/// # Panics
/// This function will panic if it is called more than once.
fn init_pic() {
    let pics = PICS.call_once(|| {
        // # Safety
        // we ensure that the PICs does not overlap and the offsets are correct
        let chained_pics = unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) };

        Mutex::new(chained_pics)
    });

    // initialize keyboard before enabling interrupts
    init_keyboard();

    // # Safety
    // we ensure that the PICs are properly configured
    unsafe {
        pics.lock().initialize();
    }
}

fn init_keyboard() {
    KEYBOARD.call_once(|| {
        let keyboard = Keyboard::new(
            pc_keyboard::ScancodeSet1::new(),
            pc_keyboard::layouts::Us104Key,
            HandleControl::Ignore,
        );

        Mutex::new(keyboard)
    });
}

// Exception handlers
extern "x86-interrupt" fn breakpoint_handler(frame: InterruptStackFrame) {
    // FIXME handle breakpoint
    println!("Exception: breakpoint\n{:#?}", frame)
}

extern "x86-interrupt" fn page_fault_handler(
    frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    let accessed_addr = Cr2::read();

    panic!(
        "Exception: page fault\n\
         Accessed address: {accessed_addr:?}\n\
         Error code: {error_code:?}\n\
         {frame:#?}",
    )
}

extern "x86-interrupt" fn invalid_tss_handler(frame: InterruptStackFrame, _error_code: u64) {
    panic!("Exception: invalid tss\n{:#?}", frame)
}

extern "x86-interrupt" fn alignment_check_handler(frame: InterruptStackFrame, _error_code: u64) {
    panic!("Exception: alignment check\n{:#?}", frame)
}

extern "x86-interrupt" fn bound_range_exceeded_handler(frame: InterruptStackFrame) {
    panic!("Exception: bounds range exceeded\n{:#?}", frame)
}

extern "x86-interrupt" fn debug_handler(frame: InterruptStackFrame) {
    panic!("Exception: debug\n{:#?}", frame)
}

extern "x86-interrupt" fn stack_segment_fault_handler(
    frame: InterruptStackFrame,
    _error_code: u64,
) {
    panic!("Exception: stack segment fault\n{:#?}", frame)
}

extern "x86-interrupt" fn virtualization_handler(frame: InterruptStackFrame) {
    panic!("Exception: virtualization\n{:#?}", frame)
}

extern "x86-interrupt" fn vmm_communication_exception_handler(
    frame: InterruptStackFrame,
    _error_code: u64,
) {
    panic!("Exception: vmm communication exception\n{:#?}", frame)
}

extern "x86-interrupt" fn general_protection_fault_handler(
    frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "Exception: general protection fault\nerror code: {error_code}\n{:#?}",
        frame
    )
}

extern "x86-interrupt" fn double_fault_handler(frame: InterruptStackFrame, _error_code: u64) -> ! {
    // FIXME handle double fault
    panic!("Exception: double fault\n{:#?}", frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // # Safety
    // we use the same interrupt number as the handler is registered for
    unsafe {
        PICS.get()
            .unwrap()
            .lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.into());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // # Safety
    // we read from the keyboard port only on keyboard interrupt
    let scancode: u8 = unsafe { Port::new(PS2_CONTROLLER_PORT).read() };

    let mut keyboard = KEYBOARD.get().unwrap().lock();

    match keyboard.add_byte(scancode) {
        Ok(Some(key_event)) => {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                use pc_keyboard::DecodedKey::Unicode;
                match key {
                    Unicode('\x1b') => print!("ESC"),
                    Unicode('\x08') => print!("BS"),
                    Unicode('\x7f') => print!("DEL"),
                    Unicode('\t') => print!("TAB"), // FIXME: tab not supported in logger?
                    Unicode(character) => print!("{}", character),
                    pc_keyboard::DecodedKey::RawKey(key) => print!("RAW[{:?}]", key),
                }
            }
        }
        Ok(None) => {}
        Err(e) => println!("Keyboard error: {:?}", e),
    }

    // # Safety
    // we use the same interrupt number as the handler is registered for
    unsafe {
        PICS.get()
            .unwrap()
            .lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.into());
    }
}
