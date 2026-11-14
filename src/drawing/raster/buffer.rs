//! The [`RasterOps`] implementation for the strided render target.

use super::ops::RasterOps;
use crate::drawing::default_color::default_text_color;
use ab_glyph::Font;
use ab_glyph::FontVec;
use ab_glyph::Glyph;
use smearor_commons_buffer::BufferViewMut;
use smearor_commons_buffer::Rgba;
use smearor_commons_color::Color;
use smearor_commons_geometry::Pixels;
use smearor_commons_geometry::Rect;
use smearor_commons_geometry::Vec2;

impl RasterOps for BufferViewMut<'_, Rgba> {
    /// Draw a single glyph onto the view with alpha blending.
    fn draw_glyph(&mut self, font: &FontVec, glyph: &Glyph, color: Color<u8>) {
        let size = self.size();
        let stride = self.stride().as_usize();
        let clip = Rect::new(Vec2::<Pixels, i32>::default(), Vec2::new(size.x as i32, size.y as i32));
        let pixels = self.data_mut();
        if let Some(outline) = font.outline_glyph(glyph.clone()) {
            let bounds: Rect<Pixels, f32> = outline.px_bounds().into();
            let pos: Vec2<Pixels, i32> = bounds.min.to_i32_round();

            outline.draw(|gx, gy, coverage| {
                let p = pos + Vec2::<Pixels, i32>::new(gx as i32, gy as i32);
                if clip.contains(&p) {
                    let idx = p.y as usize * stride + p.x as usize * 4;
                    let alpha = (coverage * color.a as f32) as u8;
                    if alpha > 0 {
                        let dst = Color::<u8>::new_rgba(pixels[idx], pixels[idx + 1], pixels[idx + 2], pixels[idx + 3]);
                        let blended: [u8; 4] = color.alpha_over(&dst, alpha).into();
                        pixels[idx..idx + 4].copy_from_slice(&blended);
                    }
                }
            });
        }
    }

    /// Simple bitmap text renderer used when no font is available.
    fn draw_text_bitmap(&mut self, text: &str, _is_active: bool) {
        let size = self.size();
        let stride = self.stride().as_usize();
        let pixels = self.data_mut();
        let color = default_text_color(false);
        let bytes: [u8; 4] = color.into();
        let char_w = 6u32;
        let char_h = 8u32;
        let text_size = Vec2::new(text.len() as u32 * char_w, char_h);
        let start = size.saturating_sub(text_size) / 2;

        for (i, _ch) in text.chars().enumerate() {
            let x = start.x + i as u32 * char_w;
            for dy in 0..char_h {
                for dx in 0..char_w {
                    if x + dx < size.x && start.y + dy < size.y {
                        let idx = (start.y + dy) as usize * stride + (x + dx) as usize * 4;
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
    use crate::drawing::raster::RasterOps;
    use smearor_commons_buffer::PixelBuffer;
    use smearor_commons_geometry::Size;

    #[test]
    fn draw_text_bitmap_renders_pixels() {
        let mut buf = PixelBuffer::<Rgba>::new(Size::new(64, 16));
        buf.view_mut().draw_text_bitmap("AB", false);

        let non_zero = buf.pixels().as_chunks::<4>().0.iter().filter(|px| px.iter().any(|&b| b != 0)).count();
        assert!(non_zero > 0, "bitmap text should render some pixels");
    }

    #[test]
    fn draw_text_bitmap_empty_string_does_not_render() {
        let mut buf = PixelBuffer::<Rgba>::new(Size::new(32, 16));
        buf.view_mut().draw_text_bitmap("", false);

        assert!(buf.pixels().iter().all(|&p| p == 0), "no pixels should be rendered for empty string");
    }
}
