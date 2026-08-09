//! W109 G6-01: race period-product formation schedules after the exact-tau
//! predicate survived its confound audit.
//!
//! The banked power-of-two-rate corpus contains the published `tau`, EXP/LN
//! intermediates, and model-free pinned PMT `em`.  The `n == 1` row for each
//! rate supplies the exact stored `log1p(r)` value.  This tool varies only how
//! integer `n` forms `-n*log1p(r)`, then evaluates both the ordinary Kahan
//! reconstruction and its per-operation x87-double-rounded counterpart.

use oxfunc_core::excel_numeric::research as rx;
use rx::{CW_PC53_RN, CW_PC64_RN, ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64};
use std::collections::BTreeMap;

const CORPUS: &str = "../../work/w109/G6-solvers/expm1_intermediates.csv";

#[derive(Clone, Copy)]
struct Row {
    k: i32,
    n: u32,
    captured_tau: f64,
    expected: u64,
}

fn parse_bits(text: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(text, 16).expect("hex f64 bits"))
}

fn x87_add(left: f64, right: f64) -> f64 {
    ext_to_f64(
        &ext_add(&ext_from_f64(left), &ext_from_f64(right), CW_PC64_RN),
        CW_PC53_RN,
    )
}

fn x87_mul(left: f64, right: f64) -> f64 {
    ext_to_f64(
        &ext_mul(&ext_from_f64(left), &ext_from_f64(right), CW_PC64_RN),
        CW_PC53_RN,
    )
}

fn x87_div(left: f64, right: f64) -> f64 {
    ext_to_f64(
        &ext_div(&ext_from_f64(left), &ext_from_f64(right), CW_PC64_RN),
        CW_PC53_RN,
    )
}

fn repeated_add(term: f64, n: u32, add: fn(f64, f64) -> f64) -> f64 {
    let mut sum = 0.0;
    for _ in 0..n {
        sum = add(sum, term);
    }
    sum
}

fn binary_add(term: f64, n: u32, add: fn(f64, f64) -> f64) -> f64 {
    let mut exponent = n;
    let mut power = term;
    let mut sum = 0.0;
    while exponent != 0 {
        if exponent & 1 != 0 {
            sum = add(sum, power);
        }
        exponent >>= 1;
        if exponent != 0 {
            power = add(power, power);
        }
    }
    sum
}

fn plain_add(left: f64, right: f64) -> f64 {
    left + right
}

fn kahan_plain(tau: f64) -> f64 {
    let u = rx::excel_exp(tau);
    if u == 1.0 {
        return tau;
    }
    if tau.abs() < 1.0 {
        (u - 1.0) * tau / rx::excel_ln(u)
    } else {
        u - 1.0
    }
}

fn kahan_x87_spill(tau: f64) -> f64 {
    let u = rx::excel_exp(tau);
    if u == 1.0 {
        return tau;
    }
    if tau.abs() < 1.0 {
        x87_div(x87_mul(u - 1.0, tau), rx::excel_ln(u))
    } else {
        u - 1.0
    }
}

