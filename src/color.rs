//! Color types for pixel-drawing.
//!
//! Provides a generic [`Color<T>`] with two concrete instantiations:
//!
//! - [`Color<f64>`] — high-level color for CSS generation and hex parsing.
//!   Components are in `[0.0, 1.0]`.
//! - [`Color<u8>`] — pixel-native color for software rendering. Components
//!   are in `[0, 255]`.
//!
//! Conversions between the two are provided via `From`/`Into`.

use std::fmt;
use std::str::FromStr;

/// Generic RGBA color with four components of type `T`.
///
/// Use [`Color<f64>`] for CSS/GTK operations (components in `[0.0, 1.0]`)
/// and [`Color<u8>`] for pixel buffer operations (components in `[0, 255]`).
///
/// Conversions between the two are available via `From`/`Into`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color<T> {
    /// Red component.
    pub r: T,
    /// Green component.
    pub g: T,
    /// Blue component.
    pub b: T,
    /// Alpha component.
    pub a: T,
}

/// Error returned when parsing an invalid hex color string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorParseError {
    /// The invalid input string.
    pub input: String,
    /// Human-readable reason for the failure.
    pub reason: &'static str,
}

impl fmt::Display for ColorParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid color '{}': {}", self.input, self.reason)
    }
}

impl std::error::Error for ColorParseError {}

// ---------------------------------------------------------------------------
// Color<f64> — high-level color for CSS and hex parsing
// ---------------------------------------------------------------------------

impl Color<f64> {
    /// Creates a new opaque color from RGB components (alpha = 1.0).
    pub const fn new(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Creates a new color from RGBA components.
    pub const fn new_rgba(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self { r, g, b, a }
    }

    /// Black.
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0);

    /// White.
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0);

    /// Converts to [`Color<u8>`] for pixel buffer operations.
    pub fn to_u8(self) -> Color<u8> {
        Color {
            r: (self.r * 255.0).round() as u8,
            g: (self.g * 255.0).round() as u8,
            b: (self.b * 255.0).round() as u8,
            a: (self.a * 255.0).round() as u8,
        }
    }

    /// Parses a hex color string into a `Color<f64>`.
    ///
    /// Accepts the following formats (with or without leading `#`):
    /// - `#RGB` — shorthand, each digit doubled (e.g. `#F0A` -> `#FF00AA`)
    /// - `#RGBA` — shorthand with alpha
    /// - `#RRGGBB` — opaque
    /// - `#RRGGBBAA` — with alpha
    ///
    /// # Errors
    ///
    /// Returns [`ColorParseError`] if the string is not a valid hex color.
    pub fn from_hex(hex: &str) -> Result<Self, ColorParseError> {
        Self::from_str(hex)
    }

    /// Formats the color as a hex string `#RRGGBBAA`.
    ///
    /// If alpha is 1.0 (opaque), the output is `#RRGGBB` (6 digits).
    /// Otherwise, the full 8-digit form `#RRGGBBAA` is used.
    pub fn to_hex(self) -> String {
        let c = self.to_u8();
        if c.a == 255 {
            format!("#{:02X}{:02X}{:02X}", c.r, c.g, c.b)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", c.r, c.g, c.b, c.a)
        }
    }
}

impl FromStr for Color<f64> {
    type Err = ColorParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hex = s.strip_prefix('#').unwrap_or(s);
        let len = hex.len();

        let (r, g, b, a) = match len {
            3 => {
                let bytes = hex.as_bytes();
                let r = u8::from_str_radix(&format!("{}{}", bytes[0] as char, bytes[0] as char), 16);
                let g = u8::from_str_radix(&format!("{}{}", bytes[1] as char, bytes[1] as char), 16);
                let b = u8::from_str_radix(&format!("{}{}", bytes[2] as char, bytes[2] as char), 16);
                (r, g, b, Ok(255u8))
            }
            4 => {
                let bytes = hex.as_bytes();
                let r = u8::from_str_radix(&format!("{}{}", bytes[0] as char, bytes[0] as char), 16);
                let g = u8::from_str_radix(&format!("{}{}", bytes[1] as char, bytes[1] as char), 16);
                let b = u8::from_str_radix(&format!("{}{}", bytes[2] as char, bytes[2] as char), 16);
                let a = u8::from_str_radix(&format!("{}{}", bytes[3] as char, bytes[3] as char), 16);
                (r, g, b, a)
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16);
                let g = u8::from_str_radix(&hex[2..4], 16);
                let b = u8::from_str_radix(&hex[4..6], 16);
                (r, g, b, Ok(255u8))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16);
                let g = u8::from_str_radix(&hex[2..4], 16);
                let b = u8::from_str_radix(&hex[4..6], 16);
                let a = u8::from_str_radix(&hex[6..8], 16);
                (r, g, b, a)
            }
            _ => {
                return Err(ColorParseError {
                    input: s.to_string(),
                    reason: "expected 3, 4, 6, or 8 hex digits",
                });
            }
        };

        match (r, g, b, a) {
            (Ok(r), Ok(g), Ok(b), Ok(a)) => Ok(Color::<u8>::from([r, g, b, a]).into()),
            _ => Err(ColorParseError {
                input: s.to_string(),
                reason: "contains non-hex characters",
            }),
        }
    }
}

impl fmt::Display for Color<f64> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

// ---------------------------------------------------------------------------
// Color<u8> — pixel-native color for software rendering
// ---------------------------------------------------------------------------

impl Color<u8> {
    /// Creates a new opaque color from RGB components (alpha = 255).
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Creates a new color from RGBA components.
    pub const fn new_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Black.
    pub const BLACK: Self = Self::new(0, 0, 0);

    /// White.
    pub const WHITE: Self = Self::new(255, 255, 255);

