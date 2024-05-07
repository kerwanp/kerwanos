#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;
use std::println;

use bootloader::{entry_point, BootInfo};
use kernel::{
    allocator,
    memory::{self, BootInfoFrameAllocator},
    task::{executor::Executor, Task},
    tty::TTY,
};
use lazy_static::lazy_static;
use pci::{
    access::{CSpaceAccess, CSpaceAccessMethod, IoCSpaceAccessMethod},
    structures::{
        device::Device,
        register::{bar::BaseAddressRegister, command::CommandRegister, status::StatusRegister},
    },
};
use shell::Shell;
use spin::Mutex;
use x86::{
    addr::{PhysAddr, VirtAddr},
    structures::paging::{
        frame::PhysFrame,
        mapper::{Mapper, Translate},
        page_table::PageTableFlags,
    },
};

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Initializing Kernel");
    kernel::init();

    println!("Initializing Frame Allocator");
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    println!("Initializing Heap");
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("failed to initialize heap");

    println!("Initializing PCI");
    let devices = pci::scan_buses(CSpaceAccessMethod::Io);

    for device in devices {
        if let Device::General(device) = device {
            let sam = IoCSpaceAccessMethod::new(device.common.location);
            std::println!(
                "pci v:{:#x} d:{:#x} bc:{:#x} sc:{:#x} ht:{:#x}",
                device.common.vendor_id,
                device.common.device_id,
                device.common.class_code,
                device.common.subclass,
                device.common.header_type,
            );

            // if device.common.device_id == 0x1009 {
            //     for (i, bar) in device.bars.iter().enumerate() {
            //         match bar {
            //             BaseAddressRegister::Memory(bar) => {
            //                 let addr = bar.address();
            //                 if addr == 0x0 {
            //                     continue;
            //                 }
            //
            //                 // let size = bar.size();
            //                 // let register = 0x10 + (i as u8) * 4;
            //                 // let phys_addr = PhysAddr::new(bar.address() as u64);
            //                 // sam.write(register, 0xFFFFFFFF);
            //                 // let res = sam.read(register);
            //                 // sam.write(register, bar.bits());
            //                 //
            //                 // let size = !(res & 0xFFFFFF0) + 1;
            //                 //
            //                 // serial::println!("-----");
            //                 // serial::println!("Addr: {:#x}", bar.address());
            //                 // serial::println!("Size: {:#x} {:#x}", res, size);
            //             }
            //             _ => {}
            //         }
            //     }
            // }

            if device.common.device_id == 0x1009 {
                std::println!("{:#b}", device.capabilities_pointer);

                let mut next_ptr = device.capabilities_pointer;
                while next_ptr != 0 {
                    let (capability_id, nptr, length, typ) = sam.read8(next_ptr);
                    let (bar_index, _, _, _) = sam.read8(device.capabilities_pointer + 0x4);
                    let offset = sam.read(device.capabilities_pointer + 0x8);
                    let length = sam.read(device.capabilities_pointer + 0xC);
                    serial::println!("------");
                    serial::println!("Capability {:#x}", capability_id);
                    if capability_id == 0x9 {
                        let bar = device.bars[bar_index as usize];
                        serial::println!("VType {:#x}", typ);
                        serial::println!("Bar {:#x}", bar.address());

                        let phys_addr = VirtAddr::new(bar.address().into());
                        // let page = {};
                        // let frame = PhysFrame::containing_address(phys_addr);
                        // let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
                        // mapper.map_to(page, frame, flags, frame_allocator)

                        let test = mapper.translate_addr(phys_addr);
                        match test {
                            Some(t) => serial::println!("Test: {:#x}", t.as_u64()),
                            None => serial::println!("Not found"),
                        };

                        // let test: &'static mut u32 = unsafe { &mut *(addr as *mut u32) };

                        // serial::println!("Offset {:#x}", length);
                    }

                    next_ptr = nptr;
                }
                // let offset = sam.read(device.capabilities_pointer + 0x4);
                // let (bar, _, _, _) = sam.read8(device.capabilities_pointer + 0x8);
                // let (typ, len, next, vndr) = sam.read8(device.capabilities_pointer + 0xC);
                //
                // std::println!("Length {}", length);
                // std::println!("Offset {:#b}", offset);
                // std::println!("Bar {:#b}", offset);
                // std::println!("Type {}", typ);
                // std::println!("len {}", len);
                // }
            }
        }
    }

    #[cfg(test)]
    test_main();

    let mut executor = Executor::new();
    executor.spawn(Task::new(shell::init()));

    executor.run();
}

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }

    qemu::exit(qemu::QemuExitCode::Success);
}

#[test_case]
fn trivial_assertion() {
    use serial::print;
    print!("Test");
    assert_eq!(1, 1);
}
