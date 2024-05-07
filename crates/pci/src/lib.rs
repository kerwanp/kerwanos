#![no_std]

pub mod access;
pub mod bus;
pub mod memory;
mod register;
pub mod structures;

extern crate alloc;

use bus::BusScan;
use structures::device::Device;

pub fn scan_buses(method: access::CSpaceAccessMethod) -> impl Iterator<Item = Device> {
    BusScan::new(method)
}
