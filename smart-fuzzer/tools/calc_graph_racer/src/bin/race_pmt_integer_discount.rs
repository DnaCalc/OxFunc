//! W109 G6-01: test integer-period discount recurrences as an explanation for
//! the exact-period-product PMT split.
//!
//! The private PMT routine receives both `rate` and integer `nper`; it need not
//! compute a generic `expm1(tau)`.  This racer evaluates stable recurrences in
//! the `em = discount - 1` domain, including binary composition
//! `(1+a)(1+b)-1 = a+b+a*b`, on the 234-row power-of-two-rate bank and the 90
//! independently pinned general-rate rows.

use oxfunc_core::excel_numeric::research as rx;
use rx::{CW_PC53_RN, CW_PC64_RN, ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64};
use std::collections::BTreeMap;

const POWER_CORPUS: &str = "../../work/w109/G6-solvers/expm1_intermediates.csv";
const GENERAL_META: &str = "../../work/w109/G6-solvers/meta-pmt-general-intermediates-20260809.csv";

#[derive(Clone, Copy)]
struct Row {
    rate: f64,
    periods: u64,
    captured_tau: f64,
    expected: u64,
    general: bool,
    exact_tau: bool,
}

#[derive(Clone, Copy, Debug)]
enum Precision {
    Plain,
    X87Spill,
}

fn parse_bits(text: &str) -> u64 {
    u64::from_str_radix(text.trim_matches('"').trim_start_matches("0x"), 16).expect("hex f64 bits")
}

fn add(left: f64, right: f64, precision: Precision) -> f64 {
    match precision {
        Precision::Plain => left + right,
        Precision::X87Spill => ext_to_f64(
            &ext_add(&ext_from_f64(left), &ext_from_f64(right), CW_PC64_RN),
            CW_PC53_RN,
        ),
    }
}

fn sub(left: f64, right: f64, precision: Precision) -> f64 {
    match precision {
        Precision::Plain => left - right,
        Precision::X87Spill => ext_to_f64(
            &ext_sub(&ext_from_f64(left), &ext_from_f64(right), CW_PC64_RN),
            CW_PC53_RN,
        ),
    }
}

fn mul(left: f64, right: f64, precision: Precision) -> f64 {
    match precision {
        Precision::Plain => left * right,
        Precision::X87Spill => ext_to_f64(
            &ext_mul(&ext_from_f64(left), &ext_from_f64(right), CW_PC64_RN),
            CW_PC53_RN,
        ),
    }
}

fn div(left: f64, right: f64, precision: Precision) -> f64 {
    match precision {
        Precision::Plain => left / right,
        Precision::X87Spill => ext_to_f64(
            &ext_div(&ext_from_f64(left), &ext_from_f64(right), CW_PC64_RN),
            CW_PC53_RN,
        ),
    }
}

fn base_em(rate: f64, precision: Precision, variant: u8) -> f64 {
    let one_plus = add(1.0, rate, precision);
    match variant {
        0 => div(-rate, one_plus, precision),
        1 => sub(div(1.0, one_plus, precision), 1.0, precision),
        2 => -div(rate, one_plus, precision),
        3 => mul(-rate, div(1.0, one_plus, precision), precision),
        _ => unreachable!(),
    }
}

fn compose(left: f64, right: f64, precision: Precision, variant: u8) -> f64 {
    match variant {
        0 => add(
            add(left, right, precision),
            mul(left, right, precision),
            precision,
        ),
        1 => add(
            left,
            mul(add(1.0, left, precision), right, precision),
            precision,
        ),
        2 => add(
            right,
            mul(add(1.0, right, precision), left, precision),
            precision,
        ),
        3 => sub(
            mul(
                add(1.0, left, precision),
                add(1.0, right, precision),
                precision,
            ),
            1.0,
            precision,
        ),
        _ => unreachable!(),
    }
}

fn double(value: f64, precision: Precision, compose_variant: u8, variant: u8) -> f64 {
    match variant {
        0 => compose(value, value, precision, compose_variant),
        1 => mul(value, add(value, 2.0, precision), precision),
        2 => add(
            add(value, value, precision),
            mul(value, value, precision),
            precision,
        ),
        3 => add(
            value,
            mul(add(1.0, value, precision), value, precision),
            precision,
        ),
        _ => unreachable!(),
    }
}

fn binary_lsb(
    rate: f64,
    mut periods: u64,
    precision: Precision,
    base_variant: u8,
    compose_variant: u8,
    double_variant: u8,
) -> f64 {
    let mut accumulator = 0.0;
    let mut power = base_em(rate, precision, base_variant);
    while periods != 0 {
        if periods & 1 != 0 {
            accumulator = compose(accumulator, power, precision, compose_variant);
        }
        periods >>= 1;
        if periods != 0 {
            power = double(power, precision, compose_variant, double_variant);
        }
    }
    accumulator
}

fn binary_msb(
    rate: f64,
    periods: u64,
    precision: Precision,
    base_variant: u8,
    compose_variant: u8,
    double_variant: u8,
) -> f64 {
    let base = base_em(rate, precision, base_variant);
    let top = 63 - periods.leading_zeros();
    let mut accumulator = 0.0;
    for bit in (0..=top).rev() {
        accumulator = double(accumulator, precision, compose_variant, double_variant);
        if periods & (1_u64 << bit) != 0 {
            accumulator = compose(accumulator, base, precision, compose_variant);
        }
    }
    accumulator
}

