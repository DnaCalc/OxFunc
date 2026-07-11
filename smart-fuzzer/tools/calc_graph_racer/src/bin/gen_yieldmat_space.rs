//! W109 Phase-2: YIELDMAT (G6-09) candidate space + paired probe pools.
//!
//! Reference decomposition (public F# / current OxFunc):
//!   b     = days-in-year, dim = days(issue,maturity), a = days(issue,settl)
//!   dsm   = dim - a
//!   term1 = dim/b*rate + 1 - price/100 - a/b*rate
//!   term2 = price/100 + a/b*rate
//!   nper  = term1 / term2 * (b/dsm)
//!
//! The arithmetic operates on exact doubles (integer day counts, `b` a
//! constant for bases 2/3), so the search runs on bases 2 (actual/360) and 3
//! (actual/365) where the generator replicates the day counts exactly. The
//! identified staging is then validated against the basis-0/1 catalog
//! witnesses through the production kernel (which owns full basis logic).
//!
//! Graph args: 0=dim, 1=a, 2=b, 3=rate, 4=price.
//! Excel probes: (settlement, maturity, issue, rate, price, basis) with the
//! same probe ids; `remap` glues answers back to graph args.
//!
//! Axes (72 candidates): arith strict/spill x price/100 div-vs-mul-0.01 x
//! day-fraction association x term1 association (3) x final association (3).

use calc_graph_racer::dsl::{Candidate, ConstVal, EvalModel, GraphBuilder, NodeId, Op};
use calc_graph_racer::eval::format_bits_hex;
use calc_graph_racer::scheduler::ProbeCase;
use calc_graph_racer::score::WitnessArg;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq)]
enum Arith {
    Strict,
    Spill,
}
#[derive(Clone, Copy, PartialEq)]
enum P100 {
    Div100,
    Mul001,
}
#[derive(Clone, Copy, PartialEq)]
enum Frac {
    DivThenMul, // (x/b)*rate
    MulThenDiv, // (x*rate)/b
}
#[derive(Clone, Copy, PartialEq)]
enum T1 {
    LeftChain,  // ((dbr + 1) - p100) - accr
    OnePGroup,  // dbr + ((1 - p100) - accr)
    PairGroup,  // (dbr + (1 - p100)) - accr
    DocsForm,   // (1 + dbr) - term2   (term2 reused, per the published formula)
}
#[derive(Clone, Copy, PartialEq)]
enum Fin {
    DivMulRatio, // (t1/t2) * (b/dsm)
    DivMulDiv,   // ((t1/t2) * b) / dsm
    MulFirst,    // (t1 * b) / (t2 * dsm)
    DivYf,       // (t1/t2) / (dsm/b)
}

