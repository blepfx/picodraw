use gungraun::{Callgrind, FlamegraphConfig, LibraryBenchmarkConfig, library_benchmark, library_benchmark_group, main};
use picodraw_core::trace::*;
use picodraw_core::*;
use picodraw_software::*;
use std::hint::black_box;

fn shader_circle() -> float4 {
    let z = float1::read_f32(0);
    let x = float2::position();
    let r = float2::resolution().x().max(float2::resolution().y());

    let sdf = (x - r * 0.5).len() - r * z * 0.25;
    let alpha = 1.0 - ((sdf + 0.6) / 1.2).clamp(0.0, 1.0);

    float4((x.x() / r, x.y() / r, 1.0, alpha * 0.5))
}

fn shader_rect() -> float4 {
    let center = float2((float1::read_f32(0), float1::read_f32(4)));
    let angle = float1::read_f32(8);
    let extents = float2((float1::read_f32(12), float1::read_f32(16)));
    let radius = float1::read_f32(20);
    let color = float4((
        float1::read_f32(24),
        float1::read_f32(28),
        float1::read_f32(32),
        float1::read_f32(36),
    ));

    let p = float2::position() - center;
    let p = float2((
        p.x() * angle.cos() - p.y() * angle.sin(),
        p.x() * angle.sin() + p.y() * angle.cos(),
    ));

    let q = p.abs() - extents + radius;
    let d = q.x().max(q.y()).min(0.0) + q.max(0.0).len() - radius;

    let mask = (0.5 - d * 0.707).clamp(0.0, 1.0);
    float4((color.x(), color.y(), color.z(), color.w() * mask))
}

#[library_benchmark]
fn bench_circle_compile() {
    let mut backend = SoftwareBackend::with_threads(1);
    let mut context = backend.open(BufferMut::default());
    let shader = context.create_shader(&ShaderData::trace(shader_circle)).unwrap();
    context.delete_shader(black_box(shader));
}

#[library_benchmark]
fn bench_rect_compile() {
    let mut backend = SoftwareBackend::with_threads(1);
    let mut context = backend.open(BufferMut::default());
    let shader = context.create_shader(&ShaderData::trace(shader_rect)).unwrap();
    context.delete_shader(black_box(shader));
}

#[library_benchmark]
fn bench_circle_draw() {
    let mut buffer = black_box(vec![Color::default(); 256 * 256]);
    let mut backend = SoftwareBackend::with_threads(1);
    let shader = backend
        .open(BufferMut::default())
        .create_shader(&ShaderData::trace(shader_circle))
        .unwrap();

    let mut context = backend.open(BufferMut::from_slice(&mut buffer, 256, 256));
    context.draw(DrawTarget::Screen, |encoder| {
        for i in 0..1000 {
            encoder.add_rect([0, 0, 256, 256].into());
            encoder.add_data(&f32::to_ne_bytes(2.0 - i as f32 * 0.015));
            encoder.draw(black_box(&shader));
        }
    });

    black_box(buffer);
}

#[library_benchmark]
fn bench_rect_draw() {
    let mut buffer = black_box(vec![Color::default(); 256 * 256]);
    let mut backend = SoftwareBackend::with_threads(1);
    let shader = backend
        .open(BufferMut::default())
        .create_shader(&ShaderData::trace(shader_rect))
        .unwrap();

    let mut context = backend.open(BufferMut::from_slice(&mut buffer, 256, 256));
    context.draw(DrawTarget::Screen, |encoder| {
        let mut random = black_box(fastrand::Rng::new());

        for _ in 0..1000 {
            let x = random.f32() * 256.0;
            let y = random.f32() * 256.0;
            let w = random.f32() * 50.0 + 10.0;
            let h = random.f32() * 50.0 + 10.0;
            let r = random.f32() * std::f32::consts::TAU;

            let col_r = random.f32();
            let col_g = random.f32();
            let col_b = random.f32();
            let col_a = random.f32() * 0.1;

            let rad = random.f32() * w.min(h);

            let min_x = (x - w.max(h)).floor() as i32;
            let min_y = (y - w.max(h)).floor() as i32;
            let max_x = (x + w.max(h)).ceil() as i32;
            let max_y = (y + w.max(h)).ceil() as i32;

            encoder.add_rect([min_x, min_y, max_x, max_y].into());
            encoder.add_data(&f32::to_ne_bytes(x));
            encoder.add_data(&f32::to_ne_bytes(y));
            encoder.add_data(&f32::to_ne_bytes(r));
            encoder.add_data(&f32::to_ne_bytes(w));
            encoder.add_data(&f32::to_ne_bytes(h));
            encoder.add_data(&f32::to_ne_bytes(rad));
            encoder.add_data(&f32::to_ne_bytes(col_r));
            encoder.add_data(&f32::to_ne_bytes(col_g));
            encoder.add_data(&f32::to_ne_bytes(col_b));
            encoder.add_data(&f32::to_ne_bytes(col_a));
            encoder.draw(black_box(&shader));
        }
    });

    black_box(buffer);
}

library_benchmark_group! {
    name = circle;
    benchmarks =
        bench_circle_compile,
        bench_circle_draw,
}

library_benchmark_group! {
    name = rect;
    benchmarks =
        bench_rect_compile,
        bench_rect_draw,
}

main!(
    config = LibraryBenchmarkConfig::default().tool(Callgrind::default().flamegraph(FlamegraphConfig::default()));
    library_benchmark_groups = circle, rect
);
