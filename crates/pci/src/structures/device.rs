use super::{
    location::Location,
    register::{bar::BaseAddressRegister, command::CommandRegister, status::StatusRegister},
};

#[derive(Debug)]
pub enum Device {
    General(GeneralDevice),
    PciToPci(PciToPciDevice),
}

#[derive(Debug)]
pub struct CommonHeaders {
    pub location: Location,
    pub vendor_id: u16,
    pub device_id: u16,
    pub command: CommandRegister,
    pub status: StatusRegister,
    pub revision_id: u8,
    pub prog_if: u8,
    pub subclass: u8,
    pub class_code: u8,
    pub cache_line_size: u8,
    pub latency_timer: u8,
    pub header_type: u8,
    pub bist: u8,
}

#[derive(Debug)]
pub struct GeneralDevice {
    pub common: CommonHeaders,
    pub bars: [BaseAddressRegister; 6],
    pub cardbus_cis_pointer: u32,
    pub subsystem_id: u16,
    pub subsystem_vendor_id: u16,
    pub expansion_rom_base_address: u32,
    pub capabilities_pointer: u8,
    pub max_latency: u8,
    pub min_grant: u8,
    pub interrupt_pin: u8,
    pub interrupt_line: u8,
}

#[derive(Debug)]
pub struct PciToPciDevice {
    pub common: CommonHeaders,
    pub bars: [BaseAddressRegister; 2],
    pub primary_bus_number: u8,
    pub secondary_bus_number: u8,
    pub subordinate_bus_number: u8,
    pub secondary_latency_timer: u8,
    pub io_base: u8,
    pub io_limit: u8,
    pub secondary_status: u16,
    pub memory_base: u16,
    pub memory_limit: u16,
    pub prefetchable_memory_base: u16,
    pub prefetchable_memory_limit: u16,
    pub prefetchable_base_upper: u32,
    pub prefetchable_limit_upper: u32,
    pub io_base_upper: u16,
    pub io_limit_upper: u16,
    pub capability_pointer: u8,
    pub expansion_rom_base_address: u32,
    pub interrupt_line: u8,
    pub interrupt_pin: u8,
    pub bridge_control: u16,
}
