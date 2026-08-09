//! Score the frozen W109 PMT general-rate timing-order discriminator.
//!
//! The private annuity helper is a nuisance parameter, not a model input.  For
//! each `(rate,nper)` context we infer a small answer-constrained set of
//! possible stored `em` values from the type-0 PV ladder, then score every
//! pre-frozen timing/rate association against both 16-row ladders.  One fitted
//! `em` must explain all 32 outputs in a context.

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use std::collections::{BTreeMap, BTreeSet};

const BATCH: &str = "../../work/w109/G6-solvers/batch-pmt-tf-order-discriminator-20260809.json";
const ANSWERS: &str = "../../work/w109/G6-solvers/answers-pmt-tf-order-discriminator-20260809.json";
const CW: u16 = rx::CW_PC64_RN;
const EM_RADIUS: usize = 24;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Stage {
    Native,
    X87,
}

impl Stage {
    fn tag(self) -> &'static str {
        match self {
            Self::Native => "n",
            Self::X87 => "x",
        }
    }
}

fn ext(value: f64) -> rx::Ext80 {
    rx::ext_from_f64(value)
}

fn to_f64(value: &rx::Ext80) -> f64 {
    rx::ext_to_f64(value, CW)
}

fn xmul(left: f64, right: f64) -> f64 {
    to_f64(&rx::ext_mul(&ext(left), &ext(right), CW))
}

fn xdiv(left: f64, right: f64) -> f64 {
    to_f64(&rx::ext_div(&ext(left), &ext(right), CW))
}

fn xadd(left: f64, right: f64) -> f64 {
    to_f64(&rx::ext_add(&ext(left), &ext(right), CW))
}

fn mul(stage: Stage, left: f64, right: f64) -> f64 {
    match stage {
        Stage::Native => left * right,
        Stage::X87 => xmul(left, right),
    }
}

fn div(stage: Stage, left: f64, right: f64) -> f64 {
    match stage {
        Stage::Native => left / right,
        Stage::X87 => xdiv(left, right),
    }
}

fn add(stage: Stage, left: f64, right: f64) -> f64 {
    match stage {
        Stage::Native => left + right,
        Stage::X87 => xadd(left, right),
    }
}

fn sub(stage: Stage, left: f64, right: f64) -> f64 {
    match stage {
        Stage::Native => left - right,
        Stage::X87 => to_f64(&rx::ext_sub(&ext(left), &ext(right), CW)),
    }
}

fn next_up(value: f64) -> f64 {
    if value == 0.0 {
        return f64::from_bits(1);
    }
    if value.is_sign_positive() {
        f64::from_bits(value.to_bits() + 1)
    } else {
        f64::from_bits(value.to_bits() - 1)
    }
}

fn next_down(value: f64) -> f64 {
    if value == 0.0 {
        return f64::from_bits((1_u64 << 63) | 1);
    }
    if value.is_sign_positive() {
        f64::from_bits(value.to_bits() - 1)
    } else {
        f64::from_bits(value.to_bits() + 1)
    }
}

fn ordered(value: f64) -> i128 {
    let bits = value.to_bits();
    if bits >> 63 == 0 {
        (bits | (1_u64 << 63)) as i128
    } else {
        (!bits) as i128
    }
}

fn ulp_distance(left: f64, right: f64) -> u128 {
    (ordered(left) - ordered(right)).unsigned_abs()
}

fn args(witness: &calc_graph_racer::score::Witness) -> Option<[f64; 5]> {
    let values = witness
        .args
        .iter()
        .filter_map(|arg| match arg {
            WitnessArg::Scalar(text) => parse_bits_hex(text),
            _ => None,
        })
        .collect::<Vec<_>>();
    (values.len() == 5).then(|| values.try_into().unwrap())
}

#[derive(Clone, Copy)]
struct Pair {
    present: f64,
    type0: f64,
    type1: f64,
}

#[derive(Clone)]
struct Context {
    rate: f64,
    periods: f64,
    pairs: Vec<Pair>,
}

