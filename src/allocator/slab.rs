
use super::Alloc;
use alloc::alloc::Layout;
const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

pub struct Cut {
    next: Option<&'static mut Cut>,
}
pub struct Slabber {
    slabs: [
        Option<&'static mut Cut>; BLOCK_SIZES.len()
    ],
    fallback: linked_list_allocator::Heap,
}

impl Slabber {
    pub const fn new() -> Self {
        Self {
            slabs: [None; BLOCK_SIZES.len()],
            fallback: linked_list_allocator::Heap::empty(),
        }
    }
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.fallback.init(heap_start, heap_size);
    }

    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        match self.fallback.allocate_first_fit(layout) {
            Ok(p) => p.as_ptr(),
            Err(_) => core::ptr::null_mut(),
        }
    }
}

impl Alloc for Slabber {
    fn alloc(&mut self, mut layout: Layout) -> *mut u8 {
        if let Some(idx) = slab_index(&layout) {
            if let Some(cut) = self.slabs[idx].take() {
                self.slabs[idx] = cut.next.take();
                return cut as *mut Cut as *mut u8;
            } else {
                let size = BLOCK_SIZES[idx];
                let align = size;
                layout = Layout::from_size_align(size, align).unwrap();
            }
        }
        self.fallback_alloc(layout)
    }
    fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        if let Some(idx) = slab_index(&layout) {
            let cut = Cut {
                next: self.slabs[idx].take()
            };
            assert!(core::mem::size_of::<Cut>() <= BLOCK_SIZES[idx]);
            assert!(core::mem::align_of::<Cut>() <= BLOCK_SIZES[idx]);
            let cut_ptr = ptr as *mut Cut;
            unsafe {
                cut_ptr.write(cut);
                self.slabs[idx] = Some(&mut *cut_ptr);
            }
        } else {
            let ptr = core::ptr::NonNull::new(ptr).unwrap();
            unsafe {
                self.fallback.deallocate(ptr, layout);
            }
        }
    }
}

fn slab_index(layout: &Layout) -> Option<usize> {
    let size = layout.size().max(layout.align());
    BLOCK_SIZES.iter().position(|&s| s >= size)
}