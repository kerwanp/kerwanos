// The physical location of a device on the bus.
#[derive(Debug, Clone)]
pub struct Location {
    bus: u8,
    device: u8,
    function: u8,
}

impl Location {
    pub fn new(bus: u8, device: u8, function: u8) -> Self {
        Self {
            bus,
            device,
            function,
        }
    }

    pub fn bus(&self) -> u8 {
        self.bus
    }

    pub fn device(&self) -> u8 {
        self.device
    }

    pub fn function(&self) -> u8 {
        self.function
    }
}
