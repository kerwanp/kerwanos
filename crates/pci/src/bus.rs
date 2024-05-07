use bit_field::BitField;

use crate::{
    access::{CSpaceAccess, CSpaceAccessMethod, IoCSpaceAccessMethod},
    structures::{
        device::{CommonHeaders, Device, GeneralDevice, PciToPciDevice},
        location::Location,
        register::{bar::BaseAddressRegister, command::CommandRegister, status::StatusRegister},
    },
};

pub struct BusScan {
    sam: CSpaceAccessMethod,
    bus: u8,
    device: u8,
}

impl BusScan {
    pub fn new(sam: CSpaceAccessMethod) -> Self {
        Self {
            sam,
            bus: 0,
            device: 0,
        }
    }

    fn done(&self) -> bool {
        self.bus == 255 && self.device == 31
    }

    fn increment(&mut self) {
        if self.device == 31 {
            // If we have read all devices of bus we scan the next bus
            self.device = 0;
            self.bus += 1;
        } else {
            self.device += 1;
        }
    }

    fn scan_device(&self) -> Option<Device> {
        // serial::println!("Scanning {} {}", self.bus, self.device);
        match self.sam {
            CSpaceAccessMethod::Io => {
                let location = Location::new(self.bus, self.device, 0);
                let sam = IoCSpaceAccessMethod::new(location.clone());
                let (vendor_id, device_id) = sam.read16(0x0);
                if vendor_id == 0xFFFF {
                    return None;
                }

                let (command, status) = sam.read16(0x4);
                let (revision_id, prog_if, subclass, class_code) = sam.read8(0x8);
                let (cache_line_size, latency_timer, header_type, bist) = sam.read8(0xC);
                let common = CommonHeaders {
                    location,
                    vendor_id,
                    device_id,
                    command: CommandRegister::from_bits_truncate(command),
                    status: StatusRegister::from_bits_truncate(status),
                    class_code,
                    subclass,
                    prog_if,
                    revision_id,
                    bist,
                    header_type,
                    latency_timer,
                    cache_line_size,
                };

                serial::println!("Status: {:#b}", status);

                match common.header_type {
                    0x0 => {
                        let bars = [
                            BaseAddressRegister::new(sam.read(0x10)),
                            BaseAddressRegister::new(sam.read(0x14)),
                            BaseAddressRegister::new(sam.read(0x18)),
                            BaseAddressRegister::new(sam.read(0x1C)),
                            BaseAddressRegister::new(sam.read(0x20)),
                            BaseAddressRegister::new(sam.read(0x24)),
                        ];

                        let cardbus_cis_pointer = sam.read(0x28);
                        let (subsystem_vendor_id, subsystem_id) = sam.read16(0x2C);
                        let expansion_rom_base_address = sam.read(0x30);
                        let (capabilities_pointer, _, _, _) = sam.read8(0x34);
                        let (max_latency, min_grant, interrupt_pin, interrupt_line) =
                            sam.read8(0x3C);

                        Some(Device::General(GeneralDevice {
                            common,
                            bars,
                            cardbus_cis_pointer,
                            subsystem_vendor_id,
                            subsystem_id,
                            expansion_rom_base_address,
                            capabilities_pointer,
                            max_latency,
                            min_grant,
                            interrupt_pin,
                            interrupt_line,
                        }))
                    }
                    0x1 => {
                        let bars = [
                            BaseAddressRegister::new(sam.read(0x10)),
                            BaseAddressRegister::new(sam.read(0x14)),
                        ];

                        let (
                            primary_bus_number,
                            secondary_bus_number,
                            subordinate_bus_number,
                            secondary_latency_timer,
                        ) = sam.read8(0x18);
                        let (io_base, io_limit, secondary_status_2, secondary_status_1) =
                            sam.read8(0x1C);
                        let secondary_status =
                            (secondary_status_1 as u16) << 8 & (secondary_status_2 as u16);
                        let (memory_base, memory_limit) = sam.read16(0x20);
                        let (prefetchable_memory_base, prefetchable_memory_limit) =
                            sam.read16(0x24);
                        let prefetchable_base_upper = sam.read(0x28);
                        let prefetchable_limit_upper = sam.read(0x2C);
                        let (io_base_upper, io_limit_upper) = sam.read16(0x30);
                        let (capability_pointer, _, _, _) = sam.read8(0x34);
                        let expansion_rom_base_address = sam.read(0x38);
                        let (interrupt_line, interrupt_pin, bridge_control_2, bridge_control_1) =
                            sam.read8(0x3C);
                        let bridge_control =
                            (bridge_control_1 as u16) << 8 & (bridge_control_2 as u16);

                        Some(Device::PciToPci(PciToPciDevice {
                            common,
                            bars,
                            primary_bus_number,
                            secondary_bus_number,
                            subordinate_bus_number,
                            secondary_latency_timer,
                            io_base,
                            io_limit,
                            secondary_status,
                            memory_base,
                            memory_limit,
                            prefetchable_memory_base,
                            prefetchable_memory_limit,
                            prefetchable_base_upper,
                            prefetchable_limit_upper,
                            io_base_upper,
                            io_limit_upper,
                            capability_pointer,
                            expansion_rom_base_address,
                            interrupt_line,
                            interrupt_pin,
                            bridge_control,
                        }))
                    }
                    _ => None, // h => panic!("Header type {:#x} not handled", h),
                }
            }
        }
    }
}

impl Iterator for BusScan {
    type Item = Device;

    fn next(&mut self) -> Option<Self::Item> {
        let mut ret = None;
        loop {
            if self.done() {
                return ret;
            }

            ret = self.scan_device();
            self.increment();

            if ret.is_some() {
                return ret;
            }
        }
    }
}
