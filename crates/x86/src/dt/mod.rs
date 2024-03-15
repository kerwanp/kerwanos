pub mod gdt;
pub mod idt;

#[repr(u8)]
pub enum DescriptorTable {
    Gdt = 0, // Use the GDT table
    Idt = 1, // Use the IDT table
    Ldt = 3, // TODO: Check if this value is correct
}

#[repr(C, packed(2))]
pub struct DescriptorTablePointer {
    limit: u16,
    base: u64,
}
