//! Software rendering utilities for pixel buffers.
//!
//! Provides color types, glyph rendering, text rendering, icon rendering,
//! and progress bar rendering onto raw RGBA pixel buffers using `ab_glyph`.
//!
//! Depends on `nerd-fonts-gtk` for font loading (`nerd_font()`, `label_font()`).

pub mod color;
pub mod drawing;
pub mod pixel_buffer;
pub mod render_context;
pub mod size;

pub use color::Color;
pub use color::ColorParseError;
pub use pixel_buffer::PixelBuffer;
pub use render_context::RenderContext;
pub use size::Size;
