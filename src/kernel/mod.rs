/// Kernel core modules
pub mod process;
pub mod memory;
pub mod interrupt;
pub mod scheduler;
pub mod user;

pub fn init() {
    memory::init();
    process::init();
    interrupt::init();
    scheduler::init();
    user::init();
}
