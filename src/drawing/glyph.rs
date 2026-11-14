//! Glyph rendering and font metric utilities.
//!
//! Provides single-glyph rasterization with alpha blending, glyph-to-pixel
//! scale computation (with caching), and text width truncation.

use super::pixel::blend_pixel;
use ab_glyph::Font;
use ab_glyph::FontVec;
use ab_glyph::Glyph;
use ab_glyph::PxScaleFont;
use ab_glyph::ScaleFont;
use ahash::AHashMap;
use smearor_commons_buffer::BufferViewMut;
use smearor_commons_buffer::Rgba;
use smearor_commons_color::Color;
use std::cell::RefCell;

thread_local! {
    /// Cache for `glyph_fit_scale` results, keyed by (glyph_id, target_px bits).
    /// Avoids redundant glyph probing on repeated renders of the same icon.
    static GLYPH_SCALE_CACHE: RefCell<AHashMap<(ab_glyph::GlyphId, u32), f32>> =
        RefCell::new(AHashMap::new());
}

/// Compute the em-scale needed so that a glyph's largest dimension (width or
/// height) equals `target_px` pixels.
///
/// `PxScale::from(x)` sets the **em** size, not the glyph size. Nerd Font
/// symbols typically occupy only a small fraction of the em square, so a
/// 76px em can produce a ~12px glyph. This function probes the glyph at 1px
/// em, measures its design-space bounds, and returns the em-scale that makes
/// the glyph's largest dimension match `target_px`.
pub(super) fn glyph_fit_scale(font: &FontVec, glyph_id: ab_glyph::GlyphId, target_px: f32) -> f32 {
    let key = (glyph_id, target_px.to_bits());

    let cached = GLYPH_SCALE_CACHE.with(|c| c.borrow().get(&key).copied());
    if let Some(scale) = cached {
        return scale;
    }

    let scale = glyph_fit_scale_uncached(font, glyph_id, target_px);

    GLYPH_SCALE_CACHE.with(|c| c.borrow_mut().insert(key, scale));

    scale
}

fn glyph_fit_scale_uncached(font: &FontVec, glyph_id: ab_glyph::GlyphId, target_px: f32) -> f32 {
    // Probe at a large enough scale to get accurate pixel measurements.
    // outline_glyph rounds to integer pixels, so scale=1.0 gives 0 or 1 px
    // which is too imprecise. Use 100.0 em and scale the result.
    let probe_scale = 100.0_f32;
    let probe = glyph_id.with_scale(probe_scale);
    if let Some(outline) = font.outline_glyph(probe) {
        let pb = outline.px_bounds();
        let gw = pb.width().max(0.0);
        let gh = pb.height().max(0.0);
        let max_g = gw.max(gh);
        if max_g > 0.01 {
            // max_g is the pixel size at probe_scale em.
            // To get target_px pixels: em_scale = target_px * probe_scale / max_g
            return target_px * probe_scale / max_g;
        }
    }
    // Fallback: use glyph_bounds (design-space, not pixel-accurate)
    let probe = glyph_id.with_scale(1.0);
    let bounds = font.glyph_bounds(&probe);
    let gw = bounds.width().max(0.0);
    let gh = bounds.height().max(0.0);
    let max_g = gw.max(gh);
    if max_g > 0.01 { target_px / max_g } else { target_px }
}

/// Truncate text to fit within a maximum pixel width.
///
/// Uses incremental width tracking — O(n) in the number of characters.
pub(super) fn truncate_text_to_width(text: &str, scaled_font: &PxScaleFont<&FontVec>, max_width: f32) -> String {
    let mut result = String::new();
    let mut current_width: f32 = 0.0;
    for ch in text.chars() {
        let glyph_id = scaled_font.glyph_id(ch);
        let char_width = scaled_font.h_advance(glyph_id);
        if current_width + char_width > max_width {
            break;
        }
        current_width += char_width;
        result.push(ch);
    }
    result
}

