//! Glyph rendering and font metric utilities.
//!
//! Provides single-glyph rasterization with alpha blending, glyph-to-pixel
//! scale computation (with caching), and text width truncation.

use ab_glyph::FontVec;
use ab_glyph::GlyphId;
use ahash::AHashMap;
use smearor_commons_geometry::Pixels;
use std::cell::RefCell;

thread_local! {
    /// Cache for `glyph_fit_scale` results, keyed by (glyph_id, target_px bits).
    /// Avoids redundant glyph probing on repeated renders of the same icon.
    static GLYPH_SCALE_CACHE: RefCell<AHashMap<(GlyphId, u32), Pixels<f32>>> =
        RefCell::new(AHashMap::new());
}

/// [`smearor_commons_font::glyph_fit_scale`] behind a thread-local
/// cache - avoids redundant glyph probing on repeated renders of the
/// same icon. The cache policy lives here; the metric itself is generic.
pub(super) fn glyph_fit_scale(font: &FontVec, glyph_id: GlyphId, target_px: Pixels<f32>) -> Pixels<f32> {
    let key = (glyph_id, target_px.as_raw().to_bits());

    let cached = GLYPH_SCALE_CACHE.with(|c| c.borrow().get(&key).copied());
    if let Some(scale) = cached {
        return scale;
    }

    let scale = smearor_commons_font::glyph_fit_scale(font, glyph_id, target_px);

    GLYPH_SCALE_CACHE.with(|c| c.borrow_mut().insert(key, scale));

    scale
}

#[cfg(test)]
mod tests {
    use super::*;
    use ab_glyph::Font;
    use ab_glyph::PxScale;
    use nerd_fonts_rs::fonts::nerd_font;
    use smearor_commons_font::truncate_text_to_width;

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
