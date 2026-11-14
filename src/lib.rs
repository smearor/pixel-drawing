//! Software rendering utilities for pixel buffers.
//!
//! Provides glyph rendering, text rendering, icon rendering, and progress
//! bar rendering onto RGBA pixel buffer views using `ab_glyph`. Buffer,
//! format and view types come from `smearor-commons-buffer`, geometry and
//! colors from `smearor-commons-geometry` and `smearor-commons-color`;
//! the drawing operations are the [`DrawOps`] extension trait.
//!
//! Depends on `nerd-fonts-gtk` for font loading (`nerd_font()`, `label_font()`).

pub mod drawing;
pub mod render_context;

pub use drawing::DrawOps;
pub use render_context::RenderContext;
