//! Drawing operations on RGBA pixel buffer views.
//!
//! [`DrawOps`] provides methods to draw Nerd Font icons, text labels,
//! progress bars, and icon grids using `ab_glyph`. It is implemented for
//! [`BufferViewMut`]`<`[`Rgba`]`>` - the strided render target - and for
//! [`PixelBuffer`]<[`Rgba`]>, which delegates to its packed view.

mod glyph;
mod icons;
mod pixel;
mod progress;
mod text;

use ab_glyph::FontVec;
use ab_glyph::Glyph;
use smearor_commons_buffer::BufferViewMut;
use smearor_commons_buffer::PixelBuffer;
use smearor_commons_buffer::Rgba;
use smearor_commons_color::Color;

/// RGBA drawing operations on a strided pixel buffer view.
///
/// The methods only write the declared pixel rows; row padding and memory
/// beyond the view extent stay untouched.
pub trait DrawOps {
    /// Fill the entire view with a solid background color.
    fn fill_background(&mut self, color: impl Into<Color<u8>>);
    /// Draw a Nerd Font icon by name, resolved via `resolve_codepoint`;
    /// falls back to [`DrawOps::draw_icon_placeholder`].
    fn draw_nerd_font_icon(&mut self, icon_name: &str, is_active: bool, resolve_codepoint: impl Fn(&str) -> Option<char>, icon_color: Option<Color<f64>>);
    /// Draw a circular placeholder when the font or icon is unavailable.
    fn draw_icon_placeholder(&mut self, is_active: bool);
    /// Draw a Nerd Font icon by Unicode codepoint at a custom position and size.
    fn draw_nerd_font_codepoint(&mut self, codepoint: char, center_x: f32, center_y: f32, icon_size: f32, color: impl Into<Color<u8>>);
    /// Draw a 2x2 or 3x3 icon grid within the view area.
    fn draw_icon_grid(&mut self, icons: &[&str], grid_cols: u32, is_active: bool, resolve_codepoint: impl Fn(&str) -> Option<char>);
    /// Draw a text label at the bottom of the view using the label font.
    fn draw_label_text(&mut self, text: &str, is_active: bool, text_color_override: Option<Color<f64>>);
    /// Draw text centered horizontally at a given y-baseline.
    fn draw_text_centered(&mut self, text: &str, baseline_y: f32, font_size: f32, color: impl Into<Color<u8>>);
    /// Draw a horizontal progress bar at the bottom of the view.
    fn draw_progress_bar(&mut self, value: f32, color: impl Into<Color<u8>>);
}

impl DrawOps for BufferViewMut<'_, Rgba> {
    fn fill_background(&mut self, color: impl Into<Color<u8>>) {
        pixel::fill_background(self, color);
    }

    fn draw_nerd_font_icon(&mut self, icon_name: &str, is_active: bool, resolve_codepoint: impl Fn(&str) -> Option<char>, icon_color: Option<Color<f64>>) {
        icons::draw_nerd_font_icon(self, icon_name, is_active, resolve_codepoint, icon_color);
    }

    fn draw_icon_placeholder(&mut self, is_active: bool) {
        icons::draw_icon_placeholder(self, is_active);
    }

    fn draw_nerd_font_codepoint(&mut self, codepoint: char, center_x: f32, center_y: f32, icon_size: f32, color: impl Into<Color<u8>>) {
        icons::draw_nerd_font_codepoint(self, codepoint, center_x, center_y, icon_size, color);
    }

    fn draw_icon_grid(&mut self, icons: &[&str], grid_cols: u32, is_active: bool, resolve_codepoint: impl Fn(&str) -> Option<char>) {
        icons::draw_icon_grid(self, icons, grid_cols, is_active, resolve_codepoint);
    }

    fn draw_label_text(&mut self, text: &str, is_active: bool, text_color_override: Option<Color<f64>>) {
        text::draw_label_text(self, text, is_active, text_color_override);
    }

    fn draw_text_centered(&mut self, text: &str, baseline_y: f32, font_size: f32, color: impl Into<Color<u8>>) {
        text::draw_text_centered(self, text, baseline_y, font_size, color);
    }

    fn draw_progress_bar(&mut self, value: f32, color: impl Into<Color<u8>>) {
        progress::draw_progress_bar(self, value, color);
    }
}

/// Internal raster primitives shared by the [`DrawOps`] implementations.
///
/// `draw_glyph` rasterizes one `ab_glyph` glyph with alpha blending;
/// `draw_text_bitmap` is the fontless bitmap fallback. Kept private so
/// they stay out of the public API surface.
pub(super) trait RasterOps {
    /// Draw a single glyph onto the view with alpha blending.
    fn draw_glyph(&mut self, font: &FontVec, glyph: &Glyph, color: Color<u8>);
    /// Simple bitmap text renderer used when no font is available.
    fn draw_text_bitmap(&mut self, text: &str, is_active: bool);
}

impl RasterOps for BufferViewMut<'_, Rgba> {
    fn draw_glyph(&mut self, font: &FontVec, glyph: &Glyph, color: Color<u8>) {
        glyph::draw_glyph(self, font, glyph, color);
    }

    fn draw_text_bitmap(&mut self, text: &str, is_active: bool) {
        pixel::draw_text_bitmap(self, text, is_active);
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

    fn draw_icon_placeholder(&mut self, is_active: bool) {
        self.view_mut().draw_icon_placeholder(is_active);
    }

    fn draw_nerd_font_codepoint(&mut self, codepoint: char, center_x: f32, center_y: f32, icon_size: f32, color: impl Into<Color<u8>>) {
        self.view_mut().draw_nerd_font_codepoint(codepoint, center_x, center_y, icon_size, color);
    }

    fn draw_icon_grid(&mut self, icons: &[&str], grid_cols: u32, is_active: bool, resolve_codepoint: impl Fn(&str) -> Option<char>) {
        self.view_mut().draw_icon_grid(icons, grid_cols, is_active, resolve_codepoint);
    }

    fn draw_label_text(&mut self, text: &str, is_active: bool, text_color_override: Option<Color<f64>>) {
        self.view_mut().draw_label_text(text, is_active, text_color_override);
    }

    fn draw_text_centered(&mut self, text: &str, baseline_y: f32, font_size: f32, color: impl Into<Color<u8>>) {
        self.view_mut().draw_text_centered(text, baseline_y, font_size, color);
    }

    fn draw_progress_bar(&mut self, value: f32, color: impl Into<Color<u8>>) {
        self.view_mut().draw_progress_bar(value, color);
    }
}
