//! Builder pattern for composing multi-element renders onto a pixel buffer.
//!
//! [`RenderContext`] encapsulates the pixel buffer, dimensions, and default
//! colors/state, then provides methods that delegate to the methods on
//! [`crate::PixelBuffer`]. This eliminates the boilerplate of passing `pixels`,
//! `width`, `height`, and colors to every `draw_*` call.

use crate::pixel_buffer::PixelBuffer;
use crate::size::Size;
use smearor_commons_color::Color;

/// Builder context for rendering Nerd Font icons, text, and UI elements
/// onto a raw RGBA pixel buffer.
///
/// Construct with [`RenderContext::new`], chain optional configuration
/// methods, then call draw methods.
///
/// # Examples
///
/// ```no_run
/// use pixel_drawing::PixelBuffer;
/// use pixel_drawing::RenderContext;
/// use pixel_drawing::Size;
/// use smearor_commons_color::Color;
///
/// let mut buf = PixelBuffer::new(Size::new(96, 96));
/// let mut ctx = RenderContext::new(&mut buf)
///     .background(Color::<f64>::new(0.12, 0.12, 0.12))
///     .icon_color(Color::<f64>::new(0.4, 0.7, 1.0))
///     .active(true);
/// ctx.fill_background();
/// ctx.draw_icon('\u{F11B}');
/// ctx.draw_label("gamepad");
/// ```
#[derive(Debug)]
pub struct RenderContext<'a> {
    /// Target pixel buffer (RGBA, row-major).
    buf: &'a mut PixelBuffer,
    /// Optional background fill color, applied by [`RenderContext::fill_background`].
    background: Option<Color<f64>>,
    /// Default color for icons and progress bars.
    icon_color: Color<f64>,
    /// Default color for text labels.
    text_color: Color<f64>,
    /// Whether the rendered element is "active" (affects default colors).
    is_active: bool,
}

impl<'a> RenderContext<'a> {
    /// Creates a new `RenderContext` with default colors and inactive state.
    pub fn new(buf: &'a mut PixelBuffer) -> Self {
        Self {
            buf,
            background: None,
            icon_color: Color::<f64>::new(0.94, 0.94, 0.94),
            text_color: Color::<f64>::new(0.94, 0.94, 0.94),
            is_active: false,
        }
    }

    /// Sets the background fill color, applied when [`fill_background`](Self::fill_background) is called.
    pub fn background(mut self, color: Color<f64>) -> Self {
        self.background = Some(color);
        self
    }

    /// Sets the default icon color (also used for progress bars).
    pub fn icon_color(mut self, color: Color<f64>) -> Self {
        self.icon_color = color;
        self
    }

    /// Sets the default text color for labels and centered text.
    pub fn text_color(mut self, color: Color<f64>) -> Self {
        self.text_color = color;
        self
    }

    /// Sets the active state, which affects default color selection.
    pub fn active(mut self, is_active: bool) -> Self {
        self.is_active = is_active;
        self
    }

    /// Fills the entire pixel buffer with the configured background color.
    ///
    /// Does nothing if no background color was set.
    pub fn fill_background(&mut self) {
        if let Some(color) = self.background {
            self.buf.fill_background(color);
        }
    }

    /// Draws a Nerd Font icon by its Unicode codepoint, centered in the upper
    /// portion of the buffer.
    ///
    /// Uses the configured `icon_color`. Falls back to a placeholder if the
    /// font is unavailable.
    pub fn draw_icon(&mut self, codepoint: char) {
        let size = self.buf.size();
        let icon_size = size.width().min(size.height()) as f32 * 0.6;
        self.buf
            .draw_nerd_font_codepoint(codepoint, size.width() as f32 / 2.0, size.height() as f32 * 0.4, icon_size, self.icon_color);
    }

    /// Draws a Nerd Font icon by name, resolving the codepoint via the
    /// provided resolver function.
    ///
    /// Uses the configured `icon_color` and `is_active` state. Falls back to
    /// a placeholder if the font or icon is unavailable.
    pub fn draw_icon_by_name(&mut self, icon_name: &str, resolve_codepoint: impl Fn(&str) -> Option<char>) {
        self.buf
            .draw_nerd_font_icon(icon_name, self.is_active, resolve_codepoint, Some(self.icon_color));
    }

