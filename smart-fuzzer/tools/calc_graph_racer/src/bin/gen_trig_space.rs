//! W109 Phase-3: trig (G4-01) candidate spaces + probe pools for
//! SIN / COS / TAN / COT / SEC / CSC below the 2^27 guard.
//!
//! Confirmed on the recon witnesses: Excel = FPREM1(x, 2·FLDPI) reduction
//! then the x87 trig instruction (the hardware's own internal reduction and
//! the platform libm are both ruled out). Remaining axes:
//! * prem vs prem1 (residue in [-π,π] vs sign-of-x [0,2π));
//! * residue kept extended into the instruction vs stored to binary64;
//! * reciprocal staging for COT/SEC/CSC: 1/f strict vs double-rounded, or
//!   the cos/sin ratio in strict / double-rounded division.
//!
//! Graph arg: 0 = x.

use calc_graph_racer::dsl::{Candidate, ConstVal, EvalModel, Graph, GraphBuilder, NodeId, Op};
use calc_graph_racer::eval::format_bits_hex;
use calc_graph_racer::scheduler::ProbeCase;
use calc_graph_racer::score::WitnessArg;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq)]
enum Reduce {
    ViaSinShift,
    Pi2Quadrant,
    PiParity,
    Prem1Cont,
    Prem1Stored,
    PremCont,
    RawInstr,
    Platform,
}

fn reduced_input(b: &mut GraphBuilder, x: NodeId, reduce: Reduce) -> NodeId {
    match reduce {
        // Composite CRT chain ops own their reduction — hand them raw x.
        Reduce::RawInstr
        | Reduce::Platform
        | Reduce::PiParity
        | Reduce::Pi2Quadrant
        | Reduce::ViaSinShift => x,
        _ => {
            let pi = b.push(Op::Const(ConstVal::RomPi), EvalModel::X87_CONT);
            let two_pi = b.push(Op::Add(pi, pi), EvalModel::X87_CONT);
            let model = if reduce == Reduce::Prem1Stored {
                EvalModel::X87_STORED
            } else {
                EvalModel::X87_CONT
            };
            match reduce {
                Reduce::PremCont => b.push(Op::Prem { x, modulus: two_pi }, model),
                _ => b.push(Op::Prem1 { x, modulus: two_pi }, model),
            }
        }
    }
}

fn trig_node(b: &mut GraphBuilder, r: NodeId, reduce: Reduce, op: fn(NodeId) -> Op) -> NodeId {
    if matches!(
        reduce,
        Reduce::PiParity | Reduce::Pi2Quadrant | Reduce::ViaSinShift
    ) {
        // Swap in the composite CRT chain op matching the direct instruction.
        let probe = op(0);
        let chain: fn(NodeId) -> Op = match (reduce, probe) {
            (Reduce::ViaSinShift, Op::Cos(_)) => Op::CosViaSinShift,
            (Reduce::PiParity, Op::Sin(_)) => Op::SinPiParity,
            (Reduce::PiParity, Op::Cos(_)) => Op::CosPiParity,
            (Reduce::PiParity, Op::Tan(_)) => Op::TanPiReduced,
            (Reduce::Pi2Quadrant, Op::Sin(_)) => Op::SinPi2Quadrant,
            (Reduce::Pi2Quadrant, Op::Cos(_)) => Op::CosPi2Quadrant,
            (Reduce::Pi2Quadrant, Op::Tan(_)) => Op::TanPi2Quadrant,
            _ => unreachable!("reduction chains only wrap trig ops"),
        };
        return b.push(chain(r), EvalModel::X87_STORED);
    }
    let model = if reduce == Reduce::Platform {
        EvalModel::Strict
    } else {
        EvalModel::X87_STORED
    };
    b.push(op(r), model)
}

/// Like `trig_node` but the result stays EXTENDED (store=false).
fn trig_node_cont(b: &mut GraphBuilder, r: NodeId, reduce: Reduce, op: fn(NodeId) -> Op) -> NodeId {
    if matches!(
        reduce,
        Reduce::PiParity | Reduce::Pi2Quadrant | Reduce::ViaSinShift
    ) {
        let probe = op(0);
        let chain: fn(NodeId) -> Op = match (reduce, probe) {
            (Reduce::ViaSinShift, Op::Cos(_)) => Op::CosViaSinShift,
            (Reduce::PiParity, Op::Sin(_)) => Op::SinPiParity,
            (Reduce::PiParity, Op::Cos(_)) => Op::CosPiParity,
            (Reduce::PiParity, Op::Tan(_)) => Op::TanPiReduced,
            (Reduce::Pi2Quadrant, Op::Sin(_)) => Op::SinPi2Quadrant,
            (Reduce::Pi2Quadrant, Op::Cos(_)) => Op::CosPi2Quadrant,
            (Reduce::Pi2Quadrant, Op::Tan(_)) => Op::TanPi2Quadrant,
            _ => unreachable!("reduction chains only wrap trig ops"),
        };
        return b.push(chain(r), EvalModel::X87_CONT);
    }
    b.push(op(r), EvalModel::X87_CONT)
}

