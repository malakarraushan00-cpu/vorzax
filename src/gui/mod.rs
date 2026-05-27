/// GUI framework for Vorzax OS
pub mod desktop;
pub mod window;
pub mod app;
pub mod widget;
pub mod event;
pub mod about;

pub fn init() {
    desktop::init();
    window::init();
    app::init();
    event::init();
}
