/// Window Management System
/// 
/// Handles window creation, destruction, events, and rendering

use spin::Mutex;
use crate::driver::framebuffer::{self, Color, COLOR_WHITE, COLOR_BLACK, COLOR_GRAY};

/// Window ID type
pub type WindowId = u32;

/// Window state
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowState {
    Minimized,
    Normal,
    Maximized,
    Focused,
    Unfocused,
}

/// Window rectangle
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Rect { x, y, width, height }
    }
    
    pub fn contains_point(&self, px: u32, py: u32) -> bool {
        px >= self.x && px < self.x + self.width &&
        py >= self.y && py < self.y + self.height
    }
    
    pub fn intersects(&self, other: &Rect) -> bool {
        !(self.x + self.width <= other.x ||
          other.x + other.width <= self.x ||
          self.y + self.height <= other.y ||
          other.y + other.height <= self.y)
    }
}

/// Window structure
pub struct Window {
    pub id: WindowId,
    pub title: [u8; 256],
    pub title_len: usize,
    pub rect: Rect,
    pub state: WindowState,
    pub background: Color,
    pub border_color: Color,
    pub has_focus: bool,
}

impl Window {
    pub fn new(id: WindowId, x: u32, y: u32, width: u32, height: u32) -> Self {
        let mut title = [0u8; 256];
        title[0] = b'W';
        
        Window {
            id,
            title,
            title_len: 7,
            rect: Rect::new(x, y, width, height),
            state: WindowState::Normal,
            background: COLOR_WHITE,
            border_color: COLOR_BLACK,
            has_focus: false,
        }
    }
    
    pub fn set_title(&mut self, title: &str) {
        let bytes = title.as_bytes();
        let len = core::cmp::min(bytes.len(), 256);
        for i in 0..len {
            self.title[i] = bytes[i];
        }
        self.title_len = len;
    }
    
    pub fn render(&self) {
        // Draw window background
        framebuffer::fill_rect(self.rect.x, self.rect.y, 
                              self.rect.width, self.rect.height,
                              self.background);
        
        // Draw border (2px)
        let color = if self.has_focus { COLOR_BLACK } else { COLOR_GRAY };
        framebuffer::draw_rect(self.rect.x, self.rect.y,
                              self.rect.width, self.rect.height, color);
        
        // Draw title bar
        let title_height = 24u32;
        let title_color = if self.has_focus { 
            Color::new(0, 0, 128, 255) 
        } else { 
            COLOR_GRAY 
        };
        framebuffer::fill_rect(self.rect.x + 1, self.rect.y + 1,
                              self.rect.width - 2, title_height - 2,
                              title_color);
        
        // Draw close button
        let close_btn_x = self.rect.x + self.rect.width - 24;
        let close_btn_y = self.rect.y + 2;
        framebuffer::fill_rect(close_btn_x, close_btn_y, 22, 20, COLOR_WHITE);
        framebuffer::draw_rect(close_btn_x, close_btn_y, 22, 20, COLOR_BLACK);
    }
    
    pub fn hit_test_close_button(&self) -> bool {
        let close_btn_x = self.rect.x + self.rect.width - 24;
        let close_btn_y = self.rect.y + 2;
        // Would use mouse coordinates here
        false
    }
}

/// Window manager
pub struct WindowManager {
    windows: [Option<Window>; 16],
    window_count: usize,
    focused_window: Option<WindowId>,
    next_id: WindowId,
}

impl WindowManager {
    pub fn new() -> Self {
        WindowManager {
            windows: [
                None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None,
            ],
            window_count: 0,
            focused_window: None,
            next_id: 1,
        }
    }
    
    pub fn create_window(&mut self, x: u32, y: u32, width: u32, height: u32) -> Option<WindowId> {
        if self.window_count < 16 {
            let id = self.next_id;
            self.next_id += 1;
            
            let mut window = Window::new(id, x, y, width, height);
            window.has_focus = true;
            
            // Unfocus previous window
            if let Some(old_id) = self.focused_window {
                if let Some(Some(w)) = self.windows.iter_mut().find(|slot| {
                    if let Some(win) = slot {
                        return win.id == old_id;
                    }
                    false
                }) {
                    w.has_focus = false;
                }
            }
            
            self.focused_window = Some(id);
            
            if let Some(slot) = self.windows.iter_mut().find(|s| s.is_none()) {
                *slot = Some(window);
                self.window_count += 1;
                return Some(id);
            }
        }
        None
    }
    
    pub fn destroy_window(&mut self, id: WindowId) {
        if let Some(pos) = self.windows.iter().position(|w| {
            if let Some(win) = w {
                return win.id == id;
            }
            false
        }) {
            self.windows[pos] = None;
            self.window_count -= 1;
            
            if self.focused_window == Some(id) {
                self.focused_window = None;
            }
        }
    }
    
    pub fn get_window(&self, id: WindowId) -> Option<&Window> {
        self.windows.iter()
            .find_map(|slot| {
                if let Some(w) = slot {
                    if w.id == id {
                        return Some(w);
                    }
                }
                None
            })
    }
    
    pub fn get_window_mut(&mut self, id: WindowId) -> Option<&mut Window> {
        self.windows.iter_mut()
            .find_map(|slot| {
                if let Some(w) = slot {
                    if w.id == id {
                        return Some(w);
                    }
                }
                None
            })
    }
    
    pub fn render_all(&self) {
        for window in self.windows.iter() {
            if let Some(w) = window {
                w.render();
            }
        }
    }
}

static WINDOW_MANAGER: Mutex<WindowManager> = Mutex::new(WindowManager {
    windows: [None, None, None, None, None, None, None, None,
              None, None, None, None, None, None, None, None],
    window_count: 0,
    focused_window: None,
    next_id: 1,
});

pub fn init() {
    // Initialize window manager
}

pub fn create_window(x: u32, y: u32, width: u32, height: u32) -> Option<WindowId> {
    let mut wm = WINDOW_MANAGER.lock();
    wm.create_window(x, y, width, height)
}

pub fn destroy_window(id: WindowId) {
    let mut wm = WINDOW_MANAGER.lock();
    wm.destroy_window(id);
}

pub fn render_windows() {
    let wm = WINDOW_MANAGER.lock();
    wm.render_all();
}

pub fn set_window_title(id: WindowId, title: &str) {
    let mut wm = WINDOW_MANAGER.lock();
    if let Some(w) = wm.get_window_mut(id) {
        w.set_title(title);
    }
}

pub fn get_focused_window() -> Option<WindowId> {
    WINDOW_MANAGER.lock().focused_window
}

pub fn set_focused_window(id: WindowId) {
    let mut wm = WINDOW_MANAGER.lock();
    if let Some(old_id) = wm.focused_window {
        if let Some(w) = wm.get_window_mut(old_id) {
            w.has_focus = false;
        }
    }
    if let Some(w) = wm.get_window_mut(id) {
        w.has_focus = true;
    }
    wm.focused_window = Some(id);
}
