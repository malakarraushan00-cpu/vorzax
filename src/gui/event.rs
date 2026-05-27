/// GUI Event System
/// 
/// Keyboard, mouse, and window events

use spin::Mutex;

/// Input event types
#[derive(Debug, Clone, Copy)]
pub enum EventType {
    KeyPress(u8),
    KeyRelease(u8),
    MouseMove { x: u32, y: u32 },
    MouseClick { x: u32, y: u32, button: MouseButton },
    MouseRelease { x: u32, y: u32, button: MouseButton },
    WindowClose,
    WindowFocus,
    WindowBlur,
    ApplicationQuit,
}

/// Mouse button
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Input event
#[derive(Debug, Clone, Copy)]
pub struct InputEvent {
    pub event_type: EventType,
    pub timestamp: u64,
    pub target_window: Option<u32>,
}

impl InputEvent {
    pub fn new(event_type: EventType) -> Self {
        InputEvent {
            event_type,
            timestamp: 0,
            target_window: None,
        }
    }
}

/// Event handler function pointer
pub type EventHandler = fn(&InputEvent) -> bool;

/// Event queue
pub struct EventQueue {
    events: [Option<InputEvent>; 256],
    head: usize,
    tail: usize,
    handlers: [Option<EventHandler>; 16],
    handler_count: usize,
}

impl EventQueue {
    pub fn new() -> Self {
        EventQueue {
            events: [None; 256],
            head: 0,
            tail: 0,
            handlers: [None; 16],
            handler_count: 0,
        }
    }
    
    pub fn push_event(&mut self, event: InputEvent) {
        if self.tail < 256 {
            self.events[self.tail] = Some(event);
            self.tail = (self.tail + 1) % 256;
        }
    }
    
    pub fn pop_event(&mut self) -> Option<InputEvent> {
        if self.head < self.tail {
            let event = self.events[self.head];
            self.head = (self.head + 1) % 256;
            event
        } else {
            None
        }
    }
    
    pub fn register_handler(&mut self, handler: EventHandler) -> bool {
        if self.handler_count < 16 {
            self.handlers[self.handler_count] = Some(handler);
            self.handler_count += 1;
            true
        } else {
            false
        }
    }
    
    pub fn dispatch(&mut self, event: InputEvent) {
        for i in 0..self.handler_count {
            if let Some(handler) = self.handlers[i] {
                if handler(&event) {
                    break; // Event consumed
                }
            }
        }
    }
    
    pub fn process_all(&mut self) {
        while let Some(event) = self.pop_event() {
            self.dispatch(event);
        }
    }
}

static EVENT_QUEUE: Mutex<EventQueue> = Mutex::new(EventQueue {
    events: [None; 256],
    head: 0,
    tail: 0,
    handlers: [None; 16],
    handler_count: 0,
});

pub fn init() {
    // Initialize event system
}

pub fn queue_event(event: InputEvent) {
    let mut queue = EVENT_QUEUE.lock();
    queue.push_event(event);
}

pub fn register_event_handler(handler: EventHandler) {
    let mut queue = EVENT_QUEUE.lock();
    queue.register_handler(handler);
}

pub fn process_events() {
    let mut queue = EVENT_QUEUE.lock();
    queue.process_all();
}

/// Keyboard event helper
pub fn on_key_press(key: u8) {
    let event = InputEvent::new(EventType::KeyPress(key));
    queue_event(event);
}

/// Keyboard event helper
pub fn on_key_release(key: u8) {
    let event = InputEvent::new(EventType::KeyRelease(key));
    queue_event(event);
}

/// Mouse event helper
pub fn on_mouse_move(x: u32, y: u32) {
    let event = InputEvent::new(EventType::MouseMove { x, y });
    queue_event(event);
}

/// Mouse event helper
pub fn on_mouse_click(x: u32, y: u32, button: MouseButton) {
    let event = InputEvent::new(EventType::MouseClick { x, y, button });
    queue_event(event);
}

/// Mouse event helper
pub fn on_mouse_release(x: u32, y: u32, button: MouseButton) {
    let event = InputEvent::new(EventType::MouseRelease { x, y, button });
    queue_event(event);
}

/// Window event helper
pub fn on_window_close(window_id: u32) {
    let mut event = InputEvent::new(EventType::WindowClose);
    event.target_window = Some(window_id);
    queue_event(event);
}

/// Application quit
pub fn on_quit() {
    let event = InputEvent::new(EventType::ApplicationQuit);
    queue_event(event);
}

/// Check if event queue is empty
pub fn is_empty() -> bool {
    let queue = EVENT_QUEUE.lock();
    queue.head >= queue.tail
}

/// Get event count
pub fn event_count() -> usize {
    let queue = EVENT_QUEUE.lock();
    queue.tail - queue.head
}
