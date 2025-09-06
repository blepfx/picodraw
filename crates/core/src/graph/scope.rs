use super::{Graph, GraphBuilder, OpAddr, OpValue};
use crate::{graph::GraphError, shader::float4};
use std::cell::RefCell;

thread_local! {
    static SCOPE_GRAPH: RefCell<Option<GraphBuilder>> = const { RefCell::new(None) };
}

impl Graph {
    pub fn push_scope(op: OpValue) -> Result<OpAddr, GraphError> {
        SCOPE_GRAPH.with(|graph| {
            let mut graph = graph.borrow_mut();
            let graph = graph
                .as_mut()
                .expect("attempt to execute a graph operation outside of a graph scope");

            graph.push(op)
        })
    }

    pub fn scope(f: impl FnOnce() -> float4) -> Self {
        let prev = SCOPE_GRAPH.with(|engine| engine.borrow_mut().replace(GraphBuilder::new()));
        let output = f();
        SCOPE_GRAPH
            .with(|engine| std::mem::replace(&mut *engine.borrow_mut(), prev))
            .unwrap()
            .finish(output.0)
            .unwrap()
    }
}
