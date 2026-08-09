//! Offline audit of the currently banked G3-04 evidence.
//!
//! The purpose of this binary is diagnostic: it demonstrates exactly how
//! underdetermined the two legacy GROWTH rows are before any new oracle work.

#[path = "growth_research/common.rs"]
mod common;

use common::*;

struct Existing {
    id: &'static str,
    y: &'static [f64],
    x: &'static [f64],
    new_x: f64,
    expected: u64,
}

const CASES: [Existing; 2] = [
    Existing {
        id: "growth-catalog",
        y: &[1.0, 3.0, 2.0, 5.0],
        x: &[1.0, 2.0, 3.0, 4.0],
        new_x: 5.0,
        expected: 0x401c48c6001f0ac0,
    },
    Existing {
        id: "growth-geometric",
        y: &[2.0, 4.0, 8.0, 16.0],
        x: &[1.0, 2.0, 3.0, 4.0],
        new_x: 5.0,
        expected: 0x403ffffffffffffe,
    },
];

#[derive(Default)]
struct Score {
    exact: usize,
    max_ulp: u64,
}

fn main() {
    let variants = regression_variants();
    let mut scores = Vec::new();
    for &ln in &LOG_PROVIDERS {
        for &regression in &variants {
            for &coefficient_exp in &COEFFICIENT_EXP_VARIANTS {
                for &prediction in &PREDICTION_GRAPHS {
                    let mut score = Score::default();
                    let mut outputs = Vec::new();
                    for case in &CASES {
                        let logged = case.y.iter().map(|&v| ln.eval(v)).collect::<Vec<_>>();
                        let coefficients = regress(case.x, &logged, regression);
                        let published = publish_coefficients(coefficients, coefficient_exp);
                        let got = predict(coefficients, published, case.new_x, prediction);
                        score.exact += usize::from(got.to_bits() == case.expected);
                        score.max_ulp = score.max_ulp.max(
                            ulp_distance(got, f64::from_bits(case.expected)).unwrap_or(u64::MAX),
                        );
                        outputs.push(format!("{}={}", case.id, hex(got)));
                    }
                    scores.push((
                        score.exact,
                        score.max_ulp,
                        format!(
                            "{}|{}|{}|{}",
                            ln.tag(),
                            regression.id(),
                            coefficient_exp.tag(),
                            prediction.tag()
                        ),
                        outputs,
                    ));
                }
            }
        }
    }
    scores.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| left.1.cmp(&right.1))
            .then_with(|| left.2.cmp(&right.2))
    });
    println!(
        "candidate_combinations={} banked_growth_rows={} (no paired LOGEST coefficients)",
        scores.len(),
        CASES.len()
    );
    let exact_both = scores.iter().filter(|entry| entry.0 == CASES.len()).count();
    let exact_one = scores.iter().filter(|entry| entry.0 == 1).count();
    println!("exact_both={exact_both} exact_one={exact_one}");
    for (rank, (exact, max_ulp, id, outputs)) in scores.iter().take(30).enumerate() {
        println!(
            "{:02} exact={}/{} max_ulp={} {} {}",
            rank + 1,
            exact,
            CASES.len(),
            max_ulp,
            id,
            outputs.join(" ")
        );
    }

    // One old scalar/array capture supplies a paired LOGEST/GROWTH control.
    // It is intentionally kept separate from the two-row numeric catalog bank.
    let factor = f64::from_bits(0x3ff8000000000002);
    let base = f64::from_bits(0x3ff5555555555553);
    let g1 = base * factor.powf(1.0);
    let g2 = base * factor.powf(2.0);
    println!(
        "legacy_pair_control native-published graph: x1={} want=0x3fffffffffffffff; x2={} want=0x4008000000000001",
        hex(g1),
        hex(g2)
    );
    assert_eq!(g1.to_bits(), 0x3fffffffffffffff);
    assert_eq!(g2.to_bits(), 0x4008000000000001);
}
