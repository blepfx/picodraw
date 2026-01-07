use picodraw_core::trace::*;
use picodraw_core::*;
use picodraw_software::*;

#[test]
fn test() {
    let mut buffer = vec![Color::default(); 512 * 512];
    let mut backend = SoftwareBackend::with_threads(1);
    let shader = backend
        .open(BufferMut::default())
        .create_shader(&ShaderData::trace(shader_rect))
        .unwrap();

    let mut context = backend.open(BufferMut::from_slice(&mut buffer, 512, 512));
    context.draw(DrawTarget::Screen, |encoder| {
        let mut random = fastrand::Rng::new();

        for _ in 0..1000 {
            let size = 128.0;
            let x = random.f32() * (512.0 - size) + size * 0.5;
            let y = random.f32() * (512.0 - size) + size * 0.5;

            let col_r = random.f32();
            let col_g = random.f32();
            let col_b = random.f32();
            let col_a = random.f32() * 0.1;

            let rad = random.f32() * 36.0 + 4.0;

            let min_x = (x - size).floor() as i32;
            let min_y = (y - size).floor() as i32;
            let max_x = (x + size).ceil() as i32;
            let max_y = (y + size).ceil() as i32;

            encoder.add_rect([min_x, min_y, max_x, max_y].into());
            encoder.add_data(&f32::to_ne_bytes(x));
            encoder.add_data(&f32::to_ne_bytes(y));
            encoder.add_data(&f32::to_ne_bytes(size));
            encoder.add_data(&f32::to_ne_bytes(size));
            encoder.add_data(&f32::to_ne_bytes(rad));
            encoder.add_data(&f32::to_ne_bytes(col_r));
            encoder.add_data(&f32::to_ne_bytes(col_g));
            encoder.add_data(&f32::to_ne_bytes(col_b));
            encoder.add_data(&f32::to_ne_bytes(col_a));
            encoder.draw(&shader);
        }
    });

    image::ImageBuffer::from_fn(512, 512, |x, y| {
        let pixel = buffer[(y * 512 + x) as usize];
        image::Rgba([pixel.r, pixel.g, pixel.b, pixel.a])
    })
    .save("test_output.webp")
    .unwrap();
}

fn shader_rect() -> float4 {
    let center = float2((float1::read_f32(0), float1::read_f32(4)));
    let extents = float2((float1::read_f32(8), float1::read_f32(12)));
    let radius = float1::read_f32(16);
    let color = float4((
        float1::read_f32(20),
        float1::read_f32(24),
        float1::read_f32(28),
        float1::read_f32(32),
    ));

    let p = float2::position() - center;
    let q = p.abs() - extents + radius;
    let d = q.x().max(q.y()).min(0.0) + q.max(0.0).len() - radius;

    let mask = (0.5 - d * 0.707).clamp(0.0, 1.0);
    float4((color.x(), color.y(), color.z(), color.w() * mask))
}
