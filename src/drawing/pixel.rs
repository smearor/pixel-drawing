//! Low-level pixel buffer operations.
//!
//! Provides direct pixel manipulation without font dependencies:
//! background fills, alpha blending, and bitmap text fallback.

use super::progress::default_text_color;
use smearor_commons_buffer::BufferViewMut;
use smearor_commons_buffer::Rgba;
use smearor_commons_color::Color;
use smearor_commons_geometry::Pixels;

/// Alpha-blend a source color over an existing pixel.
pub(super) fn blend_pixel(dst: &[u8], src: &Color<u8>, alpha: u8) -> [u8; 4] {
    let src_alpha = alpha as f32 / 255.0;
    let dst_alpha = dst[3] as f32 / 255.0;
    let out_alpha = src_alpha + dst_alpha * (1.0 - src_alpha);
    if out_alpha < 0.001 {
        return [0, 0, 0, 0];
    }
    let r = (src.r as f32 * src_alpha + dst[0] as f32 * dst_alpha * (1.0 - src_alpha)) / out_alpha;
    let g = (src.g as f32 * src_alpha + dst[1] as f32 * dst_alpha * (1.0 - src_alpha)) / out_alpha;
    let b = (src.b as f32 * src_alpha + dst[2] as f32 * dst_alpha * (1.0 - src_alpha)) / out_alpha;
    [r.round() as u8, g.round() as u8, b.round() as u8, (out_alpha * 255.0).round() as u8]
}

/// Simple bitmap text renderer used when no font is available.
pub(super) fn draw_text_bitmap(view: &mut BufferViewMut<Rgba>, text: &str, _is_active: bool) {
    let size = view.size();
    let stride = view.stride().as_usize();
    let pixels = view.data_mut();
    let color = default_text_color(false);
    let bytes: [u8; 4] = color.into();
    let char_w = 6u32;
    let char_h = 8u32;
    let total_w = text.len() as u32 * char_w;
    let start_x = size.x.saturating_sub(total_w) / 2;
    let start_y = size.y.saturating_sub(char_h) / 2;

    for (i, _ch) in text.chars().enumerate() {
        let x = start_x + i as u32 * char_w;
        for dy in 0..char_h {
            for dx in 0..char_w {
                if x + dx < size.x && start_y + dy < size.y {
                    let idx = (start_y + dy) as usize * stride + (x + dx) as usize * 4;
                    pixels[idx..idx + 4].copy_from_slice(&bytes);
                }
            }
        }
    }
}

/// Fill the entire view with a solid background color.
///
/// Packed views reinterpret the `&mut [u8]` data as `&mut [u32]` (RGBA
/// = 4 bytes = one `u32` in little-endian) and use `slice::fill()` for
/// SIMD-vectorized writes. Strided views fill each row's pixel bytes
/// and leave row padding untouched.
pub(super) fn fill_background(view: &mut BufferViewMut<Rgba>, color: impl Into<Color<u8>>) {
    let color: Color<u8> = color.into();
    if view.is_packed() {
        let len = view.metadata().required_len(4);
        let color_u32 = u32::from_le_bytes(color.into());
        let pixels_u32: &mut [u32] = bytemuck::cast_slice_mut(&mut view.data_mut()[..len]);
        pixels_u32.fill(color_u32);
    } else {
        let bytes: [u8; 4] = color.into();
        for y in 0..view.height().as_raw() {
            if let Some(row) = view.row_mut(Pixels::new(y)) {
                row.as_chunks_mut::<4>().0.fill(bytes);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DrawOps;
    use crate::drawing::RasterOps;
    use smearor_commons_buffer::BufferMetadata;
    use smearor_commons_buffer::PixelBuffer;
    use smearor_commons_buffer::Stride;
    use smearor_commons_geometry::Size;

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

    #[test]
    fn blend_pixel_full_opacity_overwrites_dst() {
        let dst = [10, 20, 30, 255];
        let src = Color::<u8>::new_rgba(255, 0, 0, 255);
        let result = blend_pixel(&dst, &src, 255);
        assert_eq!(result, [255, 0, 0, 255]);
    }

    #[test]
    fn blend_pixel_half_alpha_blends() {
        let dst = [0, 0, 0, 255];
        let src = Color::<u8>::new_rgba(255, 255, 255, 255);
        let result = blend_pixel(&dst, &src, 128);
        assert_eq!(result[3], 255, "output alpha should be fully opaque");
        assert!(result[0] > 0 && result[0] < 255, "red should be blended");
    }

    #[test]
    fn blend_pixel_over_transparent_dst() {
        let dst = [0, 0, 0, 0];
        let src = Color::<u8>::new_rgba(100, 200, 50, 255);
        let result = blend_pixel(&dst, &src, 255);
        assert_eq!(result, [100, 200, 50, 255]);
    }

    #[test]
    fn draw_text_bitmap_renders_pixels() {
        let mut buf = PixelBuffer::new(Size::new(64, 16));
        buf.view_mut().draw_text_bitmap("AB", false);

        let non_zero = buf.pixels().as_chunks::<4>().0.iter().filter(|px| px.iter().any(|&b| b != 0)).count();
        assert!(non_zero > 0, "bitmap text should render some pixels");
    }

    #[test]
    fn draw_text_bitmap_empty_string_does_not_render() {
        let mut buf = PixelBuffer::new(Size::new(32, 16));
        buf.view_mut().draw_text_bitmap("", false);

        assert!(buf.pixels().iter().all(|&p| p == 0), "no pixels should be rendered for empty string");
    }
}
