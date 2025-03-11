use super::{Graph, GraphBuilder, OpAddr, OpValue};
use crate::shader::float4;
use std::{cell::RefCell, mem::replace};

thread_local! {
    static COLLECT_GRAPH: RefCell<Option<GraphBuilder>> = RefCell::new(None);
}

impl Graph {
    pub fn push_collect(op: OpValue) -> OpAddr {
        COLLECT_GRAPH.with(|graph| {
            let mut graph = graph.borrow_mut();
            let graph = graph
                .as_mut()
                .expect("not executing in a shader graph context");

            graph.push(op)
        })
    }

    pub fn collect(f: impl FnOnce() -> float4) -> Self {
        let prev = COLLECT_GRAPH
            .with(|engine| replace(&mut *engine.borrow_mut(), Some(GraphBuilder::new())));

        let output = f();

        COLLECT_GRAPH
            .with(|engine| std::mem::replace(&mut *engine.borrow_mut(), prev))
            .unwrap()
            .finish(output.0)
            .unwrap()
    }
}