fn base_chain(op: fn(NodeId) -> Op, reduce: Reduce) -> (GraphBuilder, NodeId) {
    let mut b = GraphBuilder::new();
    let x = b.strict(Op::Arg(0));
    let r = reduced_input(&mut b, x, reduce);
    let t = trig_node(&mut b, r, reduce, op);
    (b, t)
}

fn reduce_tag(r: Reduce) -> &'static str {
    match r {
        Reduce::ViaSinShift => "viasinshift",
        Reduce::Pi2Quadrant => "pi2quadrant",
        Reduce::PiParity => "piparity",
        Reduce::Prem1Cont => "prem1cont",
        Reduce::Prem1Stored => "prem1stored",
        Reduce::PremCont => "premcont",
        Reduce::RawInstr => "raw",
        Reduce::Platform => "platform",
    }
}

const REDUCES: [Reduce; 8] = [
    Reduce::ViaSinShift,
    Reduce::Pi2Quadrant,
    Reduce::PiParity,
    Reduce::Prem1Cont,
    Reduce::Prem1Stored,
    Reduce::PremCont,
    Reduce::RawInstr,
    Reduce::Platform,
];

fn direct_candidates(name: &str, op: fn(NodeId) -> Op) -> Vec<Candidate> {
    REDUCES
        .iter()
        .filter(|&&r| r != Reduce::ViaSinShift || name == "cos")
        .map(|&reduce| {
            let (b, t) = base_chain(op, reduce);
            Candidate {
                id: format!("{name}-{}", reduce_tag(reduce)),
                description: format!("{name}: reduce={}", reduce_tag(reduce)),
                graph: b.finish(t),
            }
        })
        .collect()
}

#[derive(Clone, Copy, PartialEq)]
enum RecipKind {
    RecipStrict,
    RecipDr,
    RecipExt,
    RatioStrict,
    RatioDr,
    RatioExt,
}

/// COT = cos/sin (or 1/tan), CSC = 1/sin, SEC = 1/cos.
fn recip_candidates(
    name: &str,
    prim: fn(NodeId) -> Op,
    ratio: Option<(fn(NodeId) -> Op, fn(NodeId) -> Op)>, // (numerator, denominator)
) -> Vec<Candidate> {
    let mut out = Vec::new();
    for &reduce in REDUCES.iter() {
        if reduce == Reduce::ViaSinShift && name != "sec" {
            continue; // shift chain only exists for the cos primitive
        }
        for kind in [
            RecipKind::RecipStrict,
            RecipKind::RecipDr,
            RecipKind::RecipExt,
            RecipKind::RatioStrict,
            RecipKind::RatioDr,
            RecipKind::RatioExt,
        ] {
            if matches!(
                kind,
                RecipKind::RatioStrict | RecipKind::RatioDr | RecipKind::RatioExt
            ) && ratio.is_none()
            {
                continue;
            }
            // Extended stagings need an x87 base chain to stay unstored.
            if matches!(kind, RecipKind::RecipExt | RecipKind::RatioExt)
                && reduce == Reduce::Platform
            {
                continue;
            }
            let graph: Graph = match kind {
                RecipKind::RecipStrict | RecipKind::RecipDr => {
                    let (mut b, t) = base_chain(prim, reduce);
                    let model = if kind == RecipKind::RecipStrict {
                        EvalModel::Strict
                    } else {
                        EvalModel::X87_STORED
                    };
                    let o = b.push(Op::Recip(t), model);
                    b.finish(o)
                }
                RecipKind::RecipExt => {
                    // Trig value stays EXTENDED into the reciprocal; one store.
                    let mut b = GraphBuilder::new();
                    let x = b.strict(Op::Arg(0));
                    let r = reduced_input(&mut b, x, reduce);
                    let t = trig_node_cont(&mut b, r, reduce, prim);
                    let o = b.push(Op::Recip(t), EvalModel::X87_STORED);
                    b.finish(o)
                }
                RecipKind::RatioExt => {
                    let (num_op, den_op) = ratio.unwrap();
                    let mut b = GraphBuilder::new();
                    let x = b.strict(Op::Arg(0));
                    let r = reduced_input(&mut b, x, reduce);
                    let n = trig_node_cont(&mut b, r, reduce, num_op);
                    let d = trig_node_cont(&mut b, r, reduce, den_op);
                    let o = b.push(Op::Div(n, d), EvalModel::X87_STORED);
                    b.finish(o)
                }
                _ => {
                    let (num_op, den_op) = ratio.unwrap();
                    let mut b = GraphBuilder::new();
                    let x = b.strict(Op::Arg(0));
                    let r = reduced_input(&mut b, x, reduce);
                    let n = trig_node(&mut b, r, reduce, num_op);
                    let d = trig_node(&mut b, r, reduce, den_op);
                    let model = if kind == RecipKind::RatioStrict {
                        EvalModel::Strict
                    } else {
                        EvalModel::X87_STORED
                    };
                    let o = b.push(Op::Div(n, d), model);
                    b.finish(o)
                }
            };
            let ktag = match kind {
                RecipKind::RecipStrict => "recips",
                RecipKind::RecipDr => "recipdr",
                RecipKind::RecipExt => "recipext",
                RecipKind::RatioStrict => "ratios",
                RecipKind::RatioDr => "ratiodr",
                RecipKind::RatioExt => "ratioext",
            };
            out.push(Candidate {
                id: format!("{name}-{}-{ktag}", reduce_tag(reduce)),
                description: format!("{name}: reduce={} staging={ktag}", reduce_tag(reduce)),
                graph,
            });
        }
    }
    out
}

