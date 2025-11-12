mod math;

pub use math::*;

use crate::{OpFloat, ShaderData, ShaderOp};
use std::{cell::RefCell, sync::Arc};

thread_local! {
    static TRACE_GRAPH: RefCell<Option<Vec<ShaderOp>>> = const { RefCell::new(None) };
}

#[derive(Clone, Debug)]
pub struct TraceGraph {
    nodes: Arc<[ShaderOp]>,
    outputs: [OpFloat; 4],
}

pub(super) fn trace_emit(op: ShaderOp) -> u32 {
    TRACE_GRAPH.with(|graph| {
        let mut graph = graph.borrow_mut();
        let graph = graph
            .as_mut()
            .expect("attempt to trace a graph operation outside of `TraceGraph::new`");

        graph.push(op);
        graph.len() as u32 - 1
    })
}

pub(super) fn trace_patch(addr: u32, op: ShaderOp) {
    TRACE_GRAPH.with(|graph| {
        let mut graph = graph.borrow_mut();
        let graph = graph
            .as_mut()
            .expect("attempt to trace a graph operation outside of `TraceGraph::new`");
        graph[addr as usize] = op;
    })
}

impl TraceGraph {
    pub fn new(f: impl FnOnce() -> float4) -> Self {
        TRACE_GRAPH.with_borrow_mut(|graph| {
            if graph.is_some() {
                panic!("already tracing");
            }

            graph.replace(vec![]);
        });

        let output = f();
        let nodes = TRACE_GRAPH.with_borrow_mut(|graph| graph.take().unwrap());

        Self {
            nodes: nodes.into(),
            outputs: [output.x.0, output.y.0, output.z.0, output.w.0],
        }
    }
}

impl<'a> From<&'a TraceGraph> for ShaderData<'a> {
    fn from(value: &'a TraceGraph) -> Self {
        Self {
            nodes: &value.nodes,
            output: value.outputs,
        }
    }
}
