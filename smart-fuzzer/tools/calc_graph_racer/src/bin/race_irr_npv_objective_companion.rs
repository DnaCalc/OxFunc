//! Offline exact-graph race for the worksheet-NPV surface captured alongside
//! the frozen W109 IRR discovery rows.
//!
//! The candidate family is clean-room: the add-product discount loop comes
//! from Microsoft's published Visual Basic Reference Source, while the other
//! forms are ordinary algebraic controls.  The race never reads the sealed IRR
//! held-out battery.

use oxfunc_core::excel_numeric::research as rx;
use rx::Ext80;
use serde_json::Value;
use std::collections::BTreeSet;

const ANSWERS: &str =
    "../../work/w109/G6-solvers/answers-irr-npv-objective-companion-discovery-20260809.json";

#[derive(Clone)]
struct Obs {
    id: String,
    point: String,
    c0: f64,
    rate: f64,
    tail: Vec<f64>,
    raw: u64,
    direct: u64,
    cell: u64,
}

fn parse_hex(raw: &str) -> f64 {
    assert_eq!(raw.len(), 18);
    assert!(raw.starts_with("0x"));
    f64::from_bits(u64::from_str_radix(&raw[2..], 16).expect("hex bits"))
}

fn string<'a>(value: &'a Value, field: &str) -> &'a str {
    value[field]
        .as_str()
        .unwrap_or_else(|| panic!("missing string field {field}"))
}

fn numeric_result_bits(value: &Value, field: &str) -> u64 {
    let result = &value[field];
    assert_eq!(string(result, "kind"), "number");
    parse_hex(string(result, "bits")).to_bits()
}

fn load() -> Vec<Obs> {
    let document: Value = serde_json::from_slice(&std::fs::read(ANSWERS).expect("read answers"))
        .expect("parse answers");
    assert_eq!(
        string(&document, "schema_version"),
        "w109.irr.npv_objective_companion.answers.v1"
    );
    assert_eq!(string(&document, "function"), "NPV");
    let rows = document["probes"].as_array().expect("probe array");
    assert_eq!(rows.len(), 900);
    let mut ids = BTreeSet::new();
    let mut observations = Vec::with_capacity(rows.len());
    for row in rows {
        let probe = &row["probe"];
        let id = string(probe, "id").to_owned();
        assert!(ids.insert(id.clone()));
        let tail = probe["tail_bits"]
            .as_array()
            .expect("tail array")
            .iter()
            .map(|item| parse_hex(item.as_str().expect("tail bits")))
            .collect::<Vec<_>>();
        let readback = &row["argument_value2_readback"];
        assert_eq!(string(probe, "c0_bits"), string(readback, "c0_bits"));
        assert_eq!(string(probe, "rate_bits"), string(readback, "rate_bits"));
        assert_eq!(
            probe["tail_bits"].as_array().unwrap(),
            readback["tail_bits"].as_array().unwrap()
        );
        observations.push(Obs {
            id,
            point: string(probe, "point_class").to_owned(),
            c0: parse_hex(string(probe, "c0_bits")),
            rate: parse_hex(string(probe, "rate_bits")),
            tail,
            raw: numeric_result_bits(row, "raw_npv"),
            direct: numeric_result_bits(row, "direct_composed"),
            cell: numeric_result_bits(row, "cell_composed"),
        });
    }
    assert_eq!(ids.len(), 900);
    observations
}

#[derive(Clone, Copy)]
struct Ext(Ext80);

impl Ext {
    fn new(value: f64) -> Self {
        Self(rx::ext_from_f64(value))
    }

    fn add(self, other: Self, cw: u16) -> Self {
        Self(rx::ext_add(&self.0, &other.0, cw))
    }

    fn mul(self, other: Self, cw: u16) -> Self {
        Self(rx::ext_mul(&self.0, &other.0, cw))
    }

    fn div(self, other: Self, cw: u16) -> Self {
        Self(rx::ext_div(&self.0, &other.0, cw))
    }

    fn store(self, yes: bool, cw: u16) -> Self {
        if yes {
            Self::new(rx::ext_to_f64(&self.0, cw))
        } else {
            self
        }
    }

