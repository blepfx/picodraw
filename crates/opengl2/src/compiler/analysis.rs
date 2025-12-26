use crate::compiler::CompilerError;
use picodraw_core2::{OpBool, ShaderData, ShaderOp, ShaderOpType};
use std::collections::HashMap;

#[derive(Debug)]
pub enum ShaderValue {
    Inline(ShaderOp),
    Register(u32, ShaderOpType),
}

#[derive(Debug)]
pub enum ShaderStatement {
    Assign {
        register: u32,
        value: ShaderValue,
    },

    Conditional {
        condition: u32,
        if_true: Box<[ShaderStatement]>,
        if_false: Box<[ShaderStatement]>,
    },
}

#[derive(Debug)]
pub struct ShaderStructure {
    pub registers_float: u32,
    pub registers_int32: u32,
    pub registers_bool: u32,
    pub num_textures: u32,

    pub values: HashMap<u32, ShaderValue>,
    pub statements: Vec<ShaderStatement>,
    pub outputs: [u32; 4],
}

pub fn shader_analysis(shader: ShaderData) -> Result<ShaderStructure, CompilerError> {
    let mut nodes: Vec<Node> = Vec::new();
    let mut branches: HashMap<OpBool, Branch> = HashMap::new();
    let mut num_textures = 0;

    // visit nodes (do typechk, discover usages, dependencies, branches and textures)
    for op in shader.nodes.iter().copied() {
        let index = nodes.len() as u32;
        let mut node = Node {
            op,
            forward: vec![],
            backward: vec![],
            register: None,
            branch: None,
        };

        visit_node_dependencies(op, |arg, ty| {
            let dep = nodes
                .get_mut(arg as usize)
                .ok_or(CompilerError::InvalidInstructionReference { index })?;

            if dep.op.output_type() != ty {
                return Err(CompilerError::InvalidInstructionType { index });
            }

            dep.forward.push(index);
            node.backward.push(arg);
            Ok(())
        })?;

        match op {
            ShaderOp::FSelect(cond, left, right)
            | ShaderOp::ISelect(cond, left, right)
            | ShaderOp::BSelect(cond, left, right) => {
                let branch = branches.entry(cond).or_default();
                branch.island_false.push(right);
                branch.island_true.push(left);
            }

            ShaderOp::TexH(tex)
            | ShaderOp::TexW(tex)
            | ShaderOp::TexSampleF32(tex, ..)
            | ShaderOp::TexSampleU8(tex, ..) => {
                num_textures = num_textures.max(tex + 1);
            }

            _ => {}
        }

        nodes.push(node);
    }

    // output check
    for index in shader.output.iter().copied() {
        let node = nodes
            .get_mut(index as usize)
            .ok_or(CompilerError::InvalidOutput { index })?;

        if node.op.output_type() != ShaderOpType::Float {
            return Err(CompilerError::InvalidOutput { index });
        }

        node.forward.push(index);
    }

    // find branch islands (subsets of the graph that are only used from within that branch)
    for (cond, branch) in branches.iter_mut() {
        visit_branch_island(&mut branch.island_false, |x| &nodes[x as usize]);
        visit_branch_island(&mut branch.island_true, |x| &nodes[x as usize]);

        branch.island_false.sort();
        branch.island_true.sort();

        for node in branch.island_false.iter().copied() {
            nodes[node as usize].branch.get_or_insert((*cond, false));
        }

        for node in branch.island_true.iter().copied() {
            nodes[node as usize].branch.get_or_insert((*cond, true));
        }
    }

    // register allocation
    let mut registers: HashMap<ShaderOpType, Vec<u32>> = HashMap::new();
    for node in 0..nodes.len() {
        for dependency in nodes[node].backward.iter() {
            let dependency = &nodes[*dependency as usize];
            if let Some(register) = dependency.register {
                registers.entry(dependency.op.output_type()).or_default()[register as usize] -= 1; //evil
            }
        }

        let mut alloc_register = |ty, usages| -> u32 {
            let registers = registers.entry(ty).or_default();
            let next_free = registers.iter().position(|x| *x == 0).unwrap_or(registers.len());

            if next_free == registers.len() {
                registers.push(usages);
            } else {
                registers[next_free] = usages;
            }

            next_free as u32
        };

        match nodes[node].op {
            ShaderOp::FLit(_)
            | ShaderOp::ILit(_)
            | ShaderOp::BLit(_)
            | ShaderOp::PosX
            | ShaderOp::PosY
            | ShaderOp::ResX
            | ShaderOp::ResY
            | ShaderOp::QuadB
            | ShaderOp::QuadL
            | ShaderOp::QuadT
            | ShaderOp::QuadR => {
                // do not alloc registers for these guys, inline em
            }

            ShaderOp::FSelect(cond, _, _) | ShaderOp::BSelect(cond, _, _) | ShaderOp::ISelect(cond, _, _) => {
                let branch = branches.get_mut(&cond).unwrap();

                branch.inline = branch
                    .island_false
                    .iter()
                    .chain(branch.island_true.iter())
                    .all(|node| nodes[*node as usize].register.is_none());

                let node = &mut nodes[node];
                if node.forward.len() > 1 || !branch.inline {
                    node.register = Some(alloc_register(node.op.output_type(), node.forward.len() as u32));
                }
            }

            _ => {
                let node = &mut nodes[node];
                if node.forward.len() > 1 {
                    node.register = Some(alloc_register(node.op.output_type(), node.forward.len() as u32));
                }
            }
        }
    }

    // emit
    let statements = emit_statement_list(
        &nodes,
        &branches,
        &mut (0..nodes.len() as u32).filter(|x| nodes[*x as usize].branch.is_none()),
    );

    Ok(ShaderStructure {
        registers_float: registers
            .get(&ShaderOpType::Float)
            .map(|x| x.len() as u32)
            .unwrap_or_default(),
        registers_int32: registers
            .get(&ShaderOpType::Int32)
            .map(|x| x.len() as u32)
            .unwrap_or_default(),
        registers_bool: registers
            .get(&ShaderOpType::Bool)
            .map(|x| x.len() as u32)
            .unwrap_or_default(),
        num_textures,

        values: nodes
            .iter()
            .zip(0u32..)
            .map(|(node, id)| (id, emit_node_value(node)))
            .collect(),
        statements,
        outputs: shader.output,
    })
}

