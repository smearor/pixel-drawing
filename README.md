# pixel-drawing

[![crates.io](https://img.shields.io/crates/v/pixel-drawing.svg)](https://crates.io/crates/pixel-drawing)
[![Rust Edition 2024](https://img.shields.io/badge/Rust-Edition%202024-orange.svg)](https://doc.rust-lang.org/edition-guide/editions/2024/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

[![CI Build](https://github.com/smearor/pixel-drawing/actions/workflows/build.yml/badge.svg)](https://github.com/smearor/pixel-drawing/actions/workflows/build.yml)
[![MSRV](https://github.com/smearor/pixel-drawing/actions/workflows/msrv.yml/badge.svg)](https://github.com/smearor/pixel-drawing/actions/workflows/msrv.yml)
[![Security Audit](https://github.com/smearor/pixel-drawing/actions/workflows/audit.yml/badge.svg)](https://github.com/smearor/pixel-drawing/actions/workflows/audit.yml)
[![Book](https://github.com/smearor/pixel-drawing/actions/workflows/book.yml/badge.svg)](https://github.com/smearor/pixel-drawing/actions/workflows/book.yml)

A Rust library for software rendering onto raw RGBA pixel buffers. It provides
color types, glyph rasterization, Nerd Font icon rendering, text layout,
progress bars, and a builder-pattern render context — all without requiring a
windowing system or GPU.

## Features

- **Color Types** — Generic `Color<T>` with `Color<f64>` and `Color<u8>`,
  hex parsing, and conversions
- **Pixel Operations** — `fill_background`, `blend_pixel`, and bitmap text
  fallback
- **Glyph Rendering** — Single-glyph rasterization with alpha blending using
  `ab_glyph`, plus glyph scale caching
- **Icon Rendering** — Draw Nerd Font icons by name or codepoint, icon
  placeholders, and icon grids
- **Text Rendering** — Label text at bottom, centered text at arbitrary
  positions, with font fallback and truncation
- **Progress Bars** — Horizontal progress bar with configurable colors
- **RenderContext** — Builder pattern combining background, icon, and label
  rendering

## Quick Start

### Prerequisites

- Rust toolchain (Edition 2024)
- `nerd-fonts-gtk` as a path dependency (for font loading)

### Add to Your Project

```toml
[dependencies]
pixel-drawing = { path = "../pixel-drawing" }
nerd-fonts-gtk = { path = "../nerd-fonts-gtk", default-features = false, features = ["render", "embed-fonts"] }
```

### Usage

```rust
use pixel_drawing::color::Color;
use pixel_drawing::pixel_buffer::PixelBuffer;
use pixel_drawing::render_context::RenderContext;
use pixel_drawing::size::Size;

// PixelBuffer methods
let mut buf = PixelBuffer::new(Size::new(96, 96));
buf.fill_background(Color::<f64>::new(0.1, 0.1, 0.1));
buf.draw_nerd_font_icon(
    "nf-fa-gamepad",
    true,
    |name| Some('\u{F11B}'),
    Some(Color::<f64>::new(0.4, 0.7, 1.0)),
);
buf.draw_label_text("gamepad", false, None);

// RenderContext builder
let mut buf = PixelBuffer::new(Size::new(96, 96));
let mut ctx = RenderContext::new(&mut buf)
    .background(Color::new(0.12, 0.12, 0.12))
    .icon_color(Color::new(0.4, 0.7, 1.0))
    .active(true);
ctx.fill_background();
ctx.draw_icon('\u{F11B}');
ctx.draw_label("gamepad");
```

## Running the Example

```sh
cargo run --example interactive_demo
```

The interactive demo is a GTK4 application that demonstrates all rendering
methods: `fill_background`, `draw_nerd_font_icon`, `draw_icon_placeholder`,
`draw_nerd_font_codepoint`, `draw_icon_grid`, `draw_progress_bar`,
`draw_label_text`, `draw_text_centered`, and `RenderContext`.

## Documentation

- **User Guide**: [mdBook](https://smearor.github.io/pixel-drawing/book/)
- **API Reference**: [docs.rs](https://docs.rs/pixel-drawing)
- **Changelog**: [CHANGELOG.md](./CHANGELOG.md)

## Contributing

Contributions are welcome. Please read the
[Code of Conduct](./CODE_OF_CONDUCT.md) before contributing.

## License

Licensed under the [MIT License](./LICENSE).
