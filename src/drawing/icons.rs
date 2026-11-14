//! Nerd Font icon rendering functions.
//!
//! Provides functions to draw Nerd Font icons by name or codepoint,
//! icon placeholders, and icon grids onto raw RGBA pixel buffers.

use super::glyph::draw_glyph;
use super::glyph::glyph_fit_scale;
use super::progress::default_text_color;
use ab_glyph::Font;
use ab_glyph::Glyph;
use ab_glyph::PxScale;
use nerd_fonts_rs::fonts::nerd_font;
use smearor_commons_buffer::BufferViewMut;
use smearor_commons_buffer::Rgba;
use smearor_commons_color::Color;
use tracing::debug;

/// Draw a Nerd Font icon centered on the image, occupying the upper portion.
///
/// The icon name is resolved to a Nerd Font codepoint via the caller-provided
/// resolver function. If the font or icon is unavailable, a circular
/// placeholder is drawn instead.
pub(super) fn draw_nerd_font_icon(
    view: &mut BufferViewMut<Rgba>,
    icon_name: &str,
    is_active: bool,
    resolve_codepoint: impl Fn(&str) -> Option<char>,
    icon_color: Option<Color<f64>>,
) {
    let size = view.size();
    let font = match nerd_font() {
        Some(f) => f,
        None => {
            debug!("nerd-fonts-gtk: Nerd Font not available, drawing placeholder");
            draw_icon_placeholder(view, is_active);
            return;
        }
    };

    let codepoint = match resolve_codepoint(icon_name) {
        Some(c) => c,
        None => {
            debug!("nerd-fonts-gtk: unknown icon name '{}', drawing placeholder", icon_name);
            draw_icon_placeholder(view, is_active);
            return;
        }
    };

    let color: Color<u8> = icon_color.map(|c| c.into()).unwrap_or_else(|| default_text_color(is_active));

    let icon_size = size.x.min(size.y) as f32 * 0.6;
    let glyph_id = font.glyph_id(codepoint);
    let em_scale = glyph_fit_scale(font, glyph_id, icon_size);
    let scale = PxScale::from(em_scale);

    // Use outline px_bounds for precise centering (glyph_bounds includes side bearings)
    let probe = glyph_id.with_scale(scale);
    if let Some(outline) = font.outline_glyph(probe.clone()) {
        let pb = outline.px_bounds();
        let glyph_w = pb.width();
        let glyph_h = pb.height();
        let pos_x = (size.x as f32 - glyph_w) / 2.0 - pb.min.x;
        let pos_y = (size.y as f32 * 0.4) - glyph_h / 2.0 - pb.min.y;
        let glyph: Glyph = glyph_id.with_scale_and_position(scale, ab_glyph::point(pos_x, pos_y));
        draw_glyph(view, font, &glyph, color);
    } else {
        // Fallback: use glyph_bounds
        let bounds = font.glyph_bounds(&probe);
        let glyph_w = bounds.width();
        let glyph_h = bounds.height();
        let pos_x = (size.x as f32 - glyph_w) / 2.0 - bounds.min.x;
        let pos_y = (size.y as f32 * 0.4) - glyph_h / 2.0 - bounds.min.y;
        let glyph: Glyph = glyph_id.with_scale_and_position(scale, ab_glyph::point(pos_x, pos_y));
        draw_glyph(view, font, &glyph, color);
    }
}

