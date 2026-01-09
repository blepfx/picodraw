use picodraw_core::{ShaderData, TextureFilter};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlslType {
    Float1,
    Float2,
    Float4,
    Int1,
    Int2,
    Int4,
    Bool1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlslExpr {
    LitFloat(u32),
    LitInt(i32),
    LitBool(bool),

    Add(u32, u32),
    Sub(u32, u32),
    Mul(u32, u32),
    Div(u32, u32),
    Min(u32, u32),
    Max(u32, u32),
    Neg(u32),

    INot(u32),
    IAnd(u32, u32),
    IOr(u32, u32),
    IXor(u32, u32),
    IShl(u32, u32),
    IShr(u32, u32),

    BNot(u32),
    BAnd(u32, u32),
    BOr(u32, u32),

    IRem(u32, u32),
    FRem(u32, u32),

    Lt(u32, u32),
    Le(u32, u32),
    Gt(u32, u32),
    Ge(u32, u32),
    Eq(u32, u32),
    Ne(u32, u32),

    Sin(u32),
    Cos(u32),
    Tan(u32),
    Asin(u32),
    Acos(u32),
    Atan(u32),
    Exp(u32),
    Log(u32),
    Sqrt(u32),
    Abs(u32),
    Sign(u32),
    Floor(u32),
    Pow(u32, u32),
    Atan2(u32, u32),
    DerivX(u32),
    DerivY(u32),

    Select(u32, u32, u32),
    Mix(u32, u32, u32),

    Swizzle1(u32, u8),
    Vec4([u32; 4]),

    AsInt(u32),
    AsFloat(u32),

    // -> float4
    DataFloat(u32),

    // -> int4
    DataInt(u32),

    // -> float4
    TextureSample(u32, TextureFilter, u32, u32), // tex, x, y

    // -> int2
    TextureSize(u32), // tex

    // -> float2
    Position,

    // -> float2
    Resolution,

    // -> float4
    QuadBounds,

    // float4 ->
    Output(u32),

    // -> _
    Variable(u32, GlslType), // type, index
}

#[derive(Debug)]
pub enum GlslStmt {
    DeclareVars {
        type_: GlslType,
        count: u32,
    },

    AssignExpr {
        var: u32,
        type_: GlslType,
        expr: GlslExpr,
    },

    ReturnExpr {
        expr: GlslExpr,
    },

    Branch {
        condition: GlslExpr,
        then_branch: Vec<GlslStmt>,
        else_branch: Vec<GlslStmt>,
    },
}

#[derive(Debug)]
pub struct GlslShader {
    pub expressions: HashMap<u32, GlslExpr>,
    pub statements: Vec<GlslStmt>,
    pub num_textures: u32,
}

impl GlslExpr {
    pub fn visit_deps(self, mut f: impl FnMut(u32)) {
        self.map_deps(|dep| {
            f(dep);
            dep
        });
    }

    pub fn map_deps(mut self, mut f: impl FnMut(u32) -> u32) -> Self {
        match &mut self {
            GlslExpr::LitFloat(_)
            | GlslExpr::LitInt(_)
            | GlslExpr::LitBool(_)
            | GlslExpr::Position
            | GlslExpr::Resolution
            | GlslExpr::QuadBounds
            | GlslExpr::DataFloat(_)
            | GlslExpr::DataInt(_)
            | GlslExpr::TextureSize(_)
            | GlslExpr::Variable(_, _) => {}

            GlslExpr::Add(a, b)
            | GlslExpr::Sub(a, b)
            | GlslExpr::Mul(a, b)
            | GlslExpr::Div(a, b)
            | GlslExpr::Min(a, b)
            | GlslExpr::Max(a, b)
            | GlslExpr::IRem(a, b)
            | GlslExpr::FRem(a, b)
            | GlslExpr::Lt(a, b)
            | GlslExpr::Le(a, b)
            | GlslExpr::Gt(a, b)
            | GlslExpr::Ge(a, b)
            | GlslExpr::Eq(a, b)
            | GlslExpr::Ne(a, b)
            | GlslExpr::Pow(a, b)
            | GlslExpr::Atan2(a, b)
            | GlslExpr::IAnd(a, b)
            | GlslExpr::IOr(a, b)
            | GlslExpr::IXor(a, b)
            | GlslExpr::IShl(a, b)
            | GlslExpr::IShr(a, b)
            | GlslExpr::BAnd(a, b)
            | GlslExpr::BOr(a, b)
            | GlslExpr::TextureSample(_, _, a, b) => {
                *a = f(*a);
                *b = f(*b);
            }

            GlslExpr::Neg(a)
            | GlslExpr::INot(a)
            | GlslExpr::BNot(a)
            | GlslExpr::Sin(a)
            | GlslExpr::Cos(a)
            | GlslExpr::Tan(a)
            | GlslExpr::Asin(a)
            | GlslExpr::Acos(a)
            | GlslExpr::Atan(a)
            | GlslExpr::Exp(a)
            | GlslExpr::Log(a)
            | GlslExpr::Sqrt(a)
            | GlslExpr::Abs(a)
            | GlslExpr::Sign(a)
            | GlslExpr::Floor(a)
            | GlslExpr::DerivX(a)
            | GlslExpr::DerivY(a)
            | GlslExpr::AsInt(a)
            | GlslExpr::AsFloat(a)
            | GlslExpr::Swizzle1(a, _)
            | GlslExpr::Output(a) => {
                *a = f(*a);
            }

            GlslExpr::Select(t, a, b) | GlslExpr::Mix(a, b, t) => {
                *t = f(*t);
                *a = f(*a);
                *b = f(*b);
            }

            GlslExpr::Vec4(elements) => {
                for e in elements {
                    *e = f(*e);
                }
            }
        }

        self
    }
}

pub fn process(shader: &ShaderData) -> GlslShader {
    let graph = graph::transpile(shader);
    let graph = optimize::prune(&graph);
    let graph = optimize::hashcons(&graph);
    let vars = variable::allocate(&graph);

    let mut expressions: HashMap<u32, GlslExpr> = HashMap::new();
    let mut statements: Vec<GlslStmt> = vec![];

    //dbg!(scope::discover(&graph));

    for (type_, count) in vars.counts.into_iter() {
        statements.push(GlslStmt::DeclareVars { type_, count });
    }

    for (node, expr, ty) in graph.iter() {
        if let GlslExpr::Output(output) = expr {
            statements.push(GlslStmt::ReturnExpr {
                expr: expressions[&output],
            });
            continue;
        }

        match vars.nodes.get(&node) {
            Some(&var) => {
                expressions.insert(node, GlslExpr::Variable(var, ty));
                statements.push(GlslStmt::AssignExpr { var, expr, type_: ty });
            }
            None => {
                expressions.insert(node, expr);
            }
        }
    }

    GlslShader {
        expressions,
        statements,
        num_textures: texture::count_textures(&graph),
    }
}

/// build a DAG of GLSL expression nodes
mod graph {
    use super::{GlslExpr, GlslType};
    use picodraw_core::{ShaderData, ShaderOp, ShaderOpType};
    use std::collections::HashMap;

    pub fn transpile(shader: &ShaderData) -> GlslGraph {
        let mut graph = GlslGraph::default();
        let mut map_orig: HashMap<u32, u32> = HashMap::new();

        for (index, op) in shader.iter() {
            let (expr, ty) = transpile_single(|expr, ty| graph.add(expr, ty), |i| map_orig[&i], op);
            map_orig.insert(index, graph.add(expr, ty));
        }

        let outputs = shader.output().map(|i| map_orig[&i]);
        let output = graph.add(GlslExpr::Vec4(outputs), GlslType::Float4);
        graph.add(GlslExpr::Output(output), GlslType::Float4);
        graph
    }

    fn transpile_single(
        mut emit: impl FnMut(GlslExpr, GlslType) -> u32, //map expression (possibly new) to glsl expr index
        map: impl Fn(u32) -> u32,                        //map picodraw index to glsl expr index
        op: ShaderOp,
    ) -> (GlslExpr, GlslType) {
        let expr = match op {
            ShaderOp::FLit(v) => GlslExpr::LitFloat(v.to_bits()),
            ShaderOp::ILit(v) => GlslExpr::LitInt(v),
            ShaderOp::BLit(v) => GlslExpr::LitBool(v),
            ShaderOp::Acos(x) => GlslExpr::Acos(map(x)),
            ShaderOp::Asin(x) => GlslExpr::Asin(map(x)),
            ShaderOp::Atan(x) => GlslExpr::Atan(map(x)),
            ShaderOp::Cos(x) => GlslExpr::Cos(map(x)),
            ShaderOp::Sin(x) => GlslExpr::Sin(map(x)),
            ShaderOp::Tan(x) => GlslExpr::Tan(map(x)),
            ShaderOp::Exp(x) => GlslExpr::Exp(map(x)),
            ShaderOp::Ln(x) => GlslExpr::Log(map(x)),
            ShaderOp::Sqrt(x) => GlslExpr::Sqrt(map(x)),
            ShaderOp::PosX => GlslExpr::Swizzle1(emit(GlslExpr::Position, GlslType::Float2), 0),
            ShaderOp::PosY => GlslExpr::Swizzle1(emit(GlslExpr::Position, GlslType::Float2), 1),
            ShaderOp::ResX => GlslExpr::Swizzle1(emit(GlslExpr::Resolution, GlslType::Float2), 0),
            ShaderOp::ResY => GlslExpr::Swizzle1(emit(GlslExpr::Resolution, GlslType::Float2), 1),
            ShaderOp::QuadL => GlslExpr::Swizzle1(emit(GlslExpr::QuadBounds, GlslType::Float4), 0),
            ShaderOp::QuadT => GlslExpr::Swizzle1(emit(GlslExpr::QuadBounds, GlslType::Float4), 1),
            ShaderOp::QuadR => GlslExpr::Swizzle1(emit(GlslExpr::QuadBounds, GlslType::Float4), 2),
            ShaderOp::QuadB => GlslExpr::Swizzle1(emit(GlslExpr::QuadBounds, GlslType::Float4), 3),
            ShaderOp::FAdd(a, b) | ShaderOp::IAdd(a, b) => GlslExpr::Add(map(a), map(b)),
            ShaderOp::FSub(a, b) | ShaderOp::ISub(a, b) => GlslExpr::Sub(map(a), map(b)),
            ShaderOp::FMul(a, b) | ShaderOp::IMul(a, b) => GlslExpr::Mul(map(a), map(b)),
            ShaderOp::FDiv(a, b) | ShaderOp::IDiv(a, b) => GlslExpr::Div(map(a), map(b)),
            ShaderOp::FMin(a, b) | ShaderOp::IMin(a, b) => GlslExpr::Min(map(a), map(b)),
            ShaderOp::FMax(a, b) | ShaderOp::IMax(a, b) => GlslExpr::Max(map(a), map(b)),
            ShaderOp::FNeg(a) | ShaderOp::INeg(a) => GlslExpr::Neg(map(a)),
            ShaderOp::FSign(a) | ShaderOp::ISign(a) => GlslExpr::Sign(map(a)),
            ShaderOp::FAbs(x) | ShaderOp::IAbs(x) => GlslExpr::Abs(map(x)),
            ShaderOp::FMod(a, b) => GlslExpr::FRem(map(a), map(b)),
            ShaderOp::IMod(a, b) => GlslExpr::IRem(map(a), map(b)),
            ShaderOp::Atan2(a, b) => GlslExpr::Atan2(map(a), map(b)),
            ShaderOp::Pow(a, b) => GlslExpr::Pow(map(a), map(b)),
            ShaderOp::Floor(a) => GlslExpr::Floor(map(a)),
            ShaderOp::Lerp(t, a, b) => GlslExpr::Mix(map(a), map(b), map(t)),
            ShaderOp::DerivX(a) => GlslExpr::DerivX(map(a)),
            ShaderOp::DerivY(a) => GlslExpr::DerivY(map(a)),
            ShaderOp::IOr(a, b) => GlslExpr::IOr(map(a), map(b)),
            ShaderOp::IAnd(a, b) => GlslExpr::IAnd(map(a), map(b)),
            ShaderOp::IXor(a, b) => GlslExpr::IXor(map(a), map(b)),
            ShaderOp::IShl(a, b) => GlslExpr::IShl(map(a), map(b)),
            ShaderOp::IShr(a, b) => GlslExpr::IShr(map(a), map(b)),
            ShaderOp::INot(a) => GlslExpr::INot(map(a)),
            ShaderOp::BOr(a, b) => GlslExpr::BOr(map(a), map(b)),
            ShaderOp::BAnd(a, b) => GlslExpr::BAnd(map(a), map(b)),
            ShaderOp::BXor(a, b) => GlslExpr::Ne(map(a), map(b)),
            ShaderOp::BNot(a) => GlslExpr::BNot(map(a)),
            ShaderOp::IEq(a, b) | ShaderOp::FEq(a, b) => GlslExpr::Eq(map(a), map(b)),
            ShaderOp::INe(a, b) | ShaderOp::FNe(a, b) => GlslExpr::Ne(map(a), map(b)),
            ShaderOp::ILt(a, b) | ShaderOp::FLt(a, b) => GlslExpr::Lt(map(a), map(b)),
            ShaderOp::ILe(a, b) | ShaderOp::FLe(a, b) => GlslExpr::Le(map(a), map(b)),
            ShaderOp::IGt(a, b) | ShaderOp::FGt(a, b) => GlslExpr::Gt(map(a), map(b)),
            ShaderOp::IGe(a, b) | ShaderOp::FGe(a, b) => GlslExpr::Ge(map(a), map(b)),

            ShaderOp::FSelect(t, a, b) | ShaderOp::BSelect(t, a, b) | ShaderOp::ISelect(t, a, b) => {
                GlslExpr::Select(map(t), map(a), map(b))
            }

            ShaderOp::FCastInt32(a) => GlslExpr::AsFloat(map(a)),
            ShaderOp::ICastFloat(a) => GlslExpr::AsInt(map(a)),

            ShaderOp::TexW(t) => GlslExpr::Swizzle1(emit(GlslExpr::TextureSize(t), GlslType::Int2), 0),
            ShaderOp::TexH(t) => GlslExpr::Swizzle1(emit(GlslExpr::TextureSize(t), GlslType::Int2), 1),
            ShaderOp::TexSample(t, x, y, texture_filter, texture_channel) => GlslExpr::Swizzle1(
                emit(
                    GlslExpr::TextureSample(t, texture_filter, map(x), map(y)),
                    GlslType::Float4,
                ),
                texture_channel as u8,
            ),
            ShaderOp::ReadF32(offset) => GlslExpr::Swizzle1(
                emit(GlslExpr::DataFloat(offset / 4), GlslType::Float4),
                (offset % 4) as u8,
            ),
            ShaderOp::ReadI32(offset) => {
                GlslExpr::Swizzle1(emit(GlslExpr::DataInt(offset / 4), GlslType::Int4), (offset % 4) as u8)
            }
            ShaderOp::ReadU16(offset) => {
                let (b16, b4, b1) = ((offset * 2) >> 4, ((offset * 2) >> 2) & 3, ((offset * 2) & 3) << 3);
                let a = GlslExpr::Swizzle1(emit(GlslExpr::DataInt(b16), GlslType::Int4), b4 as u8);
                let b = GlslExpr::IShr(
                    emit(a, GlslType::Int1),
                    emit(GlslExpr::LitInt(b1 as i32), GlslType::Int1),
                );
                GlslExpr::IAnd(emit(b, GlslType::Int1), emit(GlslExpr::LitInt(65535), GlslType::Int1))
            }
            ShaderOp::ReadU8(offset) => {
                let (b16, b4, b1) = (offset >> 4, (offset >> 2) & 3, (offset & 3) << 3);
                let a = GlslExpr::Swizzle1(emit(GlslExpr::DataInt(b16), GlslType::Int4), b4 as u8);
                let b = GlslExpr::IShr(
                    emit(a, GlslType::Int1),
                    emit(GlslExpr::LitInt(b1 as i32), GlslType::Int1),
                );
                GlslExpr::IAnd(emit(b, GlslType::Int1), emit(GlslExpr::LitInt(255), GlslType::Int1))
            }
        };

        let ty = match op.output_type() {
            ShaderOpType::Float => GlslType::Float1,
            ShaderOpType::Int32 => GlslType::Int1,
            ShaderOpType::Bool => GlslType::Bool1,
        };

        (expr, ty)
    }

    #[derive(Default)]
    pub struct GlslGraph {
        nodes: Vec<GlslNode>,
    }

    struct GlslNode {
        expr: GlslExpr,
        ty: GlslType,
        forward: Vec<u32>,
        backward: Vec<u32>,
    }

    impl GlslGraph {
        pub fn add(&mut self, expr: GlslExpr, ty: GlslType) -> u32 {
            let index = self.nodes.len() as u32;
            let mut node = GlslNode {
                expr,
                ty,
                forward: vec![],
                backward: vec![],
            };

            expr.visit_deps(|dep| {
                self.nodes[dep as usize].forward.push(index);
                node.backward.push(dep);
            });

            self.nodes.push(node);
            index
        }

        pub fn get(&self, index: u32) -> (GlslExpr, GlslType) {
            let node = &self.nodes[index as usize];
            (node.expr, node.ty)
        }

        pub fn iter(&self) -> impl ExactSizeIterator<Item = (u32, GlslExpr, GlslType)> + DoubleEndedIterator + '_ {
            self.nodes
                .iter()
                .enumerate()
                .map(|(i, node)| (i as u32, node.expr, node.ty))
        }

        pub fn forward(&self, index: u32) -> impl ExactSizeIterator<Item = u32> + DoubleEndedIterator + '_ {
            self.nodes[index as usize].forward.iter().copied()
        }

        pub fn backward(&self, index: u32) -> impl ExactSizeIterator<Item = u32> + DoubleEndedIterator + '_ {
            self.nodes[index as usize].backward.iter().copied()
        }
    }
}

