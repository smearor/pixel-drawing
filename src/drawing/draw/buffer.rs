//! [`DrawOps`] implementations for the commons buffer types.

use super::ops::DrawOps;
use crate::drawing::RasterOps;
use crate::drawing::default_color::default_text_color;
use crate::drawing::glyph::centered_glyph;
use crate::drawing::glyph::glyph_fit_scale;
use crate::drawing::glyph::truncate_text_to_width;
use ab_glyph::Font;
use ab_glyph::Glyph;
use ab_glyph::PxScale;
use ab_glyph::ScaleFont;
use nerd_fonts_rs::fonts::label_font;
use nerd_fonts_rs::fonts::nerd_font;
use smearor_commons_buffer::BufferViewMut;
use smearor_commons_buffer::PixelBuffer;
use smearor_commons_buffer::Rgba;
use smearor_commons_color::Color;
use smearor_commons_geometry::Pixels;
use smearor_commons_geometry::Vec2;
use tracing::debug;

impl DrawOps for BufferViewMut<'_, Rgba> {
    /// Fill the entire view with a solid background color.
    ///
    /// Packed views reinterpret the `&mut [u8]` data as `&mut [u32]` (RGBA
    /// = 4 bytes = one `u32` in little-endian) and use `slice::fill()` for
    /// SIMD-vectorized writes. Strided views fill each row's pixel bytes
    /// and leave row padding untouched.
    fn fill_background(&mut self, color: impl Into<Color<u8>>) {
        let color: Color<u8> = color.into();
        if self.is_packed() {
            let len = self.metadata().required_len(4);
            let color_u32 = u32::from_le_bytes(color.into());
            let pixels_u32: &mut [u32] = bytemuck::cast_slice_mut(&mut self.data_mut()[..len]);
            pixels_u32.fill(color_u32);
        } else {
            let bytes: [u8; 4] = color.into();
            for y in 0..self.height().as_raw() {
                if let Some(row) = self.row_mut(Pixels::new(y)) {
                    row.as_chunks_mut::<4>().0.fill(bytes);
                }
            }
        }
    }

    /// Draw a Nerd Font icon centered on the image, occupying the upper portion.
    ///
    /// The icon name is resolved to a Nerd Font codepoint via the caller-provided
    /// resolver function. If the font or icon is unavailable, a circular
    /// placeholder is drawn instead.
    fn draw_nerd_font_icon(&mut self, icon_name: &str, is_active: bool, resolve_codepoint: impl Fn(&str) -> Option<char>, icon_color: Option<Color<f64>>) {
        let size = self.size();
        let font = match nerd_font() {
            Some(f) => f,
            None => {
                debug!("nerd-fonts-gtk: Nerd Font not available, drawing placeholder");
                self.draw_icon_placeholder(is_active, icon_color);
                return;
            }
        };

        let codepoint = match resolve_codepoint(icon_name) {
            Some(c) => c,
            None => {
                debug!("nerd-fonts-gtk: unknown icon name '{}', drawing placeholder", icon_name);
                self.draw_icon_placeholder(is_active, icon_color);
                return;
            }
        };

        let color: Color<u8> = icon_color.map(|c| c.into()).unwrap_or_else(|| default_text_color(is_active));

        let icon_size = Pixels::new(size.x.min(size.y) as f32 * 0.6);
        let glyph_id = font.glyph_id(codepoint);
        let em_scale = glyph_fit_scale(font, glyph_id, icon_size);
        let scale = PxScale::from(em_scale.as_raw());

        // Center the glyph bounds in the upper portion of the buffer
        let target = Vec2::<Pixels, f32>::new(size.x as f32 / 2.0, size.y as f32 * 0.4);
        let glyph = centered_glyph(font, glyph_id, scale, target);
        self.draw_glyph(font, &glyph, color);
    }

    /// Draw a simple circular placeholder when the font or icon is unavailable.
    fn draw_icon_placeholder(&mut self, is_active: bool, icon_color: Option<Color<f64>>) {
        let size = self.size();
        let stride = self.stride().as_usize();
        let pixels = self.data_mut();
        let center_x = size.x / 2;
        let center_y = size.x / 2;
        let radius = (size.x.min(size.y) / 3).max(8);
        let color: Color<u8> = icon_color.map(|c| c.into()).unwrap_or_else(|| default_text_color(is_active));
        let bytes: [u8; 4] = color.into();

        let cx = center_x as i32;
        let cy = (center_y as i32) - (size.y as i32 / 6);
        let r = radius as i32;

        for y in 0..size.y as i32 {
            for x in 0..size.x as i32 {
                let dx = x - cx;
                let dy = y - cy;
                if dx * dx + dy * dy <= r * r {
                    let idx = y as usize * stride + x as usize * 4;
                    pixels[idx..idx + 4].copy_from_slice(&bytes);
                }
            }
        }
    }

    /// Draw a Nerd Font icon by its Unicode codepoint at a custom position and size.
    fn draw_nerd_font_codepoint(&mut self, codepoint: char, center: Vec2<Pixels, f32>, icon_size: Pixels<f32>, color: impl Into<Color<u8>>) {
        let font = match nerd_font() {
            Some(f) => f,
            None => return,
        };

        let glyph_id = font.glyph_id(codepoint);
        let em_scale = glyph_fit_scale(font, glyph_id, icon_size);
        let scale = PxScale::from(em_scale.as_raw());

        let color: Color<u8> = color.into();
        let glyph = centered_glyph(font, glyph_id, scale, center);
        self.draw_glyph(font, &glyph, color);
    }

    /// Draw a 2x2 or 3x3 icon grid within the image area.
    fn draw_icon_grid(&mut self, icons: &[&str], grid_cols: u32, is_active: bool, resolve_codepoint: impl Fn(&str) -> Option<char>) {
        let size = self.size();
        let font = match nerd_font() {
            Some(f) => f,
            None => return,
        };

        let rows = (icons.len() as u32).div_ceil(grid_cols).max(1);
        let cell_w = size.x / grid_cols;
        let cell_h = size.y / rows;
        let icon_size = Pixels::new(cell_w.min(cell_h) as f32 * 0.6);
        let color = default_text_color(is_active);

        for (i, icon_name) in icons.iter().enumerate() {
            let codepoint = match resolve_codepoint(icon_name) {
                Some(c) => c,
                None => continue,
            };

            let col = (i as u32) % grid_cols;
            let row = (i as u32) / grid_cols;
            let cx = col * cell_w + cell_w / 2;
            let cy = row * cell_h + cell_h / 2;

            let glyph_id = font.glyph_id(codepoint);
            let em_scale = glyph_fit_scale(font, glyph_id, icon_size);
            let scale = PxScale::from(em_scale.as_raw());

            // Center the glyph bounds in its cell
            let glyph = centered_glyph(font, glyph_id, scale, Vec2::<Pixels, f32>::new(cx as f32, cy as f32));
            self.draw_glyph(font, &glyph, color);
        }
    }

    /// Draw a text label at the bottom of the image using the label font.
    ///
    /// Falls back to the Nerd Font if the label font is unavailable, and to
    /// a bitmap renderer if no font is available.
    fn draw_label_text(&mut self, text: &str, is_active: bool, text_color_override: Option<Color<f64>>) {
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

        let font_size = Pixels::new((size.y as f32 * 0.22).max(8.0));
        let scale = PxScale::from(font_size.as_raw());
        let scaled_font = font.as_scaled(scale);

        let max_width = Pixels::new(size.x as f32 * 0.9);
        let truncated = truncate_text_to_width(text, &scaled_font, max_width);
        if truncated.is_empty() {
            return;
        }

        let total_width = Pixels::new(truncated.chars().map(|c| scaled_font.h_advance(font.glyph_id(c))).sum::<f32>());
        let start_x = Pixels::new(((size.x as f32 - total_width.as_raw()) / 2.0).max(0.0));
        let baseline = Pixels::new(size.y as f32 * 0.88);

        let mut pen = Vec2::<Pixels, f32>::new(start_x.as_raw(), baseline.as_raw());
        for ch in truncated.chars() {
            let glyph_id = font.glyph_id(ch);
            let glyph: Glyph = glyph_id.with_scale_and_position(scale, pen);
            self.draw_glyph(font, &glyph, color);
            pen.x += scaled_font.h_advance(glyph_id);
        }
    }

    /// Draw text centered horizontally at a given y-baseline with a configurable font size.
    fn draw_text_centered(&mut self, text: &str, baseline_y: Pixels<f32>, font_size: Pixels<f32>, color: impl Into<Color<u8>>) {
        let size = self.size();
        let font = match label_font().or_else(nerd_font) {
            Some(f) => f,
            None => {
                self.draw_text_bitmap(text, false);
                return;
            }
        };

        let scale = PxScale::from(font_size.as_raw());
        let scaled_font = font.as_scaled(scale);
        let max_width = Pixels::new(size.x as f32 * 0.9);
        let truncated = truncate_text_to_width(text, &scaled_font, max_width);
        if truncated.is_empty() {
            return;
        }

        let total_width = Pixels::new(truncated.chars().map(|c| scaled_font.h_advance(font.glyph_id(c))).sum::<f32>());
        let start_x = Pixels::new(((size.x as f32 - total_width.as_raw()) / 2.0).max(0.0));

        let color: Color<u8> = color.into();

        let mut pen = Vec2::<Pixels, f32>::new(start_x.as_raw(), baseline_y.as_raw());
        for ch in truncated.chars() {
            let glyph_id = font.glyph_id(ch);
            let glyph: Glyph = glyph_id.with_scale_and_position(scale, pen);
            self.draw_glyph(font, &glyph, color);
            pen.x += scaled_font.h_advance(glyph_id);
        }
    }

    /// Draw a horizontal progress bar at the bottom of the image.
    fn draw_progress_bar(&mut self, value: f32, color: impl Into<Color<u8>>) {
        let size = self.size();
        let stride = self.stride().as_usize();
        let pixels = self.data_mut();
        let bar_height = 4u32;
        let color: Color<u8> = color.into();
        let bar_y = size.y.saturating_sub(bar_height + 2);
        let fill_width = ((size.x as f32 * value.clamp(0.0, 1.0)) as u32).min(size.x);
        let bytes: [u8; 4] = color.into();

        for y in bar_y..bar_y + bar_height {
            for x in 0..size.x {
                let idx = y as usize * stride + x as usize * 4;
                if x < fill_width {
                    pixels[idx..idx + 4].copy_from_slice(&bytes);
                } else {
                    pixels[idx..idx + 4].copy_from_slice(&[bytes[0] / 3, bytes[1] / 3, bytes[2] / 3, 255]);
                }
            }
        }
    }
}

