# picodraw
a smol 2d graphics abstraction library

## Features
- High level abstraction of different graphics APIs
- Dynamically draw a list of quads, each with a different custom shader
- Supports reading from textures, and drawing onto render textures
- Write custom shaders in Rust
- Heavily tested (see [drawtest.rs](tests/drawtest.rs))

## Backends
Currently `picodraw` supports the following backends:
- `opengl` - OpenGL 3.1+ GPU backend, suitable for rerendering every frame. Can draw most scenes in 1 drawcall
- `software` - Multithreaded software rasterizer backend, slower than `opengl` but more portable

## Example
```rust,no_run
use picodraw::{*, shader::*};

fn shader_red_circle(pos: float2, x: float1, y: float1, radius: float1) -> float4 {
    let dist = (pos - float2((x, y))).len();
    let mask = (radius - dist).smoothstep(-0.5, 0.5);
    float4((1.0, 0.0, 0.0, mask))
}

let context: &dyn Context = todo!() /* create context */;
let shader = context.create_shader(Graph::scope(|| 
    shader_red_circle(
        io::position(), 
        io::read::<f32>(),
        io::read::<f32>(),
        io::read::<f32>(),
    )));


context.draw_screen(&[
    Command::ObjectBegin(shader),
    Command::ObjectRect([0, 0, 512, 512].into()),
    Command::ObjectData(ObjectData::Float(256.0)),
    Command::ObjectData(ObjectData::Float(256.0)),
    Command::ObjectData(ObjectData::Float(100.0)),
    Command::ObjectEnd,
]);

context.draw(&commands);
```
## Installation

To install `picodraw` add this to your `Cargo.toml`:

```toml
[dependencies]
picodraw = { git = "https://github.com/blepfx/picodraw", features = ["derive", "opengl"], branch = "rewrite" }
```

## License
Licensed under either of
 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
