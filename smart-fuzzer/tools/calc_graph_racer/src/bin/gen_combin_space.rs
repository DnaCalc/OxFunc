//! W109 Phase-2: COMBIN + PERMUT (G4-04) candidate spaces + paired pools.
//!
//! Evidence: COMBIN(23,10) publishes the exact integer +1 ULP; PERMUT(61,20)
//! is 1 ULP below the exact product — Excel accumulates in floating point.
//!
//! COMBIN loop shapes over i = 1..=k (graph args: 0=nums[], 1=dens[]):
//!   A: nums ascending  (n-k+1 .. n),  dens 1..k
//!   B: nums descending (n .. n-k+1),  dens 1..k
//! folds: ratio-product `c *= num/den` (body staging strict / double-rounded /
//! extended) and interleaved `c = (c*num)/den` (strict / spill / extended),
//! each forward and reverse. A strict factorial-ratio control dies on
//! n > 170 (COMBIN(200,3) is finite in Excel).
//!
//! PERMUT (graph arg 0 = nums[]): plain product folds in the same stagings.

use calc_graph_racer::dsl::{Candidate, EvalModel, GraphBuilder, Op, SumOrder};
use calc_graph_racer::eval::format_bits_hex;
use calc_graph_racer::scheduler::ProbeCase;
use calc_graph_racer::score::WitnessArg;
use std::path::PathBuf;

fn scalar(v: f64) -> WitnessArg {
    WitnessArg::Scalar(format_bits_hex(v))
}
fn array(vs: &[f64]) -> WitnessArg {
    WitnessArg::Array(vs.iter().map(|v| format_bits_hex(*v)).collect())
}

fn ratio_prod(
    body_model: EvalModel,
    acc_model: EvalModel,
    order: SumOrder,
) -> calc_graph_racer::dsl::Graph {
    let mut body = GraphBuilder::new();
    let num = body.strict(Op::Arg(0));
    let den = body.strict(Op::Arg(1));
    let t = body.push(Op::Div(num, den), body_model);
    let body = body.finish(t);
    let mut b = GraphBuilder::new();
    let out = b.push(
        Op::FoldProd {
            over: vec![0, 1],
            body,
            order,
        },
        acc_model,
    );
    b.finish(out)
}

fn muldiv(model: EvalModel, order: SumOrder) -> calc_graph_racer::dsl::Graph {
    let mut b = GraphBuilder::new();
    let out = b.push(
        Op::FoldMulDiv {
            nums: 0,
            dens: 1,
            order,
        },
        model,
    );
    b.finish(out)
}

fn prod_only(
    body_model: EvalModel,
    acc_model: EvalModel,
    order: SumOrder,
) -> calc_graph_racer::dsl::Graph {
    let mut body = GraphBuilder::new();
    let num = body.strict(Op::Arg(0));
    let t = body.push(Op::Abs(num), body_model); // identity carrier for staging
    let body = body.finish(t);
    let mut b = GraphBuilder::new();
    let out = b.push(
        Op::FoldProd {
            over: vec![0],
            body,
            order,
        },
        acc_model,
    );
    b.finish(out)
}

