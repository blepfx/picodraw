mod backend;
mod dispatch;
mod simd;
mod vm;

pub use backend::*;
pub use dispatch::*;
pub use vm::*;

// TODO: implement
//
// the idea is as follows:
//  - shader compilation:
//      - lower complex (vector, smoothstep, lerps, etc) operations into simpler ones
//        (i.e. preserve only simd friendly ops)
//      - lower the shader graph ops into a simplified bytecode
//      - make the bytecode strongly typed and execution context aware
//        (i.e. per quad/per frame ops should get their own instructions that dont do any per pixel ops)
//      - because picodraw doesnt support any complicated control flow it should be a breeze to implement
//      - preferably don't do any jit (even though it would be cool)
//  - shader execution:
//      - split dispatch group (quad list) into a bunch of tiles (16x16 or 8x8)
//      - run each tile in a scoped thread pool (rayon?)
//      - each tile has a list of `actions` (shader, quad data, quad bounds) it should execute in sequence
//      - every `action` is a shader invocation, which should use SIMD to process multiple pixels in lockstep
//      - because we dont allow complex control flow no divergence is possible
//      - after each action invocation we do `masking` (cutting off the pixels that are out of bounds)
//        and `blending` (mixing the result with the local buffer)
//      - after every action is executed we blit the local tile buffer to the global target buffer
