use x86::instructions::port::Port;

pub fn read_register(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    let addr = (bus as u32) << 16
        | (device as u32) << 11
        | (function as u32) << 8
        | (offset & 0xFC) as u32
        | 0x80000000;

    let mut address_port = Port::new(0xCF8);
    unsafe { address_port.write(addr) };

    let mut data_port = Port::new(0xCFC);
    unsafe { data_port.read() }
}

pub fn read_16b_register(bus: u8, device: u8, function: u8, offset: u8) -> (u16, u16) {
    let bits = read_register(bus, device, function, offset);
    ((bits & 0xFFFF) as u16, (bits >> 16) as u16)
}

pub fn read_8b_register(bus: u8, device: u8, function: u8, offset: u8) -> (u8, u8, u8, u8) {
    let bits = read_register(bus, device, function, offset);

    (
        (bits >> 24) as u8,
        (bits >> 16) as u8,
        (bits >> 8) as u8,
        bits as u8,
    )
}

pub fn read_address_space_amount(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    let addr = (bus as u32) << 16
        | (device as u32) << 11
        | (function as u32) << 8
        | (offset & 0xFC) as u32
        | 0x80000000;

    let mut address_port = Port::new(0xCF8);
    unsafe { address_port.write(addr) };

    // Get original BAR value
    let mut data_port = Port::new(0xCFC);
    let original: u32 = unsafe { data_port.read() };

    // Write only 1
    unsafe { data_port.write(0xFFFFFFFF) }

    // Get result
    let res: u32 = unsafe { data_port.read() };

    // Write back original value
    unsafe { data_port.write(original) };

    !(res & 0xFFFF_FFF0).checked_add(1).unwrap_or(0)
}