/// An owned packed buffer draws through its packed view.
impl DrawOps for PixelBuffer<Rgba> {
    fn fill_background(&mut self, color: impl Into<Color<u8>>) {
        self.view_mut().fill_background(color);
    }

    fn draw_nerd_font_icon(&mut self, icon_name: &str, is_active: bool, resolve_codepoint: impl Fn(&str) -> Option<char>, icon_color: Option<Color<f64>>) {
        self.view_mut().draw_nerd_font_icon(icon_name, is_active, resolve_codepoint, icon_color);
    }

    fn draw_icon_placeholder(&mut self, is_active: bool, icon_color: Option<Color<f64>>) {
        self.view_mut().draw_icon_placeholder(is_active, icon_color);
    }

    fn draw_nerd_font_codepoint(&mut self, codepoint: char, center: Vec2<Pixels, f32>, icon_size: Pixels<f32>, color: impl Into<Color<u8>>) {
        self.view_mut().draw_nerd_font_codepoint(codepoint, center, icon_size, color);
    }

    fn draw_icon_grid(&mut self, icons: &[&str], grid_cols: u32, is_active: bool, resolve_codepoint: impl Fn(&str) -> Option<char>) {
        self.view_mut().draw_icon_grid(icons, grid_cols, is_active, resolve_codepoint);
    }