fn main() {
    let text = std::fs::read_to_string(CORPUS).expect("read PMT intermediate corpus");
    let mut rows = Vec::new();
    let mut logs = BTreeMap::new();
    for line in text.lines().skip(1) {
        let fields = line.split(',').collect::<Vec<_>>();
        if fields.len() < 6 {
            continue;
        }
        let k = fields[0].parse::<i32>().expect("k");
        let n = fields[1].parse::<u32>().expect("n");
        let captured_tau = parse_bits(fields[2]);
        let expected = u64::from_str_radix(fields[5], 16).expect("expected bits");
        if n == 1 {
            logs.insert(k, -captured_tau);
        }
        rows.push(Row {
            k,
            n,
            captured_tau,
            expected,
        });
    }

    type Formation = (&'static str, fn(f64, u32) -> f64);
    let formations: [Formation; 6] = [
        ("plain multiply", |log, n| -(n as f64 * log)),
        ("x87-spill multiply", |log, n| x87_mul(-(n as f64), log)),
        ("plain repeated add", |log, n| {
            repeated_add(-log, n, plain_add)
        }),
        ("x87-spill repeated add", |log, n| {
            repeated_add(-log, n, x87_add)
        }),
        ("plain binary add", |log, n| binary_add(-log, n, plain_add)),
        ("x87-spill binary add", |log, n| {
            binary_add(-log, n, x87_add)
        }),
    ];

    let mut formed = vec![vec![0.0; rows.len()]; formations.len()];
    for (formation_index, (_, formation)) in formations.iter().enumerate() {
        for (row_index, row) in rows.iter().enumerate() {
            formed[formation_index][row_index] = formation(logs[&row.k], row.n);
        }
    }

    println!("PMT tau-formation race, rows={}", rows.len());
    println!(
        "{:<30} {:>14} {:>14} {:>16}",
        "formation", "tau=captured", "plain Kahan", "x87-spill Kahan"
    );
    for (formation_index, (name, _)) in formations.iter().enumerate() {
        let same_tau = rows
            .iter()
            .enumerate()
            .filter(|(index, row)| {
                formed[formation_index][*index].to_bits() == row.captured_tau.to_bits()
            })
            .count();
        let plain = rows
            .iter()
            .enumerate()
            .filter(|(index, row)| {
                kahan_plain(formed[formation_index][*index]).to_bits() == row.expected
            })
            .count();
        let spill = rows
            .iter()
            .enumerate()
            .filter(|(index, row)| {
                kahan_x87_spill(formed[formation_index][*index]).to_bits() == row.expected
            })
            .count();
        println!(
            "{name:<30} {same_tau:>6}/{:<7} {plain:>6}/{:<7} {spill:>6}/{:<7}",
            rows.len(),
            rows.len(),
            rows.len()
        );
    }

    let captured_plain = rows
        .iter()
        .filter(|row| kahan_plain(row.captured_tau).to_bits() == row.expected)
        .count();
    let captured_spill = rows
        .iter()
        .filter(|row| kahan_x87_spill(row.captured_tau).to_bits() == row.expected)
        .count();
    println!(
        "{:<30} {:>6}/{:<7} {:>6}/{:<7} {:>6}/{:<7}",
        "captured tau control",
        rows.len(),
        rows.len(),
        captured_plain,
        rows.len(),
        captured_spill,
        rows.len()
    );

    let exact_product = |row: &Row| {
        let log = logs[&row.k];
        let product = row.n as f64 * log;
        product.to_bits() == (-row.captured_tau).to_bits()
            && (row.n as f64).mul_add(log, -product) == 0.0
    };
    let exact_count = rows.iter().filter(|row| exact_product(row)).count();
    println!(
        "\nExact-product partition: {exact_count}/{} exact, {}/{} rounded",
        rows.len(),
        rows.len() - exact_count,
        rows.len()
    );

    type EmModel = (&'static str, fn(f64) -> f64);
    let em_models: [EmModel; 8] = [
        ("plain Kahan", kahan_plain),
        ("x87-spill Kahan", kahan_x87_spill),
        ("num x87 / div plain", |tau| {
            let u = rx::excel_exp(tau);
            if u == 1.0 {
                tau
            } else if tau.abs() < 1.0 {
                x87_mul(u - 1.0, tau) / rx::excel_ln(u)
            } else {
                u - 1.0
            }
        }),
        ("num plain / div x87", |tau| {
            let u = rx::excel_exp(tau);
            if u == 1.0 {
                tau
            } else if tau.abs() < 1.0 {
                x87_div((u - 1.0) * tau, rx::excel_ln(u))
            } else {
                u - 1.0
            }
        }),
        ("plain exp minus one", |tau| rx::excel_exp(tau) - 1.0),
        ("portable faithful expm1", rx::excel_expm1),
        ("tau passthrough", |tau| tau),
        ("negative zero control", |_| -0.0),
    ];
    println!(
        "{:<28} {:>10} {:>12} {:>14}",
        "em model", "all", "exact tau", "rounded tau"
    );
    for (name, model) in em_models {
        let mut all = 0usize;
        let mut exact_hits = 0usize;
        let mut rounded_hits = 0usize;
        for row in &rows {
            let hit = model(row.captured_tau).to_bits() == row.expected;
            all += usize::from(hit);
            if exact_product(row) {
                exact_hits += usize::from(hit);
            } else {
                rounded_hits += usize::from(hit);
            }
        }
        println!(
            "{name:<28} {all:>4}/{:<5} {exact_hits:>4}/{:<7} {rounded_hits:>4}/{:<7}",
            rows.len(),
            exact_count,
            rows.len() - exact_count
        );
    }

    println!("\nHybrid: alternate formation only where the product is exact");
    for (formation_index, (name, _)) in formations.iter().enumerate().skip(1) {
        let plain = rows
            .iter()
            .enumerate()
            .filter(|(index, row)| {
                let tau = if exact_product(row) {
                    formed[formation_index][*index]
                } else {
                    row.captured_tau
                };
                kahan_plain(tau).to_bits() == row.expected
            })
            .count();
        let spill = rows
            .iter()
            .enumerate()
            .filter(|(index, row)| {
                let tau = if exact_product(row) {
                    formed[formation_index][*index]
                } else {
                    row.captured_tau
                };
                kahan_x87_spill(tau).to_bits() == row.expected
            })
            .count();
        println!(
            "  {name:<28} plain={plain:>3}/{} spill={spill:>3}/{}",
            rows.len(),
            rows.len()
        );
    }
}
