//! W109 G6-01: recover PMT's timing-factor operation order from type metamers.
//!
//! The annuity helper is deliberately opaque.  For each otherwise-identical
//! type-0/type-1 pair, call its stored binary64 result before the timing/rate
//! tail `q`.  We invert the type-0 publication `RN(q * rate)` to the finite
//! set of binary64 `q` values in that rounding cell, then ask which candidate
//! tail can publish the observed type-1 value for at least one such `q`.
//!
//! This is stronger than plugging in any PMT helper model: only the two live
//! worksheet outputs and their shared rate enter the score.

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use std::collections::{BTreeMap, BTreeSet};

const ANSWER_DIR: &str = "../../work/w109/G6-solvers";
const CW: u16 = rx::CW_PC64_RN;
const INVERSE_RADIUS: usize = 32;

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

fn x87_mul(left: f64, right: f64) -> f64 {
    rx::ext_to_f64(&rx::ext_mul(&ext(left), &ext(right), CW), CW)
}

fn x87_div(left: f64, right: f64) -> f64 {
    rx::ext_to_f64(&rx::ext_div(&ext(left), &ext(right), CW), CW)
}

fn x87_add(left: f64, right: f64) -> f64 {
    rx::ext_to_f64(&rx::ext_add(&ext(left), &ext(right), CW), CW)
}

fn mul(stage: Stage, left: f64, right: f64) -> f64 {
    match stage {
        Stage::Native => left * right,
        Stage::X87 => x87_mul(left, right),
    }
}

fn div(stage: Stage, left: f64, right: f64) -> f64 {
    match stage {
        Stage::Native => left / right,
        Stage::X87 => x87_div(left, right),
    }
}

fn add(stage: Stage, left: f64, right: f64) -> f64 {
    match stage {
        Stage::Native => left + right,
        Stage::X87 => x87_add(left, right),
    }
}

fn reciprocal(stage: Stage, value: f64) -> f64 {
    div(stage, 1.0, value)
}

fn next_up(value: f64) -> f64 {
    if value.is_nan() || value == f64::INFINITY {
        return value;
    }
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
    if value.is_nan() || value == f64::NEG_INFINITY {
        return value;
    }
    if value == 0.0 {
        return f64::from_bits((1_u64 << 63) | 1);
    }
    if value.is_sign_positive() {
        f64::from_bits(value.to_bits() - 1)
    } else {
        f64::from_bits(value.to_bits() + 1)
    }
}

#[derive(Clone, Copy)]
struct ExtInterval {
    low: rx::Ext80,
    high: rx::Ext80,
}

fn ext_compare(left: &rx::Ext80, right: &rx::Ext80) -> std::cmp::Ordering {
    let delta = rx::ext_to_f64(&rx::ext_sub(left, right, CW), CW);
    if delta < 0.0 {
        std::cmp::Ordering::Less
    } else if delta > 0.0 {
        std::cmp::Ordering::Greater
    } else {
        std::cmp::Ordering::Equal
    }
}

/// Exact-in-extended midpoint bounds of the real values that round to `value`
/// under binary64 round-to-nearest/even.  The endpoint inclusion bit is not
/// needed for this corpus: candidate intersections never collapse to a lone
/// shared midpoint; a diagnostic below reports any zero-width contact.
fn rounding_cell(value: f64) -> ExtInterval {
    let half = ext(0.5);
    let low_sum = rx::ext_add(&ext(next_down(value)), &ext(value), CW);
    let high_sum = rx::ext_add(&ext(value), &ext(next_up(value)), CW);
    ExtInterval {
        low: rx::ext_mul(&low_sum, &half, CW),
        high: rx::ext_mul(&high_sum, &half, CW),
    }
}

fn divide_interval(interval: ExtInterval, divisor: f64) -> ExtInterval {
    divide_interval_ext(interval, &ext(divisor))
}