fn build(arith: Arith, p100: P100, frac: Frac, t1: T1, fin: Fin) -> Candidate {
    let m = match arith {
        Arith::Strict => EvalModel::Strict,
        Arith::Spill => EvalModel::X87_STORED,
    };
    let mut b = GraphBuilder::new();
    let dim = b.strict(Op::Arg(0));
    let a = b.strict(Op::Arg(1));
    let byr = b.strict(Op::Arg(2));
    let rate = b.strict(Op::Arg(3));
    let price = b.strict(Op::Arg(4));
    let one = b.strict(Op::Const(ConstVal::from_f64(1.0)));

    let day_frac = |b: &mut GraphBuilder, x: NodeId| -> NodeId {
        match frac {
            Frac::DivThenMul => {
                let q = b.push(Op::Div(x, byr), m);
                b.push(Op::Mul(q, rate), m)
            }
            Frac::MulThenDiv => {
                let p = b.push(Op::Mul(x, rate), m);
                b.push(Op::Div(p, byr), m)
            }
        }
    };
    let dbr = day_frac(&mut b, dim); // dim/b*rate
    let accr = day_frac(&mut b, a); // a/b*rate
    let p_norm = match p100 {
        P100::Div100 => {
            let c = b.strict(Op::Const(ConstVal::from_f64(100.0)));
            b.push(Op::Div(price, c), m)
        }
        P100::Mul001 => {
            let c = b.strict(Op::Const(ConstVal::from_f64(0.01)));
            b.push(Op::Mul(price, c), m)
        }
    };
    let term2 = b.push(Op::Add(p_norm, accr), m);
    let term1 = match t1 {
        T1::LeftChain => {
            let s1 = b.push(Op::Add(dbr, one), m);
            let s2 = b.push(Op::Sub(s1, p_norm), m);
            b.push(Op::Sub(s2, accr), m)
        }
        T1::OnePGroup => {
            let s1 = b.push(Op::Sub(one, p_norm), m);
            let s2 = b.push(Op::Sub(s1, accr), m);
            b.push(Op::Add(dbr, s2), m)
        }
        T1::PairGroup => {
            let s1 = b.push(Op::Sub(one, p_norm), m);
            let s2 = b.push(Op::Add(dbr, s1), m);
            b.push(Op::Sub(s2, accr), m)
        }
        T1::DocsForm => {
            let s1 = b.push(Op::Add(one, dbr), m);
            b.push(Op::Sub(s1, term2), m)
        }
    };
    let dsm = b.push(Op::Sub(dim, a), m);
    let out = match fin {
        Fin::DivMulRatio => {
            let q = b.push(Op::Div(term1, term2), m);
            let r = b.push(Op::Div(byr, dsm), m);
            b.push(Op::Mul(q, r), m)
        }
        Fin::DivMulDiv => {
            let q = b.push(Op::Div(term1, term2), m);
            let s = b.push(Op::Mul(q, byr), m);
            b.push(Op::Div(s, dsm), m)
        }
        Fin::MulFirst => {
            let n = b.push(Op::Mul(term1, byr), m);
            let d = b.push(Op::Mul(term2, dsm), m);
            b.push(Op::Div(n, d), m)
        }
        Fin::DivYf => {
            let q = b.push(Op::Div(term1, term2), m);
            let yf = b.push(Op::Div(dsm, byr), m);
            b.push(Op::Div(q, yf), m)
        }
    };
    let id = format!(
        "ym-{}-{}-{}-{}-{}",
        if arith == Arith::Strict { "sarith" } else { "spill" },
        if p100 == P100::Div100 { "d100" } else { "m001" },
        if frac == Frac::DivThenMul { "dfrac" } else { "mfrac" },
        match t1 {
            T1::LeftChain => "t1left",
            T1::OnePGroup => "t1one",
            T1::PairGroup => "t1pair",
            T1::DocsForm => "t1docs",
        },
        match fin {
            Fin::DivMulRatio => "ratio",
            Fin::DivMulDiv => "divmuldiv",
            Fin::MulFirst => "mulfirst",
            Fin::DivYf => "divyf",
        }
    );
    let description = format!(
        "arith={}; price/100={}; dayfrac={}; term1={}; final={}",
        if arith == Arith::Strict { "strict" } else { "x87 spill-loop" },
        if p100 == P100::Div100 { "price/100" } else { "price*0.01" },
        if frac == Frac::DivThenMul { "(x/b)*rate" } else { "(x*rate)/b" },
        match t1 {
            T1::LeftChain => "((dbr+1)-p)-accr",
            T1::OnePGroup => "dbr+((1-p)-accr)",
            T1::PairGroup => "(dbr+(1-p))-accr",
            T1::DocsForm => "(1+dbr)-term2 (term2 reused)",
        },
        match fin {
            Fin::DivMulRatio => "(t1/t2)*(b/dsm)",
            Fin::DivMulDiv => "((t1/t2)*b)/dsm",
            Fin::MulFirst => "(t1*b)/(t2*dsm)",
            Fin::DivYf => "(t1/t2)/(dsm/b)",
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
    fn int_in(&mut self, lo: u64, hi: u64) -> u64 {
        lo + self.next_u64() % (hi - lo + 1)
    }
}

fn scalar(v: f64) -> WitnessArg {
    WitnessArg::Scalar(format_bits_hex(v))
}

/// One probe in both representations.
struct Pair {
    excel: ProbeCase,
    graph: ProbeCase,
}

fn make_pair(id: String, issue: f64, a_days: f64, dsm_days: f64, rate: f64, price: f64, basis: f64) -> Pair {
    let settlement = issue + a_days;
    let maturity = settlement + dsm_days;
    let dim = a_days + dsm_days;
    let b = if basis == 2.0 { 360.0 } else { 365.0 };
    Pair {
        excel: ProbeCase {
            id: id.clone(),
            args: vec![
                scalar(settlement),
                scalar(maturity),
                scalar(issue),
                scalar(rate),
                scalar(price),
                scalar(basis),
            ],
        },
        graph: ProbeCase {
            id,
            args: vec![scalar(dim), scalar(a_days), scalar(b), scalar(rate), scalar(price)],
        },
    }
}

fn pool(rng: &mut Rng, tag: &str, count: usize) -> Vec<Pair> {
    let mut out = Vec::new();
    let mut n = 0;
    for rate in [0.0525, 0.06, 0.011, 0.1875, 0.0333333333333333] {
        for price in [98.59811340546048, 100.0, 95.5, 101.25, 89.03] {
            for (a_days, dsm_days) in [(166.0, 398.0), (30.0, 300.0), (5.0, 1000.0), (400.0, 87.0)] {
                for basis in [2.0, 3.0] {
                    out.push(make_pair(
                        format!("{tag}-grid-{n:04}"),
                        40000.0 + (n % 700) as f64,
                        a_days,
                        dsm_days,
                        rate,
                        price,
                        basis,
                    ));
                    n += 1;
                }
            }
        }
    }
    for i in 0..count {
        let issue = rng.int_in(25000, 46000) as f64;
        let a_days = rng.int_in(1, 900) as f64;
        let dsm_days = rng.int_in(1, 2000) as f64;
        let rate = rng.uniform(0.0001, 0.4);
        let price = rng.uniform(40.0, 180.0);
        let basis = if rng.next_u64() % 2 == 0 { 2.0 } else { 3.0 };
        out.push(make_pair(
            format!("{tag}-rand-{i:04}"),
            issue,
            a_days,
            dsm_days,
            rate,
            price,
            basis,
        ));
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
            .unwrap_or_else(|| "../../work/w109/G6-09-yieldmat".into()),
    );
    let mut candidates = Vec::new();
    for arith in [Arith::Strict, Arith::Spill] {
        for p100 in [P100::Div100, P100::Mul001] {
            for frac in [Frac::DivThenMul, Frac::MulThenDiv] {
                for t1 in [T1::LeftChain, T1::OnePGroup, T1::PairGroup, T1::DocsForm] {
                    for fin in [Fin::DivMulRatio, Fin::DivMulDiv, Fin::MulFirst, Fin::DivYf] {
                        candidates.push(build(arith, p100, frac, t1, fin));
                    }
                }
            }
        }
    }
    write_json(&out_dir.join("candidates.json"), &candidates);
    println!("{} candidates", candidates.len());

    for (seed, tag, count, file) in [
        (0x0109_6E09_0001u64, "disc", 500usize, "discovery"),
        (0x0109_6E09_BEEF, "held", 350, "heldout"),
        // Large full-entropy pool for double-rounding-window hunting
        // (separating strict from spill-loop arithmetic offline).
        (0x0109_6E09_31ED, "win", 60000, "windows"),
    ] {
        let mut rng = Rng(seed);
        let pairs = pool(&mut rng, tag, count);
        let excel: Vec<&ProbeCase> = pairs.iter().map(|p| &p.excel).collect();
        let graph: Vec<&ProbeCase> = pairs.iter().map(|p| &p.graph).collect();
        write_json(&out_dir.join(format!("pool-{file}-excel.json")), &excel);
        write_json(&out_dir.join(format!("pool-{file}-graph.json")), &graph);
        println!("{} {file} probe pairs", pairs.len());
    }
}
