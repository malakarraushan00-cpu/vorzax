/// Mouse input driver.
///
/// Hardware-neutral queue for pointer events. Real PS/2 or USB HID drivers can
/// translate packets into these events.

use spin::Mutex;

pub const MOUSE_QUEUE_SIZE: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseEventKind {
    Move,
    ButtonPress(MouseButton),
    ButtonRelease(MouseButton),
    Scroll(i8),
}

#[derive(Debug, Clone, Copy)]
pub struct MouseEvent {
    pub x: i32,
    pub y: i32,
    pub dx: i32,
    pub dy: i32,
    pub kind: MouseEventKind,
}

pub struct MouseDriver {
    x: i32,
    y: i32,
    queue: [Option<MouseEvent>; MOUSE_QUEUE_SIZE],
    head: usize,
    tail: usize,
    len: usize,
    dropped: u64,
}

impl MouseDriver {
    pub const fn new() -> Self {
        MouseDriver {
            x: 0,
            y: 0,
            queue: [None; MOUSE_QUEUE_SIZE],
            head: 0,
            tail: 0,
            len: 0,
            dropped: 0,
        }
    }

    pub fn reset(&mut self) {
        self.x = 0;
        self.y = 0;
        self.queue = [None; MOUSE_QUEUE_SIZE];
        self.head = 0;
        self.tail = 0;
        self.len = 0;
        self.dropped = 0;
    }

    pub fn move_by(&mut self, dx: i32, dy: i32) -> bool {
        self.x = self.x.saturating_add(dx);
        self.y = self.y.saturating_add(dy);
        self.push_event(MouseEvent {
            x: self.x,
            y: self.y,
            dx,
            dy,
            kind: MouseEventKind::Move,
        })
    }

    pub fn push_event(&mut self, event: MouseEvent) -> bool {
        if self.len >= MOUSE_QUEUE_SIZE {
            self.dropped += 1;
            return false;
        }

        self.queue[self.tail] = Some(event);
        self.tail = (self.tail + 1) % MOUSE_QUEUE_SIZE;
        self.len += 1;
        true
    }

    pub fn pop_event(&mut self) -> Option<MouseEvent> {
        if self.len == 0 {
            return None;
        }

        let event = self.queue[self.head];
        self.queue[self.head] = None;
        self.head = (self.head + 1) % MOUSE_QUEUE_SIZE;
        self.len -= 1;
        event
    }
}

static MOUSE: Mutex<MouseDriver> = Mutex::new(MouseDriver::new());

pub fn init() {
    MOUSE.lock().reset();
}

pub fn move_by(dx: i32, dy: i32) -> bool {
    MOUSE.lock().move_by(dx, dy)
}

pub fn button(button: MouseButton, pressed: bool) -> bool {
    let mut mouse = MOUSE.lock();
    let kind = if pressed {
        MouseEventKind::ButtonPress(button)
    } else {
        MouseEventKind::ButtonRelease(button)
    };

    let event = MouseEvent {
        x: mouse.x,
        y: mouse.y,
        dx: 0,
        dy: 0,
        kind,
    };
    mouse.push_event(event)
}

pub fn read_event() -> Option<MouseEvent> {
    MOUSE.lock().pop_event()
}

pub fn position() -> (i32, i32) {
    let mouse = MOUSE.lock();
    (mouse.x, mouse.y)
}

pub fn pending_events() -> usize {
    MOUSE.lock().len
}

pub fn dropped_events() -> u64 {
    MOUSE.lock().dropped
}
