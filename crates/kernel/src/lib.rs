#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]

pub mod allocator;
pub mod interrupts;
pub mod memory;
pub mod task;

extern crate alloc;

use interrupts::PICS;
use lazy_static::lazy_static;
use vga::{print, println};
use x86::{
    dt::gdt::{Descriptor, GlobalDescriptorTable},
    dt::idt::InterruptDescriptorTable,
    instructions::{self, load_tss},
    segmentation::{SegmentSelector, CS},
    tss::TaskStateSegment,
};

use crate::interrupts::{keyboard_interrupt_handler, InterruptIndex};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::default();
        idt.divide_by_zero.set_handler(divide_by_zero_handler);
        idt.breakpoint.set_handler(breakpoint_handler);
        idt.double_fault
            .set_handler(double_fault_handler)
            .set_stack_index(DOUBLE_FAULT_IST_INDEX);
        idt[InterruptIndex::Timer.as_u8()].set_handler(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_u8()].set_handler(keyboard_interrupt_handler);
        idt
    };
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::default();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = unsafe { (&STACK) as *const _ as u64 };
            stack_start + STACK_SIZE as u64
        };

        tss
    };
    static ref GDT: (GlobalDescriptorTable, SegmentSelector, SegmentSelector) = {
        let mut gdt = GlobalDescriptorTable::default();
        let code_selector = gdt.append(Descriptor::kernel_code_segment());
        let tss_selector = gdt.append(Descriptor::tss_segment(&TSS));
        (gdt, code_selector, tss_selector)
    };
}

pub fn init() {
    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1);
        load_tss(GDT.2);
    }

    IDT.load();
    interrupts::init();
}

pub fn hlt_loop() -> ! {
    loop {
        instructions::hlt();
    }
}

extern "x86-interrupt" fn divide_by_zero_handler() {
    panic!("EXCEPTION: DIVIDE BY ZERO");
}

extern "x86-interrupt" fn breakpoint_handler() {
    println!("EXCEPTION: BREAKPOINT");
}

extern "x86-interrupt" fn double_fault_handler() {
    panic!("EXCEPTION: DOUBLE FAULT");
}

extern "x86-interrupt" fn timer_interrupt_handler() {
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8())
    };
}
