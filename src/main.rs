#![no_main]
#![no_std]
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "tmain"]
extern crate alloc;
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use os::println;

use alloc::{boxed::Box, rc::Rc};

#[cfg(not(test))]
entry_point!(rmain);
#[cfg(not(test))]
fn rmain(boot_info: &'static BootInfo) -> ! {
    os::init();
    println!("Hello, World!");
    let offset = x86_64::VirtAddr::new(boot_info.physical_memory_offset);
    let mut m = unsafe { os::memory::init(offset) };
    let mut frame_allocator = unsafe {
        os::memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    };
    os::allocator::init_heap(&mut m, &mut frame_allocator).expect("failed to create heap");
    let x = Box::new(41);
    let y = Rc::new(100);
    {
        let q = y.clone();
        println!("ref count: {}", Rc::strong_count(&y));
    }
    println!("ref count 2: {}", Rc::strong_count(&y));
    println!("It did not crash!: {:?}", x);
    os::hlt_loop()
}

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    println!("{}", info);
    os::hlt_loop()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::test_panic(info)
}

#[cfg(test)]
entry_point!(t_main);
#[cfg(test)]
fn t_main(_boot_info: &'static BootInfo) -> ! {
    tmain();
    os::hlt_loop()
}
