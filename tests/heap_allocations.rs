
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]


#[cfg(test)]
mod tests {
    use os::{serial_print, serial_println};
    use alloc::{boxed::Box, vec::Vec};
    use kern_test::kern_test;
    #[kern_test]
    fn simple_alloc() {
        let v = Box::new(42);
        assert_eq!(*v, 42);
    }
    #[kern_test]
    fn large_vec() {
        let mut v = Vec::new();
        let n = 1000;
        for i in 0..n {
            v.push(i);
        }
        assert_eq!(v.iter().sum::<u64>(), (n - 1) * n / 2);
    }
    #[kern_test]
    fn reuse_mem() {
        for i in 0..os::allocator::HEAP_SIZE {
            let x = Box::new(i);
            assert_eq!(*x, i);
        }
    }
}


extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

entry_point!(main);

fn main(info: &'static BootInfo) -> ! {
    use os::{allocator, memory::{self, BootInfoFrameAllocator}};
    use x86_64::VirtAddr;
    os::init();
    let offset = info.physical_memory_offset;
    let mut mapper = unsafe {
        memory::init(VirtAddr::new(offset))
    };
    let mut frame_alloc = unsafe {
        BootInfoFrameAllocator::init(&info.memory_map)
    };
    allocator::init_heap(&mut mapper, &mut frame_alloc)
        .expect("heap init failed");
    test_main();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::test_panic(info)
}