    fn to_f64(self, cw: u16) -> f64 {
        rx::ext_to_f64(&self.0, cw)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Graph {
    // Published LDoNPV statement graph:
    // discount = discount + discount * rate; total += value / discount.
    AddProduct,
    AddProductMulCommuted,
    AddProductSeedFirst,
    MultiplyOnePlusRate,
    ReverseHornerDivision,
    ReverseHornerReciprocal,
    ReverseDivideThenAddFinalDivide,
    ReverseDivideThenAddFinalDivideSeeded,
    ReverseMultiplyThenAddFinalMultiply,
    ForwardRepeatedDivision,
}

fn evaluate(rate: f64, values: &[f64], graph: Graph, mask: u8, cw: u16) -> f64 {
    let bit = |index: u8| mask & (1 << index) != 0;
    let rate = Ext::new(rate);
    let mut discount = Ext::new(1.0);
    let mut total = Ext::new(0.0);
    match graph {
        Graph::AddProduct | Graph::AddProductMulCommuted => {
            for &value in values {
                let increment = match graph {
                    Graph::AddProduct => discount.mul(rate, cw),
                    Graph::AddProductMulCommuted => rate.mul(discount, cw),
                    _ => unreachable!(),
                }
                .store(bit(0), cw);
                discount = discount.add(increment, cw).store(bit(1), cw);
                let term = Ext::new(value).div(discount, cw).store(bit(2), cw);
                total = total.add(term, cw).store(bit(3), cw);
            }
        }
        Graph::AddProductSeedFirst => {
            let increment = discount.mul(rate, cw).store(bit(0), cw);
            discount = discount.add(increment, cw).store(bit(1), cw);
            total = Ext::new(values[0]).div(discount, cw).store(bit(2), cw);
            for &value in &values[1..] {
                let increment = discount.mul(rate, cw).store(bit(0), cw);
                discount = discount.add(increment, cw).store(bit(1), cw);
                let term = Ext::new(value).div(discount, cw).store(bit(2), cw);
                total = total.add(term, cw).store(bit(3), cw);
            }
        }
        Graph::MultiplyOnePlusRate => {
            let w = Ext::new(1.0).add(rate, cw).store(bit(5), cw);
            for &value in values {
                discount = discount.mul(w, cw).store(bit(1), cw);
                let term = Ext::new(value).div(discount, cw).store(bit(2), cw);
                total = total.add(term, cw).store(bit(3), cw);
            }
        }
        Graph::ReverseHornerDivision | Graph::ReverseHornerReciprocal => {
            let w = Ext::new(1.0).add(rate, cw).store(bit(5), cw);
            let reciprocal = Ext::new(1.0).div(w, cw).store(bit(4), cw);
            for &value in values.iter().rev() {
                total = total.add(Ext::new(value), cw).store(bit(0), cw);
                total = match graph {
                    Graph::ReverseHornerDivision => total.div(w, cw),
                    Graph::ReverseHornerReciprocal => total.mul(reciprocal, cw),
                    _ => unreachable!(),
                }
                .store(bit(1), cw);
            }
        }
        Graph::ReverseDivideThenAddFinalDivide
        | Graph::ReverseDivideThenAddFinalDivideSeeded
        | Graph::ReverseMultiplyThenAddFinalMultiply => {
            let w = Ext::new(1.0).add(rate, cw).store(bit(5), cw);
            let reciprocal = Ext::new(1.0).div(w, cw).store(bit(4), cw);
            let mut iterator = values.iter().rev();
            if graph == Graph::ReverseDivideThenAddFinalDivideSeeded {
                total = Ext::new(*iterator.next().expect("nonempty NPV values"));
            }
            for &value in iterator {
                total = match graph {
                    Graph::ReverseMultiplyThenAddFinalMultiply => total.mul(reciprocal, cw),
                    _ => total.div(w, cw),
                }
                .store(bit(1), cw);
                total = total.add(Ext::new(value), cw).store(bit(0), cw);
            }
            total = match graph {
                Graph::ReverseMultiplyThenAddFinalMultiply => total.mul(reciprocal, cw),
                _ => total.div(w, cw),
            }
            .store(bit(1), cw);
        }
        Graph::ForwardRepeatedDivision => {
            let w = Ext::new(1.0).add(rate, cw).store(bit(5), cw);
            let mut factor = Ext::new(1.0);
            for &value in values {
                factor = factor.div(w, cw).store(bit(1), cw);
                let term = Ext::new(value).mul(factor, cw).store(bit(2), cw);
                total = total.add(term, cw).store(bit(3), cw);
            }
        }
    }
    total.store(bit(4), cw).to_f64(cw)
}

fn ordered(bits: u64) -> u64 {
    if bits >> 63 == 0 {
        bits | (1 << 63)
    } else {
        !bits
    }
}

fn ulp_distance(left: u64, right: u64) -> u64 {
    ordered(left).abs_diff(ordered(right))
}

#[derive(Clone, Copy, Debug)]
struct Score {
    exact: usize,
    ulp_sum: u128,
    max_ulp: u64,
    by_point: [usize; 3],
}

impl Score {
    fn key(self) -> (usize, std::cmp::Reverse<u128>, std::cmp::Reverse<u64>) {
        (
            self.exact,
            std::cmp::Reverse(self.ulp_sum),
            std::cmp::Reverse(self.max_ulp),
        )
    }
}

fn point_index(point: &str) -> usize {
    match point {
        "base" => 0,
        "v_h_neg" => 1,
        "v_h_pos" => 2,
        _ => panic!("unexpected point {point}"),
    }
}

fn score(observations: &[Obs], graph: Graph, mask: u8, cw: u16) -> Score {
    let mut result = Score {
        exact: 0,
        ulp_sum: 0,
        max_ulp: 0,
        by_point: [0; 3],
    };
    for obs in observations {
        let got = evaluate(obs.rate, &obs.tail, graph, mask, cw).to_bits();
        let distance = ulp_distance(got, obs.raw);
        result.ulp_sum += distance as u128;
        result.max_ulp = result.max_ulp.max(distance);
        if got == obs.raw {
            result.exact += 1;
            result.by_point[point_index(&obs.point)] += 1;
        }
    }
    result
}

fn composed_surface_report(observations: &[Obs]) {
    let mut direct_equals_cell = 0;
    let mut strict = 0;
    let mut pc64 = 0;
    let mut pc53 = 0;
    for obs in observations {
        direct_equals_cell += usize::from(obs.direct == obs.cell);
        let raw = f64::from_bits(obs.raw);
        strict += usize::from((raw + obs.c0).to_bits() == obs.direct);
        let left = Ext::new(raw);
        let right = Ext::new(obs.c0);
        pc64 += usize::from(
            left.add(right, rx::CW_PC64_RN)
                .to_f64(rx::CW_PC64_RN)
                .to_bits()
                == obs.direct,
        );
        pc53 += usize::from(
            left.add(right, rx::CW_PC53_RN)
                .to_f64(rx::CW_PC53_RN)
                .to_bits()
                == obs.direct,
        );
    }
    println!("-- composed/publication boundary --");
    println!(
        "direct == raw-cell+c0 : {direct_equals_cell}/{}",
        observations.len()
    );
    println!(
        "captured raw + c0 strict f64 : {strict}/{}",
        observations.len()
    );
    println!(
        "captured raw + c0 PC64/store : {pc64}/{}",
        observations.len()
    );
    println!(
        "captured raw + c0 PC53/store : {pc53}/{}",
        observations.len()
    );
}

fn main() {
    let observations = load();
    composed_surface_report(&observations);

    let graphs = [
        Graph::AddProduct,
        Graph::AddProductMulCommuted,
        Graph::AddProductSeedFirst,
        Graph::MultiplyOnePlusRate,
        Graph::ReverseHornerDivision,
        Graph::ReverseHornerReciprocal,
        Graph::ReverseDivideThenAddFinalDivide,
        Graph::ReverseDivideThenAddFinalDivideSeeded,
        Graph::ReverseMultiplyThenAddFinalMultiply,
        Graph::ForwardRepeatedDivision,
    ];
    let control_words = [("PC64_RN", rx::CW_PC64_RN), ("PC53_RN", rx::CW_PC53_RN)];
    let mut results = Vec::new();
    for graph in graphs {
        for (cw_name, cw) in control_words {
            for mask in 0u8..64 {
                results.push((
                    score(&observations, graph, mask, cw),
                    graph,
                    cw_name,
                    cw,
                    mask,
                ));
            }
        }
    }
    results.sort_by(|left, right| right.0.key().cmp(&left.0.key()));

    println!("-- raw worksheet NPV candidates --");
    for (score, graph, cw_name, _, mask) in results.iter().take(30) {
        println!(
            "{:3}/{} base={:3} neg={:3} pos={:3} ulp_sum={} max={} {graph:?} {cw_name} mask={mask:06b}",
            score.exact,
            observations.len(),
            score.by_point[0],
            score.by_point[1],
            score.by_point[2],
            score.ulp_sum,
            score.max_ulp,
        );
    }

    let best = results[0];
    println!("-- best misses --");
    let mut shown = 0;
    for obs in &observations {
        let got = evaluate(obs.rate, &obs.tail, best.1, best.4, best.3).to_bits();
        if got != obs.raw && shown < 30 {
            shown += 1;
            println!(
                "{} point={} n={} got=0x{got:016x} want=0x{:016x} ulp={}",
                obs.id,
                obs.point,
                obs.tail.len(),
                obs.raw,
                ulp_distance(got, obs.raw),
            );
        }
    }
}