fn divide_interval_ext(interval: ExtInterval, divisor: &rx::Ext80) -> ExtInterval {
    let first = rx::ext_div(&interval.low, &divisor, CW);
    let second = rx::ext_div(&interval.high, &divisor, CW);
    if ext_compare(&first, &second).is_le() {
        ExtInterval {
            low: first,
            high: second,
        }
    } else {
        ExtInterval {
            low: second,
            high: first,
        }
    }
}

fn multiply_interval(interval: ExtInterval, factor: f64) -> ExtInterval {
    multiply_interval_ext(interval, &ext(factor))
}

fn multiply_interval_ext(interval: ExtInterval, factor: &rx::Ext80) -> ExtInterval {
    let first = rx::ext_mul(&interval.low, &factor, CW);
    let second = rx::ext_mul(&interval.high, &factor, CW);
    if ext_compare(&first, &second).is_le() {
        ExtInterval {
            low: first,
            high: second,
        }
    } else {
        ExtInterval {
            low: second,
            high: first,
        }
    }
}

fn dividend_over_interval(dividend: f64, divisor: ExtInterval) -> ExtInterval {
    let dividend = ext(dividend);
    let first = rx::ext_div(&dividend, &divisor.low, CW);
    let second = rx::ext_div(&dividend, &divisor.high, CW);
    if ext_compare(&first, &second).is_le() {
        ExtInterval {
            low: first,
            high: second,
        }
    } else {
        ExtInterval {
            low: second,
            high: first,
        }
    }
}

fn intervals_overlap(left: ExtInterval, right: ExtInterval) -> bool {
    ext_compare(&left.low, &right.high).is_le() && ext_compare(&right.low, &left.high).is_le()
}

fn continuous_coefficient_support(type0: f64, type1: f64, rate: f64, mode: usize) -> bool {
    let q_from_type0 = divide_interval(rounding_cell(type0), rate);
    let rate_ext = ext(rate);
    let tf_stored = 1.0 + rate;
    let tf_ext = rx::ext_add(&rx::ext_one(), &rate_ext, CW);
    let coefficient = match mode {
        // No intermediate publication: r / stored-binary64 tf.
        0 => rx::ext_div(&rate_ext, &ext(tf_stored), CW),
        // No intermediate publication and tf=1+r retained in PC64.
        1 => rx::ext_div(&rate_ext, &tf_ext, CW),
        // Stored native reciprocal, then PC64-continuous multiply by rate.
        2 => rx::ext_mul(&rate_ext, &ext(1.0 / tf_stored), CW),
        // Stored x87-double-rounded reciprocal, then PC64-continuous rate.
        3 => rx::ext_mul(&rate_ext, &ext(x87_div(1.0, tf_stored)), CW),
        // Reciprocal retained in PC64, then PC64-continuous rate.
        4 => {
            let recip = rx::ext_div(&rx::ext_one(), &tf_ext, CW);
            rx::ext_mul(&rate_ext, &recip, CW)
        }
        _ => unreachable!(),
    };
    intervals_overlap(
        q_from_type0,
        divide_interval_ext(rounding_cell(type1), &coefficient),
    )
}

fn em_association_support(
    family: usize,
    stage: Stage,
    rate: f64,
    present: f64,
    type0: f64,
    type1: f64,
) -> bool {
    let tf = add(stage, 1.0, rate);
    match family {
        // type0 = RN(RN(p/em)*r); type1 = RN(RN(p/RN(em*tf))*r)
        0 => inverse_q_values(type0, rate, stage)
            .into_iter()
            .flat_map(|q| inverse_divisor_values(present, q, stage))
            .any(|em| {
                mul(stage, div(stage, present, mul(stage, em, tf)), rate).to_bits()
                    == type1.to_bits()
            }),
        // type0 = RN(RN(p*r)/em); type1 = RN(RN(p*r)/RN(em*tf))
        1 => {
            let numerator = mul(stage, present, rate);
            inverse_divisor_values(numerator, type0, stage)
                .into_iter()
                .any(|em| div(stage, numerator, mul(stage, em, tf)).to_bits() == type1.to_bits())
        }
        // type0 = RN(RN(p*r)/em); type1 = RN(RN(RN(p*r)/tf)/em)
        2 => {
            let numerator = mul(stage, present, rate);
            inverse_divisor_values(numerator, type0, stage)
                .into_iter()
                .any(|em| div(stage, div(stage, numerator, tf), em).to_bits() == type1.to_bits())
        }
        // type0 = RN(p/RN(em/r)); type1 = RN(p/RN(RN(em*tf)/r))
        3 => {
            let em_over_rate_values = inverse_divisor_values(present, type0, stage);
            em_over_rate_values.into_iter().any(|em_over_rate| {
                let em = mul(stage, em_over_rate, rate);
                let denominator = div(stage, mul(stage, em, tf), rate);
                div(stage, present, denominator).to_bits() == type1.to_bits()
            })
        }
        _ => unreachable!(),
    }
}

