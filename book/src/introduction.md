# Introduction

**pixel-drawing** is a Rust library for software rendering onto raw RGBA pixel
buffers. It provides color types, glyph rasterization, icon rendering, text
layout, progress bars, and a builder-pattern render context — all without
requiring a windowing system or GPU.

## What Does It Do?

- **Color Types** — Generic `Color<T>` with `Color<f64>` and `Color<u8>`
  instantiations, hex parsing, and conversions
- **Pixel Operations** — `fill_background`, `blend_pixel`, and a bitmap text
  fallback renderer
- **Glyph Rendering** — Single-glyph rasterization with alpha blending using
  `ab_glyph`, plus glyph-to-pixel scale computation with caching
- **Icon Rendering** — Draw Nerd Font icons by name or codepoint, icon
  placeholders, and icon grids
- **Text Rendering** — Draw label text at the bottom of an image or centered
  text at arbitrary positions, with font fallback chains and truncation
- **Progress Bars** — Horizontal progress bar rendering with configurable
  colors
- **RenderContext** — Builder pattern that combines background fill, icon
  drawing, and label text into a single fluent API

## Module Overview

```mermaid
graph TD
    Lib["pixel-drawing"]

    Color["color.rs<br/>Color<T>, Color<f64>, Color<u8>"]
    Drawing["drawing/<br/>Pixel, glyph, icon, text, progress"]
    RenderContext["render_context.rs<br/>Builder pattern"]

    Lib --> Color
    Lib --> Drawing
    Lib --> RenderContext

    Drawing --> Pixel["pixel.rs<br/>fill_background, blend_pixel"]
    Drawing --> Glyph["glyph.rs<br/>draw_glyph, glyph_fit_scale"]
    Drawing --> Icons["icons.rs<br/>draw_nerd_font_icon, draw_icon_grid"]
    Drawing --> Text["text.rs<br/>draw_label_text, draw_text_centered"]
    Drawing --> Progress["progress.rs<br/>draw_progress_bar"]

    RenderContext --> Drawing
```

## Dependency on nerd-fonts-gtk

`pixel-drawing` depends on [nerd-fonts-gtk](https://github.com/smearor/nerd-fonts-gtk)
for font loading (`nerd_font()`, `label_font()`) and icon name resolution
(`IconName`, codepoint maps). The `render` feature enables this dependency.

## License

MIT. See [LICENSE](https://github.com/smearor/pixel-drawing/blob/main/LICENSE).
