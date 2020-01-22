#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(os::test_runner)]
use kern_test::kern_test;
use os::{println, serial_print, serial_println};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();

    loop {}
}

use core::panic::PanicInfo;
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::test_panic(info)
}
#[kern_test]
fn test_println() {
    println!("integration println");
}
