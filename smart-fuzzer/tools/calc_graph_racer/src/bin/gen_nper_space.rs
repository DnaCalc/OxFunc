//! W109 Phase-2: NPER (G6-08) candidate space + probe pools.
//!
//! NPER(rate, pmt, pv, fv, type), rate != 0 main path:
//!
//!   tf    = 1 + rate*type
//!   num   = tf*pmt - fv*rate
//!   den   = tf*pmt + pv*rate
//!   nper  = ln(num/den) / ln(1 + rate)
//!
//! Axes (cartesian, 40 candidates):
//! * arith  — every arithmetic assignment strict vs x87-stored
//!   (the legacy spill-loop prior from the XNPV identification);
//! * numlog — ln(ratio): x87 worksheet ln vs platform ln;
//! * denlog — ln(1+rate): x87 ln of strict 1+r, x87 ln of double-rounded 1+r,
//!   FYL2XP1(rate) (the legacy log1p instruction), portable log1p, platform
//!   ln(1+r);
//! * fdiv   — final division strict vs double-rounded.
//!
//! Args: 0=rate, 1=pmt, 2=pv, 3=fv, 4=type.

use calc_graph_racer::dsl::{
    Candidate, ConstVal, EvalModel, GraphBuilder, NodeId, Op,
};
use calc_graph_racer::eval::format_bits_hex;
use calc_graph_racer::scheduler::ProbeCase;
use calc_graph_racer::score::WitnessArg;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq)]
enum Arith {
    Strict,
    SpillLoop,
}
#[derive(Clone, Copy, PartialEq)]
enum NumLog {
    X87Ln,
    PlatformLn,
}
#[derive(Clone, Copy, PartialEq)]
enum DenLog {
    X87LnStrictBase,
    X87LnDrBase,
    Fyl2xp1,
    PortableLog1p,
    PlatformLn,
}
#[derive(Clone, Copy, PartialEq)]
enum FinalDiv {
    Strict,
    DoubleRounded,
}