/// DAG optimization stuff
mod optimize {
    use super::{GlslExpr, graph::GlslGraph};
    use std::collections::{HashMap, HashSet};

    /// Merge duplicate nodes in the graph
    pub fn hashcons(graph: &GlslGraph) -> GlslGraph {
        let mut result = GlslGraph::default();
        let mut mapping = HashMap::new();
        let mut cache = HashMap::new();

        for (index, expr, ty) in graph.iter() {
            match expr {
                GlslExpr::Output(a) => {
                    result.add(GlslExpr::Output(mapping[&a]), ty);
                }

                _ => {
                    let new_expr = expr.map_deps(|dep| mapping[&dep]);
                    if let Some(&cached) = cache.get(&new_expr) {
                        mapping.insert(index, cached);
                    } else {
                        let new_node = result.add(new_expr, ty);
                        cache.insert(new_expr, new_node);
                        mapping.insert(index, new_node);
                    }
                }
            }
        }

        result
    }

    /// Remove unused nodes from the graph
    pub fn prune(graph: &GlslGraph) -> GlslGraph {
        let mut usage = HashSet::new();

        if let Some((output, _, _)) = graph
            .iter()
            .rev()
            .find(|(_, expr, _)| matches!(expr, GlslExpr::Output(_)))
        {
            usage.insert(output);
        }

        for (index, expr, _) in graph.iter().rev() {
            if usage.contains(&index) {
                expr.visit_deps(|dep| {
                    usage.insert(dep);
                });
            }
        }

        let mut result = GlslGraph::default();
        let mut mapping = HashMap::new();

        for (index, expr, ty) in graph.iter() {
            if !usage.contains(&index) {
                continue;
            }

            let new_expr = expr.map_deps(|dep| mapping[&dep]);
            mapping.insert(index, result.add(new_expr, ty));
        }

        result
    }
}

