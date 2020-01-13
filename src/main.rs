#![no_main]
#![no_std]
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "tmain"]

use core::panic::PanicInfo;
use os::println;

#[no_mangle]
extern "C" fn _start() -> ! {
    #[cfg(not(test))]
    rmain();
    #[cfg(test)]
    tmain();

    loop {}
}
#[cfg(not(test))]
fn rmain() {
    panic!("Hello worl{}", "d");
}
#[cfg(not(test))]
#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::test_panic(info)
}

