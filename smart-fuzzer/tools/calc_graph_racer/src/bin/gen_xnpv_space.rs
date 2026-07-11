//! W109 Phase-1 pilot: generate the XNPV (G6-11) candidate space and probe
//! pools into `smart-fuzzer/work/w109/G6-11-xnpv/`.
//!
//! XNPV(rate, values, dates) = Σ values_i / (1+rate)^((dates_i - dates_0)/365)
//!
//! Search axes (cartesian, 36 candidates):
//! * years staging      — `(d - d0) / 365` vs `(d - d0) * (1/365)`;
//! * power kernel       — Excel POWER positive staging vs platform `powf`;
//! * term staging       — strict divide, x87-stored divide, or Excel-POWER
//!   negative exponent (reciprocal staging) then multiply;
//! * summation          — strict forward, strict reverse, x87 extended
//!   accumulator (stored at the end).
//!
//! Pools: `pool-discovery.json` (structured + random), `pool-heldout.json`
//! (fresh seed, never used during search), `pool-metamorphic.json`
//! (power-of-two value scaling, joint permutation, same-date splits of
//! discovery probes).

use calc_graph_racer::dsl::{
    Candidate, ConstVal, EvalModel, Graph, GraphBuilder, Op, SumOrder,
};
use calc_graph_racer::eval::format_bits_hex;
use calc_graph_racer::scheduler::ProbeCase;
use calc_graph_racer::score::WitnessArg;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq)]
enum Years {
    Div365,
    Div365Dr,
    MulRecip365,
    MulRecip365Dr,
}
#[derive(Clone, Copy, PartialEq)]
enum BaseAdd {
    Strict,
    DoubleRounded,
}
#[derive(Clone, Copy, PartialEq)]
enum PowKind {
    /// Full worksheet POWER kernel (integer binexp dispatch + fractional x87).
    ExcelKernel,
    /// Fractional x87 staging only (no integer dispatch).
    ExcelPower,
    PlatformPowf,
}
#[derive(Clone, Copy, PartialEq)]
enum Term {
    DivStrict,
    DivX87Stored,
    DivX87Cont,
    MulNegPow,
}
#[derive(Clone, Copy, PartialEq)]
enum Sum {
    Forward,
    Reverse,
    X87Acc,
    ForwardDr,
    ReverseDr,
}

/// Body args: Arg0 = value_i, Arg1 = date_i, Arg2 = rate, Arg3 = values,
/// Arg4 = dates (over = [1, 2] at the top level).
fn build_body(years: Years, base_add: BaseAdd, pow: PowKind, term: Term) -> Graph {
    let mut b = GraphBuilder::new();
    let value = b.strict(Op::Arg(0));
    let date = b.strict(Op::Arg(1));
    let rate = b.strict(Op::Arg(2));
    let d0 = b.strict(Op::Elem { arg: 4, index: 0 });
    // Integer-serial difference: exact under both models.
    let diff = b.strict(Op::Sub(date, d0));
    let years_node = match years {
        Years::Div365 | Years::Div365Dr => {
            let c365 = b.strict(Op::Const(ConstVal::from_f64(365.0)));
            let model = if years == Years::Div365 {
                EvalModel::Strict
            } else {
                EvalModel::X87_STORED // RN53(RN64(diff/365))
            };
            b.push(Op::Div(diff, c365), model)
        }
        Years::MulRecip365 | Years::MulRecip365Dr => {
            let recip = b.strict(Op::Const(ConstVal::from_f64(1.0 / 365.0)));
            let model = if years == Years::MulRecip365 {
                EvalModel::Strict
            } else {
                EvalModel::X87_STORED
            };
            b.push(Op::Mul(diff, recip), model)
        }
    };
    let one = b.strict(Op::Const(ConstVal::from_f64(1.0)));
    let base = b.push(
        Op::Add(one, rate),
        match base_add {
            BaseAdd::Strict => EvalModel::Strict,
            BaseAdd::DoubleRounded => EvalModel::X87_STORED,
        },
    );
    let out = match term {
        Term::DivStrict | Term::DivX87Stored | Term::DivX87Cont => {
            let p = match pow {
                PowKind::ExcelKernel => b.strict(Op::PowExcelKernel {
                    base,
                    exp: years_node,
                }),
                PowKind::ExcelPower => b.strict(Op::PowExcelPositive {
                    base,
                    exp: years_node,
                }),
                PowKind::PlatformPowf => b.strict(Op::PowfStrict {
                    base,
                    exp: years_node,
                }),
            };
            let model = match term {
                Term::DivStrict => EvalModel::Strict,
                Term::DivX87Stored => EvalModel::X87_STORED,
                // Quotient stays extended and flows into the accumulator.
                _ => EvalModel::X87_CONT,
            };
            b.push(Op::Div(value, p), model)
        }
        Term::MulNegPow => match pow {
            PowKind::ExcelKernel => {
                // Worksheet POWER(base, -years) through the full kernel.
                let neg = b.strict(Op::Neg(years_node));
                let p = b.strict(Op::PowExcelKernel { base, exp: neg });
                b.strict(Op::Mul(value, p))
            }
            PowKind::ExcelPower => {
                // Excel POWER(base, -years): positive power, one x87
                // double-rounded reciprocal, then multiply.
                let p = b.strict(Op::PowExcelPositive {
                    base,
                    exp: years_node,
                });
                let r = b.push(Op::Recip(p), EvalModel::X87_STORED);
                b.strict(Op::Mul(value, r))
            }
            PowKind::PlatformPowf => {
                let neg = b.strict(Op::Neg(years_node));
                let p = b.strict(Op::PowfStrict { base, exp: neg });
                b.strict(Op::Mul(value, p))
            }
        },
    };
    b.finish(out)
}

