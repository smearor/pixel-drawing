//! Glyph rendering and font metric utilities.
//!
//! Provides single-glyph rasterization with alpha blending, glyph-to-pixel
//! scale computation (with caching), and text width truncation.

use ab_glyph::Font;
use ab_glyph::FontVec;
use ab_glyph::Glyph;
use ab_glyph::GlyphId;
use ab_glyph::PxScale;
use ab_glyph::PxScaleFont;
use ab_glyph::ScaleFont;
use ahash::AHashMap;
use smearor_commons_geometry::Pixels;
use smearor_commons_geometry::Rect;
use smearor_commons_geometry::Vec2;
use std::cell::RefCell;

thread_local! {
    /// Cache for `glyph_fit_scale` results, keyed by (glyph_id, target_px bits).
    /// Avoids redundant glyph probing on repeated renders of the same icon.
    static GLYPH_SCALE_CACHE: RefCell<AHashMap<(GlyphId, u32), Pixels<f32>>> =
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
pub(super) fn glyph_fit_scale(font: &FontVec, glyph_id: GlyphId, target_px: Pixels<f32>) -> Pixels<f32> {
    let key = (glyph_id, target_px.as_raw().to_bits());

    let cached = GLYPH_SCALE_CACHE.with(|c| c.borrow().get(&key).copied());
    if let Some(scale) = cached {
        return scale;
    }

    let scale = glyph_fit_scale_uncached(font, glyph_id, target_px);

    GLYPH_SCALE_CACHE.with(|c| c.borrow_mut().insert(key, scale));

    scale
}

fn glyph_fit_scale_uncached(font: &FontVec, glyph_id: GlyphId, target_px: Pixels<f32>) -> Pixels<f32> {
    // Probe at a large enough scale to get accurate pixel measurements.
    // outline_glyph rounds to integer pixels, so scale=1.0 gives 0 or 1 px
    // which is too imprecise. Use 100.0 em and scale the result.
    let probe_scale = Pixels::new(100.0_f32);
    let probe = glyph_id.with_scale(PxScale::from(probe_scale.as_raw()));
    if let Some(outline) = font.outline_glyph(probe) {
        let bounds: Rect<Pixels, f32> = outline.px_bounds().into();
        let max_g = Pixels::new(bounds.size.x.max(bounds.size.y).max(0.0));
        if max_g > Pixels::new(0.01) {
            // max_g is the pixel size at probe_scale em.
            // To get target_px pixels: em_scale = target_px * probe_scale / max_g
            return Pixels::new(target_px.as_raw() * probe_scale.as_raw() / max_g.as_raw());
        }
    }
    // Fallback: use glyph_bounds (design-space, not pixel-accurate)
    let probe = glyph_id.with_scale(1.0);
    let bounds: Rect<Pixels, f32> = font.glyph_bounds(&probe).into();
    let max_g = Pixels::new(bounds.size.x.max(bounds.size.y).max(0.0));
    if max_g > Pixels::new(0.01) {
        Pixels::new(target_px.as_raw() / max_g.as_raw())
    } else {
        target_px
    }
}

/// A glyph scaled to `scale` and positioned so that its pixel bounds
/// center lands on `target` - outline `px_bounds` when available
/// (precise for centering), `glyph_bounds` as fallback.
pub(super) fn centered_glyph(font: &FontVec, glyph_id: GlyphId, scale: PxScale, target: Vec2<Pixels, f32>) -> Glyph {
    let probe = glyph_id.with_scale(scale);
    let bounds: Rect<Pixels, f32> = font
        .outline_glyph(probe.clone())
        .map(|outline| outline.px_bounds().into())
        .unwrap_or_else(|| font.glyph_bounds(&probe).into());
    glyph_id.with_scale_and_position(scale, target - bounds.center())
}

/// Truncate text to fit within a maximum pixel width.
///
/// Uses incremental width tracking — O(n) in the number of characters.
pub(super) fn truncate_text_to_width(text: &str, scaled_font: &PxScaleFont<&FontVec>, max_width: Pixels<f32>) -> String {
    let mut result = String::new();
    let mut current_width = Pixels::new(0.0_f32);
    for ch in text.chars() {
        let glyph_id = scaled_font.glyph_id(ch);
        let char_width = Pixels::new(scaled_font.h_advance(glyph_id));
        if current_width + char_width > max_width {
            break;
        }
        current_width = current_width + char_width;
        result.push(ch);
    }
    result
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
        let target_px = Pixels::new(48.0_f32);

        let scale1 = glyph_fit_scale(font, glyph_id, target_px);
        let scale2 = glyph_fit_scale(font, glyph_id, target_px);

        assert_eq!(scale1.as_raw().to_bits(), scale2.as_raw().to_bits());
        assert!(scale1.as_raw() > 0.0);
    }

    #[test]
    fn glyph_fit_scale_proportional_to_target() {
        let font = nerd_font().expect("nerd font should be loaded");
        let glyph_id = font.glyph_id('\u{F11B}');

        let scale_24 = glyph_fit_scale(font, glyph_id, Pixels::new(24.0));
        let scale_48 = glyph_fit_scale(font, glyph_id, Pixels::new(48.0));

        let ratio = scale_48 / scale_24;
        assert!((ratio - 2.0).abs() < 0.01, "doubling target_px should approximately double the scale, got ratio {}", ratio);
    }

    #[test]
    fn glyph_fit_scale_positive_for_valid_glyph() {
        let font = nerd_font().expect("nerd font should be loaded");
        let glyph_id = font.glyph_id('\u{F001}');
        let scale = glyph_fit_scale(font, glyph_id, Pixels::new(32.0));
        assert!(scale.as_raw() > 0.0, "scale should be positive for a valid glyph");
    }

    #[test]
    fn truncate_text_to_width_returns_full_text_when_fits() {
        let font = nerd_font().expect("nerd font should be loaded");
        let scale = PxScale::from(16.0);
        let scaled_font = font.as_scaled(scale);
        let text = "hi";

        let result = truncate_text_to_width(text, &scaled_font, Pixels::new(1000.0));
        assert_eq!(result, "hi");
    }

    #[test]
    fn truncate_text_to_width_truncates_when_too_long() {
        let font = nerd_font().expect("nerd font should be loaded");
        let scale = PxScale::from(16.0);
        let scaled_font = font.as_scaled(scale);
        let text = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

        let result = truncate_text_to_width(text, &scaled_font, Pixels::new(20.0));
        assert!(result.len() < text.len(), "text should be truncated to fit within 20px");
        assert!(!result.is_empty(), "truncated text should not be empty");
    }

    #[test]
    fn truncate_text_to_width_empty_string_returns_empty() {
        let font = nerd_font().expect("nerd font should be loaded");
        let scale = PxScale::from(16.0);
        let scaled_font = font.as_scaled(scale);

        let result = truncate_text_to_width("", &scaled_font, Pixels::new(100.0));
        assert!(result.is_empty());
    }

    #[test]
    fn truncate_text_to_width_zero_max_width_returns_empty() {
        let font = nerd_font().expect("nerd font should be loaded");
        let scale = PxScale::from(16.0);
        let scaled_font = font.as_scaled(scale);

        let result = truncate_text_to_width("hello", &scaled_font, Pixels::new(0.0));
        assert!(result.is_empty(), "no characters should fit in zero width");
    }
}