fn continuous_em_support(
    family: usize,
    stage: Stage,
    rate: f64,
    present: f64,
    type0: f64,
    type1: f64,
) -> bool {
    let tf = add(stage, 1.0, rate);
    match family {
        // type0=RN(N/em), type1=RN(N/RN(em*tf)), N=RN(p*r).
        0 => {
            let numerator = mul(stage, present, rate);
            let em0 = dividend_over_interval(numerator, rounding_cell(type0));
            inverse_divisor_values(numerator, type1, stage)
                .into_iter()
                .any(|timed_denominator| {
                    let em1 = divide_interval(rounding_cell(timed_denominator), tf);
                    intervals_overlap(em0, em1)
                })
        }
        // type0=RN(N/em), type1=RN(RN(N/tf)/em), N=RN(p*r).
        1 => {
            let numerator = mul(stage, present, rate);
            let timed_numerator = div(stage, numerator, tf);
            let em0 = dividend_over_interval(numerator, rounding_cell(type0));
            let em1 = dividend_over_interval(timed_numerator, rounding_cell(type1));
            intervals_overlap(em0, em1)
        }
        // type0=RN(N/em), type1=RN(RN(N*recip(tf))/em).
        2 => {
            let numerator = mul(stage, present, rate);
            let timed_numerator = mul(stage, numerator, reciprocal(stage, tf));
            let em0 = dividend_over_interval(numerator, rounding_cell(type0));
            let em1 = dividend_over_interval(timed_numerator, rounding_cell(type1));
            intervals_overlap(em0, em1)
        }
        // type0=RN(RN(p/em)*r), type1=RN(RN(p/RN(em*tf))*r).
        3 => inverse_q_values(type0, rate, stage).into_iter().any(|q0| {
            let em0 = dividend_over_interval(present, rounding_cell(q0));
            inverse_q_values(type1, rate, stage).into_iter().any(|q1| {
                inverse_divisor_values(present, q1, stage)
                    .into_iter()
                    .any(|timed_denominator| {
                        let em1 = divide_interval(rounding_cell(timed_denominator), tf);
                        intervals_overlap(em0, em1)
                    })
            })
        }),
        _ => unreachable!(),
    }
}

#[derive(Clone)]
struct Observation {
    value: f64,
    locations: BTreeSet<String>,
}

