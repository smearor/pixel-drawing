# Glyph Rendering

The `drawing::glyph` module provides single-glyph rasterization and font metric
utilities. These are used internally by the icon and text rendering methods on
`PixelBuffer` and are not typically called directly.

## glyph_fit_scale

Computes the em-scale needed so that a glyph's largest dimension equals a
target pixel size. Nerd Font symbols typically occupy only a small fraction of
the em square, so a 76px em can produce a ~12px glyph. This function probes the
glyph at a reference scale and computes the correct em-scale.

Results are cached in a `thread_local` `AHashMap` keyed by `(GlyphId, target_px)`
to avoid redundant probing on repeated renders.

```rust
use nerd_fonts_gtk::fonts::nerd_font;
use pixel_drawing::drawing::glyph::glyph_fit_scale;

let font = nerd_font().unwrap();
let glyph_id = font.glyph_id('\u{F11B}');
let scale = glyph_fit_scale(font, glyph_id, 48.0);
```

## draw_glyph

Rasterizes a single glyph onto the pixel buffer with alpha blending. Uses
`ab_glyph`'s outline rasterizer and composites each pixel via `blend_pixel`.
This is a `PixelBuffer` method called internally by the text and icon rendering
methods.

## truncate_text_to_width

Truncates text to fit within a maximum pixel width, using incremental width
tracking (O(n) in the number of characters). Returns the truncated string.
