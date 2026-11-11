//! Software rendering utilities for pixel buffers.
//!
//! Provides glyph rendering, text rendering, icon rendering, and progress
//! bar rendering onto raw RGBA pixel buffers using `ab_glyph`. Color types
//! are provided by `smearor-commons-color`.
//!
//! Depends on `nerd-fonts-gtk` for font loading (`nerd_font()`, `label_font()`).

pub mod drawing;
pub mod pixel_buffer;
pub mod render_context;
pub mod size;

pub use pixel_buffer::PixelBuffer;
pub use render_context::RenderContext;
pub use size::Size;
