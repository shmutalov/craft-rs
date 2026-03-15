#![allow(dead_code)]

use portablegl::gl_context::GlContext;
use portablegl::gl_types::*;
use std::path::Path;

pub fn load_png_texture(ctx: &mut GlContext, path: &str) -> (u32, u32) {
    let file = std::fs::File::open(Path::new(path))
        .unwrap_or_else(|e| panic!("Failed to open texture {}: {}", path, e));
    let decoder = png::Decoder::new(file);
    let mut reader = decoder.read_info()
        .unwrap_or_else(|e| panic!("Failed to read PNG info {}: {}", path, e));
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf)
        .unwrap_or_else(|e| panic!("Failed to decode PNG {}: {}", path, e));

    let width = info.width;
    let height = info.height;

    // Convert to RGBA if needed
    let rgba_data = match info.color_type {
        png::ColorType::Rgba => buf[..info.buffer_size()].to_vec(),
        png::ColorType::Rgb => {
            let pixels = info.buffer_size() / 3;
            let mut rgba = vec![0u8; pixels * 4];
            for i in 0..pixels {
                rgba[i * 4]     = buf[i * 3];
                rgba[i * 4 + 1] = buf[i * 3 + 1];
                rgba[i * 4 + 2] = buf[i * 3 + 2];
                rgba[i * 4 + 3] = 255;
            }
            rgba
        }
        png::ColorType::Grayscale => {
            let pixels = info.buffer_size();
            let mut rgba = vec![0u8; pixels * 4];
            for i in 0..pixels {
                rgba[i * 4]     = buf[i];
                rgba[i * 4 + 1] = buf[i];
                rgba[i * 4 + 2] = buf[i];
                rgba[i * 4 + 3] = 255;
            }
            rgba
        }
        png::ColorType::GrayscaleAlpha => {
            let pixels = info.buffer_size() / 2;
            let mut rgba = vec![0u8; pixels * 4];
            for i in 0..pixels {
                rgba[i * 4]     = buf[i * 2];
                rgba[i * 4 + 1] = buf[i * 2];
                rgba[i * 4 + 2] = buf[i * 2];
                rgba[i * 4 + 3] = buf[i * 2 + 1];
            }
            rgba
        }
        _ => panic!("Unsupported PNG color type: {:?}", info.color_type),
    };

    // Flip image vertically (OpenGL convention: bottom-left origin)
    let row_size = (width * 4) as usize;
    let mut flipped = vec![0u8; rgba_data.len()];
    for y in 0..height as usize {
        let src_row = y * row_size;
        let dst_row = (height as usize - 1 - y) * row_size;
        flipped[dst_row..dst_row + row_size].copy_from_slice(&rgba_data[src_row..src_row + row_size]);
    }

    ctx.gl_tex_image_2d(
        GL_TEXTURE_2D,
        0,
        GL_RGBA as GLint,
        width as GLsizei,
        height as GLsizei,
        0,
        GL_RGBA,
        GL_UNSIGNED_BYTE,
        Some(&flipped),
    );

    (width, height)
}

pub struct FpsCounter {
    pub fps: i32,
    frames: i32,
    since: f64,
}

impl FpsCounter {
    pub fn new() -> Self {
        FpsCounter { fps: 0, frames: 0, since: 0.0 }
    }

    pub fn update(&mut self, now: f64) {
        self.frames += 1;
        let elapsed = now - self.since;
        if elapsed >= 1.0 {
            self.fps = (self.frames as f64 / elapsed) as i32;
            self.frames = 0;
            self.since = now;
        }
    }
}

pub fn char_width(c: char) -> i32 {
    match c {
        'M' | 'W' | 'm' | 'w' => 7,
        ' ' | 'f' | 'i' | 'j' | 'l' | 't' | '!' | '\'' | ',' | '.' | ':' | ';' | '|' => 4,
        _ => 5,
    }
}

pub fn string_width(text: &str) -> i32 {
    text.chars().map(char_width).sum()
}

pub fn wrap(text: &str, max_width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0i32;

    for word in text.split(' ') {
        let word_width: i32 = word.chars().map(char_width).sum();
        if current_width > 0 {
            if current_width + 4 + word_width > max_width as i32 {
                lines.push(current_line.clone());
                current_line.clear();
                current_width = 0;
            } else {
                current_line.push(' ');
                current_width += 4;
            }
        }
        current_line.push_str(word);
        current_width += word_width;
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    lines
}
