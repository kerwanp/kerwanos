use bit_field::BitField;
use x86::instructions::port::Port;

use crate::structures::location::Location;

const CONFIG_ADDRESS: u16 = 0xCF8;
const CONFIG_DATA: u16 = 0x0CFC;

#[derive(Debug)]
pub enum CSpaceAccessMethod {
    Io,
}

pub trait CSpaceAccess {
    fn read8(&self, register: u8) -> (u8, u8, u8, u8);
    fn read16(&self, register: u8) -> (u16, u16);
    fn read(&self, register: u8) -> u32;

    fn write(&self, register: u8, value: u32);
}

pub struct IoCSpaceAccessMethod {
    location: Location,
}

impl IoCSpaceAccessMethod {
    pub fn new(location: Location) -> Self {
        Self { location }
    }

    pub fn address(&self, offset: u8) -> u32 {
        let mut addr: u32 = 0;
        addr.set_bits(0..8, u32::from(offset))
            .set_bits(8..11, u32::from(self.location.function()))
            .set_bits(11..16, u32::from(self.location.device()))
            .set_bits(16..24, u32::from(self.location.bus()))
            .set_bit(31, true);
        addr
    }
}

impl CSpaceAccess for IoCSpaceAccessMethod {
    fn read8(&self, register: u8) -> (u8, u8, u8, u8) {
        let bits = self.read(register);

        (
            bits.get_bits(0..8) as u8,
            bits.get_bits(8..16) as u8,
            bits.get_bits(16..24) as u8,
            bits.get_bits(24..32) as u8,
        )
    }

    fn read16(&self, offset: u8) -> (u16, u16) {
        let bits = self.read(offset);

        (bits.get_bits(0..16) as u16, bits.get_bits(16..32) as u16)
    }

    fn read(&self, offset: u8) -> u32 {
        let addr = self.address(offset);
        let mut address_port = Port::new(CONFIG_ADDRESS);
        unsafe { address_port.write(addr) };

        let mut data_port = Port::new(CONFIG_DATA);
        unsafe { data_port.read() }
    }

    fn write(&self, offset: u8, value: u32) {
        let addr = self.address(offset);
        let mut address_port = Port::new(CONFIG_ADDRESS);
        unsafe { address_port.write(addr) };

        let mut data_port = Port::new(CONFIG_DATA);
        unsafe { data_port.write(value) }
    }
}
