use super::CompiledShader;
use bumpalo::Bump;
use picodraw_core::{
    Graph,
    shader::{float2, float4, io},
};

#[test]
pub fn test_circles() {
    fn shader_circles() -> float4 {
        let z = io::read::<f32>();
        let x = io::position();
        let r = io::resolution().x().max(io::resolution().y());

        let sdf = (x - r * 0.5).len() - r * z * 0.25;
        let alpha = (0.5 - 0.9 * sdf).clamp(0.0, 1.0);

        float4((x.x() / r, x.y() / r, 1.0, alpha * 0.5))
    }

    let arena = Bump::new();
    let graph = Graph::trace(shader_circles);
    let compiled = CompiledShader::compile(&arena, &graph);

    dbg!(compiled.static_program());
    dbg!(compiled.dynamic_program());
}

#[test]
pub fn test_round_rect() {
    fn shader_rect() -> float4 {
        let center = float2((io::read::<f32>(), io::read::<f32>()));
        let angle = io::read::<f32>();
        let extents = float2((io::read::<f32>(), io::read::<f32>()));
        let radius = float4((
            io::read::<f32>(),
            io::read::<f32>(),
            io::read::<f32>(),
            io::read::<f32>(),
        ));
        let color = float4((
            io::read::<f32>(),
            io::read::<f32>(),
            io::read::<f32>(),
            io::read::<f32>(),
        ));

        let p = io::position() - center;
        let p = float2((
            p.x() * angle.cos() - p.y() * angle.sin(),
            p.x() * angle.sin() + p.y() * angle.cos(),
        ));

        let r = p.x().gt(0.0).select(
            p.y().gt(0.0).select(radius.x(), radius.y()),
            p.y().gt(0.0).select(radius.z(), radius.w()),
        );

        let q = p.abs() - extents + r;
        let d = q.x().max(q.y()).min(0.0) + q.max(0.0).len() - r;

        let mask = (0.5 - d * 0.707).clamp(0.0, 1.0);
        float4((color.x(), color.y(), color.z(), color.w() * mask))
    }

    let arena = Bump::new();
    let graph = Graph::trace(shader_rect);
    let compiled = CompiledShader::compile(&arena, &graph);

    dbg!(compiled.static_program());
    dbg!(compiled.dynamic_program());
}