fn sequential(rate: f64, periods: u64, precision: Precision, variant: u8) -> f64 {
    let denominator = add(1.0, rate, precision);
    let mut em = 0.0;
    for _ in 0..periods {
        em = match variant {
            0 => div(sub(em, rate, precision), denominator, precision),
            1 => sub(
                div(add(1.0, em, precision), denominator, precision),
                1.0,
                precision,
            ),
            2 => compose(em, base_em(rate, precision, 0), precision, 0),
            _ => unreachable!(),
        };
    }
    em
}

fn load_rows() -> Vec<Row> {
    let mut rows = Vec::new();
    let power = std::fs::read_to_string(POWER_CORPUS).expect("read power corpus");
    for line in power.lines().skip(1) {
        let fields = line.split(',').collect::<Vec<_>>();
        if fields.len() < 6 {
            continue;
        }
        let exponent = fields[0].parse::<i32>().expect("rate exponent");
        rows.push(Row {
            rate: 2.0_f64.powi(exponent),
            periods: fields[1].parse().expect("periods"),
            captured_tau: f64::from_bits(parse_bits(fields[2])),
            expected: parse_bits(fields[5]),
            general: false,
            exact_tau: false,
        });
    }
    let general = std::fs::read_to_string(GENERAL_META).expect("read general metadata");
    for line in general.lines().skip(1) {
        let fields = line.split(',').collect::<Vec<_>>();
        if fields.len() < 8 {
            continue;
        }
        rows.push(Row {
            rate: f64::from_bits(parse_bits(fields[3])),
            periods: fields[4].trim_matches('"').parse().expect("periods"),
            captured_tau: f64::from_bits(parse_bits(fields[5])),
            expected: parse_bits(fields[6]),
            general: true,
            exact_tau: false,
        });
    }
    assert_eq!(rows.len(), 324);
    let mut logs = BTreeMap::new();
    for row in &rows {
        if row.periods == 1 {
            logs.insert(row.rate.to_bits(), row.captured_tau.abs());
        }
    }
    for row in &mut rows {
        let log = logs[&row.rate.to_bits()];
        let product = row.periods as f64 * log;
        row.exact_tau = product.to_bits() == row.captured_tau.abs().to_bits()
            && (row.periods as f64).mul_add(log, -product) == 0.0;
    }
    rows
}

#[derive(Clone)]
struct Score {
    name: String,
    all: usize,
    power: usize,
    general: usize,
    exact: usize,
    rounded: usize,
    n1: usize,
}

fn main() {
    let rows = load_rows();
    let mut scores = Vec::new();
    for precision in [Precision::Plain, Precision::X87Spill] {
        for base_variant in 0..4 {
            for compose_variant in 0..4 {
                for double_variant in 0..4 {
                    for schedule in 0..2 {
                        let mut score = Score {
                            name: format!(
                                "{precision:?} b{base_variant} c{compose_variant} d{double_variant} {}",
                                if schedule == 0 { "lsb" } else { "msb" }
                            ),
                            all: 0,
                            power: 0,
                            general: 0,
                            exact: 0,
                            rounded: 0,
                            n1: 0,
                        };
                        for row in &rows {
                            let value = if schedule == 0 {
                                binary_lsb(
                                    row.rate,
                                    row.periods,
                                    precision,
                                    base_variant,
                                    compose_variant,
                                    double_variant,
                                )
                            } else {
                                binary_msb(
                                    row.rate,
                                    row.periods,
                                    precision,
                                    base_variant,
                                    compose_variant,
                                    double_variant,
                                )
                            };
                            if value.to_bits() == row.expected {
                                score.all += 1;
                                if row.general {
                                    score.general += 1;
                                } else {
                                    score.power += 1;
                                }
                                if row.exact_tau {
                                    score.exact += 1;
                                } else {
                                    score.rounded += 1;
                                }
                                if row.periods == 1 {
                                    score.n1 += 1;
                                }
                            }
                        }
                        scores.push(score);
                    }
                }
            }
        }
    }

    for precision in [Precision::Plain, Precision::X87Spill] {
        for variant in 0..3 {
            let mut score = Score {
                name: format!("{precision:?} sequential v{variant}"),
                all: 0,
                power: 0,
                general: 0,
                exact: 0,
                rounded: 0,
                n1: 0,
            };
            for row in &rows {
                let value = sequential(row.rate, row.periods, precision, variant);
                if value.to_bits() == row.expected {
                    score.all += 1;
                    if row.general {
                        score.general += 1;
                    } else {
                        score.power += 1;
                    }
                    if row.exact_tau {
                        score.exact += 1;
                    } else {
                        score.rounded += 1;
                    }
                    if row.periods == 1 {
                        score.n1 += 1;
                    }
                }
            }
            scores.push(score);
        }
    }

    scores.sort_by_key(|score| std::cmp::Reverse((score.all, score.general, score.power)));
    println!("PMT integer-discount recurrence race, rows=324 (power=234, general=90)");
    println!(
        "{:<42} {:>9} {:>11} {:>11} {:>11} {:>11} {:>9}",
        "model", "all", "power", "general", "exact", "rounded", "n=1"
    );
    for score in scores.iter().take(40) {
        println!(
            "{:<42} {:>3}/324  {:>3}/234    {:>2}/90    {:>3}/239    {:>2}/85    {:>2}/33",
            score.name, score.all, score.power, score.general, score.exact, score.rounded, score.n1
        );
    }
}
