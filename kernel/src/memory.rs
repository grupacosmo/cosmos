use bootloader_api::info::{MemoryRegion, MemoryRegionKind, MemoryRegions};
use spin::{Mutex, Once};
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTableFlags, PhysFrame, Size4KiB, Translate,
};
use x86_64::{structures::paging::PageTable, PhysAddr, VirtAddr};

const PAGE_FRAME_SIZE: usize = 4096;

pub static MEMORY_MANAGER: Once<Mutex<MemoryManager>> = Once::new();

/// # Panics
/// The function will panic if it is called more than once.
pub fn init_global(physical_memory_offset: VirtAddr, memory_regions: &'static MemoryRegions) {
    let manager = unsafe { MemoryManager::new(physical_memory_offset, memory_regions) };
    MEMORY_MANAGER.call_once(|| Mutex::new(manager));
}

pub struct MemoryManager {
    mapper: OffsetPageTable<'static>,
    frame_allocator: BootInfoFrameAllocator,
}

impl MemoryManager {
    /// Creates new instance of [`MemoryManager`]
    ///
    /// ## Safety
    ///
    /// Caller of this function mus guarantee that the complete physical memory is mapped to
    /// virtual memory at the passed `physical_memory_offset`.
    ///
    /// Caller of this function  must guarantee that the passed
    /// `memory_regions` are valid. The main requirement is that all frames that are marked
    /// as `USABLE` in it are really unused.
    ///
    /// This function must be called only once to avoid aliasing `&mut` references (which is undefined behavior).
    unsafe fn new(
        physical_memory_offset: VirtAddr,
        memory_regions: &'static MemoryRegions,
    ) -> Self {
        Self {
            mapper: unsafe { init_mapper(physical_memory_offset) },
            frame_allocator: unsafe { BootInfoFrameAllocator::init(memory_regions) },
        }
    }

    /// Provides the physical address to which the virtual address has been mapped to.
    /// Returns [`None`] if there is no valid mapping for the given virtual address.
    pub fn translate_address(&self, addr: VirtAddr) -> Option<PhysAddr> {
        self.mapper.translate_addr(addr)
    }

    /// Allocates frames for virtual memory region.
    /// * `region_start` - virtual address at which the region starts
    /// * `region_size` - size of the region
    /// * `flags` - combination of flags for memory pages
    ///
    /// [`MapToError`] determines whether an error occurred during frame allocation or page
    /// mapping.
    /// # Example
    /// ```
    ///let memory_namager = MEMORY_MANAGER.get().unwrap();
    ///memory_namager
    ///    .lock()
    ///    .allocate_frames_for_memory_region(
    ///        0x4242_4242_0000_u64,
    ///        0x4000,
    ///        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
    ///    )
    ///    .expect("failed to allocate pages");
    /// ```
    pub fn allocate_frames_for_memory_region(
        &mut self,
        region_start: VirtAddr,
        region_size: usize,
        flags: PageTableFlags,
    ) -> Result<(), MapToError<Size4KiB>> {
        // Calculate which pages contain addresses from given memory region
        // and createa range of that pages
        let page_range = {
            let region_start = VirtAddr::new(region_start.as_u64());
            // Substract 1 to get inclusive bound
            let region_end = region_start + region_size - 1u64;
            let region_start_page = Page::containing_address(region_start);
            let region_end_page = Page::containing_address(region_end);
            Page::range_inclusive(region_start_page, region_end_page)
        };

        for page in page_range {
            // Allocate avalible USABLE memory frame
            let frame = self
                .frame_allocator
                .allocate_frame()
                .ok_or(MapToError::FrameAllocationFailed)?;

            // Map page to allocated memory frame
            // SAFETY
            // This is safe because we map an unmapped page to USABLE frame.
            // In case of mapping already mapped paged an apprioriate Error is returned.
            // This function can't remap pages so it's impossible to cause value invalidation and
            // provoke UB.
            unsafe {
                self.mapper
                    .map_to(page, frame, flags, &mut self.frame_allocator)?
                    .flush()
            };
        }

        Ok(())
    }
}

/// Initialize a new OffsetPageTable which allows for mapping virtual pages to the physical memory
/// frames.
///
/// ## Safety
///
/// Caller of this function must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`.
///
/// This function must be called only once to avoid aliasing `&mut` references (which is undefined behavior).
unsafe fn init_mapper(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

/// Returns a mutable reference to the active level 4 table.
///
/// ## Safety
///
/// Caller of this function  must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`.
///
/// This function must be called only once to avoid aliasing `&mut` references (which is undefined behavior).
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    // Get the frame where the Level 4 table is stored
    // That table occupies whole frame
    let (level_4_table_frame, _) = Cr3::read();

    // Get virtual address of frame's start
    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();

    // Obtain mutable pointer to the table
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
struct BootInfoFrameAllocator {
    memory_regions: &'static [MemoryRegion],
    next: usize,
}

impl BootInfoFrameAllocator {
    /// Create a FrameAllocator from the passed memory map.
    ///
    /// ## Safety
    ///
    /// Caller of this function  must guarantee that the passed
    /// `memory_regions` are valid. The main requirement is that all frames that are marked
    /// as `USABLE` in it are really unused.
    unsafe fn init(memory_regions: &'static MemoryRegions) -> Self {
        BootInfoFrameAllocator {
            memory_regions,
            next: 0,
        }
    }

    /// Returns an iterator over the usable frames specified in [`MemoryRegions`].
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        self.memory_regions
            .iter()
            .filter(|r| r.kind == MemoryRegionKind::Usable)
            // map each region to its address range
            .map(|r| r.start..r.end)
            // transform to an iterator of frame start addresses
            .flat_map(|r| r.step_by(PAGE_FRAME_SIZE))
            .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
