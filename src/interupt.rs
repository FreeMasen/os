
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static::lazy_static! {
    pub static ref IDT: InterruptDescriptorTable = {
        let mut i = InterruptDescriptorTable::new();
        i.breakpoint.set_handler_fn(breakpoint);
        unsafe {
            i.double_fault.set_handler_fn(double_fault).set_stack_index(crate::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        i
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint(frame: &mut InterruptStackFrame) {
    crate::println!("EXCEPTION BREAKPOINT");
    crate::println!("{:#?}", frame);
}

extern "x86-interrupt" fn double_fault(frame: &mut InterruptStackFrame, code: u64) -> ! {
    panic!("DOUBLE FAULT ({}):\n{:#?}", code, frame);
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