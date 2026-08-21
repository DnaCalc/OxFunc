//! Candidate-space enumerators for the layered search: association trees,
//! store-barrier masks, and eval-model assignments.

use crate::dsl::{EvalModel, Graph, GraphBuilder, Node, NodeId, Op};

/// Binary operator kinds for association enumeration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinKind {
    Add,
    Sub,
    Mul,
    Div,
}

impl BinKind {
    fn to_op(self, a: NodeId, b: NodeId) -> Op {
        match self {
            BinKind::Add => Op::Add(a, b),
            BinKind::Sub => Op::Sub(a, b),
            BinKind::Mul => Op::Mul(a, b),
            BinKind::Div => Op::Div(a, b),
        }
    }
}

/// All binary tree shapes over leaves `lo..=hi` (`ops[k]` sits at the boundary
/// between leaf `k` and leaf `k+1`, and operand order is preserved).
#[derive(Clone, Debug)]
enum Shape {
    Leaf(usize),
    Node(Box<Shape>, Box<Shape>, usize),
}

fn shapes(lo: usize, hi: usize) -> Vec<Shape> {
    if lo == hi {
        return vec![Shape::Leaf(lo)];
    }
    let mut out = Vec::new();
    for split in lo..hi {
        for left in shapes(lo, split) {
            for right in shapes(split + 1, hi) {
                out.push(Shape::Node(Box::new(left.clone()), Box::new(right), split));
            }
        }
    }
    out
}

fn materialize(
    shape: &Shape,
    b: &mut GraphBuilder,
    leaves: &[&dyn Fn(&mut GraphBuilder) -> NodeId],
    ops: &[BinKind],
    model: EvalModel,
) -> NodeId {
    match shape {
        Shape::Leaf(i) => leaves[*i](b),
        Shape::Node(l, r, k) => {
            let ln = materialize(l, b, leaves, ops, model);
            let rn = materialize(r, b, leaves, ops, model);
            b.push(ops[*k].to_op(ln, rn), model)
        }
    }
}

/// Enumerate every association (parenthesization) of the ordered expression
/// `leaf0 op0 leaf1 op1 leaf2 ...`, each as an independent [`Graph`]. Interior
/// nodes get `model`; leaves build whatever nodes they need. Catalan(n-1)
/// graphs for n leaves.
pub fn enumerate_associations(
    leaves: &[&dyn Fn(&mut GraphBuilder) -> NodeId],
    ops: &[BinKind],
    model: EvalModel,
) -> Vec<Graph> {
    assert_eq!(
        ops.len() + 1,
        leaves.len(),
        "need exactly one op per adjacent leaf pair"
    );
    shapes(0, leaves.len() - 1)
        .iter()
        .map(|shape| {
            let mut b = GraphBuilder::new();
            let root = materialize(shape, &mut b, leaves, ops, model);
            b.finish(root)
        })
        .collect()
}

/// Human-readable rendering of an association graph (for candidate
/// descriptions): the fully parenthesized expression with leaf labels.
pub fn describe_association(g: &Graph, leaf_labels: &[&str]) -> String {
    fn render(g: &Graph, id: NodeId, leaf_labels: &[&str], leaf_cursor: &mut usize) -> String {
        match &g.nodes[id].op {
            Op::Add(a, b) => binrender(g, *a, *b, "+", leaf_labels, leaf_cursor),
            Op::Sub(a, b) => binrender(g, *a, *b, "-", leaf_labels, leaf_cursor),
            Op::Mul(a, b) => binrender(g, *a, *b, "*", leaf_labels, leaf_cursor),
            Op::Div(a, b) => binrender(g, *a, *b, "/", leaf_labels, leaf_cursor),
            _ => {
                let label = leaf_labels
                    .get(*leaf_cursor)
                    .copied()
                    .unwrap_or("<leaf>")
                    .to_string();
                *leaf_cursor += 1;
                label
            }
        }
    }
    fn binrender(
        g: &Graph,
        a: NodeId,
        b: NodeId,
        op: &str,
        leaf_labels: &[&str],
        leaf_cursor: &mut usize,
    ) -> String {
        let left = render(g, a, leaf_labels, leaf_cursor);
        let right = render(g, b, leaf_labels, leaf_cursor);
        format!("({left} {op} {right})")
    }
    let mut cursor = 0usize;
    render(g, g.output, leaf_labels, &mut cursor)
}

/// Every store-barrier mask over the given x87 nodes: for each subset, `store`
/// is set on the subset and cleared on the rest. `node_ids` must all be
/// `EvalModel::X87` nodes; capped at 16 nodes (65536 variants).
pub fn store_mask_variants(g: &Graph, node_ids: &[NodeId]) -> Vec<Graph> {
    assert!(node_ids.len() <= 16, "store mask space too large");
    for id in node_ids {
        assert!(
            matches!(g.nodes[*id].model, EvalModel::X87 { .. }),
            "node {id} is not x87; store masks only apply to x87 nodes"
        );
    }
    let mut out = Vec::with_capacity(1 << node_ids.len());
    for mask in 0u32..(1u32 << node_ids.len()) {
        let mut variant = g.clone();
        for (bit, id) in node_ids.iter().enumerate() {
            if let Node {
                model: EvalModel::X87 { cw, .. },
                ..
            } = variant.nodes[*id]
            {
                variant.nodes[*id].model = EvalModel::X87 {
                    cw,
                    store: mask & (1 << bit) != 0,
                };
            }
        }
        out.push(variant);
    }
    out
}

/// Every assignment of the given models to the given nodes (cartesian
/// product). Capped so |models|^|nodes| stays under 100k variants.
pub fn model_variants(g: &Graph, node_ids: &[NodeId], models: &[EvalModel]) -> Vec<Graph> {
    let space = (models.len() as u64).pow(node_ids.len() as u32);
    assert!(
        space <= 100_000,
        "model assignment space too large ({space})"
    );
    let mut out = Vec::with_capacity(space as usize);
    let mut assignment = vec![0usize; node_ids.len()];
    loop {
        let mut variant = g.clone();
        for (slot, id) in node_ids.iter().enumerate() {
            variant.nodes[*id].model = models[assignment[slot]];
        }
        out.push(variant);
        // Odometer increment.
        let mut slot = 0;
        loop {
            if slot == node_ids.len() {
                return out;
            }
            assignment[slot] += 1;
            if assignment[slot] < models.len() {
                break;
            }
            assignment[slot] = 0;
            slot += 1;
        }
    }
}
