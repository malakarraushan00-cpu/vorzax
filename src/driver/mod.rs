/// Device drivers for Vorzax OS
pub mod framebuffer;
pub mod input;
pub mod keyboard;
pub mod mouse;
pub mod storage;
pub mod timer;
pub mod uart;

pub fn init() {
    uart::init(115200);
    timer::init();
    storage::init();
    input::init();
    framebuffer::init();
}
