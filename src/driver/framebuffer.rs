/// Linear framebuffer driver for ARM64
/// 
/// Supports 1920x1080 @ 32-bit RGBA

use spin::Mutex;

/// Display mode
#[derive(Debug, Clone, Copy)]
pub struct DisplayMode {
    pub width: u32,
    pub height: u32,
    pub stride: u32,  // bytes per scan line
    pub format: PixelFormat,
}

/// Pixel format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    RGBA8888,
    BGRA8888,
    RGB888,
    RGB565,
}

impl PixelFormat {
    pub fn bits_per_pixel(&self) -> u32 {
        match self {
            PixelFormat::RGBA8888 | PixelFormat::BGRA8888 => 32,
            PixelFormat::RGB888 => 24,
            PixelFormat::RGB565 => 16,
        }
    }
}

/// Color (32-bit RGBA)
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }
    
    pub fn to_u32(&self) -> u32 {
        ((self.r as u32) << 24) | ((self.g as u32) << 16) | 
        ((self.b as u32) << 8) | (self.a as u32)
    }
}

// Default colors
pub const COLOR_BLACK: Color = Color { r: 0, g: 0, b: 0, a: 255 };
pub const COLOR_WHITE: Color = Color { r: 255, g: 255, b: 255, a: 255 };
pub const COLOR_RED: Color = Color { r: 255, g: 0, b: 0, a: 255 };
pub const COLOR_GREEN: Color = Color { r: 0, g: 255, b: 0, a: 255 };
pub const COLOR_BLUE: Color = Color { r: 0, g: 0, b: 255, a: 255 };
pub const COLOR_GRAY: Color = Color { r: 128, g: 128, b: 128, a: 255 };

/// Framebuffer state
pub struct FramebufferDriver {
    pub mode: DisplayMode,
    pub buffer: *mut u32,
    pub back_buffer: *mut u32,
}

impl FramebufferDriver {
    pub fn new(width: u32, height: u32, buffer_addr: u64) -> Self {
        let stride = width * 4; // 32-bit pixels
        
        FramebufferDriver {
            mode: DisplayMode {
                width,
                height,
                stride,
                format: PixelFormat::RGBA8888,
            },
            buffer: buffer_addr as *mut u32,
            back_buffer: (buffer_addr + (height * stride) as u64) as *mut u32,
        }
    }
    
    pub fn put_pixel(&self, x: u32, y: u32, color: Color) {
        if x < self.mode.width && y < self.mode.height {
            unsafe {
                let offset = (y * self.mode.stride / 4) + x;
                *self.buffer.add(offset as usize) = color.to_u32();
            }
        }
    }
    
    pub fn clear(&self, color: Color) {
        unsafe {
            let pixel_count = (self.mode.stride / 4) * self.mode.height;
            let pixel_value = color.to_u32();
            for i in 0..pixel_count {
                *self.buffer.add(i as usize) = pixel_value;
            }
        }
    }
    
    pub fn fill_rect(&self, x: u32, y: u32, width: u32, height: u32, color: Color) {
        for dy in 0..height {
            for dx in 0..width {
                self.put_pixel(x + dx, y + dy, color);
            }
        }
    }
    
    pub fn draw_line(&self, x0: u32, y0: u32, x1: u32, y1: u32, color: Color) {
        // Bresenham line algorithm
        let dx = if x1 > x0 { x1 - x0 } else { x0 - x1 } as i32;
        let dy = if y1 > y0 { y1 - y0 } else { y0 - y1 } as i32;
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        
        let mut err = dx - dy;
        let mut x = x0 as i32;
        let mut y = y0 as i32;
        
        loop {
            self.put_pixel(x as u32, y as u32, color);
            
            if x == x1 as i32 && y == y1 as i32 {
                break;
            }
            
            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }
    
    pub fn draw_rect(&self, x: u32, y: u32, width: u32, height: u32, color: Color) {
        // Top and bottom lines
        for dx in 0..width {
            self.put_pixel(x + dx, y, color);
            self.put_pixel(x + dx, y + height - 1, color);
        }
        // Left and right lines
        for dy in 0..height {
            self.put_pixel(x, y + dy, color);
            self.put_pixel(x + width - 1, y + dy, color);
        }
    }
    
    pub fn swap_buffers(&mut self) {
        // Swap front and back buffers
        let temp = self.buffer;
        self.buffer = self.back_buffer;
        self.back_buffer = temp;
    }
}

static FRAMEBUFFER: Mutex<Option<FramebufferDriver>> = Mutex::new(None);

pub fn init() {
    // Initialize framebuffer driver
    // Physical address of framebuffer is typically provided by bootloader
    // Default: 0x50000000 (typical QEMU ARM64 virt machine)
    let fb = FramebufferDriver::new(1920, 1080, 0x5000_0000);
    let mut fb_guard = FRAMEBUFFER.lock();
    *fb_guard = Some(fb);
}

/// Clear screen with color
pub fn clear(color: Color) {
    let fb_guard = FRAMEBUFFER.lock();
    if let Some(ref fb) = *fb_guard {
        fb.clear(color);
    }
}

/// Put pixel on screen
pub fn put_pixel(x: u32, y: u32, color: Color) {
    let fb_guard = FRAMEBUFFER.lock();
    if let Some(ref fb) = *fb_guard {
        fb.put_pixel(x, y, color);
    }
}

/// Draw filled rectangle
pub fn fill_rect(x: u32, y: u32, width: u32, height: u32, color: Color) {
    let fb_guard = FRAMEBUFFER.lock();
    if let Some(ref fb) = *fb_guard {
        fb.fill_rect(x, y, width, height, color);
    }
}

/// Draw rectangle outline
pub fn draw_rect(x: u32, y: u32, width: u32, height: u32, color: Color) {
    let fb_guard = FRAMEBUFFER.lock();
    if let Some(ref fb) = *fb_guard {
        fb.draw_rect(x, y, width, height, color);
    }
}

/// Draw line
pub fn draw_line(x0: u32, y0: u32, x1: u32, y1: u32, color: Color) {
    let fb_guard = FRAMEBUFFER.lock();
    if let Some(ref fb) = *fb_guard {
        fb.draw_line(x0, y0, x1, y1, color);
    }
}

/// Get display mode
pub fn get_display_mode() -> Option<DisplayMode> {
    let fb_guard = FRAMEBUFFER.lock();
    fb_guard.as_ref().map(|fb| fb.mode)
}