/// split graph into scopes (if branches, etc)
mod scope {
    use super::{GlslExpr, graph::GlslGraph};
    use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

    #[derive(Debug, Clone)]
    pub enum Scoped {
        Expression(u32),
        Branch {
            condition: u32,
            outputs: Vec<(u32, u32)>,
            then_nodes: Vec<Scoped>,
            else_nodes: Vec<Scoped>,
        },
    }

    /// Find all possible branches (if statements) in the graph.
    /// For each branch, find the island of nodes that are only reachable from the branch itself (i.e. not used outside the branch).
    pub fn discover(graph: &GlslGraph) -> Vec<Scoped> {
        #[derive(Default, Debug)]
        struct Branch {
            then_island: BTreeSet<u32>,
            else_island: BTreeSet<u32>,
            outputs: Vec<(u32, u32)>,
        }

        fn emit_nodes(graph: &GlslGraph, nodes: Vec<u32>, branches: &mut BTreeMap<u32, Branch>) -> Vec<Scoped> {
            let mut result = vec![];
            for node in nodes {
                let (expr, _) = graph.get(node);
                match expr {
                    GlslExpr::Select(cond, ..) => {
                        if let Some(branch) = branches.remove(&cond) {
                            result.push(Scoped::Branch {
                                condition: cond,
                                outputs: branch.outputs,
                                then_nodes: emit_nodes(graph, branch.then_island.into_iter().collect(), branches),
                                else_nodes: emit_nodes(graph, branch.else_island.into_iter().collect(), branches),
                            });
                        }
                    }
                    _ => {
                        result.push(Scoped::Expression(node));
                    }
                }
            }
            result
        }

        // collect branch roots
        let mut branches = BTreeMap::<u32, Branch>::new();
        let mut taken = HashSet::new();

        for (_, expr, _) in graph.iter() {
            if let GlslExpr::Select(cond, then, else_) = expr {
                branches.entry(cond).or_default().outputs.push((then, else_));
            }
        }

        // for each branch, find its island
        for (_, branch) in branches.iter_mut() {
            branch.then_island = extend_island(graph, branch.outputs.iter().map(|x| x.0).collect());
            branch.else_island = extend_island(graph, branch.outputs.iter().map(|x| x.1).collect());

            branch.then_island.retain(|node| !taken.contains(node));
            branch.else_island.retain(|node| !taken.contains(node));

            taken.extend(branch.then_island.iter().chain(branch.else_island.iter()).copied());
        }

        let result = emit_nodes(
            graph,
            graph
                .iter()
                .map(|(index, _, _)| index)
                .filter(|index| !taken.contains(index))
                .collect(),
            &mut branches,
        );

        result
    }

