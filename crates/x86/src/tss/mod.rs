use core::mem::size_of;

#[repr(C, packed(4))]
pub struct TaskStateSegment {
    reserved_1: u32,
    pub privilege_stack_table: [u64; 3],
    reserved_2: u64,
    pub interrupt_stack_table: [u64; 7],
    reserved_3: u64,
    reserved_4: u64,
    pub iomap_base: u16,
}

impl Default for TaskStateSegment {
    fn default() -> Self {
        Self {
            reserved_1: Default::default(),
            privilege_stack_table: Default::default(),
            reserved_2: Default::default(),
            interrupt_stack_table: Default::default(),
            reserved_3: Default::default(),
            reserved_4: Default::default(),
            iomap_base: size_of::<TaskStateSegment>() as u16,
        }
    }
}
