use lazy_static::lazy_static;
use pic8259_simple::ChainedPics;
use spin;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
    pub static ref IDT: InterruptDescriptorTable = {
        let mut i = InterruptDescriptorTable::new();
        i.breakpoint.set_handler_fn(breakpoint);
        unsafe {
            i.double_fault
                .set_handler_fn(double_fault)
                .set_stack_index(crate::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        i[InterruptIndex::Timer.as_usize()].set_handler_fn(timer);
        i[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard);
        i
    };
}
pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;
pub const PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

pub fn init_idt() {
    IDT.load();
}

pub fn init_pics() {
    unsafe {
        PICS.lock().initialize();
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }
    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

extern "x86-interrupt" fn breakpoint(frame: &mut InterruptStackFrame) {
    crate::println!("EXCEPTION BREAKPOINT");
    crate::println!("{:#?}", frame);
}

extern "x86-interrupt" fn double_fault(frame: &mut InterruptStackFrame, code: u64) -> ! {
    panic!("DOUBLE FAULT ({}):\n{:#?}", code, frame);
}
extern "x86-interrupt" fn timer(_frame: &mut InterruptStackFrame) {
    crate::print!(".");
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}
extern "x86-interrupt" fn keyboard(_frame: &mut InterruptStackFrame) {
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;
    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(
                layouts::Us104Key,
                ScancodeSet1,
                HandleControl::MapLettersToUnicode
            ));
    }
    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_ev)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_ev) {
            match key {
                DecodedKey::Unicode(ch) => crate::print!("{}", ch),
                DecodedKey::RawKey(ch) => crate::print!("{:?}", ch),
            }
        }
    }
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::*;
    use kern_test::kern_test;
    #[kern_test]
    fn test_breakpoint() {
        x86_64::instructions::interrupts::int3();
    }
}