fn scalar_args(witness: &calc_graph_racer::score::Witness) -> Option<[f64; 5]> {
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

fn load_pairs() -> BTreeMap<[u64; 4], [Option<Observation>; 2]> {
    let mut pairs: BTreeMap<[u64; 4], [Option<Observation>; 2]> = BTreeMap::new();
    let mut sources = std::fs::read_dir(ANSWER_DIR)
        .expect("read PMT answer directory")
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            (name.starts_with("answers-pmt-") && name.ends_with(".json"))
                .then_some((name, entry.path()))
        })
        .collect::<Vec<_>>();
    sources.sort_by(|left, right| left.0.cmp(&right.0));

    for (source, path) in sources {
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        let Ok(set) = serde_json::from_str::<WitnessSet>(&text) else {
            continue;
        };
        if set.function != "PMT" {
            continue;
        }
        for witness in &set.witnesses {
            let Some([rate, periods, present, future, timing]) = scalar_args(witness) else {
                continue;
            };
            if timing != 0.0 && timing != 1.0 {
                continue;
            }
            let Some(expected) = parse_bits_hex(&witness.expected_bits) else {
                continue;
            };
            let key = [
                rate.to_bits(),
                periods.to_bits(),
                present.to_bits(),
                future.to_bits(),
            ];
            let location = format!(
                "{source}/{}",
                witness.id.as_deref().unwrap_or("<missing-id>")
            );
            let slot = &mut pairs.entry(key).or_default()[timing as usize];
            if let Some(prior) = slot {
                assert_eq!(
                    prior.value.to_bits(),
                    expected.to_bits(),
                    "conflicting PMT oracle values at {location}: prior locations {:?}",
                    prior.locations
                );
                prior.locations.insert(location);
            } else {
                *slot = Some(Observation {
                    value: expected,
                    locations: BTreeSet::from([location]),
                });
            }
        }
    }
    pairs
}

fn is_power_of_two(value: f64) -> bool {
    value.is_normal() && value.is_sign_positive() && value.to_bits() & ((1_u64 << 52) - 1) == 0
}

/// Enumerate every binary64 `q` near the exact inverse of `published = RN(q*r)`
/// and retain precisely those that reproduce the observed type-0 publication.
fn inverse_q_values(published: f64, rate: f64, stage: Stage) -> Vec<f64> {
    let mut seeds = BTreeSet::new();
    seeds.insert((published / rate).to_bits());
    seeds.insert(x87_div(published, rate).to_bits());
    let mut candidates = BTreeSet::new();
    for seed_bits in seeds {
        let seed = f64::from_bits(seed_bits);
        if !seed.is_finite() {
            continue;
        }
        if mul(stage, seed, rate).to_bits() == published.to_bits() {
            candidates.insert(seed.to_bits());
        }
        let mut lower = seed;
        let mut upper = seed;
        for _ in 0..INVERSE_RADIUS {
            lower = next_down(lower);
            upper = next_up(upper);
            if mul(stage, lower, rate).to_bits() == published.to_bits() {
                candidates.insert(lower.to_bits());
            }
            if mul(stage, upper, rate).to_bits() == published.to_bits() {
                candidates.insert(upper.to_bits());
            }
        }
    }
    candidates.into_iter().map(f64::from_bits).collect()
}

