# Icon Rendering

`PixelBuffer` provides methods to draw Nerd Font icons onto pixel buffers.

## draw_nerd_font_icon

Draws a Nerd Font icon centered in the upper portion of the image. The icon
name is resolved to a codepoint via a caller-provided resolver function. If the
font or icon is unavailable, a circular placeholder is drawn instead.

```rust
use pixel_drawing::pixel_buffer::PixelBuffer;
use pixel_drawing::size::Size;
use smearor_commons_color::Color;

let mut buf = PixelBuffer::new(Size::new(96, 96));
buf.draw_nerd_font_icon(
    "nf-fa-gamepad",
    true,
    |name| Some('\u{F11B}'),
    Some(Color::<f64>::new(0.4, 0.7, 1.0)),
);
```

Parameters:
- `icon_name` — Nerd Font icon name (e.g. `"nf-fa-gamepad"`)
- `is_active` — affects default color when `icon_color` is `None`
- `resolve_codepoint` — function mapping icon names to `char` codepoints
- `icon_color` — optional color; falls back to `default_text_color(is_active)`

## draw_icon_placeholder

Draws a simple circular placeholder when the font or icon is unavailable:

```rust
use pixel_drawing::pixel_buffer::PixelBuffer;
use pixel_drawing::size::Size;

let mut buf = PixelBuffer::new(Size::new(32, 32));
buf.draw_icon_placeholder(false);
```

## draw_nerd_font_codepoint

Draws a Nerd Font icon by its Unicode codepoint at a custom position and size:

```rust
use pixel_drawing::pixel_buffer::PixelBuffer;
use pixel_drawing::size::Size;
use smearor_commons_color::Color;

let mut buf = PixelBuffer::new(Size::new(64, 64));
buf.draw_nerd_font_codepoint(
    '\u{F11B}',
    32.0, 32.0, 48.0,
    Color::<u8>::new(255, 255, 255),
);
```

Accepts `impl Into<Color<u8>>`, so both `Color<f64>` and `Color<u8>` work.

## draw_icon_grid

Draws a 2x2 or 3x3 icon grid within the image area. Each icon is centered in
its cell:

```rust
use pixel_drawing::pixel_buffer::PixelBuffer;
use pixel_drawing::size::Size;

let icons = ["nf-fa-gamepad", "nf-fa-star", "nf-fa-heart", "nf-fa-bug"];
let mut buf = PixelBuffer::new(Size::new(192, 192));
buf.draw_icon_grid(
    &icons, 2, true,
    |name| Some('\u{F11B}'),
);
```
