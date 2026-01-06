//! Provides functionality for creating shader graphs via tracing (recording operations as they are executed).

mod math;
mod test;
pub use math::*;

use crate::{ShaderBuilder, ShaderData, ShaderOp};
use std::cell::RefCell;

thread_local! {
    static TRACE_BUILDER: RefCell<Option<ShaderBuilder>> = const { RefCell::new(None) };
}

pub(super) fn emit(op: ShaderOp) -> u32 {
    TRACE_BUILDER.with(|graph| {
        let mut builder = graph.borrow_mut();
        let graph = builder
            .as_mut()
            .expect("attempt to trace a graph operation outside of `TraceGraph::new`");

        graph.add(op)
    })
}

pub(super) fn inspect(id: u32) -> ShaderOp {
    TRACE_BUILDER.with(|graph| {
        let builder = graph.borrow();
        let graph = builder
            .as_ref()
            .expect("attempt to trace a graph operation outside of `TraceGraph::new`");

        graph.get(id)
    })
}

impl ShaderData {
    /// Create a graph via "tracing". The provided closure will be executed once, and
    /// all calls to operations in `picodraw::trace` will be recorded into a shader graph.
    ///
    /// # Panics
    /// This function will panic if it is called while another tracing session is already active.
    ///
    /// # Examples  
    /// ```rust
    /// use picodraw_core::{
    ///     trace::{float2, float4},
    ///     ShaderData,
    /// };
    ///
    /// let shader = ShaderData::trace(|| {
    ///     let uv = float2::position() / float2::resolution();
    ///     float4((uv.x(), uv.y(), 0.0, 1.0))
    /// });
    /// ```
    pub fn trace(f: impl FnOnce() -> float4) -> Self {
        TRACE_BUILDER.with_borrow_mut(|graph| {
            if graph.is_some() {
                panic!("already tracing");
            }

            graph.replace(ShaderBuilder::new());
        });

        let output = f();
        let builder = TRACE_BUILDER.with_borrow_mut(|graph| graph.take().expect("invalid tracing state"));
        builder.finish([output.x().0, output.y().0, output.z().0, output.w().0])
    }
}
