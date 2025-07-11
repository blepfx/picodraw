mod collect;
mod op;

use std::fmt::Debug;
use std::hash::{DefaultHasher, Hash, Hasher};

pub use op::*;

/// A shader graph.
///
/// Defines a computation graph for pixel color computation based on pixel position, arbitrary dynamic data and other information.
/// The graph is represented by a list of operations ([`OpValue`]) that each define a value computed based on other operations ([`OpAddr`]).
pub struct Graph {
    ops: Vec<GraphOpData>,
    output: OpAddr,
    hash: u64,
}

#[derive(Clone)]
pub struct GraphBuilder {
    ops: Vec<OpValue>,
}

#[derive(Debug)]
pub enum GraphError {
    TypeCheck { op: OpAddr, value: OpValue },
}

impl Graph {
    pub fn iter(&self) -> impl Iterator<Item = OpAddr> + DoubleEndedIterator + '_ {
        (0..self.ops.len()).map(OpAddr::from_raw)
    }

    pub fn value_of(&self, addr: OpAddr) -> OpValue {
        self.ops[addr.into_raw()].value
    }

    pub fn type_of(&self, addr: OpAddr) -> OpType {
        self.ops[addr.into_raw()].type_
    }

    pub fn dependencies_of(&self, addr: OpAddr) -> impl Iterator<Item = OpAddr> + '_ {
        self.ops[addr.into_raw()].value.iter_dependencies()
    }

    pub fn dependents_of(&self, addr: OpAddr) -> impl Iterator<Item = OpAddr> + '_ {
        self.ops[addr.into_raw()].dependants.iter().copied()
    }

    pub fn output(&self) -> OpAddr {
        self.output
    }

    pub fn hash(&self) -> u64 {
        self.hash
    }

    pub fn len(&self) -> u32 {
        self.ops.len() as u32
    }
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self { ops: vec![] }
    }

    pub fn push(&mut self, op: OpValue) -> OpAddr {
        self.ops.push(op);
        OpAddr::from_raw(self.ops.len() - 1)
    }

    pub fn finish(self, output: OpAddr) -> Result<Graph, GraphError> {
        let mut ops: Vec<GraphOpData> = Vec::new();
        let mut hasher = DefaultHasher::new();

        for (op, value) in self.ops.iter().enumerate() {
            let op = OpAddr::from_raw(op);

            let type_ = match value.type_check(|addr| Some(ops[addr.into_raw()].type_)) {
                Some(type_) => type_,
                None => {
                    return Err(GraphError::TypeCheck {
                        op,
                        value: value.clone(),
                    });
                }
            };

            for dep in value.iter_dependencies() {
                ops[dep.into_raw()].dependants.push(op);
            }

            ops.push(GraphOpData {
                value: value.clone(),
                type_,
                dependants: Vec::new(),
            });

            value.hash(&mut hasher);
        }

        Ok(Graph {
            ops,
            hash: hasher.finish(),
            output,
        })
    }
}

impl Debug for Graph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Graph(hash = {:x}) {{", self.hash())?;
        for addr in self.iter() {
            let op = self.value_of(addr);
            let ty = self.type_of(addr);
            writeln!(f, "\t{:?} {:?} = {:?}", addr, ty, op)?;
        }
        writeln!(f, "}}")?;

        Ok(())
    }
}

struct GraphOpData {
    value: OpValue,
    type_: OpType,
    dependants: Vec<OpAddr>,
}
