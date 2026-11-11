//! Software rendering utilities for pixel buffer rendering.
//!
//! Provides methods to draw Nerd Font icons, text labels, progress bars,
//! and icon grids onto RGBA pixel buffers using `ab_glyph`.
//! All drawing operations are implemented as methods on [`crate::PixelBuffer`].

mod glyph;
mod icons;
mod pixel;
mod progress;
mod text;
