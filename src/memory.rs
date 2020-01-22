use bootloader::bootinfo::{MemoryMap, MemoryRegion, MemoryRegionType};
use x86_64::{
    structures::paging::{FrameAllocator, Mapper, Page, PhysFrame, Size4KiB, UnusedPhysFrame, PageTable, OffsetPageTable},
    PhysAddr, VirtAddr,
};

pub struct BootInfoFrameAllocator {
    map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(map: &'static MemoryMap) -> Self {
        Self { map, next: 0 }
    }

    fn usable_frames(&self) -> impl Iterator<Item = UnusedPhysFrame> {
        self.map
            .iter()
            // capture only usable regions of memory
            .filter(|r| r.region_type == MemoryRegionType::Usable)
            // get the range of their start and end
            .map(|r| r.range.start_addr()..r.range.end_addr())
            // convert those ranges to page starts only
            .flat_map(|r| r.step_by(4096))
            // convert that start to a physical frame
            .map(|a| PhysFrame::containing_address(PhysAddr::new(a)))
            // convert the physical frame to an unused physical frame
            // since the first step here filters out all the reserved
            // or used frames
            .map(|f| unsafe { UnusedPhysFrame::new(f) })
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<UnusedPhysFrame> {
        let ret = self.usable_frames().nth(self.next);
        self.next += 1;
        ret
    }
}

pub unsafe fn init(offset: VirtAddr) -> OffsetPageTable<'static> {
    let l4 = active_level_4_table(offset);
    OffsetPageTable::new(l4, offset)
}

unsafe fn active_level_4_table(offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;
    let (frame, _) = Cr3::read();
    get_table_mut(&frame, offset)
}

pub unsafe fn translate(addr: VirtAddr, offset: VirtAddr) -> Option<PhysAddr> {
    translate_inner(addr, offset)
}

pub fn translate_inner(addr: VirtAddr, offset: VirtAddr) -> Option<PhysAddr> {
    use x86_64::{registers::control::Cr3, structures::paging::page_table::FrameError};
    let (l4, _) = Cr3::read();
    let table_indexes = [
        addr.p4_index(),
        addr.p3_index(),
        addr.p2_index(),
        addr.p1_index(),
    ];
    let mut frame = l4;
    for &idx in &table_indexes {
        let table = unsafe { get_table(&frame, offset) };
        let entry = &table[idx];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("Huge frames not supported"),
        }
    }
    Some(frame.start_address() + u64::from(addr.page_offset()))
}

pub unsafe fn get_table_mut(frame: &PhysFrame, offset: VirtAddr) -> &'static mut PageTable {
    let p = frame.start_address();
    let v = offset + p.as_u64();
    &mut *(v.as_mut_ptr())
}

pub unsafe fn get_table(frame: &PhysFrame, offset: VirtAddr) -> &'static PageTable {
    let p = frame.start_address();
    let v = offset + p.as_u64();
    &*(v.as_ptr() as *const PageTable)
}


pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    // FIXME: ONLY FOR TEMPORARY TESTING
    let unused_frame = unsafe { UnusedPhysFrame::new(frame) };
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = mapper.map_to(page, unused_frame, flags, frame_allocator);
    map_to_result.expect("map_to failed").flush();
}