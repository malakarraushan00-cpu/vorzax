/// Unified input driver facade.
///
/// Keeps keyboard and mouse drivers separate while giving GUI/event code a
/// single place to poll input later.

use crate::driver::{keyboard, mouse};

#[derive(Debug, Clone, Copy)]
pub enum InputEvent {
    Keyboard(keyboard::KeyboardEvent),
    Mouse(mouse::MouseEvent),
}

pub fn init() {
    keyboard::init();
    mouse::init();
}

pub fn poll_event() -> Option<InputEvent> {
    if let Some(event) = keyboard::read_event() {
        return Some(InputEvent::Keyboard(event));
    }

    mouse::read_event().map(InputEvent::Mouse)
}

pub fn pending_events() -> usize {
    keyboard::pending_events() + mouse::pending_events()
}

pub fn dropped_events() -> u64 {
    keyboard::dropped_events() + mouse::dropped_events()
}
