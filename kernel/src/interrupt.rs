use crate::println;
use spin::once::Once;
use x86_64::{
    instructions::tables,
    registers::segmentation::{Segment as _, CS},
    structures::{
        gdt::{self, GlobalDescriptorTable},
        idt::{InterruptDescriptorTable, InterruptStackFrame},
        tss::TaskStateSegment,
    },
    VirtAddr,
};

static IDT: Once<InterruptDescriptorTable> = Once::new();
static TSS: Once<TaskStateSegment> = Once::new();
static GDT: Once<GlobalDescriptorTable> = Once::new();
static SEGMENT_SELECTORS: Once<SegmentSelectors> = Once::new();

const DOUBLE_FAULT_IST_INDEX: u16 = 0;

struct SegmentSelectors {
    code_selector: gdt::SegmentSelector,
    tss_selector: gdt::SegmentSelector,
}

pub fn init() {
    init_gdt();
    init_idt();
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
pub fn init_idt() {
    let idt = IDT.call_once(|| {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        // # Safety
        // `DOUBLE_FAULT_IST_INDEX` has a corresponding entry in IST and is not used by any other
        // interrupt handler
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX);
        }
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
pub fn init_gdt() {
    let tss = TSS.call_once(|| {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[usize::from(DOUBLE_FAULT_IST_INDEX)] = {
            const STACK_SIZE: usize = 4096 * 5;
            // FIXME: use dynamic memory allocation instead of statically allocated stack
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            // # Safety
            // This code can be called only once so there will be no data races
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            // Return end address as stacks are filled from the end to the beggining
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
        CS::set_reg(selectors.code_selector);
        tables::load_tss(selectors.tss_selector);
    }
}

extern "x86-interrupt" fn breakpoint_handler(frame: InterruptStackFrame) {
    // FIXME handle breakpoint
    println!("Exception: breakpoint\n{:#?}", frame)
}

extern "x86-interrupt" fn double_fault_handler(frame: InterruptStackFrame, _error_code: u64) -> ! {
    // FIXME handle double fault
    panic!("Exception: double fault\n{:#?}", frame);
}
