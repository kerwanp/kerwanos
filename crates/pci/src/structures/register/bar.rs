use bit_field::BitField;

#[derive(Debug, Clone, Copy)]
pub enum BaseAddressRegister {
    Memory(MemoryBaseAddressRegister),
    IoSpace(IoSpaceBaseAddressRegister),
}

impl BaseAddressRegister {
    pub fn new(bits: u32) -> Self {
        match Self::type_bit(bits) {
            0 => Self::Memory(MemoryBaseAddressRegister::new(bits)),
            1 => Self::IoSpace(IoSpaceBaseAddressRegister::new(bits)),
            _ => panic!("This should never happen"),
        }
    }

    pub fn address(&self) -> u32 {
        match self {
            Self::Memory(bar) => bar.address(),
            Self::IoSpace(bar) => bar.address(),
        }
    }

    pub fn type_bit(bits: u32) -> u8 {
        (bits & 0b1) as u8
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct MemoryBaseAddressRegister(u32);

#[derive(Debug)]
pub enum MemoryBaseAddressSize {
    Bits32,
    Bits64,
}

impl MemoryBaseAddressRegister {
    pub fn new(bits: u32) -> Self {
        Self(bits)
    }

    pub fn bits(&self) -> u32 {
        self.0
    }

    pub fn size(&self) -> MemoryBaseAddressSize {
        let bits = self.0.get_bits(1..3);
        match bits {
            0x0 => MemoryBaseAddressSize::Bits32,
            0x2 => MemoryBaseAddressSize::Bits64,
            b => panic!("Memory BAR size {:#x} not supported", b),
        }
    }

    pub fn address(&self) -> u32 {
        self.0 >> 4
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IoSpaceBaseAddressRegister(u32);

impl IoSpaceBaseAddressRegister {
    pub fn new(bits: u32) -> Self {
        Self(bits)
    }

    pub fn address(&self) -> u32 {
        self.0 >> 2
    }
}