    fn draw_label_text(&mut self, text: &str, is_active: bool, text_color_override: Option<Color<f64>>) {
        self.view_mut().draw_label_text(text, is_active, text_color_override);
    }

    fn draw_text_centered(&mut self, text: &str, baseline_y: Pixels<f32>, font_size: Pixels<f32>, color: impl Into<Color<u8>>) {
        self.view_mut().draw_text_centered(text, baseline_y, font_size, color);
    }

    fn draw_progress_bar(&mut self, value: f32, color: impl Into<Color<u8>>) {
        self.view_mut().draw_progress_bar(value, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drawing::default_color::default_text_color;
    use nerd_fonts_rs::NerdFonts;
    use smearor_commons_buffer::BufferMetadata;
    use smearor_commons_buffer::Stride;
    use smearor_commons_geometry::Size;

    fn resolve_codepoint(name: &str) -> Option<char> {
        NerdFonts::codepoint(name).map(|c| c.as_char())
    }

    #[test]
    fn draw_icon_placeholder_draws_circle() {
        let size = Size::new(32, 32);
        let mut buf = PixelBuffer::new(size);
        buf.draw_icon_placeholder(false, None);

        let color = default_text_color(false);
        let bytes: [u8; 4] = color.into();
        let center_x = size.x / 2;
        let center_y = size.x / 2;
        let cx = center_x as i32;
        let cy = (center_y as i32) - (size.y as i32 / 6);
        let idx = (cy as u32 * size.x + cx as u32) as usize * 4;
        assert_eq!(&buf.pixels()[idx..idx + 4], &bytes, "center pixel should be filled by placeholder circle");

        let dx = (size.x.min(size.y) / 3).max(8) as i32 + 2;
        let outside_idx = (cy as u32 * size.x + (cx + dx) as u32) as usize * 4;
        assert_eq!(&buf.pixels()[outside_idx..outside_idx + 4], &[0, 0, 0, 0], "pixel outside circle should be untouched");
    }

    #[test]
    fn draw_icon_placeholder_active_uses_active_color() {
        let size = Size::new(16, 16);
        let mut buf = PixelBuffer::new(size);
        buf.draw_icon_placeholder(true, None);

        let active_color = default_text_color(true);
        let bytes: [u8; 4] = active_color.into();
        let center_x = size.x / 2;
        let center_y = size.x / 2;
        let cx = center_x as i32;
        let cy = (center_y as i32) - (size.y as i32 / 6);
        let idx = (cy as u32 * size.x + cx as u32) as usize * 4;
        assert_eq!(&buf.pixels()[idx..idx + 4], &bytes, "center pixel should use active color");
    }

    #[test]
    fn draw_nerd_font_codepoint_renders_pixels() {
        let size = Size::new(64, 64);
        let mut buf = PixelBuffer::new(size);
        let color = Color::<u8>::new(255, 255, 255);
        buf.draw_nerd_font_codepoint('\u{F11B}', Vec2::<Pixels, f32>::new(size.x as f32 / 2.0, size.y as f32 / 2.0), Pixels::new(48.0), color);

        let non_zero = buf.pixels().as_chunks::<4>().0.iter().filter(|px| px.iter().any(|&b| b != 0)).count();
        assert!(non_zero > 0, "at least some pixels should be rendered for a valid codepoint");
    }

    #[test]
    fn draw_nerd_font_codepoint_accepts_color_f64() {
        let size = Size::new(32, 32);
        let mut buf = PixelBuffer::new(size);
        let color_f64 = Color::<f64>::new(1.0, 0.0, 0.0);
        buf.draw_nerd_font_codepoint('\u{F11B}', Vec2::<Pixels, f32>::new(size.x as f32 / 2.0, size.y as f32 / 2.0), Pixels::new(24.0), color_f64);

        let non_zero = buf.pixels().as_chunks::<4>().0.iter().filter(|px| px.iter().any(|&b| b != 0)).count();
        assert!(non_zero > 0, "at least some pixels should be rendered with f64 color");
    }

    #[test]
    fn draw_nerd_font_icon_with_valid_name_renders() {
        let size = Size::new(64, 64);
        let mut buf = PixelBuffer::new(size);
        buf.draw_nerd_font_icon("nf-fa-gamepad", true, resolve_codepoint, Some(Color::<f64>::new(1.0, 1.0, 1.0)));

        let non_zero = buf.pixels().as_chunks::<4>().0.iter().filter(|px| px.iter().any(|&b| b != 0)).count();
        assert!(non_zero > 0, "at least some pixels should be rendered for a valid icon name");
    }

    #[test]
    fn draw_nerd_font_icon_with_unknown_name_draws_placeholder() {
        let size = Size::new(32, 32);
        let mut buf = PixelBuffer::new(size);
        buf.draw_nerd_font_icon("nf-xx-nonexistent", false, resolve_codepoint, None);

        let placeholder_color = default_text_color(false);
        let bytes: [u8; 4] = placeholder_color.into();
        let center_x = size.x / 2;
        let center_y = size.x / 2;
        let cx = center_x as i32;
        let cy = (center_y as i32) - (size.y as i32 / 6);
        let idx = (cy as u32 * size.x + cx as u32) as usize * 4;
        assert_eq!(&buf.pixels()[idx..idx + 4], &bytes, "placeholder should be drawn for unknown icon name");
    }

    #[test]
    fn draw_nerd_font_icon_placeholder_uses_color_override() {
        let size = Size::new(32, 32);
        let mut buf = PixelBuffer::new(size);
        let red = Color::<f64>::new(1.0, 0.0, 0.0);
        buf.draw_nerd_font_icon("nf-xx-nonexistent", false, resolve_codepoint, Some(red));

        let bytes: [u8; 4] = Color::<u8>::from(red).into();
        let cx = (size.x / 2) as i32;
        let cy = (size.x / 2) as i32 - (size.y as i32 / 6);
        let idx = (cy as u32 * size.x + cx as u32) as usize * 4;
        assert_eq!(&buf.pixels()[idx..idx + 4], &bytes, "placeholder should honor the icon color override");
    }

    #[test]
    fn draw_nerd_font_icon_with_none_color_uses_default() {
        let size = Size::new(64, 64);
        let mut buf = PixelBuffer::new(size);
        buf.draw_nerd_font_icon("nf-fa-gamepad", true, resolve_codepoint, None);

        let non_zero = buf.pixels().as_chunks::<4>().0.iter().filter(|px| px.iter().any(|&b| b != 0)).count();
        assert!(non_zero > 0, "at least some pixels should be rendered with default active color");
    }

    #[test]
    fn draw_icon_grid_renders_multiple_icons() {
        let size = Size::new(96, 96);
        let mut buf = PixelBuffer::new(size);
        let icons = ["nf-fa-gamepad", "nf-fa-star", "nf-fa-heart", "nf-fa-bug"];
        buf.draw_icon_grid(&icons, 2, true, resolve_codepoint);

        let non_zero = buf.pixels().as_chunks::<4>().0.iter().filter(|px| px.iter().any(|&b| b != 0)).count();
        assert!(non_zero > 0, "at least some pixels should be rendered for icon grid");
    }

    #[test]
    fn draw_icon_grid_with_empty_icons_does_not_panic() {
        let size = Size::new(32, 32);
        let mut buf = PixelBuffer::new(size);
        let icons: [&str; 0] = [];
        buf.draw_icon_grid(&icons, 2, false, resolve_codepoint);

        assert!(buf.pixels().iter().all(|&p| p == 0), "no pixels should be rendered for empty icon list");
    }

    #[test]
    fn draw_icon_grid_with_unknown_icons_skips_them() {
        let size = Size::new(64, 64);
        let mut buf = PixelBuffer::new(size);
        let icons = ["nf-xx-nonexistent", "nf-yy-also-nonexistent"];
        buf.draw_icon_grid(&icons, 2, false, resolve_codepoint);

        assert!(buf.pixels().iter().all(|&p| p == 0), "no pixels should be rendered when all icons are unknown");
    }

    #[test]
    fn draw_label_text_renders_pixels() {
        let size = Size::new(96, 96);
        let mut buf = PixelBuffer::new(size);
        buf.draw_label_text("hello", true, None);

        let non_zero = buf.pixels().as_chunks::<4>().0.iter().filter(|px| px.iter().any(|&b| b != 0)).count();
        assert!(non_zero > 0, "at least some pixels should be rendered for label text");
    }

    #[test]
    fn draw_label_text_with_color_override() {
        let size = Size::new(96, 96);
        let mut buf = PixelBuffer::new(size);
        let color = Color::<f64>::new(1.0, 0.0, 0.0);
        buf.draw_label_text("test", false, Some(color));

        let non_zero = buf.pixels().as_chunks::<4>().0.iter().filter(|px| px.iter().any(|&b| b != 0)).count();
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
        buf.draw_text_centered("center", Pixels::new(size.y as f32 / 2.0), Pixels::new(16.0), color);

        let non_zero = buf.pixels().as_chunks::<4>().0.iter().filter(|px| px.iter().any(|&b| b != 0)).count();
        assert!(non_zero > 0, "at least some pixels should be rendered for centered text");
    }

    #[test]
    fn draw_text_centered_accepts_color_f64() {
        let size = Size::new(64, 64);
        let mut buf = PixelBuffer::new(size);
        let color_f64 = Color::<f64>::new(0.0, 1.0, 0.0);
        buf.draw_text_centered("abc", Pixels::new(32.0), Pixels::new(12.0), color_f64);

        let non_zero = buf.pixels().as_chunks::<4>().0.iter().filter(|px| px.iter().any(|&b| b != 0)).count();
        assert!(non_zero > 0, "at least some pixels should be rendered with f64 color");
    }

    #[test]
    fn draw_text_centered_empty_string_does_not_render() {
        let size = Size::new(32, 32);
        let mut buf = PixelBuffer::new(size);
        let color = Color::<u8>::new(255, 255, 255);
        buf.draw_text_centered("", Pixels::new(16.0), Pixels::new(12.0), color);

        assert!(buf.pixels().iter().all(|&p| p == 0), "no pixels should be rendered for empty string");
    }

    #[test]
    fn draw_text_centered_truncates_long_text() {
        let size = Size::new(32, 32);
        let mut buf = PixelBuffer::new(size);
        let color = Color::<u8>::new(255, 255, 255);
        let long_text = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
        buf.draw_text_centered(long_text, Pixels::new(16.0), Pixels::new(12.0), color);

        let non_zero = buf.pixels().as_chunks::<4>().0.iter().filter(|px| px.iter().any(|&b| b != 0)).count();
        assert!(non_zero > 0, "truncated text should still render some pixels");
    }

    #[test]
    fn draw_progress_bar_full_fill() {
        let size = Size::new(8, 8);
        let mut buf = PixelBuffer::new(size);
        let color = Color::<u8>::new(255, 0, 0);
        buf.draw_progress_bar(1.0, color);

        let bar_height = 4u32;
        let bar_y = size.y.saturating_sub(bar_height + 2);
        let bytes: [u8; 4] = color.into();
        let pixels = buf.pixels();
        for y in bar_y..bar_y + bar_height {
            for x in 0..size.x {
                let idx = (y * size.x + x) as usize * 4;
                assert_eq!(&pixels[idx..idx + 4], &bytes, "pixel at ({},{}) should be fully filled", x, y);
            }
        }
    }

    #[test]
    fn draw_progress_bar_half_fill() {
        let size = Size::new(8, 8);
        let mut buf = PixelBuffer::new(size);
        let color = Color::<u8>::new(255, 0, 0);
        buf.draw_progress_bar(0.5, color);

        let bar_height = 4u32;
        let bar_y = size.y.saturating_sub(bar_height + 2);
        let bytes: [u8; 4] = color.into();
        let fill_width = (size.x as f32 * 0.5) as u32;
        let pixels = buf.pixels();
        for y in bar_y..bar_y + bar_height {
            for x in 0..size.x {
                let idx = (y * size.x + x) as usize * 4;
                if x < fill_width {
                    assert_eq!(&pixels[idx..idx + 4], &bytes, "pixel at ({},{}) should be filled", x, y);
                } else {
                    assert_eq!(pixels[idx], bytes[0] / 3, "pixel at ({},{}) should be dimmed", x, y);
                    assert_eq!(pixels[idx + 3], 255, "pixel at ({},{}) should be opaque", x, y);
                }
            }
        }
    }

    #[test]
    fn draw_progress_bar_zero_fill() {
        let size = Size::new(4, 8);
        let mut buf = PixelBuffer::new(size);
        let color = Color::<u8>::new(100, 200, 50);
        buf.draw_progress_bar(0.0, color);

        let bar_height = 4u32;
        let bar_y = size.y.saturating_sub(bar_height + 2);
        let bytes: [u8; 4] = color.into();
        let pixels = buf.pixels();
        for y in bar_y..bar_y + bar_height {
            for x in 0..size.x {
                let idx = (y * size.x + x) as usize * 4;
                assert_eq!(pixels[idx], bytes[0] / 3, "pixel at ({},{}) should be dimmed", x, y);
                assert_eq!(pixels[idx + 3], 255, "pixel at ({},{}) should be opaque", x, y);
            }
        }
    }

    #[test]
    fn draw_progress_bar_clamps_value_above_one() {
        let size = Size::new(4, 8);
        let mut buf = PixelBuffer::new(size);
        let color = Color::<u8>::new(255, 255, 255);
        buf.draw_progress_bar(2.0, color);

        let bar_height = 4u32;
        let bar_y = size.y.saturating_sub(bar_height + 2);
        let bytes: [u8; 4] = color.into();
        let pixels = buf.pixels();
        for y in bar_y..bar_y + bar_height {
            for x in 0..size.x {
                let idx = (y * size.x + x) as usize * 4;
                assert_eq!(&pixels[idx..idx + 4], &bytes, "pixel at ({},{}) should be fully filled (clamped)", x, y);
            }
        }
    }

    #[test]
    fn draw_progress_bar_clamps_negative_value() {
        let size = Size::new(4, 8);
        let mut buf = PixelBuffer::new(size);
        let color = Color::<u8>::new(255, 255, 255);
        buf.draw_progress_bar(-1.0, color);

        let bar_height = 4u32;
        let bar_y = size.y.saturating_sub(bar_height + 2);
        let pixels = buf.pixels();
        for y in bar_y..bar_y + bar_height {
            for x in 0..size.x {
                let idx = (y * size.x + x) as usize * 4;
                assert_eq!(pixels[idx + 3], 255, "pixel at ({},{}) should be opaque", x, y);
            }
        }
    }

    #[test]
    fn draw_progress_bar_accepts_color_f64() {
        let size = Size::new(4, 8);
        let mut buf = PixelBuffer::new(size);
        let color_f64 = Color::<f64>::new(1.0, 0.0, 0.0);
        buf.draw_progress_bar(1.0, color_f64);

        let bar_height = 4u32;
        let bar_y = size.y.saturating_sub(bar_height + 2);
        let expected: [u8; 4] = Color::<u8>::from(color_f64).into();
        let pixels = buf.pixels();
        for y in bar_y..bar_y + bar_height {
            for x in 0..size.x {
                let idx = (y * size.x + x) as usize * 4;
                assert_eq!(&pixels[idx..idx + 4], &expected, "pixel at ({},{}) should match f64-converted color", x, y);
            }
        }
    }

    #[test]
    fn fill_background_fills_all_pixels() {
        let mut buf = PixelBuffer::new(Size::new(4, 3));
        let color = Color::<f64>::new(10.0 / 255.0, 20.0 / 255.0, 30.0 / 255.0);
        let expected: [u8; 4] = Color::<u8>::from(color).into();
        buf.fill_background(color);

        for px in buf.pixels().as_chunks::<4>().0.iter() {
            assert_eq!(px, &expected);
        }
    }

    #[test]
    fn fill_background_single_pixel() {
        let mut buf = PixelBuffer::new(Size::new(1, 1));
        let color = Color::<f64>::new_rgba(1.0, 0.0, 128.0 / 255.0, 200.0 / 255.0);
        let expected: [u8; 4] = Color::<u8>::from(color).into();
        buf.fill_background(color);
        assert_eq!(buf.pixels(), &expected[..]);
    }

    #[test]
    fn fill_background_strided_view_leaves_padding_untouched() {
        // 2x2 pixels, stride 16 bytes (8 padding bytes per row):
        // required = (2 - 1) * 16 + 2 * 4 = 24, plus a trailing slice tail.
        let mut data = vec![0u8; 32];
        let metadata = BufferMetadata::new(Pixels::new(2), Pixels::new(2), Stride::new(16));
        let color = Color::<u8>::new(200, 100, 50);

        BufferViewMut::<Rgba>::from_raw(&mut data, metadata).unwrap().fill_background(color);

        let bytes: [u8; 4] = color.into();
        // row 0 pixels at [0..8), padding at [8..16)
        assert_eq!(&data[0..4], &bytes);
        assert_eq!(&data[4..8], &bytes);
        assert!(data[8..16].iter().all(|&b| b == 0), "row padding must stay untouched");
        // row 1 pixels at [16..24), memory beyond the view extent untouched
        assert_eq!(&data[16..20], &bytes);
        assert_eq!(&data[20..24], &bytes);
        assert!(data[24..].iter().all(|&b| b == 0), "memory beyond the view extent must stay untouched");
    }

    #[test]
    fn fill_background_preserves_color_via_u32_round_trip() {
        let mut buf = PixelBuffer::new(Size::new(2, 2));
        let color = Color::<f64>::new_rgba(0xAB as f64 / 255.0, 0xCD as f64 / 255.0, 0xEF as f64 / 255.0, 0x12 as f64 / 255.0);
        let expected_bytes: [u8; 4] = Color::<u8>::from(color).into();
        buf.fill_background(color);

        let pixels_u32: &[u32] = bytemuck::cast_slice(buf.pixels());
        let expected = u32::from_le_bytes(expected_bytes);
        for &px in pixels_u32 {
            assert_eq!(px, expected);
        }
    }
}