    fn extend_island(graph: &GlslGraph, mut island: BTreeSet<u32>) -> BTreeSet<u32> {
        let mut waiting = HashMap::new();
        let mut exits = vec![];

        while let Some(entry) = island.pop_first() {
            match graph.forward(entry).count() {
                0 => unreachable!(),
                1 => exits.push(entry),
                n => {
                    waiting.insert(entry, n - 1);
                }
            }
        }

        island.extend(&exits);

        while !exits.is_empty() {
            for exit in exits.drain(..) {
                for back in graph.backward(exit) {
                    *waiting.entry(back).or_insert_with(|| graph.forward(back).count()) -= 1;
                }
            }

            waiting.retain(|node, uses| {
                if *uses == 0 {
                    exits.push(*node);
                    island.insert(*node);
                    return false;
                }

                true
            });
        }

        island
    }
}

/// assign variables to each node so they can be efficiently reused
mod variable {
    use super::{GlslType, graph::GlslGraph};
    use crate::compiler::analysis::GlslExpr;
    use std::collections::HashMap;

    pub struct Variables {
        pub counts: HashMap<GlslType, u32>,
        pub nodes: HashMap<u32, u32>,
    }

    /// Allocate variables for each node in the graph
    pub fn allocate(graph: &GlslGraph) -> Variables {
        let mut stacks = HashMap::new();
        let mut allocd = HashMap::new();

        for (index, expr, ty) in graph.iter() {
            if always_inline(&expr) || graph.forward(index).count() <= 1 {
                continue;
            }

            let output = alloc(&mut stacks, ty, graph.forward(index).count() as u32);
            allocd.insert(index, output);

            expr.visit_deps(|dep| {
                let (_, ty) = graph.get(dep);
                if let Some(reg) = allocd.get(&dep).copied() {
                    release(&mut stacks, ty, reg);
                }
            });
        }

        Variables {
            counts: stacks.into_iter().map(|(k, v)| (k, v.len() as u32)).collect(),
            nodes: allocd,
        }
    }