/// Draw a single glyph onto the view with alpha blending.
pub(super) fn draw_glyph(view: &mut BufferViewMut<Rgba>, font: &FontVec, glyph: &Glyph, color: Color<u8>) {
    let size = view.size();
    let stride = view.stride().as_usize();
    let pixels = view.data_mut();
    if let Some(outline) = font.outline_glyph(glyph.clone()) {
        let bounds = outline.px_bounds();
        let px = bounds.min.x.round() as i32;
        let py = bounds.min.y.round() as i32;
        let bytes: [u8; 4] = color.into();

        outline.draw(|gx, gy, coverage| {
            let x = px + gx as i32;
            let y = py + gy as i32;
            if x >= 0 && x < size.x as i32 && y >= 0 && y < size.y as i32 {
                let idx = y as usize * stride + x as usize * 4;
                let alpha = (coverage * bytes[3] as f32) as u8;
                if alpha > 0 {
                    let blended = blend_pixel(&pixels[idx..idx + 4], &bytes, alpha);
                    pixels[idx..idx + 4].copy_from_slice(&blended);
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ab_glyph::PxScale;
    use nerd_fonts_rs::fonts::nerd_font;

    #[test]
    fn glyph_fit_scale_returns_consistent_results() {
        let font = nerd_font().expect("nerd font should be loaded");
        let glyph_id = font.glyph_id('\u{F11B}');
        let target_px = 48.0_f32;

        let scale1 = glyph_fit_scale(font, glyph_id, target_px);
        let scale2 = glyph_fit_scale(font, glyph_id, target_px);

        assert_eq!(scale1.to_bits(), scale2.to_bits());
        assert!(scale1 > 0.0);
    }

    #[test]
    fn glyph_fit_scale_proportional_to_target() {
        let font = nerd_font().expect("nerd font should be loaded");
        let glyph_id = font.glyph_id('\u{F11B}');

        let scale_24 = glyph_fit_scale(font, glyph_id, 24.0);
        let scale_48 = glyph_fit_scale(font, glyph_id, 48.0);

        let ratio = scale_48 / scale_24;
        assert!((ratio - 2.0).abs() < 0.01, "doubling target_px should approximately double the scale, got ratio {}", ratio);
    }

    #[test]
    fn glyph_fit_scale_positive_for_valid_glyph() {
        let font = nerd_font().expect("nerd font should be loaded");
        let glyph_id = font.glyph_id('\u{F001}');
        let scale = glyph_fit_scale(font, glyph_id, 32.0);
        assert!(scale > 0.0, "scale should be positive for a valid glyph");
    }

    #[test]
    fn truncate_text_to_width_returns_full_text_when_fits() {
        let font = nerd_font().expect("nerd font should be loaded");
        let scale = PxScale::from(16.0);
        let scaled_font = font.as_scaled(scale);
        let text = "hi";

        let result = truncate_text_to_width(text, &scaled_font, 1000.0);
        assert_eq!(result, "hi");
    }

    #[test]
    fn truncate_text_to_width_truncates_when_too_long() {
        let font = nerd_font().expect("nerd font should be loaded");
        let scale = PxScale::from(16.0);
        let scaled_font = font.as_scaled(scale);
        let text = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

        let result = truncate_text_to_width(text, &scaled_font, 20.0);
        assert!(result.len() < text.len(), "text should be truncated to fit within 20px");
        assert!(!result.is_empty(), "truncated text should not be empty");
    }

    #[test]
    fn truncate_text_to_width_empty_string_returns_empty() {
        let font = nerd_font().expect("nerd font should be loaded");
        let scale = PxScale::from(16.0);
        let scaled_font = font.as_scaled(scale);

        let result = truncate_text_to_width("", &scaled_font, 100.0);
        assert!(result.is_empty());
    }

    #[test]
    fn truncate_text_to_width_zero_max_width_returns_empty() {
        let font = nerd_font().expect("nerd font should be loaded");
        let scale = PxScale::from(16.0);
        let scaled_font = font.as_scaled(scale);

        let result = truncate_text_to_width("hello", &scaled_font, 0.0);
        assert!(result.is_empty(), "no characters should fit in zero width");
    }
}
