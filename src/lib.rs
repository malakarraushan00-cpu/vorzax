#![no_std]
#![feature(asm_const)]
#![feature(const_fn_floating_point_arithmetic)]
#![allow(dead_code)]

pub mod boot;
pub mod kernel;
pub mod driver;
pub mod gui;
pub mod util;

// Panic handler for no_std
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        unsafe {
            asm!("wfe");
        }
    }
}

// Core initialization
pub fn init() {
    boot::init();
    kernel::init();
}
