use alloc::alloc::{
    GlobalAlloc,
    Layout,
};
use crate::error::Error;
use spin::{Mutex, MutexGuard};
use x86_64::{
    structures::paging::{
        FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

pub mod slab;
use slab::Slabber;

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024;

pub trait Alloc {
    fn alloc(&mut self, layout: Layout) -> *mut u8;
    fn dealloc(&mut self, _ptr: *mut u8, _layout: Layout);
}

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
static ALLOCATOR: Locked<Slabber> = Locked::new(Slabber::new());
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

unsafe impl<I: Alloc> GlobalAlloc for Locked<I> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut me = self.lock();
        me.alloc(layout)

    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut me = self.lock();
        me.dealloc(ptr, layout);
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