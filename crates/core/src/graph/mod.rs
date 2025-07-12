mod op;
mod scope;

use std::fmt::{Debug, Display};
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
    ops: Vec<GraphOpData>,
}

#[derive(Debug)]
pub enum GraphError {
    InvalidReference { index: usize },
    InvalidType,
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

    pub fn push(&mut self, op: OpValue) -> Result<OpAddr, GraphError> {
        let output_addr = OpAddr::from_raw(self.ops.len());
        let output_type = op.infer_type(|input_addr| self.ops.get(input_addr.into_raw()).map(|op| op.type_));
        let output_type = match output_type {
            Some(ty) => ty,
            None => {
                for (index, dep) in op.iter_dependencies().enumerate() {
                    if self.ops.get(dep.into_raw()).is_none() {
                        return Err(GraphError::InvalidReference { index });
                    }
                }

                return Err(GraphError::InvalidType);
            }
        };

        for dep in op.iter_dependencies() {
            self.ops[dep.into_raw()].dependants.push(output_addr);
        }

        self.ops.push(GraphOpData {
            value: op,
            type_: output_type,
            dependants: Vec::new(),
        });

        Ok(output_addr)
    }

    pub fn finish(self, output: OpAddr) -> Result<Graph, GraphError> {
        match self.ops.get(output.into_raw()) {
            Some(op) if op.type_ != OpType::F4 => return Err(GraphError::InvalidType),
            None => return Err(GraphError::InvalidReference { index: 0 }),
            _ => {}
        }

        let hash = self
            .ops
            .iter()
            .fold(DefaultHasher::new(), |mut hasher, op| {
                op.value.hash(&mut hasher);
                hasher
            })
            .finish();

        Ok(Graph {
            ops: self.ops,
            hash,
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

impl Display for GraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphError::InvalidReference { index } => {
                write!(f, "argument #{index} refers to an invalid operation")
            }
            GraphError::InvalidType => {
                write!(f, "failed to infer the type of the operation")
            }
        }
    }
}

#[derive(Clone)]
struct GraphOpData {
    value: OpValue,
    type_: OpType,
    dependants: Vec<OpAddr>,
}
