//! Two-dimensional size type for pixel buffer dimensions.

use std::fmt::Display;
use std::fmt::Formatter;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Sub;
use std::ops::SubAssign;

/// Width and height of a pixel buffer or drawable region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    /// Width in pixels.
    width: u32,
    /// Height in pixels.
    height: u32,
}

impl Size {
    /// Creates a new `Size` with the given width and height in pixels.
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Returns the width in pixels.
    pub const fn width(self) -> u32 {
        self.width
    }

    /// Returns the height in pixels.
    pub const fn height(self) -> u32 {
        self.height
    }

    /// Returns the total number of pixels (width × height).
    pub const fn area(self) -> u64 {
        self.width as u64 * self.height as u64
    }

    /// Returns a new `Size` with the maximum width and height of both sizes.
    pub const fn max(self, other: Self) -> Self {
        Self {
            width: if self.width > other.width { self.width } else { other.width },
            height: if self.height > other.height { self.height } else { other.height },
        }
    }
}

impl Default for Size {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

impl Display for Size {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

impl Add for Size {
    type Output = Size;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            width: self.width + rhs.width,
            height: self.height + rhs.height,
        }
    }
}

impl AddAssign for Size {
    fn add_assign(&mut self, other: Self) {
        self.width += other.width;
        self.height += other.height;
    }
}

impl Sub for Size {
    type Output = Size;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            width: self.width - rhs.width,
            height: self.height - rhs.height,
        }
    }
}

impl SubAssign for Size {
    fn sub_assign(&mut self, other: Self) {
        self.width -= other.width;
        self.height -= other.height;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_width_and_height() {
        let size = Size::new(96, 64);
        assert_eq!(size.width(), 96);
        assert_eq!(size.height(), 64);
    }

    #[test]
    fn area_returns_width_times_height() {
        let size = Size::new(96, 64);
        assert_eq!(size.area(), 6144);
    }

    #[test]
    fn zero_size_has_zero_area() {
        let size = Size::new(0, 0);
        assert_eq!(size.area(), 0);
    }

    #[test]
    fn equality_compares_both_dimensions() {
        assert_eq!(Size::new(10, 20), Size::new(10, 20));
        assert_ne!(Size::new(10, 20), Size::new(10, 21));
        assert_ne!(Size::new(10, 20), Size::new(11, 20));
    }

    #[test]
    fn default_is_zero() {
        let size = Size::default();
        assert_eq!(size.width(), 0);
        assert_eq!(size.height(), 0);
    }

    #[test]
    fn display_formats_as_width_x_height() {
        let size = Size::new(100, 200);
        assert_eq!(format!("{}", size), "100x200");
    }

    #[test]
    fn add_sums_both_dimensions() {
        let result = Size::new(100, 200) + Size::new(50, 150);
        assert_eq!(result, Size::new(150, 350));
    }

    #[test]
    fn add_assign_modifies_in_place() {
        let mut size = Size::new(100, 200);
        size += Size::new(50, 150);
        assert_eq!(size, Size::new(150, 350));
    }

    #[test]
    fn sub_subtracts_both_dimensions() {
        let result = Size::new(100, 200) - Size::new(50, 150);
        assert_eq!(result, Size::new(50, 50));
    }

    #[test]
    fn sub_assign_modifies_in_place() {
        let mut size = Size::new(100, 200);
        size -= Size::new(50, 150);
        assert_eq!(size, Size::new(50, 50));
    }

    #[test]
    fn max_takes_largest_width_and_height() {
        let result = Size::new(100, 200).max(Size::new(50, 300));
        assert_eq!(result, Size::new(100, 300));
    }
}
