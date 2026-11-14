//! The [`DrawOps`] extension trait.

use smearor_commons_color::Color;
use smearor_commons_geometry::Pixels;
use smearor_commons_geometry::Vec2;

/// RGBA drawing operations on a strided pixel buffer view.
///
/// The methods only write the declared pixel rows; row padding and memory
/// beyond the view extent stay untouched.
pub trait DrawOps {
    /// Fill the entire view with a solid background color.
    fn fill_background(&mut self, color: impl Into<Color<u8>>);
    /// Draw a Nerd Font icon by name, resolved via `resolve_codepoint`;
    /// falls back to [`DrawOps::draw_icon_placeholder`]. The `icon_color`
    /// override applies to the placeholder as well.
    fn draw_nerd_font_icon(&mut self, icon_name: &str, is_active: bool, resolve_codepoint: impl Fn(&str) -> Option<char>, icon_color: Option<Color<f64>>);
    /// Draw a circular placeholder when the font or icon is unavailable.
    /// `icon_color` overrides the active-state default color when `Some`.
    fn draw_icon_placeholder(&mut self, is_active: bool, icon_color: Option<Color<f64>>);
    /// Draw a Nerd Font icon by Unicode codepoint at a custom position and size.
    fn draw_nerd_font_codepoint(&mut self, codepoint: char, center: Vec2<Pixels, f32>, icon_size: Pixels<f32>, color: impl Into<Color<u8>>);
    /// Draw a 2x2 or 3x3 icon grid within the view area.
    fn draw_icon_grid(&mut self, icons: &[&str], grid_cols: u32, is_active: bool, resolve_codepoint: impl Fn(&str) -> Option<char>);
    /// Draw a text label at the bottom of the view using the label font.
    fn draw_label_text(&mut self, text: &str, is_active: bool, text_color_override: Option<Color<f64>>);
    /// Draw text centered horizontally at a given y-baseline.
    fn draw_text_centered(&mut self, text: &str, baseline_y: Pixels<f32>, font_size: Pixels<f32>, color: impl Into<Color<u8>>);
    /// Draw a horizontal progress bar at the bottom of the view.
    fn draw_progress_bar(&mut self, value: f32, color: impl Into<Color<u8>>);
}
