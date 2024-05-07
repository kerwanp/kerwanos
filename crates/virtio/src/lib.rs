#![no_std]

#[derive(Clone)]
#[repr(C)]
struct VirtioPciCap {
    vndr: u8,
    next: u8,
    cfg_type: u8,
    bar: u8,
    padding1: [u8; 3],
    offset: u32,
    length: u32,
}

impl VirtioPciCap {}
