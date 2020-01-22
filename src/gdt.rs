use x86_64::{
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
        tss::TaskStateSegment,
    },
    VirtAddr,
};
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub const SIZE: usize = 4096;
lazy_static::lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            static mut STACK: [u8; SIZE] = [0; SIZE];
            let start = VirtAddr::from_ptr(unsafe { &STACK });
            let end = start + SIZE;
            end
        };
        tss
    };
}

struct Gdt {
    table: GlobalDescriptorTable,
    selectors: Selectors,
}
impl Gdt {
    pub fn new(table: GlobalDescriptorTable, selectors: Selectors) -> Self {
        Self { table, selectors }
    }

    pub fn load(&'static self) {
        self.table.load();
    }
}

struct Selectors {
    code: SegmentSelector,
    tss: SegmentSelector,
}

impl Selectors {
    pub fn new(code: SegmentSelector, tss: SegmentSelector) -> Self {
        Self { code, tss }
    }
}

lazy_static::lazy_static! {
    static ref GDT: Gdt = {
        let mut g = GlobalDescriptorTable::new();
        let code = g.add_entry(Descriptor::kernel_code_segment());
        let tss = g.add_entry(Descriptor::tss_segment(&TSS));
        let s = Selectors::new(code, tss);
        Gdt::new(g, s)
    };
}
pub fn init() {
    use x86_64::instructions::{segmentation::set_cs, tables::load_tss};
    GDT.load();
    unsafe {
        set_cs(GDT.selectors.code);
        load_tss(GDT.selectors.tss);
    }
}
