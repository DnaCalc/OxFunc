//! W109 Phase-2: ACOTH (G4-03) candidate space + probe pools.
//!
//! ACOTH(x) = 0.5·ln1p(2/(x-1)) on the positive branch (Excel is exactly
//! odd-symmetric — negatives validated separately through the kernel).
//!
//! Leading hypothesis: the legacy CRT `log1p` (`87tran` fFLNP1 shape) —
//! `FYL2XP1` when `t` is inside its architectural range (`|t| < 1-sqrt(2)/2`),
//! else `FYL2X` on an EXTENDED, unstored `1+t`. Both witnesses (x=5, x=10)
//! are 1 ULP off the platform-`ln_1p` path and straddle the branch threshold
//! (t = 0.5 vs 2/9).
//!
//! Graph arg: 0 = x (> 1). The 0.5 scaling is exact (power of two) — not an
//! axis. Candidates:
//! * crt-{s,dr}    — legacy CRT log1p, t staged strict vs double-rounded;
//! * ucrt          — platform ln_1p (current OxFunc, control);
//! * portable      — in-crate faithful log1p (control);
//! * xlnratio-{s,spill} — x87 worksheet ln of (x+1)/(x-1) (control; should
//!   die at large x);
//! * fyl2xp1-always — FYL2XP1 with no range branch (control).

use calc_graph_racer::dsl::{
    Candidate, ConstVal, EvalModel, GraphBuilder, NodeId, Op, Pred,
};
use calc_graph_racer::eval::format_bits_hex;
use calc_graph_racer::scheduler::ProbeCase;
use calc_graph_racer::score::WitnessArg;
use std::path::PathBuf;

/// 1 - sqrt(2)/2 as binary64 (the FYL2XP1 architectural bound).
const FYL2XP1_BOUND: f64 = 0.292_893_218_813_452_5;

fn half_scale(b: &mut GraphBuilder, l: NodeId) -> NodeId {
    let half = b.strict(Op::Const(ConstVal::from_f64(0.5)));
    b.strict(Op::Mul(half, l)) // exact power-of-two scaling
}

fn t_node(b: &mut GraphBuilder, x: NodeId, dr: bool) -> NodeId {
    let m = if dr { EvalModel::X87_STORED } else { EvalModel::Strict };
    let one = b.strict(Op::Const(ConstVal::from_f64(1.0)));
    let two = b.strict(Op::Const(ConstVal::from_f64(2.0)));
    let d = b.push(Op::Sub(x, one), m);
    b.push(Op::Div(two, d), m)
}

/// Legacy CRT log1p: branch on the FYL2XP1 range.
fn crt_log1p(b: &mut GraphBuilder, t: NodeId) -> NodeId {
    let ln2 = b.push(Op::Const(ConstVal::RomLn2), EvalModel::X87_CONT);
    let bound = b.strict(Op::Const(ConstVal::from_f64(FYL2XP1_BOUND)));
    let near = b.push(Op::Fyl2xp1 { y: ln2, x: t }, EvalModel::X87_STORED);
    let one = b.strict(Op::Const(ConstVal::from_f64(1.0)));
    // Far branch: 1+t formed in EXTENDED (unstored) feeding FYL2X.
    let u = b.push(Op::Add(one, t), EvalModel::X87_CONT);
    let far = b.push(Op::Fyl2x { y: ln2, x: u }, EvalModel::X87_STORED);
    b.strict(Op::Branch {
        pred: Pred::Lt(t, bound),
        then_node: near,
        else_node: far,
    })
}

