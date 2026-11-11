# Getting Started

## Prerequisites

### Rust

A working Rust toolchain is required. The project uses Rust Edition 2024.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Minimum Supported Rust Version (MSRV)

The crate declares `rust-version = "1.85"` in its `Cargo.toml`. The MSRV is
verified by CI on every push and pull request.

### System Dependencies

The crate requires `nerd-fonts-gtk` with `render` and `embed-fonts`
features, which bundle font files and use `ab_glyph` for rasterization. No
system libraries are required beyond a Rust toolchain.

If you use the `interactive_demo` example (GTK4), you need GTK 4 development
libraries:

**Ubuntu/Debian:**

```bash
sudo apt-get install -y pkg-config libgtk-4-dev libglib2.0-dev
```

## Installation

### From Source

```bash
git clone https://github.com/smearor/pixel-drawing.git
cd pixel-drawing
cargo build --release
```

### As a Dependency

Since `pixel-drawing` depends on `nerd-fonts-gtk` via a path dependency, add
both to your workspace:

```toml
[dependencies]
pixel-drawing = { path = "../pixel-drawing" }
nerd-fonts-gtk = { path = "../nerd-fonts-gtk", default-features = false, features = ["render", "embed-fonts"] }
```

## Quick Start

### Fill and Draw an Icon

```rust
use pixel_drawing::color::Color;
use pixel_drawing::pixel_buffer::PixelBuffer;
use pixel_drawing::size::Size;

let mut buf = PixelBuffer::new(Size::new(96, 96));
buf.fill_background(Color::<f64>::new(0.1, 0.1, 0.1));
buf.draw_nerd_font_icon(
    "nf-fa-gamepad",
    true,
    |name| Some('\u{F11B}'),
    Some(Color::<f64>::new(0.4, 0.7, 1.0)),
);
```

### Using RenderContext

```rust
use pixel_drawing::color::Color;
use pixel_drawing::pixel_buffer::PixelBuffer;
use pixel_drawing::render_context::RenderContext;
use pixel_drawing::size::Size;

let mut buf = PixelBuffer::new(Size::new(96, 96));
let mut ctx = RenderContext::new(&mut buf)
    .background(Color::new(0.12, 0.12, 0.12))
    .icon_color(Color::new(0.4, 0.7, 1.0))
    .active(true);
ctx.fill_background();
ctx.draw_icon('\u{F11B}');
ctx.draw_label("gamepad");
```

## Running Tests

```bash
cargo test
```

## Running the Example

```bash
cargo run --example interactive_demo
```

## Building the Documentation

```bash
cd book
mdbook build
```

The rendered HTML is placed in `book/book/`. Open `book/book/index.html` in a
browser, or run `mdbook serve` for live preview during development.
