//! W109 G4-03 ACOTH reciprocal-series calculation-graph racer.
//!
//! This program is oracle-offline. It enumerates explicit arithmetic graphs
//! for the odd power series
//!
//!     atanh(1/a) = 1/a + 1/(3*a^3) + 1/(5*a^5) + ...
//!
//! against previously captured Excel witnesses.  The candidate family keeps
//! reciprocal, square, power, coefficient, and accumulator staging separate;
//! this is intended to identify a calculation graph, not merely fit values.

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const CW64: u16 = rx::CW_PC64_RN;
const CW53: u16 = rx::CW_PC53_RN;
const TERMS: usize = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Stage {
    F64,
    X87P64Store,
    X87P53Store,
}

impl Stage {
    const ALL: [Self; 3] = [Self::F64, Self::X87P64Store, Self::X87P53Store];

    fn tag(self) -> &'static str {
        match self {
            Self::F64 => "f64",
            Self::X87P64Store => "x64s",
            Self::X87P53Store => "x53s",
        }
    }

    fn cw(self) -> Option<u16> {
        match self {
            Self::F64 => None,
            Self::X87P64Store => Some(CW64),
            Self::X87P53Store => Some(CW53),
        }
    }

    fn add(self, left: f64, right: f64) -> f64 {
        match self.cw() {
            None => left + right,
            Some(cw) => rx::ext_to_f64(
                &rx::ext_add(&rx::ext_from_f64(left), &rx::ext_from_f64(right), cw),
                cw,
            ),
        }
    }

    fn mul(self, left: f64, right: f64) -> f64 {
        match self.cw() {
            None => left * right,
            Some(cw) => rx::ext_to_f64(
                &rx::ext_mul(&rx::ext_from_f64(left), &rx::ext_from_f64(right), cw),
                cw,
            ),
        }
    }

    fn div(self, left: f64, right: f64) -> f64 {
        match self.cw() {
            None => left / right,
            Some(cw) => rx::ext_to_f64(
                &rx::ext_div(&rx::ext_from_f64(left), &rx::ext_from_f64(right), cw),
                cw,
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Scheme {
    PowerDiv,
    PowerMulCoeff,
    RecurrenceMulDiv,
    RecurrenceRatio,
    TailPowerDiv,
    SquareFromA,
    InversePower,
    Horner,
    HornerMulCoeff,
}

impl Scheme {
    const ALL: [Self; 9] = [
        Self::PowerDiv,
        Self::PowerMulCoeff,
        Self::RecurrenceMulDiv,
        Self::RecurrenceRatio,
        Self::TailPowerDiv,
        Self::SquareFromA,
        Self::InversePower,
        Self::Horner,
        Self::HornerMulCoeff,
    ];

    fn tag(self) -> &'static str {
        match self {
            Self::PowerDiv => "power/div",
            Self::PowerMulCoeff => "power*coef",
            Self::RecurrenceMulDiv => "term*z*prev/odd",
            Self::RecurrenceRatio => "term*z*(prev/odd)",
            Self::TailPowerDiv => "x+tail",
            Self::SquareFromA => "z=1/(a*a)",
            Self::InversePower => "1/(odd*a^odd)",
            Self::Horner => "horner/div",
            Self::HornerMulCoeff => "horner*coef",
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Graph {
    scheme: Scheme,
    recip: Stage,
    mul: Stage,
    div: Stage,
    add: Stage,
    terms: usize,
    terminate_when_stable: bool,
}

impl Graph {
    fn name(self) -> String {
        format!(
            "{} r={} m={} d={} a={} n={}{}",
            self.scheme.tag(),
            self.recip.tag(),
            self.mul.tag(),
            self.div.tag(),
            self.add.tag(),
            self.terms,
            if self.terminate_when_stable {
                " stable"
            } else {
                " fixed"
            }
        )
    }

    fn eval(self, a: f64) -> f64 {
        let x = self.recip.div(1.0, a);
        // Current-reference Excel observes DAZ/FTZ in this legacy function
        // body: subnormal reciprocal magnitudes publish positive zero.
        if x < f64::MIN_POSITIVE {
            return 0.0;
        }

        match self.scheme {
            Scheme::PowerDiv => self.power_div(x, self.mul.mul(x, x), false),
            Scheme::PowerMulCoeff => self.power_mul_coeff(x, self.mul.mul(x, x)),
            Scheme::RecurrenceMulDiv => self.recurrence_mul_div(x),
            Scheme::RecurrenceRatio => self.recurrence_ratio(x),
            Scheme::TailPowerDiv => self.power_div(x, self.mul.mul(x, x), true),
            Scheme::SquareFromA => {
                let z = self.div.div(1.0, self.mul.mul(a, a));
                self.power_div(x, z, false)
            }
            Scheme::InversePower => self.inverse_power(a, x),
            Scheme::Horner => self.horner(x, false),
            Scheme::HornerMulCoeff => self.horner(x, true),
        }
    }

    fn power_div(self, x: f64, z: f64, tail_first: bool) -> f64 {
        let mut power = x;
        let mut sum = if tail_first { 0.0 } else { x };
        for k in 1..self.terms {
            power = self.mul.mul(power, z);
            let term = self.div.div(power, (2 * k + 1) as f64);
            let next = self.add.add(sum, term);
            if self.terminate_when_stable && next == sum {
                break;
            }
            sum = next;
        }
        if tail_first {
            self.add.add(x, sum)
        } else {
            sum
        }
    }

    fn power_mul_coeff(self, x: f64, z: f64) -> f64 {
        let mut power = x;
        let mut sum = x;
        for k in 1..self.terms {
            power = self.mul.mul(power, z);
            let coefficient = 1.0 / ((2 * k + 1) as f64);
            let term = self.mul.mul(power, coefficient);
            let next = self.add.add(sum, term);
            if self.terminate_when_stable && next == sum {
                break;
            }
            sum = next;
        }
        sum
    }

    fn recurrence_mul_div(self, x: f64) -> f64 {
        let z = self.mul.mul(x, x);
        let mut term = x;
        let mut sum = x;
        for k in 1..self.terms {
            term = self.mul.mul(term, z);
            term = self.mul.mul(term, (2 * k - 1) as f64);
            term = self.div.div(term, (2 * k + 1) as f64);
            let next = self.add.add(sum, term);
            if self.terminate_when_stable && next == sum {
                break;
            }
            sum = next;
        }
        sum
    }

    fn recurrence_ratio(self, x: f64) -> f64 {
        let z = self.mul.mul(x, x);
        let mut term = x;
        let mut sum = x;
        for k in 1..self.terms {
            term = self.mul.mul(term, z);
            let ratio = (2 * k - 1) as f64 / (2 * k + 1) as f64;
            term = self.mul.mul(term, ratio);
            let next = self.add.add(sum, term);
            if self.terminate_when_stable && next == sum {
                break;
            }
            sum = next;
        }
        sum
    }

    fn inverse_power(self, a: f64, x: f64) -> f64 {
        let square = self.mul.mul(a, a);
        let mut denominator_power = a;
        let mut sum = x;
        for k in 1..self.terms {
            denominator_power = self.mul.mul(denominator_power, square);
            let denominator = self.mul.mul((2 * k + 1) as f64, denominator_power);
            let term = self.div.div(1.0, denominator);
            let next = self.add.add(sum, term);
            if self.terminate_when_stable && next == sum {
                break;
            }
            sum = next;
        }
        sum
    }

    fn horner(self, x: f64, multiply_coefficients: bool) -> f64 {
        let z = self.mul.mul(x, x);
        let mut polynomial = if multiply_coefficients {
            1.0 / ((2 * self.terms - 1) as f64)
        } else {
            self.div.div(1.0, (2 * self.terms - 1) as f64)
        };
        for k in (0..self.terms - 1).rev() {
            let coefficient = if multiply_coefficients {
                1.0 / ((2 * k + 1) as f64)
            } else {
                self.div.div(1.0, (2 * k + 1) as f64)
            };
            polynomial = self.add.add(coefficient, self.mul.mul(z, polynomial));
        }
        self.mul.mul(x, polynomial)
    }
}

fn ratio(a: f64) -> f64 {
    let numerator = a + 1.0;
    let denominator = a - 1.0;
    let quotient = Stage::X87P64Store.div(numerator, denominator);
    let logarithm = rx::ext_fyl2x(&rx::ext_ln2(), &rx::ext_from_f64(quotient), CW64);
    rx::ext_to_f64(&rx::ext_mul(&logarithm, &rx::ext_from_f64(0.5), CW64), CW64)
}

fn load(path: &Path) -> Vec<(String, f64, u64)> {
    let document: WitnessSet =
        serde_json::from_str(&std::fs::read_to_string(path).expect("read answers"))
            .expect("parse answers");
    document
        .witnesses
        .iter()
        .filter_map(|witness| {
            let input = match &witness.args[0] {
                WitnessArg::Scalar(text) => parse_bits_hex(text)?,
                _ => return None,
            };
            let expected = parse_bits_hex(&witness.expected_bits)?;
            Some((
                witness.id.clone().unwrap_or_else(|| "<missing-id>".into()),
                input,
                expected.to_bits(),
            ))
        })
        .collect()
}

fn combined_rows(base: &Path) -> Vec<(String, f64, u64)> {
    let paths = [
        base.join("G4-hyp-answers-acoth.json"),
        base.join("G4-03-acoth/answers-acoth-dense-discovery-20260809.json"),
        base.join("G4-03-acoth/answers-acoth-graph-discovery-20260809.json"),
        base.join("G4-03-acoth/answers-acoth-switch-r1-20260809.json"),
        base.join("G4-03-acoth/answers-acoth-switch-r2-20260809.json"),
        base.join("G4-03-acoth/answers-acoth-exact-heldout-20260809.json"),
    ];
    let mut rows = BTreeMap::new();
    for path in paths {
        for row in load(&path) {
            rows.insert(row.1.to_bits(), row);
        }
    }
    rows.into_values().collect()
}

fn all_graphs() -> Vec<Graph> {
    let mut graphs = Vec::new();
    for scheme in Scheme::ALL {
        for recip in Stage::ALL {
            for mul in Stage::ALL {
                for div in Stage::ALL {
                    for add in Stage::ALL {
                        for terminate_when_stable in [false, true] {
                            graphs.push(Graph {
                                scheme,
                                recip,
                                mul,
                                div,
                                add,
                                terms: TERMS,
                                terminate_when_stable,
                            });
                        }
                    }
                }
            }
        }
    }
    graphs
}

fn main() {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../work/w109");
    let all_rows = combined_rows(&base);
    let mut rows: Vec<_> = all_rows
        .iter()
        .cloned()
        .into_iter()
        .filter(|(_, input, _)| input.is_sign_positive())
        .collect();
    rows.sort_by(|left, right| left.1.partial_cmp(&right.1).unwrap());
    let graphs = all_graphs();
    println!(
        "{} positive rows; {} explicit series graphs",
        rows.len(),
        graphs.len()
    );

    let ratio_hits: Vec<bool> = rows
        .iter()
        .map(|(_, input, expected)| ratio(*input).to_bits() == *expected)
        .collect();
    let mut ranked = Vec::with_capacity(graphs.len());
    let mut union = vec![false; rows.len()];
    for graph in &graphs {
        let hits: Vec<bool> = rows
            .iter()
            .map(|(_, input, expected)| graph.eval(*input).to_bits() == *expected)
            .collect();
        for (covered, hit) in union.iter_mut().zip(&hits) {
            *covered |= *hit;
        }

        let mut suffix = hits.iter().filter(|hit| **hit).count();
        let mut best = (suffix, 0_usize);
        for index in 0..rows.len() {
            suffix -= usize::from(hits[index]);
            suffix += usize::from(ratio_hits[index]);
            if suffix > best.0 {
                best = (suffix, index + 1);
            }
        }
        ranked.push((best.0, best.1, *graph, hits));
    }
    ranked.sort_by_key(|entry| std::cmp::Reverse(entry.0));

    println!("\nbest ratio(<T) | series(>=T) graphs (DAZ included):");
    for (score, cut, graph, _) in ranked.iter().take(40) {
        let threshold = rows.get(*cut).map_or(f64::INFINITY, |row| row.1);
        println!(
            "  {score:5}/{} T={threshold:.17e} bits=0x{:016x}  {}",
            rows.len(),
            threshold.to_bits(),
            graph.name()
        );
    }

    let union_count = union.iter().filter(|hit| **hit).count();
    println!(
        "\nseries-family oracle union: {union_count}/{} ({} uncovered)",
        rows.len(),
        rows.len() - union_count
    );

    println!("\ndirect inverse-power term-count scan:");
    for terms in 4_usize..=40 {
        let graph = Graph {
            scheme: Scheme::InversePower,
            recip: Stage::X87P64Store,
            mul: Stage::X87P64Store,
            div: Stage::F64,
            add: Stage::X87P64Store,
            terms,
            terminate_when_stable: false,
        };
        let hits: Vec<bool> = rows
            .iter()
            .map(|(_, input, expected)| graph.eval(*input).to_bits() == *expected)
            .collect();
        let mut suffix = hits.iter().filter(|hit| **hit).count();
        let mut best = (suffix, 0_usize);
        for index in 0..rows.len() {
            suffix -= usize::from(hits[index]);
            suffix += usize::from(ratio_hits[index]);
            if suffix > best.0 {
                best = (suffix, index + 1);
            }
        }
        if best.0 + 8 >= rows.len() {
            let threshold = rows.get(best.1).map_or(f64::INFINITY, |row| row.1);
            println!(
                "  n={terms:2} {}/{} T={threshold:.17e} bits=0x{:016x}",
                best.0,
                rows.len(),
                threshold.to_bits()
            );
        }
    }

    let (_, cut, best_graph, best_hits) = &ranked[0];
    let threshold = rows.get(*cut).map_or(f64::INFINITY, |row| row.1);
    let signed_score = all_rows
        .iter()
        .filter(|(_, input, expected)| {
            let magnitude = input.abs();
            let value = if magnitude < threshold {
                ratio(magnitude)
            } else {
                best_graph.eval(magnitude)
            };
            let published = if value == 0.0 {
                0.0
            } else {
                value.copysign(*input)
            };
            published.to_bits() == *expected
        })
        .count();
    println!(
        "signed score with odd publication: {signed_score}/{}",
        all_rows.len()
    );
    let mut residuals = Vec::new();
    for (index, (id, input, expected)) in rows.iter().enumerate() {
        let got = if index < *cut {
            ratio(*input)
        } else {
            best_graph.eval(*input)
        };
        if got.to_bits() != *expected {
            residuals.push((id, *input, *expected, got.to_bits(), union[index]));
        }
    }
    println!(
        "\nbest residuals: {} using cut={} and {}",
        residuals.len(),
        cut,
        best_graph.name()
    );
    for (id, input, expected, got, any_series) in &residuals {
        println!(
            "  {id:26} x=0x{:016x} {:.17e} want=0x{expected:016x} got=0x{got:016x} d={:+} any-series={any_series}",
            input.to_bits(),
            input,
            *got as i64 - *expected as i64,
        );
    }

    println!("\nbranch discriminator bracket in 2 <= x < 5:");
    let mut last_ratio_only = None;
    let mut first_series_only = None;
    for (index, (id, input, expected)) in rows.iter().enumerate() {
        if *input < 2.0 || *input >= 5.0 {
            continue;
        }
        let r = ratio_hits[index];
        let s = best_graph.eval(*input).to_bits() == *expected;
        if r && !s {
            last_ratio_only = Some((id, input, expected));
        } else if s && !r && first_series_only.is_none() {
            first_series_only = Some((id, input, expected));
        }
    }
    for (label, row) in [
        ("last ratio-only", last_ratio_only),
        ("first series-only", first_series_only),
    ] {
        if let Some((id, input, expected)) = row {
            println!(
                "  {label:18} {id:26} x=0x{:016x} {:.17e} want=0x{expected:016x}",
                input.to_bits(),
                input
            );
        }
    }

    let mut owners = BTreeSet::new();
    for (index, (_, _input, _expected)) in rows.iter().enumerate() {
        if best_hits[index] {
            continue;
        }
        for (_, _, graph, hits) in &ranked {
            if hits[index] {
                owners.insert(graph.name());
                break;
            }
        }
    }
    println!("alternate owner graph count: {}", owners.len());
}