fn emit_statement_list(
    nodes: &[Node],
    branches: &HashMap<OpBool, Branch>,
    node_ids: &mut dyn Iterator<Item = u32>,
) -> Vec<ShaderStatement> {
    let mut statements = vec![];

    for node_id in node_ids {
        let node = &nodes[node_id as usize];
        if node.forward.is_empty() {
            continue;
        }

        match node.op {
            ShaderOp::FSelect(cond, left, right)
            | ShaderOp::BSelect(cond, left, right)
            | ShaderOp::ISelect(cond, left, right) => {
                let branch = branches.get(&cond).unwrap();

                if branch.inline {
                    if let Some(register) = node.register {
                        statements.push(ShaderStatement::Assign {
                            register,
                            value: ShaderValue::Inline(node.op),
                        });
                    }

                    continue;
                }

                let register = node.register.unwrap();

                let mut true_block = emit_statement_list(
                    nodes,
                    branches,
                    &mut branch
                        .island_true
                        .iter()
                        .copied()
                        .filter(|id| nodes[*id as usize].branch == Some((cond, true))),
                );

                let mut false_block = emit_statement_list(
                    nodes,
                    branches,
                    &mut branch
                        .island_false
                        .iter()
                        .copied()
                        .filter(|id| nodes[*id as usize].branch == Some((cond, false))),
                );

                true_block.push(ShaderStatement::Assign {
                    register,
                    value: emit_node_value(&nodes[left as usize]),
                });

                false_block.push(ShaderStatement::Assign {
                    register,
                    value: emit_node_value(&nodes[right as usize]),
                });

                statements.push(ShaderStatement::Conditional {
                    condition: cond,
                    if_true: true_block.into_boxed_slice(),
                    if_false: false_block.into_boxed_slice(),
                });
            }

            _ => {
                if let Some(register) = node.register {
                    statements.push(ShaderStatement::Assign {
                        register,
                        value: ShaderValue::Inline(node.op),
                    });
                }
            }
        }
    }

    statements
}

fn emit_node_value(node: &Node) -> ShaderValue {
    match node.register {
        Some(register) => ShaderValue::Register(register, node.op.output_type()),
        None => ShaderValue::Inline(node.op),
    }
}

fn visit_branch_island<'a>(island: &mut Vec<u32>, info: impl Fn(u32) -> &'a Node) {
    island.dedup();

    let mut waiting = HashMap::new();
    let mut exits = vec![];

    for entry in island.drain(..) {
        match info(entry).forward.len() as u32 {
            0 => {} //??
            1 => exits.push(entry),
            n => {
                waiting.insert(entry, n - 1);
            }
        }
    }

    while exits.len() > 0 {
        for exit in exits.drain(..) {
            for back in info(exit).backward.iter().copied() {
                match waiting.entry(back) {
                    std::collections::hash_map::Entry::Occupied(mut entry) => {
                        *entry.get_mut() -= 1;
                    }
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        entry.insert(info(back).forward.len() as u32 - 1);
                    }
                }
            }
        }

        waiting.retain(|node, uses| {
            if *uses == 0 {
                exits.push(*node);
                island.push(*node);
                return false;
            }

            true
        });
    }
}

