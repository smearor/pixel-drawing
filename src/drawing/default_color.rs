//! Shared default colors for the drawing operations.

use smearor_commons_color::Color;

/// Default text color based on active state.
pub(super) fn default_text_color(is_active: bool) -> Color<u8> {
    if is_active {
        Color::<u8>::new_rgba(100, 180, 255, 255)
    } else {
        Color::<u8>::new_rgba(240, 240, 240, 255)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_text_color_active_is_blue() {
        let color = default_text_color(true);
        assert_eq!(color, Color::<u8>::new_rgba(100, 180, 255, 255));
    }

    #[test]
    fn default_text_color_inactive_is_light_gray() {
        let color = default_text_color(false);
        assert_eq!(color, Color::<u8>::new_rgba(240, 240, 240, 255));
    }
}
