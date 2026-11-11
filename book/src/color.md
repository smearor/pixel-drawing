# Color Types

The `color` module provides a generic `Color<T>` struct with two primary
instantiations: `Color<f64>` for normalized colors and `Color<u8>` for
8-bit-per-channel colors.

## Color<f64>

Normalized RGBA with channels in `[0.0, 1.0]`:

```rust
use pixel_drawing::color::Color;

let red = Color::<f64>::new(1.0, 0.0, 0.0);
let semi_blue = Color::<f64>::new_rgba(0.0, 0.0, 1.0, 0.5);
```

## Color<u8>

8-bit RGBA with channels in `[0, 255]`:

```rust
use pixel_drawing::color::Color;

let white = Color::<u8>::new(255, 255, 255);
let transparent = Color::<u8>::new_rgba(0, 0, 0, 0);
```

## Conversions

`Color<f64>` converts to `Color<u8>` via `to_u8()` or `Into`:

```rust
let f64_color = Color::<f64>::new(0.5, 0.5, 0.5);
let u8_color: Color<u8> = f64_color.into();
```

`Color<u8>` converts to `Color<f64>` via `to_f64()` or `Into`:

```rust
let u8_color = Color::<u8>::new(128, 64, 32);
let f64_color: Color<f64> = u8_color.into();
```

## Hex Parsing

Parse from hex strings:

```rust
use std::str::FromStr;
use pixel_drawing::color::Color;

let color = Color::<u8>::from_str("#FF6B35").unwrap();
let with_alpha = Color::<u8>::from_str("#FF6B3580").unwrap();
```

## Serialization

With the `serde` feature, `Color<T>` implements `Serialize` and `Deserialize`.

## Methods

| Method               | Description                              |
|----------------------|------------------------------------------|
| `new(r, g, b)`       | Create with alpha = max                  |
| `new_rgba(r, g, b, a)` | Create with explicit alpha             |
| `to_bytes()`         | Get `[r, g, b, a]` as `&[u8]`           |
| `to_u8()`            | Convert `Color<f64>` to `Color<u8>`      |
| `to_f64()`           | Convert `Color<u8>` to `Color<f64>`      |
| `to_hex()`           | Format as `#RRGGBB` or `#RRGGBBAA`      |
