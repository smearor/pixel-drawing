//! Text label rendering functions.
//!
//! Provides functions to draw text labels and centered text onto raw RGBA
//! pixel buffers using `ab_glyph`, with font fallback chains.

use super::glyph::truncate_text_to_width;
use super::progress::default_text_color;
use crate::color::Color;
use crate::pixel_buffer::PixelBuffer;
use ab_glyph::Font;
use ab_glyph::Glyph;
use ab_glyph::PxScale;
use ab_glyph::ScaleFont;
use nerd_fonts_rs::fonts::label_font;
use nerd_fonts_rs::fonts::nerd_font;
use tracing::debug;

impl PixelBuffer {
    /// Draw a text label at the bottom of the image using the label font.
    ///
    /// Falls back to the Nerd Font if the label font is unavailable, and to
    /// a bitmap renderer if no font is available.
    pub fn draw_label_text(&mut self, text: &str, is_active: bool, text_color_override: Option<Color<f64>>) {
        let size = self.size();
        let font = match label_font() {
            Some(f) => f,
            None => {
                debug!("nerd-fonts-gtk: label font not available, trying Nerd Font");
                match nerd_font() {
                    Some(f) => f,
                    None => {
                        debug!("nerd-fonts-gtk: no font available for label, using bitmap fallback");
                        self.draw_text_bitmap(text, is_active);
                        return;
                    }
                }
            }
        };

        let color: Color<u8> = text_color_override.map(|c| c.into()).unwrap_or_else(|| default_text_color(is_active));

        let font_size = (size.height() as f32 * 0.22).max(8.0);
        let scale = PxScale::from(font_size);
        let scaled_font = font.as_scaled(scale);

        let max_width = size.width() as f32 * 0.9;
        let truncated = truncate_text_to_width(text, &scaled_font, max_width);
        if truncated.is_empty() {
            return;
        }

        let total_width: f32 = truncated.chars().map(|c| scaled_font.h_advance(font.glyph_id(c))).sum();
        let start_x = ((size.width() as f32 - total_width) / 2.0).max(0.0);
        let baseline = size.height() as f32 * 0.88;

        let mut pen_x = start_x;
        for ch in truncated.chars() {
            let glyph_id = font.glyph_id(ch);
            let glyph: Glyph = glyph_id.with_scale_and_position(scale, ab_glyph::point(pen_x, baseline));
            self.draw_glyph(font, &glyph, color);
            pen_x += scaled_font.h_advance(glyph_id);
        }
    }

    /// Draw text centered horizontally at a given y-baseline with a configurable font size.
    pub fn draw_text_centered(&mut self, text: &str, baseline_y: f32, font_size: f32, color: impl Into<Color<u8>>) {
        let size = self.size();
        let font = match label_font().or_else(nerd_font) {
            Some(f) => f,
            None => {
                self.draw_text_bitmap(text, false);
                return;
            }
        };

        let scale = PxScale::from(font_size);
        let scaled_font = font.as_scaled(scale);
        let max_width = size.width() as f32 * 0.9;
        let truncated = truncate_text_to_width(text, &scaled_font, max_width);
        if truncated.is_empty() {
            return;
        }

        let total_width: f32 = truncated.chars().map(|c| scaled_font.h_advance(font.glyph_id(c))).sum();
        let start_x = ((size.width() as f32 - total_width) / 2.0).max(0.0);

        let color: Color<u8> = color.into();

        let mut pen_x = start_x;
        for ch in truncated.chars() {
            let glyph_id = font.glyph_id(ch);
            let glyph: Glyph = glyph_id.with_scale_and_position(scale, ab_glyph::point(pen_x, baseline_y));
            self.draw_glyph(font, &glyph, color);
            pen_x += scaled_font.h_advance(glyph_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::size::Size;

    #[test]
    fn draw_label_text_renders_pixels() {
        let size = Size::new(96, 96);
        let mut buf = PixelBuffer::new(size);
        buf.draw_label_text("hello", true, None);

        let non_zero = buf.pixels().chunks_exact(4).filter(|px| px.iter().any(|&b| b != 0)).count();
        assert!(non_zero > 0, "at least some pixels should be rendered for label text");
    }

    #[test]
    fn draw_label_text_with_color_override() {
        let size = Size::new(96, 96);
        let mut buf = PixelBuffer::new(size);
        let color = Color::<f64>::new(1.0, 0.0, 0.0);
        buf.draw_label_text("test", false, Some(color));

        let non_zero = buf.pixels().chunks_exact(4).filter(|px| px.iter().any(|&b| b != 0)).count();
        assert!(non_zero > 0, "at least some pixels should be rendered with color override");
    }

    #[test]
    fn draw_label_text_empty_string_does_not_render() {
        let size = Size::new(32, 32);
        let mut buf = PixelBuffer::new(size);
        buf.draw_label_text("", false, None);

        assert!(buf.pixels().iter().all(|&p| p == 0), "no pixels should be rendered for empty string");
    }

    #[test]
    fn draw_text_centered_renders_pixels() {
        let size = Size::new(96, 96);
        let mut buf = PixelBuffer::new(size);
        let color = Color::<u8>::new(255, 255, 255);
        buf.draw_text_centered("center", size.height() as f32 / 2.0, 16.0, color);

        let non_zero = buf.pixels().chunks_exact(4).filter(|px| px.iter().any(|&b| b != 0)).count();
        assert!(non_zero > 0, "at least some pixels should be rendered for centered text");
    }

    #[test]
    fn draw_text_centered_accepts_color_f64() {
        let size = Size::new(64, 64);
        let mut buf = PixelBuffer::new(size);
        let color_f64 = Color::<f64>::new(0.0, 1.0, 0.0);
        buf.draw_text_centered("abc", 32.0, 12.0, color_f64);

        let non_zero = buf.pixels().chunks_exact(4).filter(|px| px.iter().any(|&b| b != 0)).count();
        assert!(non_zero > 0, "at least some pixels should be rendered with f64 color");
    }

    #[test]
    fn draw_text_centered_empty_string_does_not_render() {
        let size = Size::new(32, 32);
        let mut buf = PixelBuffer::new(size);
        let color = Color::<u8>::new(255, 255, 255);
        buf.draw_text_centered("", 16.0, 12.0, color);

        assert!(buf.pixels().iter().all(|&p| p == 0), "no pixels should be rendered for empty string");
    }

    #[test]
    fn draw_text_centered_truncates_long_text() {
        let size = Size::new(32, 32);
        let mut buf = PixelBuffer::new(size);
        let color = Color::<u8>::new(255, 255, 255);
        let long_text = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
        buf.draw_text_centered(long_text, 16.0, 12.0, color);

        let non_zero = buf.pixels().chunks_exact(4).filter(|px| px.iter().any(|&b| b != 0)).count();
        assert!(non_zero > 0, "truncated text should still render some pixels");
    }
}
