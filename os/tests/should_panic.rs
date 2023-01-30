#![no_std]
#![no_main]
// #![feature(custom_test_frameworks, catch_unwind)]
// #![test_runner(test_runner)]
// #![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use os::{QemuExitCode, exit_qemu, serial_print, serial_println};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // test_main();
    should_fail();
    serial_println!("[test did not panic]");
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

// pub fn test_runner(tests: &[&dyn Fn()]) {
//     serial_println!("Running {} tests", tests.len());
//     for test in tests {
//         let is_panic = core::panic::panic_any(|| test()).is_err();
//         serial_println!("[test did not panic], {}", is_panic);
//         exit_qemu(QemuExitCode::Failed);
//     }
//     exit_qemu(QemuExitCode::Success);
// }

// #[test_case]
fn should_fail() {
    serial_print!("should_fail... ");
    assert_eq!(0, 1);
}