#![cfg(test)]
use crate::{
    ShaderOp,
    trace::{ShaderData, boolean, float1, float2, float4, int1},
};

#[test]
fn trace_simple() {
    let shader = ShaderData::trace(|| {
        let a = float1(0.0);
        let b = float1(1.0);
        let c = a + b;
        let d = a - b;

        float4((c, c.sin(), c.cos(), d.abs()))
    });

    assert_eq!(
        shader.iter().map(|x| x.1).collect::<Vec<_>>(),
        vec![
            ShaderOp::FLit(0.0),  // a
            ShaderOp::FLit(1.0),  // b
            ShaderOp::FAdd(0, 1), // c
            ShaderOp::FSub(0, 1), // d
            ShaderOp::Sin(2),     // c.sin()
            ShaderOp::Cos(2),     // c.cos()
            ShaderOp::FAbs(3)     // d.abs()
        ]
    );

    assert!(!shader.is_empty());
    assert_eq!(shader.len(), 7);
    assert_eq!(shader.output(), [2, 4, 5, 6]);
}

#[test]
fn trace_literal() {
    ShaderData::trace(|| {
        let float = float1(42.0);
        let bool = boolean(true);
        let int = int1(64);

        let dynamic = float2::position().x();

        assert_eq!(float.as_lit(), Some(42.0));
        assert_eq!(bool.as_lit(), Some(true));
        assert_eq!(int.as_lit(), Some(64));
        assert_eq!(dynamic.as_lit(), None);

        float4(float)
    });
}
