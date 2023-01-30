#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]
#[allow(dead_code)]

use core::panic::PanicInfo;
use os::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    os::init();
    println!("Hello World{}", "!");

    #[cfg(test)]
    test_main();

    os::hlt_loop();
}

/// 这个函数将在panic时被调用
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);

    os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::test_panic_handler(info);
}