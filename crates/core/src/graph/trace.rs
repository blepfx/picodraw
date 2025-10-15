use super::{Graph, GraphBuilder, OpAddr, OpValue};
use crate::{graph::GraphError, shader::float4};
use std::cell::RefCell;

thread_local! {
    static TRACE_GRAPH: RefCell<Option<GraphBuilder>> = const { RefCell::new(None) };
}

impl Graph {
    pub fn push_trace(op: OpValue) -> Result<OpAddr, GraphError> {
        TRACE_GRAPH.with(|graph| {
            let mut graph = graph.borrow_mut();
            let graph = graph
                .as_mut()
                .expect("attempt to trace a graph operation outside of `Graph::trace`");

            graph.push(op)
        })
    }

    pub fn trace(f: impl FnOnce() -> float4) -> Self {
        let prev = TRACE_GRAPH.with(|engine| engine.borrow_mut().replace(GraphBuilder::new()));
        let output = f();
        TRACE_GRAPH
            .with(|engine| std::mem::replace(&mut *engine.borrow_mut(), prev))
            .unwrap()
            .finish(output.0)
            .unwrap()
    }
}
