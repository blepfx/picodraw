use super::CompiledShader;
use bumpalo::Bump;
use picodraw_core::{
    Graph,
    shader::{float4, io},
};

#[test]
pub fn test_circles() {
    fn shader_circles() -> float4 {
        let z = io::read::<f32>();
        let x = io::position();
        let r = io::resolution().x().max(io::resolution().y());

        let sdf = (x - r * 0.5).len() - r * z * 0.25;
        let alpha = 1.0 - sdf.smoothstep(-0.6, 0.6);

        float4((x.x() / r, x.y() / r, 1.0, alpha * 0.5))
    }

    let arena = Bump::new();
    let graph = Graph::trace(shader_circles);
    let compiled = CompiledShader::compile(&arena, &graph);
    dbg!(compiled.static_program());
    dbg!(compiled.dynamic_program());
}
