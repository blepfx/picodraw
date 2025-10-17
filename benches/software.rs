use gungraun::{Callgrind, FlamegraphConfig, LibraryBenchmarkConfig, library_benchmark, library_benchmark_group, main};
use picodraw::shader::*;
use picodraw::software::*;
use picodraw::*;
use std::hint::black_box;

fn shader_circles() -> float4 {
    let z = io::read::<f32>();
    let x = io::position();
    let r = io::resolution().x().max(io::resolution().y());

    let sdf = (x - r * 0.5).len() - r * z * 0.25;
    let alpha = 1.0 - sdf.smoothstep(-0.6, 0.6);

    float4((x.x() / r, x.y() / r, 1.0, alpha * 0.5))
}

#[library_benchmark]
fn bench_circles_compile() {
    let mut backend = SoftwareBackend::single_threaded();
    let mut cx = backend.open(BufferMut::default());
    let shader = cx.create_shader(Graph::trace(shader_circles));
    black_box(shader);
    cx.delete_shader(shader);
}

#[library_benchmark]
fn bench_circles_draw() {
    let mut buffer = vec![0u32; 256 * 256];
    let mut backend = SoftwareBackend::single_threaded();
    let shader = backend
        .open(BufferMut::default())
        .create_shader(Graph::trace(shader_circles));

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
fn bench_circles_draw_threaded() {
    let mut buffer = vec![0u32; 256 * 256];
    let mut backend = SoftwareBackend::multi_threaded();
    let shader = backend
        .open(BufferMut::default())
        .create_shader(Graph::trace(shader_circles));

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

library_benchmark_group! {
    name = picodraw_software;
    benchmarks =
        bench_circles_compile,
        bench_circles_draw,
        bench_circles_draw_threaded
}

main!(
    config = LibraryBenchmarkConfig::default().tool(Callgrind::default().flamegraph(FlamegraphConfig::default()));
    library_benchmark_groups = picodraw_software
);
