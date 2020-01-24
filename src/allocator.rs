use alloc::alloc::{
    GlobalAlloc,
    Layout,
};
use core::ptr::null_mut;
use crate::error::Error;
use spin::{Mutex, MutexGuard};
use x86_64::{
    structures::paging::{
        FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

pub mod bump;
use bump::Bumper;

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024;

pub struct Locked<A>(Mutex<A>);

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Self(Mutex::new(inner))
    }
    pub fn lock(&self) -> MutexGuard<A> {
        self.0.lock()
    }
}

#[global_allocator]
static ALLOCATOR: Locked<Bumper> = Locked::new(Bumper::new());
pub fn init_heap(mapper: &mut impl Mapper<Size4KiB>, frame_alloc: &mut impl FrameAllocator<Size4KiB>) -> Result<(), Error> {
    let pages = {
        let start = VirtAddr::new(HEAP_START as u64);
        let end = start + HEAP_SIZE - 1u64;
        let start = Page::containing_address(start);
        let end = Page::containing_address(end);
        Page::range_inclusive(start, end)
    };
    for page in pages {
        let frame = frame_alloc
            .allocate_frame()
            .ok_or(Error::OutOfFrames)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        mapper.map_to(page, frame, flags, frame_alloc)?.flush();
    }
    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }
    Ok(())
}

pub struct DumbAlloc;

unsafe impl GlobalAlloc for DumbAlloc {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        unreachable!("dealloc should never be called")
    }
}

pub fn align(addr: usize, align: usize) -> usize {
    let rem = addr % align;
    if rem == 0 {
        addr
    } else {
        addr - rem + align
    }
}