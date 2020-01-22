#![no_main]
#![no_std]
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "tmain"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use os::println;
use x86_64::{structures::paging::{MapperAllSizes, Page}, VirtAddr};

#[cfg(not(test))]
entry_point!(rmain);
#[cfg(not(test))]
fn rmain(boot_info: &'static BootInfo) -> ! {
    os::init();
    println!("Hello, World!");
    let offset = x86_64::VirtAddr::new(boot_info.physical_memory_offset);
    
    let addrs = [
        0xb8000,
        0x201008,
        0x200_0020_1a10,
        boot_info.physical_memory_offset,
    ];
    let mut m = unsafe { os::memory::init(offset) };
    let page = Page::containing_address(VirtAddr::new(0));
    let mut frame_allocator = unsafe {
        os::memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    };
    os::memory::create_example_mapping(page, &mut m, &mut frame_allocator);
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e)};
    println!("It did not crash!");
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
