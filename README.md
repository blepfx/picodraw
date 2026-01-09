# picodraw
a smol 2d graphics abstraction library

## Features
- Medium[^1] level abstraction of different graphics APIs
- Dynamically draw a list of quads, each with a different custom shader
- Supports reading from textures, and drawing onto render textures
- Write custom shaders in Rust
- Heavily tested (see [drawtest.rs](tests/drawtest.rs))

[^1]: Not as low level as raw OpenGL/Vulkan/DirectX, but not as high level as Vello or Skia.

## Backends
Currently `picodraw` supports the following backends:
- `opengl` - OpenGL 3.1+ GPU backend, suitable for rerendering every frame. Can draw most scenes in a single drawcall by using clever batching techniques.
- `software` - Multithreaded software rasterizer backend, slower than `opengl` but more portable.

I might implement a Vulkan backend in the future. I probably won't implement a Metal backend out of spite (unless they decide to remove OpenGL support), but PRs are welcome!

## Example
```rust,no_run
use picodraw::{*, trace::*};

fn shader_red_circle(pos: float2, x: float1, y: float1, radius: float1) -> float4 {
    let dist = (pos - float2((x, y))).len();
    let mask = (radius - dist - 0.5).clamp(0.0, 1.0);
    float4((1.0, 0.0, 0.0, mask))
}

let context: &mut dyn dynamic::DynContext = todo!() /* create context */;
let shader = context.create_shader(&ShaderData::trace(|| 
    shader_red_circle(
        float2::position(), 
        float1::read_f32(0),
        float1::read_f32(4),
        float1::read_f32(8),
    ))).unwrap();

context.draw(DrawTarget::Screen, |encoder| {
    encoder.add_rect([0, 0, 512, 512].into());
    encoder.add_data(&f32::to_ne_bytes(256.0));
    encoder.add_data(&f32::to_ne_bytes(256.0));
    encoder.add_data(&f32::to_ne_bytes(100.0));
    encoder.draw(&shader);
});
```
## Installation

To install `picodraw` add this to your `Cargo.toml`:

```toml
[dependencies]
picodraw = { git = "https://github.com/blepfx/picodraw", features = ["opengl"], branch = "rewrite" }
```

## License
Licensed under either of
 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
