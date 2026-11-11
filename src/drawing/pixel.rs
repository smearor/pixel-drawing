//! Low-level pixel buffer operations.
//!
//! Provides direct pixel manipulation without font dependencies:
//! background fills, alpha blending, and bitmap text fallback.

use super::progress::default_text_color;
use crate::pixel_buffer::PixelBuffer;
use smearor_commons_color::Color;

/// Alpha-blend a source color over an existing pixel.
pub(super) fn blend_pixel(dst: &[u8], src: &[u8; 4], alpha: u8) -> [u8; 4] {
    let src_alpha = alpha as f32 / 255.0;
    let dst_alpha = dst[3] as f32 / 255.0;
    let out_alpha = src_alpha + dst_alpha * (1.0 - src_alpha);
    if out_alpha < 0.001 {
        return [0, 0, 0, 0];
    }
    let r = (src[0] as f32 * src_alpha + dst[0] as f32 * dst_alpha * (1.0 - src_alpha)) / out_alpha;
    let g = (src[1] as f32 * src_alpha + dst[1] as f32 * dst_alpha * (1.0 - src_alpha)) / out_alpha;
    let b = (src[2] as f32 * src_alpha + dst[2] as f32 * dst_alpha * (1.0 - src_alpha)) / out_alpha;
    [r.round() as u8, g.round() as u8, b.round() as u8, (out_alpha * 255.0).round() as u8]
}

impl PixelBuffer {
    /// Fill the entire pixel buffer with a solid background color.
    ///
    /// Reinterprets the `&mut [u8]` buffer as `&mut [u32]` (RGBA = 4 bytes =
    /// one `u32` in little-endian) and uses `slice::fill()` for SIMD-vectorized
    /// writes instead of a pixel-by-pixel nested loop.
    ///
    /// If the buffer is smaller than `width * height * 4`, only the available
    /// bytes are filled. If it is larger, only the expected region is written.
    pub fn fill_background(&mut self, color: impl Into<Color<u8>>) {
        let size = self.size();
        let pixels = self.pixels_mut();
        let expected = (size.width() as usize) * (size.height() as usize) * 4;
        let len = pixels.len().min(expected);
        let color: Color<u8> = color.into();
        let color_u32 = u32::from_le_bytes(color.into());
        let pixels_u32: &mut [u32] = bytemuck::cast_slice_mut(&mut pixels[..len]);
        pixels_u32.fill(color_u32);
    }

    /// Simple bitmap text renderer used when no font is available.
    pub(super) fn draw_text_bitmap(&mut self, text: &str, _is_active: bool) {
        let size = self.size();
        let pixels = self.pixels_mut();
        let color = default_text_color(false);
        let bytes: [u8; 4] = color.into();
        let char_w = 6u32;
        let char_h = 8u32;
        let total_w = text.len() as u32 * char_w;
        let start_x = size.width().saturating_sub(total_w) / 2;
        let start_y = size.height().saturating_sub(char_h) / 2;

        for (i, _ch) in text.chars().enumerate() {
            let x = start_x + i as u32 * char_w;
            for dy in 0..char_h {
                for dx in 0..char_w {
                    if x + dx < size.width() && start_y + dy < size.height() {
                        let idx = (((start_y + dy) * size.width() + x + dx) * 4) as usize;
                        pixels[idx..idx + 4].copy_from_slice(&bytes);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::size::Size;

    #[test]
    fn fill_background_fills_all_pixels() {
        let mut buf = PixelBuffer::new(Size::new(4, 3));
        let color = Color::<f64>::new(10.0 / 255.0, 20.0 / 255.0, 30.0 / 255.0);
        let expected: [u8; 4] = Color::<u8>::from(color).into();
        buf.fill_background(color);

        for px in buf.pixels().chunks_exact(4) {
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
    fn fill_background_undersized_buffer_fills_available() {
        // Simulate an undersized buffer: Size says 2x2 (16 bytes) but only 8 bytes present
        let mut buf = PixelBuffer::from_raw(vec![0u8; 8], Size::new(2, 2));
        let color = Color::<f64>::new(10.0 / 255.0, 20.0 / 255.0, 30.0 / 255.0);
        let expected: [u8; 4] = Color::<u8>::from(color).into();
        buf.fill_background(color);
        // Only 2 pixels fit in 8 bytes
        assert_eq!(&buf.pixels()[..4], &expected);
        assert_eq!(&buf.pixels()[4..8], &expected);
    }

    #[test]
    fn fill_background_oversized_buffer_fills_expected_region() {
        // Simulate an oversized buffer: Size says 2x2 (16 bytes) but 24 bytes present
        let mut buf = PixelBuffer::from_raw(vec![0u8; 24], Size::new(2, 2));
        let color = Color::<f64>::new(10.0 / 255.0, 20.0 / 255.0, 30.0 / 255.0);
        buf.fill_background(color);
        // First 16 bytes (2x2x4) should be filled
        assert_eq!(&buf.pixels()[..16], &[10, 20, 30, 255, 10, 20, 30, 255, 10, 20, 30, 255, 10, 20, 30, 255]);
        // Remaining 8 bytes should be untouched
        assert_eq!(&buf.pixels()[16..], &[0, 0, 0, 0, 0, 0, 0, 0]);
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

    #[test]
    fn blend_pixel_full_opacity_overwrites_dst() {
        let dst = [10, 20, 30, 255];
        let src = [255, 0, 0, 255];
        let result = blend_pixel(&dst, &src, 255);
        assert_eq!(result, [255, 0, 0, 255]);
    }

    #[test]
    fn blend_pixel_zero_alpha_preserves_dst() {
        let dst = [10, 20, 30, 255];
        let src = [255, 0, 0, 0];
        let result = blend_pixel(&dst, &src, 0);
        assert_eq!(result, [10, 20, 30, 255]);
    }

    #[test]
    fn blend_pixel_half_alpha_blends() {
        let dst = [0, 0, 0, 255];
        let src = [255, 255, 255, 255];
        let result = blend_pixel(&dst, &src, 128);
        assert_eq!(result[3], 255, "output alpha should be fully opaque");
        assert!(result[0] > 0 && result[0] < 255, "red should be blended");
    }

    #[test]
    fn blend_pixel_over_transparent_dst() {
        let dst = [0, 0, 0, 0];
        let src = [100, 200, 50, 255];
        let result = blend_pixel(&dst, &src, 255);
        assert_eq!(result, [100, 200, 50, 255]);
    }

    #[test]
    fn draw_text_bitmap_renders_pixels() {
        let mut buf = PixelBuffer::new(Size::new(64, 16));
        buf.draw_text_bitmap("AB", false);

        let non_zero = buf.pixels().chunks_exact(4).filter(|px| px.iter().any(|&b| b != 0)).count();
        assert!(non_zero > 0, "bitmap text should render some pixels");
    }

    #[test]
    fn draw_text_bitmap_empty_string_does_not_render() {
        let mut buf = PixelBuffer::new(Size::new(32, 16));
        buf.draw_text_bitmap("", false);

        assert!(buf.pixels().iter().all(|&p| p == 0), "no pixels should be rendered for empty string");
    }
}
