#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use lazy_static::lazy_static;
use x86_64::structures::idt::InterruptDescriptorTable;
use x86_64::structures::idt::InterruptStackFrame;
use os::{exit_qemu, QemuExitCode, serial_println};

lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault.set_handler_fn(test_double_fault_handler).set_stack_index(os::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}

extern "x86-interrupt" fn test_double_fault_handler(_stack_frame: InterruptStackFrame, _error_code: u64, ) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);

    os::hlt_loop();
}

pub fn init_test_idt() {
    TEST_IDT.load();
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    os::gdt::init();
    init_test_idt();

    // trigger a stack overflow
    stack_overflow();

    panic!("Execution continued after stack overflow");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::test_panic_handler(info)
}

#[derive(Copy, Clone)]
struct MyBox {}
impl core::ops::Deref for MyBox {
    type Target = MyBox;
    fn deref(&self) -> &Self::Target { self }
}
impl core::ops::DerefMut for MyBox {
    fn deref_mut(&mut self) -> &mut Self::Target { self }
}
#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow(); // for each recursion, the return address is pushed
    volatile::Volatile::new(MyBox {}).read(); // prevent tail recursion optimizations
}