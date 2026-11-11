# Pixel Operations

`PixelBuffer` provides low-level pixel buffer manipulation methods.

## fill_background

Fills the pixel buffer with a solid color. Accepts any type that implements
`Into<Color<u8>>`, so both `Color<f64>` and `Color<u8>` work:

```rust
use pixel_drawing::color::Color;
use pixel_drawing::pixel_buffer::PixelBuffer;
use pixel_drawing::size::Size;

let mut buf = PixelBuffer::new(Size::new(64, 64));
buf.fill_background(Color::<f64>::new(0.2, 0.5, 0.8));
```

The buffer is interpreted as row-major RGBA (4 bytes per pixel). If the
underlying buffer is larger than `width * height * 4`, only the expected
region is filled.

## blend_pixel

Composites a source color over a destination pixel with alpha blending:

```rust
use pixel_drawing::drawing::blend_pixel;

let dst = [10, 20, 30, 255];
let src = [255, 0, 0, 255];
let result = blend_pixel(&dst, &src, 128);
```

This is used internally by `draw_glyph` and is not typically called directly.

## draw_text_bitmap

A fallback bitmap text renderer used when no font is available. It renders
text using a built-in 5x7 bitmap font. This is primarily used as a last-resort
fallback by `draw_label_text` and `draw_text_centered`.
