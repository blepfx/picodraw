use criterion::{Criterion, criterion_group, criterion_main};
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

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("circles (compile)", |b| {
        let mut backend = SoftwareBackend::new();

        b.iter(|| {
            let mut cx = backend.open(BufferMut::default());
            let shader = cx.create_shader(Graph::trace(shader_circles));
            black_box(shader);
            cx.delete_shader(shader);
        });
    });

    c.bench_function("circles (draw)", |b| {
        let mut buffer = vec![0u32; 512 * 512];
        let mut backend = SoftwareBackend::new();
        let shader = backend
            .open(BufferMut::default())
            .create_shader(Graph::trace(shader_circles));

        b.iter(|| {
            let mut cx = backend.open(BufferMut::from_slice(&mut buffer, 512, 512));
            let mut commands = vec![];

            for i in 0..10 {
                commands.push(Command::ObjectBegin([0, 0, 512, 512].into(), shader));
                commands.push(Command::ObjectData(QuadData::Float(2.0 - i as f32 * 0.15)));
                commands.push(Command::ObjectEnd);
            }

            cx.draw_screen(&commands).unwrap();
            black_box(&buffer);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
