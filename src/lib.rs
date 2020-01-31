#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]
#![feature(const_in_array_repeat_expressions)]
#![feature(const_fn)]

extern crate alloc;

pub mod allocator;
pub mod error;
pub mod gdt;
pub mod interupt;
pub mod memory;
pub mod serial;
pub mod vga_buffer;

use core::panic::PanicInfo;

/// Initialize the kernel
/// for normal operation
pub fn init() {
    gdt::init();
    interupt::init_idt();
    interupt::init_pics();
    x86_64::instructions::interrupts::enable();
}

#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();
    test_main();
    hlt_loop()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn test_panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("{}", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop()
}

#[cfg(test)]
#[panic_handler]
fn test_handler(info: &PanicInfo) -> ! {
    test_panic(info)
}

pub fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for f in tests {
        f()
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

/// Call the `hlt` instruction in
/// a tight loop. This will sleep
/// the processor until the next
/// interrupt arrives
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}


#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout)  -> ! {
    panic!("allocation error: {:?}", layout);
}