fn build(arith: Arith, numlog: NumLog, denlog: DenLog, fdiv: FinalDiv) -> Candidate {
    let am = match arith {
        Arith::Strict => EvalModel::Strict,
        Arith::SpillLoop => EvalModel::X87_STORED,
    };
    let mut b = GraphBuilder::new();
    let rate = b.strict(Op::Arg(0));
    let pmt = b.strict(Op::Arg(1));
    let pv = b.strict(Op::Arg(2));
    let fv = b.strict(Op::Arg(3));
    let ty = b.strict(Op::Arg(4));
    let one = b.strict(Op::Const(ConstVal::from_f64(1.0)));

    let r_ty = b.push(Op::Mul(rate, ty), am);
    let tf = b.push(Op::Add(one, r_ty), am);
    let tf_pmt = b.push(Op::Mul(tf, pmt), am);
    let fv_r = b.push(Op::Mul(fv, rate), am);
    let pv_r = b.push(Op::Mul(pv, rate), am);
    let num = b.push(Op::Sub(tf_pmt, fv_r), am);
    let den = b.push(Op::Add(tf_pmt, pv_r), am);
    let ratio = b.push(Op::Div(num, den), am);

    let ln_num = match numlog {
        NumLog::X87Ln => b.push(Op::Ln(ratio), EvalModel::X87_STORED),
        NumLog::PlatformLn => b.strict(Op::Ln(ratio)),
    };
    let ln_den: NodeId = match denlog {
        DenLog::X87LnStrictBase => {
            let base = b.strict(Op::Add(one, rate));
            b.push(Op::Ln(base), EvalModel::X87_STORED)
        }
        DenLog::X87LnDrBase => {
            let base = b.push(Op::Add(one, rate), EvalModel::X87_STORED);
            b.push(Op::Ln(base), EvalModel::X87_STORED)
        }
        DenLog::Fyl2xp1 => {
            let ln2 = b.push(Op::Const(ConstVal::RomLn2), EvalModel::X87_CONT);
            b.push(Op::Fyl2xp1 { y: ln2, x: rate }, EvalModel::X87_STORED)
        }
        DenLog::PortableLog1p => b.strict(Op::Log1p(rate)),
        DenLog::PlatformLn => {
            let base = b.strict(Op::Add(one, rate));
            b.strict(Op::Ln(base))
        }
    };
    let out = b.push(
        Op::Div(ln_num, ln_den),
        match fdiv {
            FinalDiv::Strict => EvalModel::Strict,
            FinalDiv::DoubleRounded => EvalModel::X87_STORED,
        },
    );
    let id = format!(
        "nper-{}-{}-{}-{}",
        match arith {
            Arith::Strict => "sarith",
            Arith::SpillLoop => "spill",
        },
        match numlog {
            NumLog::X87Ln => "xln",
            NumLog::PlatformLn => "pln",
        },
        match denlog {
            DenLog::X87LnStrictBase => "xlns",
            DenLog::X87LnDrBase => "xlndr",
            DenLog::Fyl2xp1 => "fyl2xp1",
            DenLog::PortableLog1p => "log1p",
            DenLog::PlatformLn => "plnden",
        },
        match fdiv {
            FinalDiv::Strict => "sdiv",
            FinalDiv::DoubleRounded => "drdiv",
        }
    );
    let description = format!(
        "arith={}; ln(ratio)={}; ln(1+r)={}; final div={}",
        match arith {
            Arith::Strict => "strict",
            Arith::SpillLoop => "x87 spill-loop double-rounded",
        },
        match numlog {
            NumLog::X87Ln => "x87 worksheet ln",
            NumLog::PlatformLn => "platform ln",
        },
        match denlog {
            DenLog::X87LnStrictBase => "x87 ln(strict 1+r)",
            DenLog::X87LnDrBase => "x87 ln(RN53(RN64(1+r)))",
            DenLog::Fyl2xp1 => "x87 FYL2XP1 (ln2*log2(1+r))",
            DenLog::PortableLog1p => "portable log1p(r)",
            DenLog::PlatformLn => "platform ln(1+r)",
        },
        match fdiv {
            FinalDiv::Strict => "strict",
            FinalDiv::DoubleRounded => "double-rounded",
        }
    );
    Candidate {
        id,
        description,
        graph: b.finish(out),
    }
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

fn probe(id: String, rate: f64, pmt: f64, pv: f64, fv: f64, ty: f64) -> ProbeCase {
    ProbeCase {
        id,
        args: [rate, pmt, pv, fv, ty]
            .iter()
            .map(|v| WitnessArg::Scalar(format_bits_hex(*v)))
            .collect(),
    }
}

fn pool(rng: &mut Rng, tag: &str, count: usize) -> Vec<ProbeCase> {
    let mut out = Vec::new();
    // Structured loan-shaped grid (annuity domain: pmt sign opposite pv).
    let rates = [
        0.0008333333333333334, // NPER-0000's monthly rate
        0.006666666666666667,
        0.05,
        0.1 / 12.0,
        0.03,
        0.25,
        1.0e-4,
        1.0e-7,
    ];
    let mut n = 0;
    for rate in rates {
        for pv in [1000.0, 10000.0, 250000.0, 12345.6789] {
            for pmt_scale in [0.010001, 0.02, 0.1, 0.5] {
                for (fv, ty) in [(0.0, 0.0), (0.0, 1.0), (500.0, 0.0), (-250.0, 1.0)] {
                    let pmt = -(pv * (rate + pmt_scale));
                    out.push(probe(format!("{tag}-grid-{n:04}"), rate, pmt, pv, fv, ty));
                    n += 1;
                }
            }
        }
    }
    for i in 0..count {
        let rate = match rng.next_u64() % 3 {
            0 => rng.uniform(1.0e-6, 0.02),
            1 => rng.uniform(0.02, 0.5),
            _ => rng.uniform(1.0e-9, 1.0e-5),
        };
        let pv = rng.uniform(10.0, 1.0e6);
        let pmt = -(pv * (rate + rng.uniform(0.001, 0.3)));
        let fv = match rng.next_u64() % 3 {
            0 => 0.0,
            1 => rng.uniform(-0.4, 0.4) * pv,
            _ => rng.uniform(0.0, 0.2) * pv,
        };
        let ty = (rng.next_u64() % 2) as f64;
        out.push(probe(format!("{tag}-rand-{i:04}"), rate, pmt, pv, fv, ty));
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
            .unwrap_or_else(|| "../../work/w109/G6-08-nper".into()),
    );
    let mut candidates = Vec::new();
    for arith in [Arith::Strict, Arith::SpillLoop] {
        for numlog in [NumLog::X87Ln, NumLog::PlatformLn] {
            for denlog in [
                DenLog::X87LnStrictBase,
                DenLog::X87LnDrBase,
                DenLog::Fyl2xp1,
                DenLog::PortableLog1p,
                DenLog::PlatformLn,
            ] {
                for fdiv in [FinalDiv::Strict, FinalDiv::DoubleRounded] {
                    candidates.push(build(arith, numlog, denlog, fdiv));
                }
            }
        }
    }
    write_json(&out_dir.join("candidates.json"), &candidates);
    println!("{} candidates", candidates.len());

    let mut rng = Rng(0x0109_6E08_0001);
    let discovery = pool(&mut rng, "disc", 400);
    write_json(&out_dir.join("pool-discovery.json"), &discovery);
    println!("{} discovery probes", discovery.len());

    let mut rng_h = Rng(0x0109_6E08_BEEF);
    let heldout = pool(&mut rng_h, "held", 300);
    write_json(&out_dir.join("pool-heldout.json"), &heldout);
    println!("{} held-out probes", heldout.len());

    // Zero/tiny-rate branch probes: identify Excel's small-rate branch
    // threshold separately from the main path.
    let mut branch = Vec::new();
    for (i, rate) in [
        0.0, -0.0, 1.0e-9, 1.0e-12, 1.0e-15, 1.0e-18, 1.0e-30, 1.0e-300, -1.0e-9, -0.05,
    ]
    .iter()
    .enumerate()
    {
        branch.push(probe(
            format!("branch-{i}"),
            *rate,
            -120.0,
            10000.0,
            0.0,
            0.0,
        ));
    }
    write_json(&out_dir.join("pool-branch.json"), &branch);
    println!("{} branch probes", branch.len());
}