    /// Draws a text label at the bottom of the buffer using the label font.
    ///
    /// Uses the configured `text_color` and `is_active` state.
    pub fn draw_label(&mut self, text: &str) {
        self.buf.draw_label_text(text, self.is_active, Some(self.text_color));
    }

    /// Draws text centered horizontally at the given y-baseline.
    ///
    /// Uses the configured `text_color`. The font size is derived from the
    /// buffer height unless overridden.
    pub fn draw_text(&mut self, text: &str, baseline_y: f32) {
        let font_size = (self.buf.height() as f32 * 0.22).max(8.0);
        self.buf.draw_text_centered(text, baseline_y, font_size, self.text_color);
    }

    /// Draws text centered horizontally at the given y-baseline with a
    /// custom font size.
    pub fn draw_text_sized(&mut self, text: &str, baseline_y: f32, font_size: f32) {
        self.buf.draw_text_centered(text, baseline_y, font_size, self.text_color);
    }

    /// Draws a horizontal progress bar at the bottom of the buffer.
    ///
    /// `value` is clamped to `[0.0, 1.0]`. Uses the configured `icon_color`.
    pub fn draw_progress_bar(&mut self, value: f32) {
        self.buf.draw_progress_bar(value, self.icon_color);
    }

    /// Draws a grid of icons by name, resolving codepoints via the provided
    /// resolver function.
    ///
    /// Uses the configured `is_active` state for default color selection.
    pub fn draw_icon_grid(&mut self, icons: &[&str], grid_cols: u32, resolve_codepoint: impl Fn(&str) -> Option<char>) {
        self.buf.draw_icon_grid(icons, grid_cols, self.is_active, resolve_codepoint);
    }

    /// Returns the buffer width in pixels.
    pub fn width(&self) -> u32 {
        self.buf.width()
    }

    /// Returns the buffer height in pixels.
    pub fn height(&self) -> u32 {
        self.buf.height()
    }

    /// Returns the buffer dimensions.
    pub fn size(&self) -> Size {
        self.buf.size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_context_fill_background_applies_configured_color() {
        let size = Size::new(4, 4);
        let mut buf = PixelBuffer::new(size);
        let color = Color::<f64>::new(50.0 / 255.0, 60.0 / 255.0, 70.0 / 255.0);
        let expected: [u8; 4] = Color::<u8>::from(color).into();

        let mut ctx = RenderContext::new(&mut buf).background(color);
        ctx.fill_background();

        for px in buf.pixels().chunks_exact(4) {
            assert_eq!(px, &expected);
        }
    }

    #[test]
    fn render_context_fill_background_noop_without_color() {
        let mut buf = PixelBuffer::new(Size::new(2, 2));
        buf.pixels_mut().fill(10);

        let mut ctx = RenderContext::new(&mut buf);
        ctx.fill_background();

        assert!(buf.pixels().iter().all(|&p| p == 10));
    }

    #[test]
    fn render_context_builder_methods_set_fields() {
        let mut buf = PixelBuffer::new(Size::new(2, 2));
        let bg = Color::<f64>::new_rgba(1.0 / 255.0, 2.0 / 255.0, 3.0 / 255.0, 4.0 / 255.0);
        let ic = Color::<f64>::new_rgba(5.0 / 255.0, 6.0 / 255.0, 7.0 / 255.0, 8.0 / 255.0);
        let tc = Color::<f64>::new_rgba(9.0 / 255.0, 10.0 / 255.0, 11.0 / 255.0, 12.0 / 255.0);
        let ctx = RenderContext::new(&mut buf).background(bg).icon_color(ic).text_color(tc).active(true);

        assert_eq!(ctx.width(), 2);
        assert_eq!(ctx.height(), 2);
        assert_eq!(ctx.icon_color, ic);
        assert_eq!(ctx.text_color, tc);
        assert!(ctx.is_active);
        assert_eq!(ctx.background, Some(bg));
    }
}