fn build_candidate(years: Years, base_add: BaseAdd, pow: PowKind, term: Term, sum: Sum) -> Candidate {
    let body = build_body(years, base_add, pow, term);
    let (order, model) = match sum {
        Sum::Forward => (SumOrder::Forward, EvalModel::Strict),
        Sum::Reverse => (SumOrder::Reverse, EvalModel::Strict),
        Sum::X87Acc => (SumOrder::Forward, EvalModel::X87_STORED),
        Sum::ForwardDr => (SumOrder::ForwardStoredStep, EvalModel::X87_STORED),
        Sum::ReverseDr => (SumOrder::ReverseStoredStep, EvalModel::X87_STORED),
    };
    let mut b = GraphBuilder::new();
    let out = b.push(
        Op::FoldSum {
            over: vec![1, 2],
            body,
            order,
        },
        model,
    );
    let id = format!(
        "xnpv-{}-{}-{}-{}-{}",
        match years {
            Years::Div365 => "div365",
            Years::Div365Dr => "div365dr",
            Years::MulRecip365 => "mul365",
            Years::MulRecip365Dr => "mul365dr",
        },
        match base_add {
            BaseAdd::Strict => "sbase",
            BaseAdd::DoubleRounded => "drbase",
        },
        match pow {
            PowKind::ExcelKernel => "kpow",
            PowKind::ExcelPower => "xpow",
            PowKind::PlatformPowf => "powf",
        },
        match term {
            Term::DivStrict => "sdiv",
            Term::DivX87Stored => "xdiv",
            Term::DivX87Cont => "extdiv",
            Term::MulNegPow => "negpow",
        },
        match sum {
            Sum::Forward => "fwd",
            Sum::Reverse => "rev",
            Sum::X87Acc => "x87acc",
            Sum::ForwardDr => "fwddr",
            Sum::ReverseDr => "revdr",
        }
    );
    let description = format!(
        "years={}; base={}; pow={}; term={}; sum={}",
        match years {
            Years::Div365 => "(d-d0)/365 strict",
            Years::Div365Dr => "(d-d0)/365 double-rounded",
            Years::MulRecip365 => "(d-d0)*(1/365) strict",
            Years::MulRecip365Dr => "(d-d0)*(1/365) double-rounded",
        },
        match base_add {
            BaseAdd::Strict => "1+rate strict",
            BaseAdd::DoubleRounded => "1+rate double-rounded",
        },
        match pow {
            PowKind::ExcelKernel => "full excel POWER kernel (integer binexp dispatch)",
            PowKind::ExcelPower => "excel POWER fractional staging only",
            PowKind::PlatformPowf => "platform powf",
        },
        match term {
            Term::DivStrict => "value/pow strict",
            Term::DivX87Stored => "value/pow x87-stored",
            Term::DivX87Cont => "value/pow extended-continuous",
            Term::MulNegPow => "value*pow(base,-years) (recip staging)",
        },
        match sum {
            Sum::Forward => "strict forward",
            Sum::Reverse => "strict reverse",
            Sum::X87Acc => "x87 extended accumulator",
            Sum::ForwardDr => "forward per-step-stored x87 (legacy spill loop)",
            Sum::ReverseDr => "reverse per-step-stored x87 (legacy spill loop)",
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
    fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[(self.next_u64() % items.len() as u64) as usize]
    }
}

fn probe(id: String, rate: f64, values: &[f64], dates: &[f64]) -> ProbeCase {
    assert_eq!(values.len(), dates.len());
    ProbeCase {
        id,
        args: vec![
            WitnessArg::Scalar(format_bits_hex(rate)),
            WitnessArg::Array(values.iter().map(|v| format_bits_hex(*v)).collect()),
            WitnessArg::Array(dates.iter().map(|v| format_bits_hex(*v)).collect()),
        ],
    }
}

fn structured_pool(rng: &mut Rng, tag: &str, random_count: usize) -> Vec<ProbeCase> {
    let mut pool = Vec::new();
    let rates = [0.05, 0.1, -0.07, 0.001, 2.5, 1.0e-4, 0.075, 0.3333333333333333];
    let date_patterns: &[&[f64]] = &[
        &[0.0, 365.0, 730.0],                       // exact whole years
        &[0.0, 182.0, 547.0],                       // fractional years
        &[0.0, 91.0, 365.0, 456.0],                 // mixed
        &[0.0, 182.0, 182.0, 400.0],                // same-date pair
        &[0.0, 37.0, 142.0, 365.0, 891.0, 1204.0],  // long irregular
        &[0.0, 1.0, 2.0, 3.0],                      // dense small deltas
    ];
    let value_patterns: &[&[f64]] = &[
        &[-1000.0, 500.0, 600.0],
        &[-1000.0, 1100.0, 250.0],
        &[-1024.0, 512.0, 256.0, 768.0],            // powers of two (exact ops)
        &[1.0e9, -999999999.5, 250.25, -0.75],      // cancellation-heavy
        &[-3333.33, 1111.11, 1111.11, 1111.11],
        &[10.0, 20.0, 30.0, 40.0, 50.0, -100.0],
    ];
    let anchors = [43831.0, 36526.0, 45000.0];
    let mut n = 0;
    for rate in rates {
        for dp in date_patterns {
            for vp in value_patterns {
                if vp.len() < dp.len() {
                    continue;
                }
                let anchor = anchors[n % anchors.len()];
                let dates: Vec<f64> = dp.iter().map(|d| anchor + d).collect();
                let values: Vec<f64> = vp[..dp.len()].to_vec();
                pool.push(probe(format!("{tag}-grid-{n:04}"), rate, &values, &dates));
                n += 1;
            }
        }
    }
    for i in 0..random_count {
        let rate_choices = [
            rng.uniform(-0.5, 3.0),
            rng.uniform(-0.05, 0.25),
            rng.uniform(0.0, 0.001),
        ];
        let rate = *rng.pick(&rate_choices);
        let len = rng.int_in(2, 8) as usize;
        let anchor = rng.int_in(30000, 47000) as f64;
        let mut dates = vec![anchor];
        for _ in 1..len {
            let last = *dates.last().unwrap();
            dates.push(last + rng.int_in(1, 700) as f64);
        }
        let values: Vec<f64> = (0..len)
            .map(|k| {
                let mag = rng.uniform(0.01, 1.0e6);
                if k == 0 { -mag } else if rng.next_u64() % 3 == 0 { -mag } else { mag }
            })
            .collect();
        pool.push(probe(format!("{tag}-rand-{i:04}"), rate, &values, &dates));
    }
    pool
}

fn parse_scalar(a: &WitnessArg) -> f64 {
    match a {
        WitnessArg::Scalar(s) => calc_graph_racer::eval::parse_bits_hex(s).unwrap(),
        WitnessArg::Array(_) => panic!("scalar expected"),
    }
}
fn parse_array(a: &WitnessArg) -> Vec<f64> {
    match a {
        WitnessArg::Array(items) => items
            .iter()
            .map(|s| calc_graph_racer::eval::parse_bits_hex(s).unwrap())
            .collect(),
        WitnessArg::Scalar(_) => panic!("array expected"),
    }
}

/// Metamorphic transforms of discovery probes (math-preserving or
/// exactly-scaling): the survivor must match Excel on these too.
fn metamorphic_pool(discovery: &[ProbeCase], rng: &mut Rng) -> Vec<ProbeCase> {
    let mut pool = Vec::new();
    for (i, p) in discovery.iter().enumerate() {
        if i % 7 != 0 {
            continue; // sample, don't explode
        }
        let rate = parse_scalar(&p.args[0]);
        let values = parse_array(&p.args[1]);
        let dates = parse_array(&p.args[2]);
        // Power-of-two scaling (every op scales exactly, absent overflow).
        for k in [10i32, -10] {
            let s = (2.0f64).powi(k);
            let scaled: Vec<f64> = values.iter().map(|v| v * s).collect();
            pool.push(probe(format!("meta-scale{k}-{i:04}"), rate, &scaled, &dates));
        }
        // Joint permutation of (value, date) pairs after the anchor: preserves
        // the mathematical sum, exposes accumulation order.
        if values.len() > 2 {
            let mut idx: Vec<usize> = (1..values.len()).collect();
            for j in (1..idx.len()).rev() {
                let k = (rng.next_u64() % (j as u64 + 1)) as usize;
                idx.swap(j, k);
            }
            let mut pv = vec![values[0]];
            let mut pd = vec![dates[0]];
            for j in idx {
                pv.push(values[j]);
                pd.push(dates[j]);
            }
            pool.push(probe(format!("meta-perm-{i:04}"), rate, &pv, &pd));
        }
        // Same-date split of the last cashflow: v -> v/2 + v/2.
        let (last_v, last_d) = (*values.last().unwrap(), *dates.last().unwrap());
        let mut sv = values.clone();
        let mut sd = dates.clone();
        *sv.last_mut().unwrap() = last_v / 2.0;
        sv.push(last_v / 2.0);
        sd.push(last_d);
        pool.push(probe(format!("meta-split-{i:04}"), rate, &sv, &sd));
    }
    pool
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
            .unwrap_or_else(|| "../../work/w109/G6-11-xnpv".into()),
    );

    let mut candidates = Vec::new();
    for years in [
        Years::Div365,
        Years::Div365Dr,
        Years::MulRecip365,
        Years::MulRecip365Dr,
    ] {
        for base_add in [BaseAdd::Strict, BaseAdd::DoubleRounded] {
            for pow in [
                PowKind::ExcelKernel,
                PowKind::ExcelPower,
                PowKind::PlatformPowf,
            ] {
                for term in [
                    Term::DivStrict,
                    Term::DivX87Stored,
                    Term::DivX87Cont,
                    Term::MulNegPow,
                ] {
                    for sum in [
                        Sum::Forward,
                        Sum::Reverse,
                        Sum::X87Acc,
                        Sum::ForwardDr,
                        Sum::ReverseDr,
                    ] {
                        candidates.push(build_candidate(years, base_add, pow, term, sum));
                    }
                }
            }
        }
    }
    write_json(&out_dir.join("candidates.json"), &candidates);
    println!("{} candidates", candidates.len());

    let mut rng = Rng(0x0109_6E11_0001);
    let discovery = structured_pool(&mut rng, "disc", 600);
    write_json(&out_dir.join("pool-discovery.json"), &discovery);
    println!("{} discovery probes", discovery.len());

    let mut rng_h = Rng(0x0109_6E11_BEEF_0002);
    let heldout = structured_pool(&mut rng_h, "held", 400);
    write_json(&out_dir.join("pool-heldout.json"), &heldout);
    println!("{} held-out probes", heldout.len());

    let meta = metamorphic_pool(&discovery, &mut rng);
    write_json(&out_dir.join("pool-metamorphic.json"), &meta);
    println!("{} metamorphic probes", meta.len());
}
