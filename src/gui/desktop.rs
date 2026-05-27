/// Desktop Environment
/// 
/// Main GUI desktop with taskbar, windows, and background

use crate::driver::framebuffer::{self, Color, COLOR_BLACK, COLOR_WHITE, COLOR_GRAY};

/// Desktop state
pub struct Desktop {
    pub width: u32,
    pub height: u32,
    pub taskbar_height: u32,
    pub background_color: Color,
    pub taskbar_color: Color,
}

impl Desktop {
    pub fn new(width: u32, height: u32) -> Self {
        Desktop {
            width,
            height,
            taskbar_height: 40,
            background_color: Color::new(60, 120, 200, 255),
            taskbar_color: COLOR_GRAY,
        }
    }
    
    pub fn render(&self) {
        // Clear background
        framebuffer::fill_rect(0, 0, self.width, self.height - self.taskbar_height, 
                              self.background_color);
        
        // Draw taskbar
        framebuffer::fill_rect(0, self.height - self.taskbar_height, 
                              self.width, self.taskbar_height, 
                              self.taskbar_color);
        
        // Draw taskbar border
        framebuffer::draw_line(0, self.height - self.taskbar_height, 
                              self.width, self.height - self.taskbar_height,
                              COLOR_BLACK);
    }
    
    pub fn get_taskbar_rect(&self) -> (u32, u32, u32, u32) {
        (0, self.height - self.taskbar_height, self.width, self.taskbar_height)
    }
    
    pub fn get_client_area(&self) -> (u32, u32, u32, u32) {
        (0, 0, self.width, self.height - self.taskbar_height)
    }
}

static mut DESKTOP: Option<Desktop> = None;

pub fn init() {
    unsafe {
        if let Some(mode) = framebuffer::get_display_mode() {
            DESKTOP = Some(Desktop::new(mode.width, mode.height));
            if let Some(ref desktop) = DESKTOP {
                desktop.render();
            }
        }
    }
}

pub fn render() {
    unsafe {
        if let Some(ref desktop) = DESKTOP {
            desktop.render();
        }
    }
}

pub fn get_desktop() -> Option<Desktop> {
    unsafe {
        DESKTOP.as_ref().map(|d| Desktop {
            width: d.width,
            height: d.height,
            taskbar_height: d.taskbar_height,
            background_color: d.background_color,
            taskbar_color: d.taskbar_color,
        })
    }
}

/// Draw wallpaper pattern
pub fn draw_wallpaper() {
    if let Some(mode) = framebuffer::get_display_mode() {
        let mut color_idx = 0u32;
        let colors = [
            Color::new(60, 120, 200, 255),
            Color::new(50, 110, 190, 255),
        ];
        
        // Draw checkerboard pattern
        for y in (0..mode.height).step_by(32) {
            for x in (0..mode.width).step_by(32) {
                color_idx ^= 1;
                framebuffer::fill_rect(x, y, 32, 32, colors[color_idx as usize]);
            }
        }
    }
}

/// Update desktop clock/status
pub fn update_status(status_text: &str) {
    // This would render status text to taskbar
}