fn build_all() -> Vec<Candidate> {
    let mut out = Vec::new();
    for (id, desc, f) in [
        (
            "acoth-crt-s",
            "legacy CRT log1p (fyl2xp1 | fyl2x on extended 1+t), t strict",
            Box::new(|b: &mut GraphBuilder, x: NodeId| {
                let t = t_node(b, x, false);
                let l = crt_log1p(b, t);
                half_scale(b, l)
            }) as Box<dyn Fn(&mut GraphBuilder, NodeId) -> NodeId>,
        ),
        (
            "acoth-crt-dr",
            "legacy CRT log1p, t double-rounded",
            Box::new(|b, x| {
                let t = t_node(b, x, true);
                let l = crt_log1p(b, t);
                half_scale(b, l)
            }),
        ),
        (
            "acoth-ucrt",
            "platform ln_1p of strict t (current OxFunc path)",
            Box::new(|b, x| {
                let t = t_node(b, x, false);
                let l = b.strict(Op::Log1p(t));
                half_scale(b, l)
            }),
        ),
        (
            "acoth-portable",
            "in-crate portable log1p of strict t",
            Box::new(|b, x| {
                let t = t_node(b, x, false);
                let l = b.push(Op::Log1p(t), EvalModel::X87_STORED);
                half_scale(b, l)
            }),
        ),
        (
            "acoth-xlnratio-s",
            "x87 worksheet ln((x+1)/(x-1)), strict ratio",
            Box::new(|b, x| {
                let one = b.strict(Op::Const(ConstVal::from_f64(1.0)));
                let np = b.strict(Op::Add(x, one));
                let nm = b.strict(Op::Sub(x, one));
                let q = b.strict(Op::Div(np, nm));
                let l = b.push(Op::Ln(q), EvalModel::X87_STORED);
                half_scale(b, l)
            }),
        ),
        (
            "acoth-xlnratio-spill",
            "x87 worksheet ln((x+1)/(x-1)), spill-loop ratio",
            Box::new(|b, x| {
                let one = b.strict(Op::Const(ConstVal::from_f64(1.0)));
                let np = b.push(Op::Add(x, one), EvalModel::X87_STORED);
                let nm = b.push(Op::Sub(x, one), EvalModel::X87_STORED);
                let q = b.push(Op::Div(np, nm), EvalModel::X87_STORED);
                let l = b.push(Op::Ln(q), EvalModel::X87_STORED);
                half_scale(b, l)
            }),
        ),
        (
            "acoth-fyl2xp1-always",
            "FYL2XP1 with no range branch (control)",
            Box::new(|b, x| {
                let t = t_node(b, x, false);
                let ln2 = b.push(Op::Const(ConstVal::RomLn2), EvalModel::X87_CONT);
                let l = b.push(Op::Fyl2xp1 { y: ln2, x: t }, EvalModel::X87_STORED);
                half_scale(b, l)
            }),
        ),
        (
            "acoth-extratio-cont",
            "fully extended (x+1)/(x-1) feeding FYL2X, no intermediate stores",
            Box::new(|b, x| {
                let one = b.strict(Op::Const(ConstVal::from_f64(1.0)));
                let a = b.push(Op::Add(x, one), EvalModel::X87_CONT);
                let d = b.push(Op::Sub(x, one), EvalModel::X87_CONT);
                let q = b.push(Op::Div(a, d), EvalModel::X87_CONT);
                let ln2 = b.push(Op::Const(ConstVal::RomLn2), EvalModel::X87_CONT);
                let l = b.push(Op::Fyl2x { y: ln2, x: q }, EvalModel::X87_STORED);
                half_scale(b, l)
            }),
        ),
        (
            "acoth-extlog1p-cont",
            "fully extended t=2/(x-1) feeding FYL2XP1|FYL2X branch",
            Box::new(|b, x| {
                let one = b.strict(Op::Const(ConstVal::from_f64(1.0)));
                let two = b.strict(Op::Const(ConstVal::from_f64(2.0)));
                let d = b.push(Op::Sub(x, one), EvalModel::X87_CONT);
                let t = b.push(Op::Div(two, d), EvalModel::X87_CONT);
                let l = crt_log1p(b, t);
                half_scale(b, l)
            }),
        ),
        (
            "acoth-atanh-recip-sxln",
            "atanh(1/x): strict r and ratio, x87 worksheet ln",
            Box::new(|b, x| {
                let one = b.strict(Op::Const(ConstVal::from_f64(1.0)));
                let r = b.strict(Op::Div(one, x));
                let num = b.strict(Op::Add(one, r));
                let den = b.strict(Op::Sub(one, r));
                let q = b.strict(Op::Div(num, den));
                let l = b.push(Op::Ln(q), EvalModel::X87_STORED);
                half_scale(b, l)
            }),
        ),
        (
            "acoth-atanh-recip-dr",
            "atanh(1/x): double-rounded r and ratio, x87 worksheet ln",
            Box::new(|b, x| {
                let one = b.strict(Op::Const(ConstVal::from_f64(1.0)));
                let r = b.push(Op::Div(one, x), EvalModel::X87_STORED);
                let num = b.push(Op::Add(one, r), EvalModel::X87_STORED);
                let den = b.push(Op::Sub(one, r), EvalModel::X87_STORED);
                let q = b.push(Op::Div(num, den), EvalModel::X87_STORED);
                let l = b.push(Op::Ln(q), EvalModel::X87_STORED);
                half_scale(b, l)
            }),
        ),
        (
            "acoth-atanh-recip-cont",
            "atanh(1/x): fully extended r and ratio feeding FYL2X",
            Box::new(|b, x| {
                let one = b.strict(Op::Const(ConstVal::from_f64(1.0)));
                let r = b.push(Op::Div(one, x), EvalModel::X87_CONT);
                let num = b.push(Op::Add(one, r), EvalModel::X87_CONT);
                let den = b.push(Op::Sub(one, r), EvalModel::X87_CONT);
                let q = b.push(Op::Div(num, den), EvalModel::X87_CONT);
                let ln2 = b.push(Op::Const(ConstVal::RomLn2), EvalModel::X87_CONT);
                let l = b.push(Op::Fyl2x { y: ln2, x: q }, EvalModel::X87_STORED);
                half_scale(b, l)
            }),
        ),
        (
            "acoth-atanh-recip-crtlog1p",
            "atanh(1/x) as CRT log1p(2r/(1-r)), strict staging",
            Box::new(|b, x| {
                let one = b.strict(Op::Const(ConstVal::from_f64(1.0)));
                let two = b.strict(Op::Const(ConstVal::from_f64(2.0)));
                let r = b.strict(Op::Div(one, x));
                let num = b.strict(Op::Mul(two, r));
                let den = b.strict(Op::Sub(one, r));
                let t = b.strict(Op::Div(num, den));
                let l = crt_log1p(b, t);
                half_scale(b, l)
            }),
        ),
    ] {
        let mut b = GraphBuilder::new();
        let x = b.strict(Op::Arg(0));
        let out_node = f(&mut b, x);
        out.push(Candidate {
            id: id.into(),
            description: desc.into(),
            graph: b.finish(out_node),
        });
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

fn probe(id: String, x: f64) -> ProbeCase {
    ProbeCase {
        id,
        args: vec![WitnessArg::Scalar(format_bits_hex(x))],
    }
}

fn pool(rng: &mut Rng, tag: &str, count: usize) -> Vec<ProbeCase> {
    let mut out = Vec::new();
    // Catalog witnesses + structured bands.
    for (i, x) in [5.0, 10.0, 2.0, 3.0, 1.5, 100.0, 1000.0, 1.0e6, 1.0e9, 1.0001]
        .iter()
        .enumerate()
    {
        out.push(probe(format!("{tag}-fixed-{i}"), *x));
    }
    // Branch-threshold band: t = 2/(x-1) crosses the FYL2XP1 bound near
    // x = 1 + 2/bound = 7.828...
    for i in 0..80 {
        let x = 7.5 + 0.8 * (i as f64) / 80.0;
        out.push(probe(format!("{tag}-band-{i:03}"), x));
    }
    // Near-1 boundary (t large).
    for k in 1..40 {
        out.push(probe(format!("{tag}-near1-{k:02}"), 1.0 + (2.0f64).powi(-k)));
    }
    // Log-uniform random coverage.
    for i in 0..count {
        let e = rng.uniform(0.0, 9.0);
        let x = 1.0 + (10.0f64).powf(e) * rng.uniform(1.0e-6, 1.0);
        if x > 1.0 {
            out.push(probe(format!("{tag}-rand-{i:04}"), x));
        }
    }
    out
}

fn write_json<T: serde::Serialize>(path: &PathBuf, value: &T) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, serde_json::to_string_pretty(value).unwrap()).unwrap();
    println!("wrote {}", path.display());
}

fn main() {
    let out_dir = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "../../work/w109/G4-03-acoth".into()),
    );
    let candidates = build_all();
    write_json(&out_dir.join("candidates.json"), &candidates);
    println!("{} candidates", candidates.len());
    let mut rng = Rng(0x0109_4703_0001);
    let discovery = pool(&mut rng, "disc", 600);
    write_json(&out_dir.join("pool-discovery.json"), &discovery);
    let mut rng_h = Rng(0x0109_4703_BEEF);
    let heldout = pool(&mut rng_h, "held", 400);
    write_json(&out_dir.join("pool-heldout.json"), &heldout);
    println!("pools written");
}
