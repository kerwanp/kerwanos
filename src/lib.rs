#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[cfg(test)]
use bootloader::{entry_point, BootInfo};
#[cfg(test)]
use core::panic::PanicInfo;

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    serial::println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }

    qemu::exit(qemu::QemuExitCode::Success);
}

#[cfg(test)]
entry_point!(test_kernel_main);

#[cfg(test)]
pub fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    kernel::init();
    test_main();
    kernel::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    kernel::hlt_loop();
}

#[test_case]
fn trivial_assertion() {
    use vga::print;

    print!("Test");
    assert_eq!(1, 1);
}
