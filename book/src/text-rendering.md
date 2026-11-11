# Text Rendering

`PixelBuffer` provides methods to render text onto pixel buffers
using `ab_glyph`, with font fallback chains.

## draw_label_text

Draws a text label at the bottom of the image using the label font. Falls back
to the Nerd Font if the label font is unavailable, and to a bitmap renderer if
no font is available.

```rust
use pixel_drawing::color::Color;
use pixel_drawing::pixel_buffer::PixelBuffer;
use pixel_drawing::size::Size;

let mut buf = PixelBuffer::new(Size::new(96, 96));
buf.draw_label_text(
    "gamepad",
    true,
    Some(Color::<f64>::new(0.9, 0.9, 0.9)),
);
```

The text is automatically truncated to fit within 90% of the image width.

## draw_text_centered

Draws text centered horizontally at a given y-baseline with a configurable
font size:

```rust
use pixel_drawing::color::Color;
use pixel_drawing::pixel_buffer::PixelBuffer;
use pixel_drawing::size::Size;

let mut buf = PixelBuffer::new(Size::new(128, 64));
buf.draw_text_centered(
    "Hello",
    32.0, 16.0,
    Color::<u8>::new(255, 255, 255),
);
```

Accepts `impl Into<Color<u8>>` for the color parameter, so both `Color<f64>`
and `Color<u8>` work.

## Font Fallback Chain

1. `label_font()` — primary label font (loaded from bundled WOFF2)
2. `nerd_font()` — Nerd Font as fallback
3. `draw_text_bitmap` — built-in 5x7 bitmap font as last resort

## Truncation

Both methods use `truncate_text_to_width` to ensure text fits within 90% of
the buffer width. Characters are added incrementally until the next character
would exceed the maximum width.
