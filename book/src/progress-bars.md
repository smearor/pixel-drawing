# Progress Bars

`PixelBuffer` provides a progress bar rendering method with shared
color defaults.

## draw_progress_bar

Draws a horizontal progress bar at the bottom of the image:

```rust
use pixel_drawing::color::Color;
use pixel_drawing::pixel_buffer::PixelBuffer;
use pixel_drawing::size::Size;

let mut buf = PixelBuffer::new(Size::new(256, 32));
buf.draw_progress_bar(
    0.65,
    Color::<f64>::new(0.4, 0.7, 1.0),
);
```

The bar is 4 pixels high, positioned 2 pixels from the bottom. The fill width
is proportional to `value` (clamped to `[0.0, 1.0]`). Unfilled portions use
a dimmed version of the color (each channel divided by 3).

Accepts `impl Into<Color<u8>>` for the color parameter.

## default_text_color

Returns the default text color based on active state:

- **Active**: `Color::<u8>::new_rgba(100, 180, 255, 255)` — blue
- **Inactive**: `Color::<u8>::new_rgba(240, 240, 240, 255)` — light gray

This function is used internally by `draw_nerd_font_icon`,
`draw_icon_grid`, and `draw_label_text` when no explicit color is provided.
