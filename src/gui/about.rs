/// About application for the Vorzax desktop.
///
/// This module is intentionally self-contained: the GUI stack does not yet
/// provide a shared text renderer, so the About page carries a compact 5x7
/// bitmap font for the labels it draws.

use crate::driver::framebuffer::{self, Color, COLOR_BLACK, COLOR_GRAY, COLOR_WHITE};
use crate::gui::{app, window};

const VERSION: &str = "0.1.0";
const BUILD_DATE: &str = "2026-05-27";
const AUTHOR: &str = "Vorzax Team";

const ABOUT_WIDTH: u32 = 560;
const ABOUT_HEIGHT: u32 = 380;
const TITLE_HEIGHT: u32 = 28;
const PADDING: u32 = 18;

const HEADER_BG: Color = Color::new(18, 72, 142, 255);
const PANEL_BG: Color = Color::new(242, 246, 250, 255);
const ACCENT: Color = Color::new(0, 128, 96, 255);
const TEXT: Color = Color::new(24, 31, 42, 255);
const MUTED: Color = Color::new(82, 92, 106, 255);
const INFO_LINES: [&str; 8] = [
    "VORZAX OS",
    "ARM64 OPERATING SYSTEM WITH GUI DESKTOP",
    "VERSION 0.1.0",
    "BUILD DATE 2026-05-27",
    "AUTHOR VORZAX TEAM",
    "TARGET AARCH64",
    "BOOT UEFI",
    "KERNEL MONOLITHIC",
];

pub struct AboutApp {
    pub id: u32,
    pub window_id: window::WindowId,
    pub version: &'static str,
    pub build_date: &'static str,
    pub author: &'static str,
}

impl AboutApp {
    pub fn new(id: u32, window_id: window::WindowId) -> Self {
        AboutApp {
            id,
            window_id,
            version: VERSION,
            build_date: BUILD_DATE,
            author: AUTHOR,
        }
    }

    pub fn info_text(&self) -> &'static [&'static str] {
        &INFO_LINES
    }

    pub fn render(&self) {
        let x = 80;
        let y = 72;

        framebuffer::fill_rect(x, y, ABOUT_WIDTH, ABOUT_HEIGHT, COLOR_WHITE);
        framebuffer::draw_rect(x, y, ABOUT_WIDTH, ABOUT_HEIGHT, COLOR_BLACK);
        framebuffer::fill_rect(x + 1, y + 1, ABOUT_WIDTH - 2, TITLE_HEIGHT, HEADER_BG);
        framebuffer::fill_rect(x + ABOUT_WIDTH - 26, y + 4, 20, 20, COLOR_WHITE);
        framebuffer::draw_rect(x + ABOUT_WIDTH - 26, y + 4, 20, 20, COLOR_BLACK);
        draw_text("X", x + ABOUT_WIDTH - 20, y + 10, 2, COLOR_BLACK);
        draw_text("ABOUT VORZAX", x + 12, y + 9, 2, COLOR_WHITE);

        let content_x = x + PADDING;
        let content_y = y + TITLE_HEIGHT + PADDING;
        framebuffer::fill_rect(
            content_x,
            content_y,
            ABOUT_WIDTH - (PADDING * 2),
            ABOUT_HEIGHT - TITLE_HEIGHT - (PADDING * 2),
            PANEL_BG,
        );
        framebuffer::draw_rect(
            content_x,
            content_y,
            ABOUT_WIDTH - (PADDING * 2),
            ABOUT_HEIGHT - TITLE_HEIGHT - (PADDING * 2),
            COLOR_GRAY,
        );

        draw_text("VORZAX OS", content_x + 18, content_y + 18, 3, TEXT);
        draw_text(
            "INDIAN APNA OS HACKING BUILD",
            content_x + 20,
            content_y + 48,
            1,
            MUTED,
        );

        framebuffer::fill_rect(content_x + 20, content_y + 72, 200, 4, ACCENT);

        let lines = [
            "VERSION      0.1.0",
            "BUILD DATE   2026-05-27",
            "TARGET       ARM64 AARCH64",
            "BOOT         UEFI",
            "KERNEL       MONOLITHIC",
            "LANGUAGE     RUST + ASSEMBLY",
            "GUI          DESKTOP WINDOWS WIDGETS",
        ];

        let mut line_y = content_y + 96;
        for line in lines.iter() {
            draw_text(line, content_x + 22, line_y, 2, TEXT);
            line_y += 24;
        }

        draw_text(
            "FOR RESEARCH EDUCATION AND DEVELOPMENT",
            content_x + 22,
            content_y + 280,
            1,
            MUTED,
        );
    }

    pub fn handle_close(&mut self) {
        app::close_app(self.id);
        window::destroy_window(self.window_id);
    }
}