/// Draw a simple circular placeholder when the font or icon is unavailable.
pub(super) fn draw_icon_placeholder(view: &mut BufferViewMut<Rgba>, is_active: bool) {
    let size = view.size();
    let stride = view.stride().as_usize();
    let pixels = view.data_mut();
    let center_x = size.x / 2;
    let center_y = size.x / 2;
    let radius = (size.x.min(size.y) / 3).max(8);
    let color = default_text_color(is_active);
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
pub(super) fn draw_nerd_font_codepoint(
    view: &mut BufferViewMut<Rgba>,
    codepoint: char,
    center_x: f32,
    center_y: f32,
    icon_size: f32,
    color: impl Into<Color<u8>>,
) {
    let font = match nerd_font() {
        Some(f) => f,
        None => return,
    };

    let glyph_id = font.glyph_id(codepoint);
    let em_scale = glyph_fit_scale(font, glyph_id, icon_size);
    let scale = PxScale::from(em_scale);

    // Center the glyph at (center_x, center_y) using outline px_bounds
    let color: Color<u8> = color.into();
    let probe = glyph_id.with_scale(scale);
    if let Some(outline) = font.outline_glyph(probe.clone()) {
        let pb = outline.px_bounds();
        let pos_x = center_x - pb.width() / 2.0 - pb.min.x;
        let pos_y = center_y - pb.height() / 2.0 - pb.min.y;
        let glyph: Glyph = glyph_id.with_scale_and_position(scale, ab_glyph::point(pos_x, pos_y));
        draw_glyph(view, font, &glyph, color);
    } else {
        let bounds = font.glyph_bounds(&probe);
        let pos_x = center_x - bounds.width() / 2.0 - bounds.min.x;
        let pos_y = center_y - bounds.height() / 2.0 - bounds.min.y;
        let glyph: Glyph = glyph_id.with_scale_and_position(scale, ab_glyph::point(pos_x, pos_y));
        draw_glyph(view, font, &glyph, color);
    }
}

/// Draw a 2x2 or 3x3 icon grid within the image area.
pub(super) fn draw_icon_grid(
    view: &mut BufferViewMut<Rgba>,
    icons: &[&str],
    grid_cols: u32,
    is_active: bool,
    resolve_codepoint: impl Fn(&str) -> Option<char>,
) {
    let size = view.size();
    let font = match nerd_font() {
        Some(f) => f,
        None => return,
    };

    let rows = (icons.len() as u32).div_ceil(grid_cols).max(1);
    let cell_w = size.x / grid_cols;
    let cell_h = size.y / rows;
    let icon_size = cell_w.min(cell_h) as f32 * 0.6;
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
        let scale = PxScale::from(em_scale);

        // Center the glyph in its cell using outline px_bounds
        let probe = glyph_id.with_scale(scale);
        if let Some(outline) = font.outline_glyph(probe.clone()) {
            let pb = outline.px_bounds();
            let pos_x = cx as f32 - pb.width() / 2.0 - pb.min.x;
            let pos_y = cy as f32 - pb.height() / 2.0 - pb.min.y;
            let glyph: Glyph = glyph_id.with_scale_and_position(scale, ab_glyph::point(pos_x, pos_y));
            draw_glyph(view, font, &glyph, color);
        } else {
            let bounds = font.glyph_bounds(&probe);
            let pos_x = cx as f32 - bounds.width() / 2.0 - bounds.min.x;
            let pos_y = cy as f32 - bounds.height() / 2.0 - bounds.min.y;
            let glyph: Glyph = glyph_id.with_scale_and_position(scale, ab_glyph::point(pos_x, pos_y));
            draw_glyph(view, font, &glyph, color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DrawOps;
    use nerd_fonts_rs::NerdFonts;
    use smearor_commons_buffer::PixelBuffer;
    use smearor_commons_geometry::Size;

    fn resolve_codepoint(name: &str) -> Option<char> {
        NerdFonts::codepoint(name).map(|c| c.as_char())
    }

    #[test]
    fn draw_icon_placeholder_draws_circle() {
        let size = Size::new(32, 32);
        let mut buf = PixelBuffer::new(size);
        buf.draw_icon_placeholder(false);

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
        buf.draw_icon_placeholder(true);

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
        buf.draw_nerd_font_codepoint('\u{F11B}', size.x as f32 / 2.0, size.y as f32 / 2.0, 48.0, color);

        let non_zero = buf.pixels().as_chunks::<4>().0.iter().filter(|px| px.iter().any(|&b| b != 0)).count();
        assert!(non_zero > 0, "at least some pixels should be rendered for a valid codepoint");
    }

    #[test]
    fn draw_nerd_font_codepoint_accepts_color_f64() {
        let size = Size::new(32, 32);
        let mut buf = PixelBuffer::new(size);
        let color_f64 = Color::<f64>::new(1.0, 0.0, 0.0);
        buf.draw_nerd_font_codepoint('\u{F11B}', size.x as f32 / 2.0, size.y as f32 / 2.0, 24.0, color_f64);

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
}