fn combin_candidates() -> Vec<Candidate> {
    let mut out = Vec::new();
    let strict = EvalModel::Strict;
    let dr = EvalModel::X87_STORED;
    let cont = EvalModel::X87_CONT;
    let stored = EvalModel::X87_STORED;
    for (dir_tag, order, step_order) in [
        ("fwd", SumOrder::Forward, SumOrder::ForwardStoredStep),
        ("rev", SumOrder::Reverse, SumOrder::ReverseStoredStep),
    ] {
        out.push(Candidate {
            id: format!("combin-ratio-strict-{dir_tag}"),
            description: format!("c *= num/den, all strict, {dir_tag}"),
            graph: ratio_prod(strict, strict, order),
        });
        out.push(Candidate {
            id: format!("combin-ratio-spill-{dir_tag}"),
            description: format!("c *= RN(num/den), spill accumulate, {dir_tag}"),
            graph: ratio_prod(dr, stored, step_order),
        });
        out.push(Candidate {
            id: format!("combin-ratio-ext-{dir_tag}"),
            description: format!("c *= num/den fully extended, one final store, {dir_tag}"),
            graph: ratio_prod(cont, stored, order),
        });
        out.push(Candidate {
            id: format!("combin-ratio-extterm-drstep-{dir_tag}"),
            description: format!("t=num/den extended, c=RN53(RN64(c*t)) per step, {dir_tag}"),
            graph: ratio_prod(cont, stored, step_order),
        });
        out.push(Candidate {
            id: format!("combin-muldiv-strict-{dir_tag}"),
            description: format!("c=(c*num)/den all strict, {dir_tag}"),
            graph: muldiv(strict, order),
        });
        out.push(Candidate {
            id: format!("combin-muldiv-spill-{dir_tag}"),
            description: format!("c=(c*num)/den spill loop, {dir_tag}"),
            graph: muldiv(stored, step_order),
        });
        out.push(Candidate {
            id: format!("combin-muldiv-ext-{dir_tag}"),
            description: format!("c=(c*num)/den fully extended, one final store, {dir_tag}"),
            graph: muldiv(stored, order),
        });
    }
    out
}