pub fn launch() -> Option<AboutApp> {
    let app_id = app::register_app("About Vorzax")?;
    let window_id = match window::create_window(80, 72, ABOUT_WIDTH, ABOUT_HEIGHT) {
        Some(id) => id,
        None => {
            app::close_app(app_id);
            return None;
        }
    };

    window::set_window_title(window_id, "About Vorzax");
    app::set_app_window(app_id, window_id);
    app::launch_app(app_id);

    let about = AboutApp::new(app_id, window_id);
    about.render();
    Some(about)
}

fn draw_text(text: &str, x: u32, y: u32, scale: u32, color: Color) {
    let mut cursor_x = x;
    for byte in text.bytes() {
        if byte == b'\n' {
            cursor_x = x;
            continue;
        }

        draw_char(byte, cursor_x, y, scale, color);
        cursor_x += 6 * scale;
    }
}

fn draw_char(ch: u8, x: u32, y: u32, scale: u32, color: Color) {
    let glyph = glyph_for(ch);
    for row in 0..7 {
        let bits = glyph[row];
        for col in 0..5 {
            if bits & (1 << (4 - col)) != 0 {
                framebuffer::fill_rect(
                    x + (col as u32 * scale),
                    y + (row as u32 * scale),
                    scale,
                    scale,
                    color,
                );
            }
        }
    }
}

fn glyph_for(ch: u8) -> [u8; 7] {
    match ch {
        b'A' => [0x0E, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        b'B' => [0x1E, 0x11, 0x11, 0x1E, 0x11, 0x11, 0x1E],
        b'C' => [0x0E, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0E],
        b'D' => [0x1E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1E],
        b'E' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x1F],
        b'F' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x10],
        b'G' => [0x0E, 0x11, 0x10, 0x17, 0x11, 0x11, 0x0F],
        b'H' => [0x11, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        b'I' => [0x0E, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0E],
        b'J' => [0x01, 0x01, 0x01, 0x01, 0x11, 0x11, 0x0E],
        b'K' => [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11],
        b'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1F],
        b'M' => [0x11, 0x1B, 0x15, 0x15, 0x11, 0x11, 0x11],
        b'N' => [0x11, 0x19, 0x15, 0x13, 0x11, 0x11, 0x11],
        b'O' => [0x0E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        b'P' => [0x1E, 0x11, 0x11, 0x1E, 0x10, 0x10, 0x10],
        b'Q' => [0x0E, 0x11, 0x11, 0x11, 0x15, 0x12, 0x0D],
        b'R' => [0x1E, 0x11, 0x11, 0x1E, 0x14, 0x12, 0x11],
        b'S' => [0x0F, 0x10, 0x10, 0x0E, 0x01, 0x01, 0x1E],
        b'T' => [0x1F, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        b'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        b'V' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x0A, 0x04],
        b'W' => [0x11, 0x11, 0x11, 0x15, 0x15, 0x15, 0x0A],
        b'X' => [0x11, 0x11, 0x0A, 0x04, 0x0A, 0x11, 0x11],
        b'Y' => [0x11, 0x11, 0x0A, 0x04, 0x04, 0x04, 0x04],
        b'Z' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1F],
        b'0' => [0x0E, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0E],
        b'1' => [0x04, 0x0C, 0x04, 0x04, 0x04, 0x04, 0x0E],
        b'2' => [0x0E, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1F],
        b'3' => [0x1E, 0x01, 0x01, 0x0E, 0x01, 0x01, 0x1E],
        b'4' => [0x02, 0x06, 0x0A, 0x12, 0x1F, 0x02, 0x02],
        b'5' => [0x1F, 0x10, 0x10, 0x1E, 0x01, 0x01, 0x1E],
        b'6' => [0x0E, 0x10, 0x10, 0x1E, 0x11, 0x11, 0x0E],
        b'7' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
        b'8' => [0x0E, 0x11, 0x11, 0x0E, 0x11, 0x11, 0x0E],
        b'9' => [0x0E, 0x11, 0x11, 0x0F, 0x01, 0x01, 0x0E],
        b'.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x0C],
        b':' => [0x00, 0x0C, 0x0C, 0x00, 0x0C, 0x0C, 0x00],
        b'-' => [0x00, 0x00, 0x00, 0x1F, 0x00, 0x00, 0x00],
        b'+' => [0x00, 0x04, 0x04, 0x1F, 0x04, 0x04, 0x00],
        b'/' => [0x01, 0x02, 0x02, 0x04, 0x08, 0x08, 0x10],
        b' ' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        _ => [0x1F, 0x11, 0x01, 0x02, 0x04, 0x00, 0x04],
    }
}
