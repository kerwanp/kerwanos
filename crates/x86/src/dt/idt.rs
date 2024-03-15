use core::{
    arch::asm,
    ops::{Index, IndexMut},
};

use crate::segmentation::{self, SegmentSelector};
use bit_field::BitField;

use super::DescriptorTablePointer;

type HandlerFunc = extern "x86-interrupt" fn();

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct EntryOptions(u16);

impl EntryOptions {
    pub fn set_present(&mut self, present: bool) {
        self.0.set_bit(15, present);
    }

    pub fn set_stack_index(&mut self, index: u16) -> &mut Self {
        self.0.set_bits(0..3, index + 1);
        self
    }
}

impl Default for EntryOptions {
    fn default() -> Self {
        Self(0b1110_0000_0000)
    }
}

#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct Entry {
    pointer_low: u16,
    segment_selector: SegmentSelector,
    options: EntryOptions,
    pointer_middle: u16,
    pointer_high: u32,
    reserved: u32,
}

impl Entry {
    fn set_handler_addr(&mut self, addr: u64) -> &mut EntryOptions {
        self.pointer_low = addr as u16;
        self.pointer_middle = (addr >> 16) as u16;
        self.pointer_high = (addr >> 32) as u32;
        self.segment_selector = segmentation::CS::get_reg();
        self.options.set_present(true);
        &mut self.options
    }

    pub fn set_handler(&mut self, f: HandlerFunc) -> &mut EntryOptions {
        self.set_handler_addr(f as u64)
    }
}

#[repr(C)]
pub struct InterruptDescriptorTable {
    pub divide_by_zero: Entry,
    pub debug: Entry,
    pub non_maskable_interrupt: Entry,
    pub breakpoint: Entry,
    pub overflow: Entry,
    pub bound_range_exceeded: Entry,
    pub invalid_opcode: Entry,
    pub device_not_available: Entry,
    pub double_fault: Entry,
    pub coprocessor_segment_overrun: Entry,
    pub invalid_tss: Entry,
    pub segment_not_present: Entry,
    pub stack_segment_fault: Entry,
    pub general_protection_fault: Entry,
    pub page_fault: Entry,
    reserved1: Entry,
    pub x87_floating_point: Entry,
    pub alignment_check: Entry,
    pub machine_check: Entry,
    pub simd_floating_point: Entry,
    pub virtualization: Entry,
    pub cp_protection_exception: Entry,
    reserved2: [Entry; 6],
    pub hv_injection_exception: Entry,
    pub vmm_communication_exception: Entry,
    pub security_exception: Entry,
    reserved3: Entry,
    interrupts: [Entry; 256 - 32],
}

impl InterruptDescriptorTable {
    pub fn pointer(&self) -> DescriptorTablePointer {
        use core::mem::size_of;
        // TODO: Add new func to DTPtr
        DescriptorTablePointer {
            base: self as *const _ as u64,
            limit: (size_of::<Self>() - 1) as u16,
        }
    }

    pub fn load(&self) {
        unsafe {
            asm!("lidt [{}]", in(reg) &self.pointer(), options(readonly, nostack, preserves_flags) )
        }
    }
}

impl Default for InterruptDescriptorTable {
    fn default() -> Self {
        Self {
            divide_by_zero: Entry::default(),
            debug: Entry::default(),
            non_maskable_interrupt: Entry::default(),
            breakpoint: Entry::default(),
            overflow: Entry::default(),
            bound_range_exceeded: Entry::default(),
            invalid_opcode: Entry::default(),
            device_not_available: Entry::default(),
            double_fault: Entry::default(),
            coprocessor_segment_overrun: Entry::default(),
            invalid_tss: Entry::default(),
            segment_not_present: Entry::default(),
            stack_segment_fault: Entry::default(),
            general_protection_fault: Entry::default(),
            page_fault: Entry::default(),
            reserved1: Entry::default(),
            x87_floating_point: Entry::default(),
            alignment_check: Entry::default(),
            machine_check: Entry::default(),
            simd_floating_point: Entry::default(),
            virtualization: Entry::default(),
            cp_protection_exception: Entry::default(),
            reserved2: [Entry::default(); 6],
            hv_injection_exception: Entry::default(),
            vmm_communication_exception: Entry::default(),
            security_exception: Entry::default(),
            reserved3: Entry::default(),
            interrupts: [Entry::default(); 256 - 32],
        }
    }
}

impl Index<u8> for InterruptDescriptorTable {
    type Output = Entry;

    fn index(&self, index: u8) -> &Self::Output {
        match index {
            0 => &self.divide_by_zero,
            1 => &self.debug,
            2 => &self.non_maskable_interrupt,
            3 => &self.breakpoint,
            4 => &self.overflow,
            5 => &self.bound_range_exceeded,
            6 => &self.invalid_opcode,
            7 => &self.device_not_available,
            9 => &self.coprocessor_segment_overrun,
            16 => &self.x87_floating_point,
            19 => &self.simd_floating_point,
            20 => &self.virtualization,
            28 => &self.hv_injection_exception,
            i @ 32..=255 => &self.interrupts[usize::from(i) - 32],
            i @ 15 | i @ 31 | i @ 22..=27 => panic!("entry {} is reserved", i),
            i @ 8 | i @ 10..=14 | i @ 17 | i @ 21 | i @ 29 | i @ 30 => {
                panic!("entry {} is an exception with error code", i)
            }
            i @ 18 => panic!("entry {} is a diverging exception (must not return)", i),
        }
    }
}

impl IndexMut<u8> for InterruptDescriptorTable {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        match index {
            0 => &mut self.divide_by_zero,
            1 => &mut self.debug,
            2 => &mut self.non_maskable_interrupt,
            3 => &mut self.breakpoint,
            4 => &mut self.overflow,
            5 => &mut self.bound_range_exceeded,
            6 => &mut self.invalid_opcode,
            7 => &mut self.device_not_available,
            9 => &mut self.coprocessor_segment_overrun,
            16 => &mut self.x87_floating_point,
            19 => &mut self.simd_floating_point,
            20 => &mut self.virtualization,
            28 => &mut self.hv_injection_exception,
            i @ 32..=255 => &mut self.interrupts[usize::from(i) - 32],
            i @ 15 | i @ 31 | i @ 22..=27 => panic!("entry {} is reserved", i),
            i @ 8 | i @ 10..=14 | i @ 17 | i @ 21 | i @ 29 | i @ 30 => {
                panic!("entry {} is an exception with error code", i)
            }
            i @ 18 => panic!("entry {} is a diverging exception (must not return)", i),
        }
    }
}