    /// Converts to [`Color<f64>`] for CSS/GTK operations.
    pub fn to_f64(self) -> Color<f64> {
        self.into()
    }

    /// Converts to `[u8; 4]` for direct pixel buffer writes.
    pub fn to_bytes(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl From<[u8; 4]> for Color<u8> {
    fn from(c: [u8; 4]) -> Self {
        Self {
            r: c[0],
            g: c[1],
            b: c[2],
            a: c[3],
        }
    }
}

// ---------------------------------------------------------------------------
// Cross-type conversions
// ---------------------------------------------------------------------------

impl From<Color<u8>> for Color<f64> {
    fn from(c: Color<u8>) -> Self {
        Self {
            r: c.r as f64 / 255.0,
            g: c.g as f64 / 255.0,
            b: c.b as f64 / 255.0,
            a: c.a as f64 / 255.0,
        }
    }
}

impl From<Color<f64>> for Color<u8> {
    fn from(c: Color<f64>) -> Self {
        c.to_u8()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f64_new_sets_alpha_to_one() {
        let color = Color::<f64>::new(0.5, 0.25, 0.75);
        assert_eq!(color.a, 1.0);
    }

    #[test]
    fn f64_new_rgba_preserves_alpha() {
        let color = Color::<f64>::new_rgba(0.1, 0.2, 0.3, 0.4);
        assert_eq!(color.a, 0.4);
    }

    #[test]
    fn f64_black_constant() {
        assert_eq!(Color::<f64>::BLACK, Color::<f64>::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn f64_white_constant() {
        assert_eq!(Color::<f64>::WHITE, Color::<f64>::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn f64_to_u8_round_trip() {
        let color = Color::<f64>::new_rgba(1.0, 0.0, 1.0, 0.0);
        let bytes: Color<u8> = color.into();
        assert_eq!(bytes, Color::<u8>::new_rgba(255, 0, 255, 0));
        let restored: Color<f64> = bytes.into();
        assert_eq!(restored, color);
    }

    #[test]
    fn f64_to_u8_black() {
        let c: Color<u8> = Color::<f64>::BLACK.into();
        assert_eq!(c, Color::<u8>::new(0, 0, 0));
    }

    #[test]
    fn f64_to_u8_white() {
        let c: Color<u8> = Color::<f64>::WHITE.into();
        assert_eq!(c, Color::<u8>::new(255, 255, 255));
    }

    #[test]
    fn u8_from_array() {
        let color: Color<u8> = [128, 64, 32, 255].into();
        assert_eq!(color.r, 128);
        assert_eq!(color.g, 64);
        assert_eq!(color.b, 32);
        assert_eq!(color.a, 255);
    }

    #[test]
    fn u8_to_bytes() {
        let color = Color::<u8>::new_rgba(10, 20, 30, 40);
        assert_eq!(color.to_bytes(), [10, 20, 30, 40]);
    }

    #[test]
    fn u8_to_f64_conversion() {
        let color = Color::<u8>::new_rgba(128, 64, 32, 255);
        let f: Color<f64> = color.into();
        assert_eq!(f.r, 128.0 / 255.0);
        assert_eq!(f.g, 64.0 / 255.0);
        assert_eq!(f.b, 32.0 / 255.0);
        assert_eq!(f.a, 1.0);
    }

    #[test]
    fn from_hex_rgb() {
        let color = Color::<f64>::from_hex("#FF6B4A").unwrap();
        assert_eq!(color, Color::<f64>::new(1.0, 0x6B as f64 / 255.0, 0x4A as f64 / 255.0));
    }

    #[test]
    fn from_hex_rgba() {
        let color = Color::<f64>::from_hex("#FF6B4A80").unwrap();
        assert_eq!(color.a, 0x80 as f64 / 255.0);
    }

    #[test]
    fn from_hex_shorthand_rgb() {
        let color = Color::<f64>::from_hex("#F0A").unwrap();
        assert_eq!(color, Color::<f64>::new(1.0, 0.0, 0xAA as f64 / 255.0));
    }

    #[test]
    fn from_hex_shorthand_rgba() {
        let color = Color::<f64>::from_hex("#F0AF").unwrap();
        assert_eq!(color.a, 1.0);
    }

    #[test]
    fn from_hex_without_hash() {
        let color = Color::<f64>::from_hex("FF6B4A").unwrap();
        assert_eq!(color, Color::<f64>::from_hex("#FF6B4A").unwrap());
    }

    #[test]
    fn from_hex_invalid_length() {
        assert!(Color::<f64>::from_hex("#FF").is_err());
        assert!(Color::<f64>::from_hex("#FFGGFF").is_err());
    }

    #[test]
    fn from_hex_invalid_chars() {
        assert!(Color::<f64>::from_hex("#GGGGGG").is_err());
    }

    #[test]
    fn to_hex_opaque() {
        let color = Color::<f64>::new(1.0, 0.0, 1.0);
        assert_eq!(color.to_hex(), "#FF00FF");
    }

    #[test]
    fn to_hex_with_alpha() {
        let color = Color::<f64>::new_rgba(1.0, 0.0, 1.0, 0.5);
        assert_eq!(color.to_hex(), "#FF00FF80");
    }

    #[test]
    fn from_str_round_trip() {
        let original = Color::<f64>::new_rgba(0x12 as f64 / 255.0, 0x34 as f64 / 255.0, 0x56 as f64 / 255.0, 0x78 as f64 / 255.0);
        let parsed: Color<f64> = original.to_hex().parse().unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn display_uses_hex() {
        let color = Color::<f64>::new(1.0, 0.0, 0.0);
        assert_eq!(format!("{}", color), "#FF0000");
    }
}
