#![no_main]
#![no_std]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "tmain"]

use core::panic::PanicInfo;

mod serial;
mod vga_buffer;

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
fn test_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("{}", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for f in tests {
        f()
    }
    exit_qemu(QemuExitCode::Success);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}
