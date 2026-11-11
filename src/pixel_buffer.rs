//! Owned RGBA pixel buffer with associated dimensions.
//!
//! [`PixelBuffer`] encapsulates a `Vec<u8>` and a [`Size`] to ensure
//! the buffer length is always consistent with its declared dimensions.

use crate::size::Size;

/// An owned RGBA pixel buffer with associated dimensions.
///
/// The buffer is always `width * height * 4` bytes long (RGBA, row-major).
/// Construct with [`PixelBuffer::new`] to get a zeroed buffer of the correct size.
///
/// # Examples
///
/// ```
/// use pixel_drawing::PixelBuffer;
/// use pixel_drawing::Size;
///
/// let mut buf = PixelBuffer::new(Size::new(96, 96));
/// assert_eq!(buf.size(), Size::new(96, 96));
/// assert_eq!(buf.pixels().len(), 96 * 96 * 4);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PixelBuffer {
    /// Raw RGBA pixel data (row-major).
    pixels: Vec<u8>,
    /// Buffer dimensions in pixels.
    size: Size,
}

impl PixelBuffer {
    /// Creates a new zeroed pixel buffer of the given size.
    pub fn new(size: Size) -> Self {
        Self {
            pixels: vec![0u8; (size.area() * 4) as usize],
            size,
        }
    }

    /// Returns the buffer dimensions.
    pub const fn size(&self) -> Size {
        self.size
    }

    /// Returns the buffer width in pixels.
    pub const fn width(&self) -> u32 {
        self.size.width()
    }

    /// Returns the buffer height in pixels.
    pub const fn height(&self) -> u32 {
        self.size.height()
    }

    /// Returns the buffer area in pixels (width * height).
    pub const fn area(&self) -> u64 {
        self.size.area()
    }

    /// Creates a `PixelBuffer` from raw pixel data and dimensions.
    ///
    /// This bypasses the size invariant and is intended only for tests that
    /// need to simulate undersized or oversized buffers.
    #[cfg(test)]
    pub(crate) fn from_raw(pixels: Vec<u8>, size: Size) -> Self {
        Self { pixels, size }
    }

    /// Returns a reference to the raw pixel data.
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// Returns a mutable reference to the raw pixel data.
    pub fn pixels_mut(&mut self) -> &mut [u8] {
        &mut self.pixels
    }
}

impl AsRef<[u8]> for PixelBuffer {
    fn as_ref(&self) -> &[u8] {
        &self.pixels
    }
}

impl AsMut<[u8]> for PixelBuffer {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.pixels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_zeroed_buffer_of_correct_size() {
        let buf = PixelBuffer::new(Size::new(2, 3));
        assert_eq!(buf.size(), Size::new(2, 3));
        assert_eq!(buf.width(), 2);
        assert_eq!(buf.height(), 3);
        assert_eq!(buf.area(), 6);
        assert_eq!(buf.pixels().len(), 24);
        assert!(buf.pixels().iter().all(|&b| b == 0));
    }

    #[test]
    fn new_zero_size_creates_empty_buffer() {
        let buf = PixelBuffer::new(Size::new(0, 0));
        assert_eq!(buf.pixels().len(), 0);
    }

    #[test]
    fn pixels_mut_allows_modification() {
        let mut buf = PixelBuffer::new(Size::new(1, 1));
        buf.pixels_mut()[0] = 255;
        assert_eq!(buf.pixels()[0], 255);
    }

    #[test]
    fn as_ref_returns_pixel_slice() {
        let buf = PixelBuffer::new(Size::new(1, 1));
        let slice: &[u8] = buf.as_ref();
        assert_eq!(slice.len(), 4);
    }

    #[test]
    fn as_mut_returns_mutable_pixel_slice() {
        let mut buf = PixelBuffer::new(Size::new(1, 1));
        let slice: &mut [u8] = buf.as_mut();
        slice[0] = 128;
        assert_eq!(buf.pixels()[0], 128);
    }

    #[test]
    fn equality_compares_pixels_and_size() {
        let a = PixelBuffer::new(Size::new(2, 2));
        let b = PixelBuffer::new(Size::new(2, 2));
        assert_eq!(a, b);

        let c = PixelBuffer::new(Size::new(3, 3));
        assert_ne!(a, c);
    }
}