fn permut_candidates() -> Vec<Candidate> {
    let mut out = Vec::new();
    for (dir_tag, order, step_order) in [
        ("fwd", SumOrder::Forward, SumOrder::ForwardStoredStep),
        ("rev", SumOrder::Reverse, SumOrder::ReverseStoredStep),
    ] {
        out.push(Candidate {
            id: format!("permut-strict-{dir_tag}"),
            description: format!("plain double product, {dir_tag}"),
            graph: prod_only(EvalModel::Strict, EvalModel::Strict, order),
        });
        out.push(Candidate {
            id: format!("permut-spill-{dir_tag}"),
            description: format!("spill-loop product (RN53(RN64) per step), {dir_tag}"),
            graph: prod_only(EvalModel::Strict, EvalModel::X87_STORED, step_order),
        });
        out.push(Candidate {
            id: format!("permut-ext-{dir_tag}"),
            description: format!("extended product, one final store, {dir_tag}"),
            graph: prod_only(EvalModel::Strict, EvalModel::X87_STORED, order),
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
    fn int_in(&mut self, lo: u64, hi: u64) -> u64 {
        lo + self.next_u64() % (hi - lo + 1)
    }
}

struct Pair {
    excel: ProbeCase,
    graph_a: ProbeCase, // ascending numerators
    graph_b: ProbeCase, // descending numerators
}

fn combin_pair(id: String, n: u64, k: u64) -> Pair {
    let nums_a: Vec<f64> = (1..=k).map(|i| (n - k + i) as f64).collect();
    let nums_b: Vec<f64> = (1..=k).map(|i| (n + 1 - i) as f64).collect();
    let dens: Vec<f64> = (1..=k).map(|i| i as f64).collect();
    Pair {
        excel: ProbeCase {
            id: id.clone(),
            args: vec![scalar(n as f64), scalar(k as f64)],
        },
        graph_a: ProbeCase {
            id: id.clone(),
            args: vec![array(&nums_a), array(&dens)],
        },
        graph_b: ProbeCase {
            id,
            args: vec![array(&nums_b), array(&dens)],
        },
    }
}

fn permut_pair(id: String, n: u64, k: u64) -> Pair {
    let nums_desc: Vec<f64> = (0..k).map(|i| (n - i) as f64).collect();
    Pair {
        excel: ProbeCase {
            id: id.clone(),
            args: vec![scalar(n as f64), scalar(k as f64)],
        },
        graph_a: ProbeCase {
            id: id.clone(),
            args: vec![array(&nums_desc)],
        },
        graph_b: ProbeCase {
            id,
            args: vec![array(&nums_desc)],
        },
    }
}

/// COMBIN(n,k) stays finite below ~1.8e308: bound k by n via a rough log2 sum.
fn combin_finite(n: u64, k: u64) -> bool {
    let k = k.min(n - k.min(n));
    let mut bits = 0.0f64;
    for i in 1..=k {
        bits += (((n - k + i) as f64) / (i as f64)).log2();
    }
    bits < 1020.0
}

fn write_json<T: serde::Serialize>(path: &PathBuf, value: &T) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, serde_json::to_string_pretty(value).unwrap()).unwrap();
    println!("wrote {}", path.display());
}

fn emit(out_dir: &PathBuf, tag: &str, pairs: &[Pair]) {
    let excel: Vec<&ProbeCase> = pairs.iter().map(|p| &p.excel).collect();
    let ga: Vec<&ProbeCase> = pairs.iter().map(|p| &p.graph_a).collect();
    let gb: Vec<&ProbeCase> = pairs.iter().map(|p| &p.graph_b).collect();
    write_json(&out_dir.join(format!("pool-{tag}-excel.json")), &excel);
    write_json(&out_dir.join(format!("pool-{tag}-graph-a.json")), &ga);
    write_json(&out_dir.join(format!("pool-{tag}-graph-b.json")), &gb);
}

fn main() {
    let root = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "../../work/w109".into()),
    );

    // ---- COMBIN ----
    let dir = root.join("G4-04-combin");
    write_json(&dir.join("candidates.json"), &combin_candidates());
    let mut rng = Rng(0x0109_4404_0001);
    let mut pairs = vec![
        combin_pair("combin-catalog".into(), 23, 10),
        combin_pair("combin-symmetry".into(), 23, 13), // k > n/2 mirror
        combin_pair("combin-k1".into(), 77, 1),
        combin_pair("combin-large-n".into(), 200, 3), // kills factorial forms
        combin_pair("combin-max".into(), 1029, 3),
    ];
    for i in 0..500 {
        let n = rng.int_in(2, 1029);
        let mut k = rng.int_in(1, n);
        if !combin_finite(n, k) {
            k = rng.int_in(1, 8.min(n));
        }
        if combin_finite(n, k) {
            pairs.push(combin_pair(format!("combin-rand-{i:04}"), n, k));
        }
    }
    emit(&dir, "discovery", &pairs);
    let mut held = Vec::new();
    let mut rng_h = Rng(0x0109_4404_BEEF);
    for i in 0..400 {
        let n = rng_h.int_in(2, 1029);
        let mut k = rng_h.int_in(1, n);
        if !combin_finite(n, k) {
            k = rng_h.int_in(1, 8.min(n));
        }
        if combin_finite(n, k) {
            held.push(combin_pair(format!("combin-held-{i:04}"), n, k));
        }
    }
    emit(&dir, "heldout", &held);
    println!("COMBIN: {} discovery, {} heldout", pairs.len(), held.len());

    // ---- PERMUT ----
    let dir = root.join("G4-04-permut");
    write_json(&dir.join("candidates.json"), &permut_candidates());
    let mut rng = Rng(0x0109_4404_0002);
    let mut pairs = vec![
        permut_pair("permut-catalog".into(), 61, 20),
        permut_pair("permut-k1".into(), 500, 1),
    ];
    for i in 0..400 {
        let n = rng.int_in(2, 5000);
        // keep the product finite: bound k by a log2 budget
        let mut bits = 0.0;
        let mut k = 0u64;
        while k < n && bits < 1000.0 {
            bits += ((n - k) as f64).log2();
            k += 1;
        }
        let k = rng.int_in(1, k.max(1));
        pairs.push(permut_pair(format!("permut-rand-{i:04}"), n, k));
    }
    emit(&dir, "discovery", &pairs);
    println!("PERMUT: {} discovery", pairs.len());
}