fn load() -> Vec<Context> {
    let batch_text = std::fs::read_to_string(BATCH).expect("read batch");
    let batch: serde_json::Value = serde_json::from_str(&batch_text).expect("parse batch");
    let text = std::fs::read_to_string(ANSWERS).expect("read answers");
    let raw: serde_json::Value = serde_json::from_str(&text).expect("parse raw answers");
    assert_eq!(batch["function"], "PMT");
    assert_eq!(raw["function"], "PMT");
    let probes = batch["probes"].as_array().expect("batch probes array");
    let raw_witnesses = raw["witnesses"].as_array().expect("answer witnesses array");
    assert_eq!(probes.len(), 480);
    assert_eq!(raw_witnesses.len(), probes.len());
    for (index, (wrapped, witness)) in probes.iter().zip(raw_witnesses).enumerate() {
        let probe = &wrapped["probe"];
        assert_eq!(probe["id"], witness["id"], "id mismatch at {index}");
        assert_eq!(
            probe["args"], witness["args"],
            "argument mismatch at {index}"
        );
    }
    let string_at = |pointer: &str| raw.pointer(pointer).and_then(serde_json::Value::as_str);
    assert_eq!(
        string_at("/capture_provenance/schema_version"),
        Some("w109-capture-provenance-v1")
    );
    assert_eq!(
        string_at("/capture_provenance/environment/excel_version"),
        Some("16.0")
    );
    assert_eq!(
        string_at("/capture_provenance/environment/excel_build"),
        Some("20228")
    );
    assert_eq!(
        string_at("/capture_provenance/environment/excel_bitness"),
        Some("64-bit")
    );
    assert_eq!(
        string_at("/capture_provenance/environment/workbook_compatibility"),
        Some("2")
    );
    assert_eq!(
        string_at("/capture_provenance/environment/excel_input_plumbing"),
        Some("cell_value2_bulk")
    );
    assert_eq!(
        string_at("/capture_provenance/oracle_cache/mode"),
        Some("no_cache")
    );
    assert_eq!(
        raw.pointer("/capture_provenance/oracle_cache/hits")
            .and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(
        raw.pointer("/capture_provenance/oracle_cache/misses")
            .and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(
        string_at("/capture_provenance/runner/version"),
        Some("w109-bulk-batch-v2")
    );
    let set: WitnessSet = serde_json::from_str(&text).expect("parse typed answers");
    assert_eq!(set.function, "PMT");
    assert_eq!(set.witnesses.len(), 480);
    let mut rows: BTreeMap<[u64; 3], [Option<f64>; 2]> = BTreeMap::new();
    let mut ids = BTreeSet::new();
    for witness in &set.witnesses {
        let id = witness.id.as_deref().expect("nonempty witness id");
        assert!(!id.is_empty());
        assert!(ids.insert(id));
        let [rate, periods, present, future, timing] = args(witness).expect("five scalar args");
        assert_eq!(future, 0.0);
        assert!(timing == 0.0 || timing == 1.0);
        let value = parse_bits_hex(&witness.expected_bits).expect("numeric PMT answer");
        let slot = &mut rows
            .entry([rate.to_bits(), periods.to_bits(), present.to_bits()])
            .or_default()[timing as usize];
        assert!(slot.replace(value).is_none());
    }
    let mut contexts: BTreeMap<[u64; 2], Vec<Pair>> = BTreeMap::new();
    for (key, values) in rows {
        let [Some(type0), Some(type1)] = values else {
            panic!("incomplete frozen timing pair");
        };
        contexts.entry([key[0], key[1]]).or_default().push(Pair {
            present: f64::from_bits(key[2]),
            type0,
            type1,
        });
    }
    let contexts = contexts
        .into_iter()
        .map(|(key, mut pairs)| {
            pairs.sort_by_key(|pair| pair.present.to_bits());
            assert_eq!(pairs.len(), 16);
            Context {
                rate: f64::from_bits(key[0]),
                periods: f64::from_bits(key[1]),
                pairs,
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(contexts.len(), 15);
    contexts
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Family {
    QRecipBeforeRate,
    QDivideBeforeRate,
    QRecipAfterRate,
    QDivideAfterRate,
    QStoredRatioDivide,
    QStoredRatioRecip,
    ProductOverTimedDenominator,
    TimedProductOverEmDivide,
    TimedProductOverEmRecip,
    QuotientTimedDenominatorThenRate,
    QPc64Ratio,
    QPc64Recip,
    FullPc64Divide,
    FullPc64Recip,
    QSubtractRatioBeforeRate,
    QSubtractRatioAfterRate,
    QOneMinusRatioBeforeRate,
    QOneMinusRatioAfterRate,
}

#[derive(Clone, Copy, Debug)]
struct Graph {
    family: Family,
    base: Stage,
    rate: Stage,
    timing: Stage,
    reciprocal: Stage,
    tf: Stage,
}

impl Graph {
    fn name(self) -> String {
        let family = match self.family {
            Family::QRecipBeforeRate => "(q*recip(tf))*r",
            Family::QDivideBeforeRate => "(q/tf)*r",
            Family::QRecipAfterRate => "(q*r)*recip(tf)",
            Family::QDivideAfterRate => "(q*r)/tf",
            Family::QStoredRatioDivide => "q*store(r/tf)",
            Family::QStoredRatioRecip => "q*store(r*recip(tf))",
            Family::ProductOverTimedDenominator => "store(p*r)/store(em*tf)",
            Family::TimedProductOverEmDivide => "store(store(p*r)/tf)/em",
            Family::TimedProductOverEmRecip => "store(store(p*r)*recip(tf))/em",
            Family::QuotientTimedDenominatorThenRate => "p/store(em*tf),then*r",
            Family::QPc64Ratio => "q*r/tf PC64-continuous",
            Family::QPc64Recip => "q*recip(tf)*r PC64-continuous",
            Family::FullPc64Divide => "p/em/tf*r PC64-continuous",
            Family::FullPc64Recip => "p/em*recip(tf)*r PC64-continuous",
            Family::QSubtractRatioBeforeRate => "(q-q*store(r/tf))*r",
            Family::QSubtractRatioAfterRate => "q*r-(q*r)*store(r/tf)",
            Family::QOneMinusRatioBeforeRate => "q*store(1-store(r/tf))*r",
            Family::QOneMinusRatioAfterRate => "q*r*store(1-store(r/tf))",
        };
        format!(
            "{family} [b={},r={},t={},i={},f={}]",
            self.base.tag(),
            self.rate.tag(),
            self.timing.tag(),
            self.reciprocal.tag(),
            self.tf.tag(),
        )
    }

    fn eval(self, present: f64, rate: f64, em: f64, timing_value: f64) -> f64 {
        let tf = if timing_value == 0.0 {
            1.0
        } else {
            add(self.tf, 1.0, rate)
        };
        let recip = div(self.reciprocal, 1.0, tf);
        let q = div(self.base, present, em);
        match self.family {
            Family::QRecipBeforeRate => mul(self.rate, mul(self.timing, q, recip), rate),
            Family::QDivideBeforeRate => mul(self.rate, div(self.timing, q, tf), rate),
            Family::QRecipAfterRate => mul(self.timing, mul(self.rate, q, rate), recip),
            Family::QDivideAfterRate => div(self.timing, mul(self.rate, q, rate), tf),
            Family::QStoredRatioDivide => mul(self.timing, q, div(self.reciprocal, rate, tf)),
            Family::QStoredRatioRecip => mul(self.timing, q, mul(self.reciprocal, rate, recip)),
            Family::ProductOverTimedDenominator => div(
                self.timing,
                mul(self.rate, present, rate),
                mul(self.base, em, tf),
            ),
            Family::TimedProductOverEmDivide => div(
                self.base,
                div(self.timing, mul(self.rate, present, rate), tf),
                em,
            ),
            Family::TimedProductOverEmRecip => div(
                self.base,
                mul(self.timing, mul(self.rate, present, rate), recip),
                em,
            ),
            Family::QuotientTimedDenominatorThenRate => mul(
                self.rate,
                div(self.base, present, mul(self.timing, em, tf)),
                rate,
            ),
            Family::QPc64Ratio => {
                let qe = ext(q);
                let re = ext(rate);
                let tfe = if timing_value == 0.0 {
                    rx::ext_one()
                } else {
                    rx::ext_add(&rx::ext_one(), &re, CW)
                };
                to_f64(&rx::ext_div(&rx::ext_mul(&qe, &re, CW), &tfe, CW))
            }
            Family::QPc64Recip => {
                let qe = ext(q);
                let re = ext(rate);
                let tfe = if timing_value == 0.0 {
                    rx::ext_one()
                } else {
                    rx::ext_add(&rx::ext_one(), &re, CW)
                };
                let inverse = rx::ext_div(&rx::ext_one(), &tfe, CW);
                to_f64(&rx::ext_mul(&rx::ext_mul(&qe, &inverse, CW), &re, CW))
            }
            Family::FullPc64Divide => {
                let pe = ext(present);
                let ee = ext(em);
                let re = ext(rate);
                let tfe = if timing_value == 0.0 {
                    rx::ext_one()
                } else {
                    rx::ext_add(&rx::ext_one(), &re, CW)
                };
                let q = rx::ext_div(&pe, &ee, CW);
                to_f64(&rx::ext_mul(&rx::ext_div(&q, &tfe, CW), &re, CW))
            }
            Family::FullPc64Recip => {
                let pe = ext(present);
                let ee = ext(em);
                let re = ext(rate);
                let tfe = if timing_value == 0.0 {
                    rx::ext_one()
                } else {
                    rx::ext_add(&rx::ext_one(), &re, CW)
                };
                let q = rx::ext_div(&pe, &ee, CW);
                let inverse = rx::ext_div(&rx::ext_one(), &tfe, CW);
                to_f64(&rx::ext_mul(&rx::ext_mul(&q, &inverse, CW), &re, CW))
            }
            Family::QSubtractRatioBeforeRate => {
                if timing_value == 0.0 {
                    return mul(self.rate, q, rate);
                }
                let ratio = div(self.reciprocal, rate, tf);
                let correction = mul(self.timing, q, ratio);
                mul(self.rate, sub(self.timing, q, correction), rate)
            }
            Family::QSubtractRatioAfterRate => {
                if timing_value == 0.0 {
                    return mul(self.rate, q, rate);
                }
                let ratio = div(self.reciprocal, rate, tf);
                let end = mul(self.rate, q, rate);
                sub(self.timing, end, mul(self.timing, end, ratio))
            }
            Family::QOneMinusRatioBeforeRate => {
                if timing_value == 0.0 {
                    return mul(self.rate, q, rate);
                }
                let ratio = div(self.reciprocal, rate, tf);
                let inverse = sub(self.timing, 1.0, ratio);
                mul(self.rate, mul(self.timing, q, inverse), rate)
            }
            Family::QOneMinusRatioAfterRate => {
                if timing_value == 0.0 {
                    return mul(self.rate, q, rate);
                }
                let ratio = div(self.reciprocal, rate, tf);
                let inverse = sub(self.timing, 1.0, ratio);
                mul(self.timing, mul(self.rate, q, rate), inverse)
            }
        }
    }
}

fn catalog() -> Vec<Graph> {
    let mut graphs = Vec::new();
    for family in [
        Family::QRecipBeforeRate,
        Family::QDivideBeforeRate,
        Family::QRecipAfterRate,
        Family::QDivideAfterRate,
        Family::QStoredRatioDivide,
        Family::QStoredRatioRecip,
        Family::ProductOverTimedDenominator,
        Family::TimedProductOverEmDivide,
        Family::TimedProductOverEmRecip,
        Family::QuotientTimedDenominatorThenRate,
        Family::QPc64Ratio,
        Family::QPc64Recip,
        Family::FullPc64Divide,
        Family::FullPc64Recip,
        Family::QSubtractRatioBeforeRate,
        Family::QSubtractRatioAfterRate,
        Family::QOneMinusRatioBeforeRate,
        Family::QOneMinusRatioAfterRate,
    ] {
        for base in [Stage::Native, Stage::X87] {
            for rate in [Stage::Native, Stage::X87] {
                for timing in [Stage::Native, Stage::X87] {
                    for reciprocal in [Stage::Native, Stage::X87] {
                        for tf in [Stage::Native, Stage::X87] {
                            graphs.push(Graph {
                                family,
                                base,
                                rate,
                                timing,
                                reciprocal,
                                tf,
                            });
                        }
                    }
                }
            }
        }
    }
    graphs
}

fn em_candidates(context: &Context) -> Vec<f64> {
    let mut candidates = BTreeSet::new();
    for pair in &context.pairs {
        let seeds = [
            (pair.present * context.rate) / pair.type0,
            xdiv(xmul(pair.present, context.rate), pair.type0),
            pair.present / (pair.type0 / context.rate),
            xdiv(pair.present, xdiv(pair.type0, context.rate)),
        ];
        for seed in seeds {
            if !seed.is_finite() || seed == 0.0 {
                continue;
            }
            candidates.insert(seed.to_bits());
            let mut lower = seed;
            let mut upper = seed;
            for _ in 0..EM_RADIUS {
                lower = next_down(lower);
                upper = next_up(upper);
                candidates.insert(lower.to_bits());
                candidates.insert(upper.to_bits());
            }
        }
    }
    candidates.into_iter().map(f64::from_bits).collect()
}

#[derive(Clone, Copy, Default)]
struct Fitness {
    exact: usize,
    exact_type0: usize,
    exact_type1: usize,
    sum_ulp: u128,
    max_ulp: u128,
    em: u64,
}

fn better(left: Fitness, right: Fitness) -> bool {
    left.exact > right.exact
        || (left.exact == right.exact && left.sum_ulp < right.sum_ulp)
        || (left.exact == right.exact
            && left.sum_ulp == right.sum_ulp
            && left.max_ulp < right.max_ulp)
}

fn fit_context(graph: Graph, context: &Context, candidates: &[f64]) -> Fitness {
    let mut best = Fitness {
        sum_ulp: u128::MAX,
        max_ulp: u128::MAX,
        ..Fitness::default()
    };
    for em in candidates {
        let mut fitness = Fitness {
            em: em.to_bits(),
            ..Fitness::default()
        };
        for pair in &context.pairs {
            for (timing, expected) in [(0.0, pair.type0), (1.0, pair.type1)] {
                let got = graph.eval(pair.present, context.rate, *em, timing);
                let distance = ulp_distance(got, expected);
                fitness.exact += usize::from(distance == 0);
                if distance == 0 && timing == 0.0 {
                    fitness.exact_type0 += 1;
                }
                if distance == 0 && timing == 1.0 {
                    fitness.exact_type1 += 1;
                }
                fitness.sum_ulp = fitness.sum_ulp.saturating_add(distance);
                fitness.max_ulp = fitness.max_ulp.max(distance);
            }
        }
        if better(fitness, best) {
            best = fitness;
        }
    }
    best
}

#[derive(Default)]
struct Aggregate {
    exact: usize,
    exact_type0: usize,
    exact_type1: usize,
    exact_contexts: usize,
    sum_ulp: u128,
    max_ulp: u128,
    fitted_em: Vec<u64>,
}

fn main() {
    let contexts = load();
    let candidate_banks = contexts.iter().map(em_candidates).collect::<Vec<_>>();
    println!(
        "frozen PMT tf-order gate: contexts={} pairs={} calls={} em-candidate-counts={:?}",
        contexts.len(),
        contexts
            .iter()
            .map(|context| context.pairs.len())
            .sum::<usize>(),
        contexts
            .iter()
            .map(|context| context.pairs.len() * 2)
            .sum::<usize>(),
        candidate_banks.iter().map(Vec::len).collect::<Vec<_>>()
    );
    let graphs = catalog();
    let mut results = Vec::new();
    for graph in graphs {
        let mut aggregate = Aggregate::default();
        for (context, candidates) in contexts.iter().zip(&candidate_banks) {
            let fitness = fit_context(graph, context, candidates);
            aggregate.exact += fitness.exact;
            aggregate.exact_type0 += fitness.exact_type0;
            aggregate.exact_type1 += fitness.exact_type1;
            aggregate.exact_contexts += usize::from(fitness.exact == context.pairs.len() * 2);
            aggregate.sum_ulp = aggregate.sum_ulp.saturating_add(fitness.sum_ulp);
            aggregate.max_ulp = aggregate.max_ulp.max(fitness.max_ulp);
            aggregate.fitted_em.push(fitness.em);
        }
        results.push((graph, aggregate));
    }
    results.sort_by(|left, right| {
        right
            .1
            .exact
            .cmp(&left.1.exact)
            .then_with(|| right.1.exact_contexts.cmp(&left.1.exact_contexts))
            .then_with(|| left.1.sum_ulp.cmp(&right.1.sum_ulp))
            .then_with(|| left.1.max_ulp.cmp(&right.1.max_ulp))
            .then_with(|| left.0.name().cmp(&right.0.name()))
    });
    println!("\n exact    t0    t1  ctx  sum_ulp  max_ulp  graph");
    for (graph, aggregate) in results.iter().take(40) {
        println!(
            "{:>3}/480 {:>3}/240 {:>3}/240 {:>2}/15 {:>8} {:>8}  {}",
            aggregate.exact,
            aggregate.exact_type0,
            aggregate.exact_type1,
            aggregate.exact_contexts,
            aggregate.sum_ulp,
            aggregate.max_ulp,
            graph.name()
        );
    }

    println!("\nbest representative per frozen family:");
    let mut seen = BTreeSet::new();
    for (graph, aggregate) in &results {
        let family_tag = format!("{:?}", graph.family);
        if !seen.insert(family_tag) {
            continue;
        }
        println!(
            "{:>3}/480 {:>3}/240 {:>3}/240 {:>2}/15 {:>8} {:>8}  {}",
            aggregate.exact,
            aggregate.exact_type0,
            aggregate.exact_type1,
            aggregate.exact_contexts,
            aggregate.sum_ulp,
            aggregate.max_ulp,
            graph.name()
        );
    }

    let (leader, aggregate) = &results[0];
    println!("\nleader per-context fitted nuisance em and exact score:");
    for ((context, candidates), em_bits) in contexts
        .iter()
        .zip(&candidate_banks)
        .zip(&aggregate.fitted_em)
    {
        let fitness = fit_context(*leader, context, candidates);
        println!(
            "  r={:#018x} n={:#018x}: {}/32 em={:#018x} sum={} max={}",
            context.rate.to_bits(),
            context.periods.to_bits(),
            fitness.exact,
            em_bits,
            fitness.sum_ulp,
            fitness.max_ulp,
        );
    }
}