fn visit_node_dependencies(
    op: ShaderOp,
    mut dep: impl FnMut(u32, ShaderOpType) -> Result<(), CompilerError>,
) -> Result<(), CompilerError> {
    match op {
        ShaderOp::FLit(_)
        | ShaderOp::ILit(_)
        | ShaderOp::BLit(_)
        | ShaderOp::ReadF32(_)
        | ShaderOp::ReadI32(_)
        | ShaderOp::ReadU16(_)
        | ShaderOp::ReadU8(_)
        | ShaderOp::PosX
        | ShaderOp::PosY
        | ShaderOp::ResX
        | ShaderOp::ResY
        | ShaderOp::QuadB
        | ShaderOp::QuadL
        | ShaderOp::QuadT
        | ShaderOp::QuadR
        | ShaderOp::TexW(_)
        | ShaderOp::TexH(_) => {}

        ShaderOp::FAdd(a, b)
        | ShaderOp::FSub(a, b)
        | ShaderOp::FMul(a, b)
        | ShaderOp::FDiv(a, b)
        | ShaderOp::FMod(a, b)
        | ShaderOp::FMin(a, b)
        | ShaderOp::FMax(a, b)
        | ShaderOp::Atan2(a, b)
        | ShaderOp::Pow(a, b)
        | ShaderOp::FEq(a, b)
        | ShaderOp::FNe(a, b)
        | ShaderOp::FLt(a, b)
        | ShaderOp::FLe(a, b)
        | ShaderOp::FGt(a, b)
        | ShaderOp::FGe(a, b) => {
            dep(a, ShaderOpType::Float)?;
            dep(b, ShaderOpType::Float)?;
        }

        ShaderOp::IAdd(a, b)
        | ShaderOp::ISub(a, b)
        | ShaderOp::IMul(a, b)
        | ShaderOp::IDiv(a, b)
        | ShaderOp::IMod(a, b)
        | ShaderOp::IMin(a, b)
        | ShaderOp::IMax(a, b)
        | ShaderOp::IOr(a, b)
        | ShaderOp::IAnd(a, b)
        | ShaderOp::IXor(a, b)
        | ShaderOp::IShl(a, b)
        | ShaderOp::IShr(a, b)
        | ShaderOp::IEq(a, b)
        | ShaderOp::INe(a, b)
        | ShaderOp::ILt(a, b)
        | ShaderOp::ILe(a, b)
        | ShaderOp::IGt(a, b)
        | ShaderOp::IGe(a, b) => {
            dep(a, ShaderOpType::Int32)?;
            dep(b, ShaderOpType::Int32)?;
        }

        ShaderOp::FNeg(a)
        | ShaderOp::FAbs(a)
        | ShaderOp::Sin(a)
        | ShaderOp::Cos(a)
        | ShaderOp::Tan(a)
        | ShaderOp::Asin(a)
        | ShaderOp::Acos(a)
        | ShaderOp::Atan(a)
        | ShaderOp::Sqrt(a)
        | ShaderOp::Ln(a)
        | ShaderOp::Exp(a)
        | ShaderOp::Floor(a)
        | ShaderOp::DerivX(a)
        | ShaderOp::DerivY(a)
        | ShaderOp::ICastFloat(a) => {
            dep(a, ShaderOpType::Float)?;
        }

        ShaderOp::INot(a) | ShaderOp::INeg(a) | ShaderOp::IAbs(a) | ShaderOp::FCastInt32(a) => {
            dep(a, ShaderOpType::Int32)?;
        }

        ShaderOp::BOr(a, b) | ShaderOp::BAnd(a, b) | ShaderOp::BXor(a, b) => {
            dep(a, ShaderOpType::Bool)?;
            dep(b, ShaderOpType::Bool)?;
        }

        ShaderOp::BNot(a) => {
            dep(a, ShaderOpType::Bool)?;
        }

        ShaderOp::Lerp(a, b, c) => {
            dep(a, ShaderOpType::Float)?;
            dep(b, ShaderOpType::Float)?;
            dep(c, ShaderOpType::Float)?;
        }

        ShaderOp::FSelect(a, b, c) => {
            dep(a, ShaderOpType::Bool)?;
            dep(b, ShaderOpType::Float)?;
            dep(c, ShaderOpType::Float)?;
        }
        ShaderOp::ISelect(a, b, c) => {
            dep(a, ShaderOpType::Bool)?;
            dep(b, ShaderOpType::Int32)?;
            dep(c, ShaderOpType::Int32)?;
        }
        ShaderOp::BSelect(a, b, c) => {
            dep(a, ShaderOpType::Bool)?;
            dep(b, ShaderOpType::Bool)?;
            dep(c, ShaderOpType::Bool)?;
        }

        ShaderOp::TexSampleF32(_, a, b, _, _) | ShaderOp::TexSampleU8(_, a, b, _, _) => {
            dep(a, ShaderOpType::Float)?;
            dep(b, ShaderOpType::Float)?;
        }
    }

    Ok(())
}

struct Node {
    op: ShaderOp,
    forward: Vec<u32>,
    backward: Vec<u32>,
    branch: Option<(OpBool, bool)>,
    register: Option<u32>,
}

#[derive(Default)]
struct Branch {
    island_false: Vec<u32>,
    island_true: Vec<u32>,
    inline: bool,
}
