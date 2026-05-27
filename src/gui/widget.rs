/// GUI Widget Library
/// 
/// Reusable UI components (buttons, text fields, panels, etc.)

use crate::driver::framebuffer::{self, Color, COLOR_WHITE, COLOR_BLACK, COLOR_GRAY};

/// Widget type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetType {
    Button,
    TextBox,
    Label,
    Panel,
    Checkbox,
    Slider,
}

/// Widget state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetState {
    Normal,
    Hovered,
    Pressed,
    Disabled,
    Focused,
}

/// Button widget
pub struct Button {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub label: [u8; 256],
    pub label_len: usize,
    pub state: WidgetState,
    pub background: Color,
    pub text_color: Color,
}

impl Button {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Button {
            x,
            y,
            width,
            height,
            label: [0; 256],
            label_len: 0,
            state: WidgetState::Normal,
            background: COLOR_GRAY,
            text_color: COLOR_BLACK,
        }
    }
    
    pub fn set_label(&mut self, text: &str) {
        let bytes = text.as_bytes();
        let len = core::cmp::min(bytes.len(), 256);
        for i in 0..len {
            self.label[i] = bytes[i];
        }
        self.label_len = len;
    }
    
    pub fn render(&self) {
        let bg_color = match self.state {
            WidgetState::Normal => self.background,
            WidgetState::Hovered => Color::new(150, 150, 150, 255),
            WidgetState::Pressed => Color::new(100, 100, 100, 255),
            WidgetState::Disabled => Color::new(180, 180, 180, 255),
            WidgetState::Focused => Color::new(120, 120, 180, 255),
        };
        
        // Draw button background
        framebuffer::fill_rect(self.x, self.y, self.width, self.height, bg_color);
        
        // Draw button border
        framebuffer::draw_rect(self.x, self.y, self.width, self.height, COLOR_BLACK);
    }
    
    pub fn hit_test(&self, px: u32, py: u32) -> bool {
        px >= self.x && px < self.x + self.width &&
        py >= self.y && py < self.y + self.height
    }
}

/// Text input widget
pub struct TextBox {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub text: [u8; 256],
    pub text_len: usize,
    pub cursor: usize,
    pub state: WidgetState,
}

impl TextBox {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        TextBox {
            x,
            y,
            width,
            height,
            text: [0; 256],
            text_len: 0,
            cursor: 0,
            state: WidgetState::Normal,
        }
    }
    
    pub fn insert_char(&mut self, c: u8) {
        if self.text_len < 256 {
            self.text[self.cursor] = c;
            self.text_len += 1;
            self.cursor += 1;
        }
    }
    
    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.text_len -= 1;
        }
    }
    
    pub fn render(&self) {
        // Draw textbox background
        framebuffer::fill_rect(self.x, self.y, self.width, self.height, COLOR_WHITE);
        
        // Draw border
        let border_color = if self.state == WidgetState::Focused {
            COLOR_BLACK
        } else {
            COLOR_GRAY
        };
        framebuffer::draw_rect(self.x, self.y, self.width, self.height, border_color);
    }
}

/// Label widget (text display)
pub struct Label {
    pub x: u32,
    pub y: u32,
    pub text: [u8; 256],
    pub text_len: usize,
    pub color: Color,
}

impl Label {
    pub fn new(x: u32, y: u32, text: &str) -> Self {
        let mut label_text = [0u8; 256];
        let bytes = text.as_bytes();
        let len = core::cmp::min(bytes.len(), 256);
        for i in 0..len {
            label_text[i] = bytes[i];
        }
        
        Label {
            x,
            y,
            text: label_text,
            text_len: len,
            color: COLOR_BLACK,
        }
    }
    
    pub fn render(&self) {
        // Simple text rendering (placeholder)
        // In a real implementation, this would use a font renderer
    }
}

/// Panel widget (container)
pub struct Panel {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub background: Color,
}

impl Panel {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Panel {
            x,
            y,
            width,
            height,
            background: COLOR_GRAY,
        }
    }
    
    pub fn render(&self) {
        framebuffer::fill_rect(self.x, self.y, self.width, self.height, self.background);
    }
}

/// Checkbox widget
pub struct Checkbox {
    pub x: u32,
    pub y: u32,
    pub checked: bool,
    pub label: [u8; 256],
    pub label_len: usize,
}

impl Checkbox {
    pub fn new(x: u32, y: u32) -> Self {
        Checkbox {
            x,
            y,
            checked: false,
            label: [0; 256],
            label_len: 0,
        }
    }
    
    pub fn toggle(&mut self) {
        self.checked = !self.checked;
    }
    
    pub fn render(&self) {
        let box_size = 20u32;
        
        // Draw checkbox background
        let bg_color = if self.checked { COLOR_BLACK } else { COLOR_WHITE };
        framebuffer::fill_rect(self.x, self.y, box_size, box_size, bg_color);
        
        // Draw checkbox border
        framebuffer::draw_rect(self.x, self.y, box_size, box_size, COLOR_BLACK);
    }
}

/// Slider widget
pub struct Slider {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub min_value: i32,
    pub max_value: i32,
    pub current_value: i32,
}

impl Slider {
    pub fn new(x: u32, y: u32, width: u32, min: i32, max: i32) -> Self {
        Slider {
            x,
            y,
            width,
            height: 20,
            min_value: min,
            max_value: max,
            current_value: min,
        }
    }
    
    pub fn set_value(&mut self, value: i32) {
        self.current_value = core::cmp::max(self.min_value, 
                                           core::cmp::min(self.max_value, value));
    }
    
    pub fn render(&self) {
        // Draw slider track
        framebuffer::fill_rect(self.x, self.y + 8, self.width, 4, COLOR_GRAY);
        
        // Draw slider thumb
        let progress = ((self.current_value - self.min_value) as f64) / 
                       ((self.max_value - self.min_value) as f64);
        let thumb_x = self.x + ((progress * self.width as f64) as u32);
        framebuffer::fill_rect(thumb_x - 5, self.y, 10, self.height, COLOR_BLACK);
    }
}
