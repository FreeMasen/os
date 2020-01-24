use alloc::alloc::Layout;


use super::{align, Locked};

pub struct Bumper {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocs: usize,
}

impl Bumper {
    pub const fn new() -> Self {
        Self {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocs: 0,
        }
    }
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }
}

unsafe impl alloc::alloc::GlobalAlloc for Locked<Bumper> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut b = self.lock();
        let start = align(b.next, layout.align());
        let end = start + layout.size();
        if end > b.heap_end {
            core::ptr::null_mut()
        } else {
            b.next = end;
            b.allocs += 1;
            start as _
        }
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let mut b = self.lock();
        b.allocs -= 1;
        if b.allocs == 0 {
            b.next = b.heap_start;
        }
    }
}

