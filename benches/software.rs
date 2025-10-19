use gungraun::{Callgrind, FlamegraphConfig, LibraryBenchmarkConfig, library_benchmark, library_benchmark_group, main};
use picodraw::shader::*;
use picodraw::software::*;
use picodraw::*;
use std::hint::black_box;

fn shader_circle() -> float4 {
    let z = io::read::<f32>();
    let x = io::position();
    let r = io::resolution().x().max(io::resolution().y());

    let sdf = (x - r * 0.5).len() - r * z * 0.25;
    let alpha = 1.0 - sdf.smoothstep(-0.6, 0.6);

    float4((x.x() / r, x.y() / r, 1.0, alpha * 0.5))
}

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

#[library_benchmark]
fn bench_circle_compile() {
    let mut backend = SoftwareBackend::single_threaded();
    let mut cx = backend.open(BufferMut::default());
    let shader = cx.create_shader(Graph::trace(shader_circle));
    black_box(shader);
    cx.delete_shader(shader);
}

#[library_benchmark]
fn bench_rect_compile() {
    let mut backend = SoftwareBackend::single_threaded();
    let mut cx = backend.open(BufferMut::default());
    let shader = cx.create_shader(Graph::trace(shader_rect));
    black_box(shader);
    cx.delete_shader(shader);
}

#[library_benchmark]
fn bench_circle_draw() {
    let mut buffer = vec![0u32; 256 * 256];
    let mut backend = SoftwareBackend::single_threaded();
    let shader = backend
        .open(BufferMut::default())
        .create_shader(Graph::trace(shader_circle));

    let mut cx = backend.open(BufferMut::from_slice(&mut buffer, 256, 256));
    let mut commands = vec![];

    for i in 0..1000 {
        commands.push(Command::ObjectBegin(shader));
        commands.push(Command::ObjectRect([0, 0, 256, 256].into()));
        commands.push(Command::ObjectData(ObjectData::Float(2.0 - i as f32 * 0.15)));
        commands.push(Command::ObjectEnd);
    }

    cx.draw_screen(&commands).unwrap();
    black_box(&buffer);
}

#[library_benchmark]
fn bench_rect_draw() {
    let mut buffer = vec![0u32; 256 * 256];
    let mut backend = SoftwareBackend::single_threaded();
    let shader = backend
        .open(BufferMut::default())
        .create_shader(Graph::trace(shader_rect));

    let mut cx = backend.open(BufferMut::from_slice(&mut buffer, 256, 256));
    let mut commands = vec![];

    for i in 0..1000 {
        commands.push(Command::ObjectBegin(shader));
        commands.push(Command::ObjectRect([0, 0, 256, 256].into()));
        commands.push(Command::ObjectData(ObjectData::Float(256.0)));
        commands.push(Command::ObjectData(ObjectData::Float(256.0)));
        commands.push(Command::ObjectData(ObjectData::Float(i as f32 * 0.01)));
        commands.push(Command::ObjectData(ObjectData::Float(100.0)));
        commands.push(Command::ObjectData(ObjectData::Float(50.0)));
        commands.push(Command::ObjectData(ObjectData::Float(0.0)));
        commands.push(Command::ObjectData(ObjectData::Float(10.0)));
        commands.push(Command::ObjectData(ObjectData::Float(25.0)));
        commands.push(Command::ObjectData(ObjectData::Float(50.0)));
        commands.push(Command::ObjectData(ObjectData::Float(1.0)));
        commands.push(Command::ObjectData(ObjectData::Float(0.0)));
        commands.push(Command::ObjectData(ObjectData::Float(1.0)));
        commands.push(Command::ObjectData(ObjectData::Float(0.1)));
        commands.push(Command::ObjectEnd);
    }

    cx.draw_screen(&commands).unwrap();
    black_box(&buffer);
}

#[library_benchmark]
fn bench_circle_draw_threaded() {
    let mut buffer = vec![0u32; 256 * 256];
    let mut backend = SoftwareBackend::multi_threaded();
    let shader = backend
        .open(BufferMut::default())
        .create_shader(Graph::trace(shader_circle));

    let mut cx = backend.open(BufferMut::from_slice(&mut buffer, 256, 256));
    let mut commands = vec![];

    for i in 0..1000 {
        commands.push(Command::ObjectBegin(shader));
        commands.push(Command::ObjectRect([0, 0, 256, 256].into()));
        commands.push(Command::ObjectData(ObjectData::Float(2.0 - i as f32 * 0.15)));
        commands.push(Command::ObjectEnd);
    }

    cx.draw_screen(&commands).unwrap();
    black_box(&buffer);
}

#[library_benchmark]
fn bench_rect_draw_threaded() {
    let mut buffer = vec![0u32; 256 * 256];
    let mut backend = SoftwareBackend::multi_threaded();
    let shader = backend
        .open(BufferMut::default())
        .create_shader(Graph::trace(shader_rect));

    let mut cx = backend.open(BufferMut::from_slice(&mut buffer, 256, 256));
    let mut commands = vec![];

    for i in 0..1000 {
        commands.push(Command::ObjectBegin(shader));
        commands.push(Command::ObjectRect([0, 0, 256, 256].into()));
        commands.push(Command::ObjectData(ObjectData::Float(256.0)));
        commands.push(Command::ObjectData(ObjectData::Float(256.0)));
        commands.push(Command::ObjectData(ObjectData::Float(i as f32 * 0.01)));
        commands.push(Command::ObjectData(ObjectData::Float(100.0)));
        commands.push(Command::ObjectData(ObjectData::Float(50.0)));
        commands.push(Command::ObjectData(ObjectData::Float(0.0)));
        commands.push(Command::ObjectData(ObjectData::Float(10.0)));
        commands.push(Command::ObjectData(ObjectData::Float(25.0)));
        commands.push(Command::ObjectData(ObjectData::Float(50.0)));
        commands.push(Command::ObjectData(ObjectData::Float(1.0)));
        commands.push(Command::ObjectData(ObjectData::Float(0.0)));
        commands.push(Command::ObjectData(ObjectData::Float(1.0)));
        commands.push(Command::ObjectData(ObjectData::Float(0.1)));
        commands.push(Command::ObjectEnd);
    }

    cx.draw_screen(&commands).unwrap();
    black_box(&buffer);
}

library_benchmark_group! {
    name = circle;
    benchmarks =
        bench_circle_compile,
        bench_circle_draw,
        bench_circle_draw_threaded
}

library_benchmark_group! {
    name = rect;
    benchmarks =
        bench_rect_compile,
        bench_rect_draw,
        bench_rect_draw_threaded
}

main!(
    config = LibraryBenchmarkConfig::default().tool(Callgrind::default().flamegraph(FlamegraphConfig::default()));
    library_benchmark_groups = circle, rect
);
