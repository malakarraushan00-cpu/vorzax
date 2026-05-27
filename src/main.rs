#![no_std]
#![feature(asm_const)]
#![feature(const_fn_floating_point_arithmetic)]

use vorzax::init;

/// ARM64 Kernel Entry Point
/// Called by bootloader after firmware initialization
#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();
    kernel_main();
}

#[no_mangle]
fn kernel_main() -> ! {
    // Initialize kernel subsystems
    vorzax::kernel::memory::init();
    vorzax::kernel::interrupt::setup_exception_vectors();
    vorzax::kernel::scheduler::init();
    
    // Initialize drivers
    vorzax::driver::init();
    let _ = vorzax::kernel::user::load_or_create_default();
    
    // Initialize GUI and show the default About page.
    vorzax::gui::init();
    let _about = vorzax::gui::about::launch();
    let _tabs = vorzax::kernel::process::open_100_tabs(tab_process_entry as usize as u64);
    
    // Main event loop
    loop {
        vorzax::kernel::scheduler::switch_context();
    }
}

extern "C" fn tab_process_entry() -> ! {
    loop {
        vorzax::kernel::scheduler::yield_process();
    }
}