struct Rng(u64);
impl Rng {
    fn next_u64(&mut self) -> u64 {
        let mut v = self.0;
        v ^= v >> 12;
        v ^= v << 25;
        v ^= v >> 27;
        self.0 = v;
        v.wrapping_mul(0x2545F4914F6CDD1D)
    }
    fn uniform(&mut self, lo: f64, hi: f64) -> f64 {
        let u = (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
        lo + u * (hi - lo)
    }
}

fn pool(rng: &mut Rng, tag: &str, count: usize) -> Vec<ProbeCase> {
    let mut out = Vec::new();
    let mut push = |id: String, x: f64| {
        out.push(ProbeCase {
            id,
            args: vec![WitnessArg::Scalar(format_bits_hex(x))],
        })
    };
    // Catalog/fixed rows.
    for (i, x) in [
        797601.58,
        134217727.0,
        100000.0,
        961281.44,
        49.214601836,
        149.214601836,
        0.5,
        1.0e-8,
        -100000.0,
        -797601.58,
    ]
    .iter()
    .enumerate()
    {
        push(format!("{tag}-fixed-{i}"), *x);
    }
    // prem-vs-prem1 band: x mod 2π in (π, 2π).
    for i in 0..60 {
        let x = std::f64::consts::PI * (1.05 + 0.9 * (i as f64) / 60.0);
        push(format!("{tag}-band-{i:03}"), x);
        push(format!("{tag}-bandneg-{i:03}"), -x);
    }
    // Near multiples of π/2 across magnitudes (reduction stress).
    for i in 0..80 {
        let k = 1.0 + rng.uniform(0.0, 8.5e7).floor();
        let x = k * std::f64::consts::FRAC_PI_2 + rng.uniform(-1.0e-3, 1.0e-3);
        if x.abs() < 134217728.0 {
            push(format!("{tag}-nearpi2-{i:03}"), x);
        }
    }
    // Log-uniform magnitudes below the 2^27 guard.
    for i in 0..count {
        let e = rng.uniform(-8.0, 8.12);
        let mut x = (10.0f64).powf(e) * rng.uniform(0.1, 1.0);
        if rng.next_u64() % 2 == 0 {
            x = -x;
        }
        if x.abs() < 134217728.0 && x != 0.0 {
            push(format!("{tag}-rand-{i:04}"), x);
        }
    }
    out
}

fn write_json<T: serde::Serialize>(path: &PathBuf, value: &T) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, serde_json::to_string_pretty(value).unwrap()).unwrap();
}

fn main() {
    let root = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "../../work/w109".into()),
    );
    let sets: Vec<(&str, Vec<Candidate>)> = vec![
        ("SIN", direct_candidates("sin", Op::Sin)),
        ("COS", direct_candidates("cos", Op::Cos)),
        ("TAN", direct_candidates("tan", Op::Tan)),
        (
            "COT",
            recip_candidates("cot", Op::Tan, Some((Op::Cos, Op::Sin))),
        ),
        ("CSC", recip_candidates("csc", Op::Sin, None)),
        ("SEC", recip_candidates("sec", Op::Cos, None)),
    ];
    for (func, candidates) in sets {
        let dir = root.join(format!("G4-01-{}", func.to_lowercase()));
        write_json(&dir.join("candidates.json"), &candidates);
        let mut rng = Rng(0x0109_4701_0001 ^ func.len() as u64);
        let discovery = pool(&mut rng, "disc", 350);
        write_json(&dir.join("pool-discovery.json"), &discovery);
        let mut rng_h = Rng(0x0109_4701_BEEF ^ func.len() as u64);
        let heldout = pool(&mut rng_h, "held", 250);
        write_json(&dir.join("pool-heldout.json"), &heldout);
        println!(
            "{func}: {} candidates, {} discovery, {} heldout",
            candidates.len(),
            discovery.len(),
            heldout.len()
        );
    }
}
