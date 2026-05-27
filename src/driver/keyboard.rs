/// Keyboard input driver.
///
/// This is a hardware-neutral event queue. A PS/2, USB HID, or UART console
/// backend can feed scancodes into it later.

use spin::Mutex;

pub const KEYBOARD_QUEUE_SIZE: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyState {
    Pressed,
    Released,
}

#[derive(Debug, Clone, Copy)]
pub struct KeyboardEvent {
    pub scancode: u16,
    pub ascii: Option<u8>,
    pub state: KeyState,
    pub modifiers: u8,
}

pub struct KeyboardDriver {
    queue: [Option<KeyboardEvent>; KEYBOARD_QUEUE_SIZE],
    head: usize,
    tail: usize,
    len: usize,
    dropped: u64,
}

impl KeyboardDriver {
    pub const fn new() -> Self {
        KeyboardDriver {
            queue: [None; KEYBOARD_QUEUE_SIZE],
            head: 0,
            tail: 0,
            len: 0,
            dropped: 0,
        }
    }

    pub fn reset(&mut self) {
        self.queue = [None; KEYBOARD_QUEUE_SIZE];
        self.head = 0;
        self.tail = 0;
        self.len = 0;
        self.dropped = 0;
    }

    pub fn push_event(&mut self, event: KeyboardEvent) -> bool {
        if self.len >= KEYBOARD_QUEUE_SIZE {
            self.dropped += 1;
            return false;
        }

        self.queue[self.tail] = Some(event);
        self.tail = (self.tail + 1) % KEYBOARD_QUEUE_SIZE;
        self.len += 1;
        true
    }

    pub fn pop_event(&mut self) -> Option<KeyboardEvent> {
        if self.len == 0 {
            return None;
        }

        let event = self.queue[self.head];
        self.queue[self.head] = None;
        self.head = (self.head + 1) % KEYBOARD_QUEUE_SIZE;
        self.len -= 1;
        event
    }
}

static KEYBOARD: Mutex<KeyboardDriver> = Mutex::new(KeyboardDriver::new());

pub fn init() {
    KEYBOARD.lock().reset();
}

pub fn push_scancode(scancode: u16, pressed: bool) -> bool {
    let state = if pressed {
        KeyState::Pressed
    } else {
        KeyState::Released
    };

    let event = KeyboardEvent {
        scancode,
        ascii: scancode_to_ascii(scancode),
        state,
        modifiers: 0,
    };

    KEYBOARD.lock().push_event(event)
}

pub fn push_event(event: KeyboardEvent) -> bool {
    KEYBOARD.lock().push_event(event)
}

pub fn read_event() -> Option<KeyboardEvent> {
    KEYBOARD.lock().pop_event()
}

pub fn pending_events() -> usize {
    KEYBOARD.lock().len
}

pub fn dropped_events() -> u64 {
    KEYBOARD.lock().dropped
}

fn scancode_to_ascii(scancode: u16) -> Option<u8> {
    match scancode {
        0x02 => Some(b'1'),
        0x03 => Some(b'2'),
        0x04 => Some(b'3'),
        0x05 => Some(b'4'),
        0x06 => Some(b'5'),
        0x07 => Some(b'6'),
        0x08 => Some(b'7'),
        0x09 => Some(b'8'),
        0x0A => Some(b'9'),
        0x0B => Some(b'0'),
        0x10 => Some(b'q'),
        0x11 => Some(b'w'),
        0x12 => Some(b'e'),
        0x13 => Some(b'r'),
        0x14 => Some(b't'),
        0x15 => Some(b'y'),
        0x16 => Some(b'u'),
        0x17 => Some(b'i'),
        0x18 => Some(b'o'),
        0x19 => Some(b'p'),
        0x1E => Some(b'a'),
        0x1F => Some(b's'),
        0x20 => Some(b'd'),
        0x21 => Some(b'f'),
        0x22 => Some(b'g'),
        0x23 => Some(b'h'),
        0x24 => Some(b'j'),
        0x25 => Some(b'k'),
        0x26 => Some(b'l'),
        0x2C => Some(b'z'),
        0x2D => Some(b'x'),
        0x2E => Some(b'c'),
        0x2F => Some(b'v'),
        0x30 => Some(b'b'),
        0x31 => Some(b'n'),
        0x32 => Some(b'm'),
        0x39 => Some(b' '),
        _ => None,
    }
}
