# RenderContext

`RenderContext` is a builder pattern that encapsulates the pixel buffer,
dimensions, and default colors, then provides methods that delegate to the
methods on `PixelBuffer`.

## Basic Usage

```rust
use pixel_drawing::pixel_buffer::PixelBuffer;
use pixel_drawing::render_context::RenderContext;
use pixel_drawing::size::Size;
use smearor_commons_color::Color;

let mut buf = PixelBuffer::new(Size::new(96, 96));
let mut ctx = RenderContext::new(&mut buf)
    .background(Color::new(0.12, 0.12, 0.12))
    .icon_color(Color::new(0.4, 0.7, 1.0))
    .active(true);
ctx.fill_background();
ctx.draw_icon('\u{F11B}');
ctx.draw_label("gamepad");
```

## Configuration Methods

These methods consume `self` and return a new `RenderContext`, enabling
chaining during construction:

| Method         | Description                                      |
|----------------|--------------------------------------------------|
| `background()` | Sets the background fill color                   |
| `icon_color()` | Sets the default icon/progress bar color         |
| `text_color()` | Sets the default text label color                |
| `active()`     | Sets the active state (affects default colors)   |

## Drawing Methods

These methods take `&mut self` and write to the pixel buffer:

| Method               | Description                                          |
|----------------------|------------------------------------------------------|
| `fill_background()`  | Fills the buffer with the configured background      |
| `draw_icon()`        | Draws an icon by codepoint                           |
| `draw_icon_by_name()`| Draws an icon by name (requires resolver function)   |
| `draw_label()`       | Draws label text at the bottom                       |
| `draw_text()`        | Draws centered text at a given y-baseline            |
| `draw_progress_bar()`| Draws a progress bar with the configured color       |

## Example: Icon with Label

```rust
let mut buf = PixelBuffer::new(Size::new(96, 96));
let mut ctx = RenderContext::new(&mut buf)
    .background(Color::new(0.1, 0.1, 0.1))
    .icon_color(Color::new(0.9, 0.3, 0.4))
    .active(true);
ctx.fill_background();
ctx.draw_icon_by_name("nf-fa-heart", resolve_codepoint);
ctx.draw_label("heart");
```

## Default Colors

If not configured, the defaults are:
- `icon_color`: `Color::<f64>::new(0.94, 0.94, 0.94)` — light gray
- `text_color`: `Color::<f64>::new(0.94, 0.94, 0.94)` — light gray
- `is_active`: `false`
- `background`: `None` (no fill)
