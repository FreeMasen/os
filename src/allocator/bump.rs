use alloc::alloc::Layout;


use super::{align, Alloc};

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

impl Alloc for Bumper {
    fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let start = align(self.next, layout.align());
        let end = start + layout.size();
        if end > self.heap_end {
            core::ptr::null_mut()
        } else {
            self.next = end;
            self.allocs += 1;
            start as _
        }
    }
    fn dealloc(&mut self, _ptr: *mut u8, _layout: Layout) {
        self.allocs -= 1;
        if self.allocs == 0 {
            self.next = self.heap_start;
        }
    }
}

