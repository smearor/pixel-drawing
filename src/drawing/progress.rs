//! UI element rendering and shared color defaults.
//!
//! Provides progress bar rendering and the default text color used across
//! icon, text, and bitmap rendering modules.

use crate::color::Color;
use crate::pixel_buffer::PixelBuffer;

/// Default text color based on active state.
pub(super) fn default_text_color(is_active: bool) -> Color<u8> {
    if is_active {
        Color::<u8>::new_rgba(100, 180, 255, 255)
    } else {
        Color::<u8>::new_rgba(240, 240, 240, 255)
    }
}

impl PixelBuffer {
    /// Draw a horizontal progress bar at the bottom of the image.
    pub fn draw_progress_bar(&mut self, value: f32, color: impl Into<Color<u8>>) {
        let size = self.size();
        let pixels = self.pixels_mut();
        let bar_height = 4u32;
        let color: Color<u8> = color.into();
        let bar_y = size.height().saturating_sub(bar_height + 2);
        let fill_width = ((size.width() as f32 * value.clamp(0.0, 1.0)) as u32).min(size.width());
        let bytes = color.to_bytes();

        for y in bar_y..bar_y + bar_height {
            for x in 0..size.width() {
                let idx = ((y * size.width() + x) * 4) as usize;
                if x < fill_width {
                    pixels[idx..idx + 4].copy_from_slice(&bytes);
                } else {
                    pixels[idx..idx + 4].copy_from_slice(&[bytes[0] / 3, bytes[1] / 3, bytes[2] / 3, 255]);
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
    fn default_text_color_active_is_blue() {
        let color = default_text_color(true);
        assert_eq!(color, Color::<u8>::new_rgba(100, 180, 255, 255));
    }

    #[test]
    fn default_text_color_inactive_is_light_gray() {
        let color = default_text_color(false);
        assert_eq!(color, Color::<u8>::new_rgba(240, 240, 240, 255));
    }

    #[test]
    fn draw_progress_bar_full_fill() {
        let size = Size::new(8, 8);
        let mut buf = PixelBuffer::new(size);
        let color = Color::<u8>::new(255, 0, 0);
        buf.draw_progress_bar(1.0, color);

        let bar_height = 4u32;
        let bar_y = size.height().saturating_sub(bar_height + 2);
        let bytes = color.to_bytes();
        let pixels = buf.pixels();
        for y in bar_y..bar_y + bar_height {
            for x in 0..size.width() {
                let idx = ((y * size.width() + x) * 4) as usize;
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
        let bar_y = size.height().saturating_sub(bar_height + 2);
        let bytes = color.to_bytes();
        let fill_width = (size.width() as f32 * 0.5) as u32;
        let pixels = buf.pixels();
        for y in bar_y..bar_y + bar_height {
            for x in 0..size.width() {
                let idx = ((y * size.width() + x) * 4) as usize;
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
        let bar_y = size.height().saturating_sub(bar_height + 2);
        let bytes = color.to_bytes();
        let pixels = buf.pixels();
        for y in bar_y..bar_y + bar_height {
            for x in 0..size.width() {
                let idx = ((y * size.width() + x) * 4) as usize;
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
        let bar_y = size.height().saturating_sub(bar_height + 2);
        let bytes = color.to_bytes();
        let pixels = buf.pixels();
        for y in bar_y..bar_y + bar_height {
            for x in 0..size.width() {
                let idx = ((y * size.width() + x) * 4) as usize;
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
        let bar_y = size.height().saturating_sub(bar_height + 2);
        let pixels = buf.pixels();
        for y in bar_y..bar_y + bar_height {
            for x in 0..size.width() {
                let idx = ((y * size.width() + x) * 4) as usize;
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
        let bar_y = size.height().saturating_sub(bar_height + 2);
        let expected = color_f64.to_u8().to_bytes();
        let pixels = buf.pixels();
        for y in bar_y..bar_y + bar_height {
            for x in 0..size.width() {
                let idx = ((y * size.width() + x) * 4) as usize;
                assert_eq!(&pixels[idx..idx + 4], &expected, "pixel at ({},{}) should match f64-converted color", x, y);
            }
        }
    }
}
