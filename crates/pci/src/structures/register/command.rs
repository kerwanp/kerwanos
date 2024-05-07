use bitflags::bitflags;

bitflags! {
    #[derive(Debug)]
    pub struct CommandRegister: u16 {
        const IO_SPACE = 1;
        const MEMORY_SPACE = 1 << 1;
        const BUS_MASTER = 1 << 2;
        const SPECIAL_CYCLES = 1 << 3;
        const MEMORY_WRITE_AND_INVALIDATE_ENABLE = 1 << 4;
        const VGA_PALETTE_SNOOP = 1 << 5;
        const PARITY_ERROR_RESPONSE = 1 << 6;
        const SERR_ENABLE = 1 << 8;
        const FAST_BACK_TO_BACK_ENABLE = 1 << 9;
        const INTERRUPT_DISABLE = 1 << 10;
    }
}