fn inverse_divisor_values(numerator: f64, published: f64, stage: Stage) -> Vec<f64> {
    let mut seeds = BTreeSet::new();
    seeds.insert((numerator / published).to_bits());
    seeds.insert(x87_div(numerator, published).to_bits());
    let mut candidates = BTreeSet::new();
    for seed_bits in seeds {
        let seed = f64::from_bits(seed_bits);
        if !seed.is_finite() {
            continue;
        }
        if div(stage, numerator, seed).to_bits() == published.to_bits() {
            candidates.insert(seed.to_bits());
        }
        let mut lower = seed;
        let mut upper = seed;
        for _ in 0..INVERSE_RADIUS {
            lower = next_down(lower);
            upper = next_up(upper);
            if div(stage, numerator, lower).to_bits() == published.to_bits() {
                candidates.insert(lower.to_bits());
            }
            if div(stage, numerator, upper).to_bits() == published.to_bits() {
                candidates.insert(upper.to_bits());
            }
        }
    }
    candidates.into_iter().map(f64::from_bits).collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Family {
    ReciprocalBeforeRate,
    ReciprocalAfterRate,
    DivideBeforeRate,
    DivideAfterRate,
    StoredRatioDivide,
    StoredRatioReciprocal,
}

#[derive(Clone, Copy, Debug)]
struct Graph {
    family: Family,
    rate_stage: Stage,
    timing_stage: Stage,
    reciprocal_stage: Stage,
    tf_stage: Stage,
}

impl Graph {
    fn name(self) -> String {
        let family = match self.family {
            Family::ReciprocalBeforeRate => "q*recip(tf),then*r",
            Family::ReciprocalAfterRate => "q*r,then*recip(tf)",
            Family::DivideBeforeRate => "q/tf,then*r",
            Family::DivideAfterRate => "q*r,then/tf",
            Family::StoredRatioDivide => "q*(r/tf)",
            Family::StoredRatioReciprocal => "q*(r*recip(tf))",
        };
        format!(
            "{family} [rate={},timing={},recip={},tf={}]",
            self.rate_stage.tag(),
            self.timing_stage.tag(),
            self.reciprocal_stage.tag(),
            self.tf_stage.tag()
        )
    }

    fn predict(self, q: f64, rate: f64) -> f64 {
        let tf = add(self.tf_stage, 1.0, rate);
        let recip = reciprocal(self.reciprocal_stage, tf);
        match self.family {
            Family::ReciprocalBeforeRate => {
                mul(self.rate_stage, mul(self.timing_stage, q, recip), rate)
            }
            Family::ReciprocalAfterRate => {
                mul(self.timing_stage, mul(self.rate_stage, q, rate), recip)
            }
            Family::DivideBeforeRate => mul(self.rate_stage, div(self.timing_stage, q, tf), rate),
            Family::DivideAfterRate => div(self.timing_stage, mul(self.rate_stage, q, rate), tf),
            Family::StoredRatioDivide => {
                mul(self.timing_stage, q, div(self.reciprocal_stage, rate, tf))
            }
            Family::StoredRatioReciprocal => {
                let coefficient = mul(self.reciprocal_stage, rate, recip);
                mul(self.timing_stage, q, coefficient)
            }
        }
    }
}

/// Existential interval score with no assumption that the hidden `q` itself
/// was spilled to binary64.  Each visible operation still has its candidate
/// binary64 publication barrier.  Extended endpoints retain enough bits to
/// represent every binary64 midpoint exactly and compare the divided cells
/// without collapsing them back to binary64.
fn continuous_support(graph: Graph, type0: f64, type1: f64, rate: f64) -> bool {
    let q_from_type0 = divide_interval(rounding_cell(type0), rate);
    let tf = add(graph.tf_stage, 1.0, rate);
    let recip = reciprocal(graph.reciprocal_stage, tf);
    match graph.family {
        Family::ReciprocalAfterRate => {
            mul(graph.timing_stage, type0, recip).to_bits() == type1.to_bits()
        }
        Family::DivideAfterRate => div(graph.timing_stage, type0, tf).to_bits() == type1.to_bits(),
        Family::StoredRatioDivide => {
            let coefficient = div(graph.reciprocal_stage, rate, tf);
            intervals_overlap(
                q_from_type0,
                divide_interval(rounding_cell(type1), coefficient),
            )
        }
        Family::StoredRatioReciprocal => {
            let coefficient = mul(graph.reciprocal_stage, rate, recip);
            intervals_overlap(
                q_from_type0,
                divide_interval(rounding_cell(type1), coefficient),
            )
        }
        Family::ReciprocalBeforeRate | Family::DivideBeforeRate => {
            inverse_q_values(type1, rate, graph.rate_stage)
                .into_iter()
                .any(|timed| {
                    let q_from_type1 = match graph.family {
                        Family::ReciprocalBeforeRate => {
                            divide_interval(rounding_cell(timed), recip)
                        }
                        Family::DivideBeforeRate => multiply_interval(rounding_cell(timed), tf),
                        _ => unreachable!(),
                    };
                    intervals_overlap(q_from_type0, q_from_type1)
                })
        }
    }
}

fn graph_catalog() -> Vec<Graph> {
    let mut graphs = Vec::new();
    for family in [
        Family::ReciprocalBeforeRate,
        Family::ReciprocalAfterRate,
        Family::DivideBeforeRate,
        Family::DivideAfterRate,
        Family::StoredRatioDivide,
        Family::StoredRatioReciprocal,
    ] {
        for rate_stage in [Stage::Native, Stage::X87] {
            for timing_stage in [Stage::Native, Stage::X87] {
                for reciprocal_stage in [Stage::Native, Stage::X87] {
                    for tf_stage in [Stage::Native, Stage::X87] {
                        graphs.push(Graph {
                            family,
                            rate_stage,
                            timing_stage,
                            reciprocal_stage,
                            tf_stage,
                        });
                    }
                }
            }
        }
    }
    graphs
}

#[derive(Default)]
struct Score {
    all: usize,
    power_two: usize,
    general: usize,
    fv_zero: usize,
    unambiguous: usize,
    continuous: usize,
    continuous_power_two: usize,
    continuous_general: usize,
}

fn main() {
    let pairs = load_pairs();
    let complete = pairs
        .iter()
        .filter_map(|(key, observations)| match observations {
            [Some(type0), Some(type1)] => Some((*key, type0, type1)),
            _ => None,
        })
        .filter(|(key, _, _)| {
            let rate = f64::from_bits(key[0]);
            rate.is_finite() && rate != 0.0 && 1.0 + rate > 0.0
        })
        .collect::<Vec<_>>();
    let power_two = complete
        .iter()
        .filter(|(key, _, _)| is_power_of_two(f64::from_bits(key[0])))
        .count();
    let general = complete.len() - power_two;
    let fv_zero = complete
        .iter()
        .filter(|(key, _, _)| f64::from_bits(key[3]) == 0.0)
        .count();
    println!(
        "PMT type-metamer audit: unique-keys={} complete={} power-two={} general={} fv-zero={}",
        pairs.len(),
        complete.len(),
        power_two,
        general,
        fv_zero
    );

    let graphs = graph_catalog();
    let mut scores = (0..graphs.len())
        .map(|_| Score::default())
        .collect::<Vec<_>>();
    let mut no_preimage = [0usize; 2];
    let mut inverse_cardinality = BTreeMap::new();

    for (key, type0, type1) in &complete {
        let rate = f64::from_bits(key[0]);
        let want = type1.value.to_bits();
        let mut row_hits = vec![false; graphs.len()];
        for (graph_index, graph) in graphs.iter().enumerate() {
            if continuous_support(*graph, type0.value, type1.value, rate) {
                scores[graph_index].continuous += 1;
                if is_power_of_two(rate) {
                    scores[graph_index].continuous_power_two += 1;
                } else {
                    scores[graph_index].continuous_general += 1;
                }
            }
        }
        for (stage_index, stage) in [Stage::Native, Stage::X87].into_iter().enumerate() {
            let q_values = inverse_q_values(type0.value, rate, stage);
            *inverse_cardinality
                .entry((stage, q_values.len()))
                .or_insert(0usize) += 1;
            no_preimage[stage_index] += usize::from(q_values.is_empty());
            for (graph_index, graph) in graphs.iter().enumerate() {
                if graph.rate_stage != stage {
                    continue;
                }
                row_hits[graph_index] = q_values
                    .iter()
                    .any(|q| graph.predict(*q, rate).to_bits() == want);
            }
        }
        let survivors = row_hits.iter().filter(|hit| **hit).count();
        for (index, hit) in row_hits.into_iter().enumerate() {
            if !hit {
                continue;
            }
            scores[index].all += 1;
            if is_power_of_two(rate) {
                scores[index].power_two += 1;
            } else {
                scores[index].general += 1;
            }
            if f64::from_bits(key[3]) == 0.0 {
                scores[index].fv_zero += 1;
            }
            if survivors == 1 {
                scores[index].unambiguous += 1;
            }
        }
    }

    println!("inverse preimage cardinalities: {inverse_cardinality:?}");
    println!(
        "no inverse q preimage: native={} x87={}",
        no_preimage[0], no_preimage[1]
    );

    let mut order = (0..graphs.len()).collect::<Vec<_>>();
    order.sort_by(|left, right| {
        scores[*right]
            .all
            .cmp(&scores[*left].all)
            .then_with(|| scores[*right].general.cmp(&scores[*left].general))
            .then_with(|| graphs[*left].name().cmp(&graphs[*right].name()))
    });
    println!(
        "\n{:>7} {:>7} {:>7} {:>7} {:>7}  graph",
        "all", "po2", "general", "fv0", "unique"
    );
    for index in order.into_iter().take(40) {
        let score = &scores[index];
        println!(
            "{:>3}/{:<3} {:>3}/{:<3} {:>3}/{:<3} {:>3}/{:<3} {:>7}  {}",
            score.all,
            complete.len(),
            score.power_two,
            power_two,
            score.general,
            general,
            score.fv_zero,
            fv_zero,
            score.unambiguous,
            graphs[index].name()
        );
    }

    let mut continuous_order = (0..graphs.len()).collect::<Vec<_>>();
    continuous_order.sort_by(|left, right| {
        scores[*right]
            .continuous
            .cmp(&scores[*left].continuous)
            .then_with(|| {
                scores[*right]
                    .continuous_general
                    .cmp(&scores[*left].continuous_general)
            })
            .then_with(|| graphs[*left].name().cmp(&graphs[*right].name()))
    });
    println!("\ncontinuous-q inverse-interval score (x87 endpoints; no helper model):");
    println!("{:>7} {:>7} {:>7}  graph", "all", "po2", "general");
    for index in continuous_order.into_iter().take(40) {
        let score = &scores[index];
        println!(
            "{:>3}/{:<3} {:>3}/{:<3} {:>3}/{:<3}  {}",
            score.continuous,
            complete.len(),
            score.continuous_power_two,
            power_two,
            score.continuous_general,
            general,
            graphs[index].name()
        );
    }

    let leading = Graph {
        family: Family::StoredRatioDivide,
        rate_stage: Stage::Native,
        timing_stage: Stage::Native,
        reciprocal_stage: Stage::Native,
        tf_stage: Stage::Native,
    };
    let mut failures_by_source = BTreeMap::new();
    let mut failures_by_sign = BTreeMap::new();
    let mut printed = 0usize;
    for (key, type0, type1) in &complete {
        let rate = f64::from_bits(key[0]);
        if continuous_support(leading, type0.value, type1.value, rate) {
            continue;
        }
        *failures_by_sign
            .entry(if rate.is_sign_positive() {
                "positive"
            } else {
                "negative"
            })
            .or_insert(0usize) += 1;
        for location in type0.locations.iter().chain(&type1.locations) {
            let source = location.split('/').next().unwrap_or(location);
            *failures_by_source
                .entry(source.to_owned())
                .or_insert(0usize) += 1;
        }
        if printed < 12 {
            let tf = 1.0 + rate;
            println!(
                "leading-miss r={:#018x} n={:#018x} pv={:#018x} fv={:#018x} y0={:#018x} y1={:#018x} y0/tf={:#018x} y0*recip={:#018x} sources0={:?} sources1={:?}",
                key[0],
                key[1],
                key[2],
                key[3],
                type0.value.to_bits(),
                type1.value.to_bits(),
                (type0.value / tf).to_bits(),
                (type0.value * (1.0 / tf)).to_bits(),
                type0.locations,
                type1.locations,
            );
            printed += 1;
        }
    }
    println!("leading q*(r/tf) failures by sign: {failures_by_sign:?}");
    println!("leading q*(r/tf) failures by source occurrence: {failures_by_source:?}");

    println!("\nPC64-continuous coefficient inverse-interval score:");
    for (mode, name) in [
        "r / stored-tf",
        "r / PC64(1+r)",
        "r * stored-native-recip",
        "r * stored-x87-recip",
        "r * PC64-recip(PC64(1+r))",
    ]
    .into_iter()
    .enumerate()
    {
        let mut all = 0usize;
        let mut po2 = 0usize;
        let mut general_hits = 0usize;
        for (key, type0, type1) in &complete {
            let rate = f64::from_bits(key[0]);
            if continuous_coefficient_support(type0.value, type1.value, rate, mode) {
                all += 1;
                if is_power_of_two(rate) {
                    po2 += 1;
                } else {
                    general_hits += 1;
                }
            }
        }
        println!(
            "  {all:>5}/{} all  {po2:>4}/{power_two} po2  {general_hits:>5}/{general} general  {name}",
            complete.len()
        );
    }

    println!("\nknown-pv / latent-em association score (fv=0 only):");
    for (family, name) in [
        "p/(em*tf), then*r",
        "(p*r)/(em*tf)",
        "((p*r)/tf)/em",
        "p / ((em*tf)/r)",
    ]
    .into_iter()
    .enumerate()
    {
        for stage in [Stage::Native, Stage::X87] {
            let mut hits = 0usize;
            let mut total = 0usize;
            let mut po2_hits = 0usize;
            let mut po2_total = 0usize;
            for (key, type0, type1) in &complete {
                if f64::from_bits(key[3]) != 0.0 {
                    continue;
                }
                total += 1;
                let rate = f64::from_bits(key[0]);
                let present = f64::from_bits(key[2]);
                let hit =
                    em_association_support(family, stage, rate, present, type0.value, type1.value);
                hits += usize::from(hit);
                if is_power_of_two(rate) {
                    po2_total += 1;
                    po2_hits += usize::from(hit);
                }
            }
            println!(
                "  {hits:>4}/{total} (po2 {po2_hits}/{po2_total}) stage={}  {name}",
                stage.tag()
            );
        }
    }

    println!("\nperiods=1, fv=0 exact-cancellation stratum:");
    let n1 = complete
        .iter()
        .filter(|(key, _, _)| f64::from_bits(key[1]) == 1.0 && f64::from_bits(key[3]) == 0.0)
        .collect::<Vec<_>>();
    println!("  rows={}", n1.len());
    for (mode, name) in [
        "r / stored-tf",
        "r / PC64(1+r)",
        "r * stored-native-recip",
        "r * stored-x87-recip",
        "r * PC64-recip(PC64(1+r))",
    ]
    .into_iter()
    .enumerate()
    {
        let hits = n1
            .iter()
            .filter(|(key, type0, type1)| {
                continuous_coefficient_support(
                    type0.value,
                    type1.value,
                    f64::from_bits(key[0]),
                    mode,
                )
            })
            .count();
        println!("  {hits:>4}/{}  {name}", n1.len());
    }
    let exact_type1 = n1
        .iter()
        .filter(|(key, _, type1)| type1.value.to_bits() == (-f64::from_bits(key[2])).to_bits())
        .count();
    println!("  Excel type1 == -pv exactly: {exact_type1}/{}", n1.len());

    println!("\ncontinuous latent-em inverse-interval score (fv=0 only):");
    for (family, name) in [
        "N / RN(em*tf), N=RN(p*r)",
        "RN(N/tf) / em, N=RN(p*r)",
        "RN(N*recip(tf)) / em",
        "RN(p/RN(em*tf)), then*r",
    ]
    .into_iter()
    .enumerate()
    {
        for stage in [Stage::Native, Stage::X87] {
            let mut hits = 0usize;
            let mut total = 0usize;
            let mut po2_hits = 0usize;
            let mut po2_total = 0usize;
            for (key, type0, type1) in &complete {
                if f64::from_bits(key[3]) != 0.0 {
                    continue;
                }
                total += 1;
                let rate = f64::from_bits(key[0]);
                let present = f64::from_bits(key[2]);
                let hit =
                    continuous_em_support(family, stage, rate, present, type0.value, type1.value);
                hits += usize::from(hit);
                if is_power_of_two(rate) {
                    po2_total += 1;
                    po2_hits += usize::from(hit);
                }
            }
            println!(
                "  {hits:>4}/{total} (po2 {po2_hits}/{po2_total}) stage={}  {name}",
                stage.tag()
            );
        }
    }
}