    fn alloc(vars: &mut HashMap<GlslType, Vec<u32>>, ty: GlslType, usages: u32) -> u32 {
        let vars = vars.entry(ty).or_default();

        // find first with 0 usages or allocate new
        match vars.iter().position(|&u| u == 0) {
            Some(pos) => {
                vars[pos] = usages;
                pos as u32
            }
            None => {
                let index = vars.len() as u32;
                vars.push(usages);
                index
            }
        }
    }

    fn release(vars: &mut HashMap<GlslType, Vec<u32>>, ty: GlslType, reg: u32) {
        let vars = vars.entry(ty).or_default();
        vars[reg as usize] -= 1;
    }

    fn always_inline(expr: &GlslExpr) -> bool {
        matches!(expr, GlslExpr::LitFloat(_) | GlslExpr::LitInt(_) | GlslExpr::LitBool(_))
    }
}

/// analyze texture usage
mod texture {
    use super::{GlslExpr, graph::GlslGraph};

    pub fn count_textures(graph: &GlslGraph) -> u32 {
        let mut num_textures = 0;

        for (_, expr, _) in graph.iter() {
            match expr {
                GlslExpr::TextureSize(tex) | GlslExpr::TextureSample(tex, _, _, _) => {
                    num_textures = num_textures.max(tex + 1);
                }
                _ => {}
            }
        }

        num_textures
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use picodraw_core::{select, trace::*};

    #[test]
    fn test_shader_op() {
        process(&ShaderData::trace(|| {
            fn sdf_circle(pos: float2, center: float2, radius: float1) -> float1 {
                (radius - (center - pos).len() + 0.5).clamp(0.0, 1.0)
            }

            let x = float1::read_f32(0);
            let y = float1::read_f32(4);
            let r = float1::read_f32(8);
            let q = int1::read_u8(12);

            let mask = sdf_circle(float2::position(), float2((x, y)), r);
            let color = select! {
                q.eq(0) => {
                    let x = x.sin();
                    let y = y.cos();
                    float4((x, y, 1.0, 1.0))
                },
                q.eq(1) => {
                    let z = select! {
                        x.lt(0.0) => x.cos(),
                        else => x.tan()
                    };

                    float4((z, z, z, z))
                },
                else => float4((0.5, 0.5, 1.0, 1.0))
            };

            float4((1.0, 1.0, 1.0, mask)) * color
        }));
    }
}
