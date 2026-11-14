//! Drawing operations on RGBA pixel buffer views.
//!
//! [`DrawOps`] provides methods to draw Nerd Font icons, text labels,
//! progress bars, and icon grids using `ab_glyph`. It is implemented for
//! `BufferViewMut`<`Rgba`> - the strided render target - and for
//! `PixelBuffer`<`Rgba`>, which delegates to its packed view.

mod default_color;
mod draw;
mod glyph;
mod raster;

pub use draw::DrawOps;
pub(super) use raster::RasterOps;
