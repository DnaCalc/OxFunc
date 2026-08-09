//! Freeze answer-blind IRR discovery and held-out batteries for W109 G6-06.
//!
//! The existing bank contains only two cash-flow shapes.  These batteries vary
//! polynomial degree, root sign, scale, sparsity, cancellation, and starting
//! guess distance.  No Excel result participates in selection.  The held-out
//! shapes are emitted separately so they can remain unopened until a graph is
//! fixed from discovery evidence.

use serde_json::json;
use std::collections::BTreeSet;

const ROOT: &str = "../../work/w109/G6-solvers";

#[derive(Clone)]
struct Shape {
    id: &'static str,
    class: &'static str,
    values: Vec<f64>,
    seed: f64,
}

fn hex(value: f64) -> String {
    format!("0x{:016x}", value.to_bits())
}

fn npv_and_derivative(values: &[f64], rate: f64) -> (f64, f64) {
    let base = 1.0 + rate;
    let mut power = 1.0;
    let mut value = values[0];
    let mut derivative = 0.0;
    for (period, cash) in values.iter().copied().enumerate().skip(1) {
        power *= base;
        value += cash / power;
        derivative -= period as f64 * cash / (power * base);
    }
    (value, derivative)
}

fn refine_root(shape: &Shape) -> f64 {
    let mut rate = shape.seed;
    for _ in 0..100 {
        let (value, derivative) = npv_and_derivative(&shape.values, rate);
        if !value.is_finite() || !derivative.is_finite() || derivative == 0.0 {
            break;
        }
        let next = rate - value / derivative;
        if !next.is_finite() || next <= -0.99 || next > 32.0 {
            rate = (rate + shape.seed) * 0.5;
            continue;
        }
        if next.to_bits() == rate.to_bits() {
            break;
        }
        rate = next;
    }
    let scale = shape.values.iter().map(|x| x.abs()).sum::<f64>();
    let residual = npv_and_derivative(&shape.values, rate).0.abs() / scale.max(1.0);
    assert!(
        rate.is_finite() && rate > -0.99 && residual < 1.0e-11,
        "{} root refinement failed: rate={rate:?} scaled residual={residual:e}",
        shape.id
    );
    rate
}

// Map finite f64s to an integer order in which adjacent integers are adjacent
// floating-point values, then move by a deliberately large ULP stride.
fn offset_ulps(value: f64, delta: i64) -> f64 {
    let bits = value.to_bits();
    let ordered = if bits >> 63 == 0 {
        bits | (1_u64 << 63)
    } else {
        !bits
    };
    let shifted = (ordered as i128 + delta as i128).clamp(1, u64::MAX as i128 - 1) as u64;
    let result_bits = if shifted >> 63 == 0 {
        !shifted
    } else {
        shifted & !(1_u64 << 63)
    };
    f64::from_bits(result_bits)
}

fn guess_bank(root: f64) -> Vec<(&'static str, f64)> {
    let mut rows = Vec::new();
    for (label, delta) in [
        ("m46", -(1_i64 << 46)),
        ("m40", -(1_i64 << 40)),
        ("m34", -(1_i64 << 34)),
        ("m28", -(1_i64 << 28)),
        ("root", 0),
        ("p28", 1_i64 << 28),
        ("p34", 1_i64 << 34),
        ("p40", 1_i64 << 40),
        ("p46", 1_i64 << 46),
    ] {
        rows.push((label, offset_ulps(root, delta)));
    }
    rows.extend([
        ("far_n075", -0.75),
        ("far_n025", -0.25),
        ("far_zero", 0.0),
        ("far_p010", 0.1),
        ("far_p050", 0.5),
        ("far_p200", 2.0),
    ]);
    rows
}

fn discovery_shapes() -> Vec<Shape> {
    let scale_base = [-1024.0, 123.25, 456.5, 789.75];
    vec![
        Shape {
            id: "d00",
            class: "linear-control",
            values: vec![-997.0, 1103.0],
            seed: 0.1,
        },
        Shape {
            id: "d01",
            class: "quadratic",
            values: vec![-1000.0, 100.0, 1150.0],
            seed: 0.1,
        },
        Shape {
            id: "d02",
            class: "cubic",
            values: vec![-777.25, 250.5, 350.75, 450.125],
            seed: 0.1,
        },
        Shape {
            id: "d03",
            class: "cubic-skew",
            values: vec![-1234.5, 50.25, 300.75, 1200.125],
            seed: 0.05,
        },
        Shape {
            id: "d04",
            class: "quartic",
            values: vec![-100.0, 10.0, 20.0, 30.0, 60.0],
            seed: 0.05,
        },
        Shape {
            id: "d05",
            class: "degree5",
            values: vec![-5000.0, 100.0, 600.0, 900.0, 1200.0, 3500.0],
            seed: 0.05,
        },
        Shape {
            id: "d06",
            class: "degree7",
            values: vec![-2048.0, 512.0, 0.5, 384.0, 640.0, 256.0, 128.0, 1024.0],
            seed: 0.05,
        },
        Shape {
            id: "d07",
            class: "trailing-sparse",
            values: vec![-1000.0, 0.0, 0.0, 0.0, 2000.0],
            seed: 0.2,
        },
        Shape {
            id: "d08",
            class: "split-sparse",
            values: vec![-1000.0, 700.0, 0.0, 0.0, 0.0, 700.0],
            seed: 0.1,
        },
        Shape {
            id: "d09",
            class: "negative-root",
            values: vec![-1000.0, 100.0, 200.0, 300.0],
            seed: -0.2,
        },
        Shape {
            id: "d10",
            class: "negative-sparse",
            values: vec![-1000.0, 0.0, 0.0, 900.0],
            seed: -0.05,
        },
        Shape {
            id: "d11",
            class: "huge-scale",
            values: vec![-1.0e150, 2.0e149, 3.0e149, 7.0e149],
            seed: 0.05,
        },
        Shape {
            id: "d12",
            class: "tiny-scale",
            values: vec![-1.0e-150, 2.5e-151, 3.25e-151, 6.75e-151],
            seed: 0.05,
        },
        Shape {
            id: "d13",
            class: "power2-scale-base",
            values: scale_base.to_vec(),
            seed: 0.1,
        },
        Shape {
            id: "d14",
            class: "power2-scale-large",
            values: scale_base.iter().map(|x| x * 2.0_f64.powi(400)).collect(),
            seed: 0.1,
        },
        Shape {
            id: "d15",
            class: "nonpower-scale",
            values: scale_base.iter().map(|x| x * 3.141592653589793).collect(),
            seed: 0.1,
        },
        Shape {
            id: "d16",
            class: "near-zero-root",
            values: vec![-1000.0, 999.999999, 0.000_011],
            seed: 1.0e-8,
        },
        Shape {
            id: "d17",
            class: "late-mass",
            values: vec![-1000.0, 1.0, 1.0, 1.0, 1.0, 1200.0],
            seed: 0.04,
        },
        Shape {
            id: "d18",
            class: "near-cancellation",
            values: vec![-1000.0, 1000.0, 1.0],
            seed: 0.001,
        },
        Shape {
            id: "d19",
            class: "degree7-geometric",
            values: vec![-1000.0, 0.001, 0.01, 0.1, 1.0, 10.0, 100.0, 1000.0],
            seed: 0.02,
        },
    ]
}

fn heldout_shapes() -> Vec<Shape> {
    let scale_base = [-8191.5, 911.25, 2222.75, 6789.125];
    vec![
        Shape {
            id: "h00",
            class: "linear-control",
            values: vec![-4093.0, 4637.0],
            seed: 0.1,
        },
        Shape {
            id: "h01",
            class: "quadratic",
            values: vec![-2500.25, 333.5, 2600.75],
            seed: 0.05,
        },
        Shape {
            id: "h02",
            class: "degree4",
            values: vec![-1500.0, 72.0, 310.0, 590.0, 880.0],
            seed: 0.05,
        },
        Shape {
            id: "h03",
            class: "degree8",
            values: vec![
                -9000.0, 125.0, 250.0, 375.0, 500.0, 625.0, 750.0, 875.0, 7000.0,
            ],
            seed: 0.01,
        },
        Shape {
            id: "h04",
            class: "negative-root",
            values: vec![-2048.0, 111.0, 222.0, 333.0, 444.0],
            seed: -0.15,
        },
        Shape {
            id: "h05",
            class: "interior-sparse",
            values: vec![-3000.0, 1200.0, 0.0, 0.0, 2100.0, 0.0, 400.0],
            seed: 0.05,
        },
        Shape {
            id: "h06",
            class: "power2-scale-base",
            values: scale_base.to_vec(),
            seed: 0.05,
        },
        Shape {
            id: "h07",
            class: "power2-scale-small",
            values: scale_base.iter().map(|x| x * 2.0_f64.powi(-400)).collect(),
            seed: 0.05,
        },
        Shape {
            id: "h08",
            class: "nonpower-scale",
            values: scale_base.iter().map(|x| x * 2.718281828459045).collect(),
            seed: 0.05,
        },
        Shape {
            id: "h09",
            class: "near-zero-root",
            values: vec![-65536.0, 65535.9375, 0.125],
            seed: 1.0e-6,
        },
        Shape {
            id: "h10",
            class: "trailing-sparse",
            values: vec![-777.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1555.0],
            seed: 0.12,
        },
        Shape {
            id: "h11",
            class: "degree9-geometric",
            values: vec![
                -5000.0, 0.005, 0.05, 0.5, 5.0, 50.0, 500.0, 1000.0, 1500.0, 3000.0,
            ],
            seed: 0.02,
        },
    ]
}

fn emit(split: &str, shapes: Vec<Shape>) {
    let mut probes = Vec::new();
    let mut meta = String::from(
        "id,split,shape,class,length,cashflow_bits,local_root_bits,guess_class,guess_bits\n",
    );
    let mut exact_rows = BTreeSet::new();

    for shape in shapes {
        let root = refine_root(&shape);
        let cashflow_bits = shape.values.iter().map(|x| hex(*x)).collect::<Vec<_>>();
        for (guess_class, guess) in guess_bank(root) {
            assert!(guess.is_finite() && guess > -1.0);
            let row_key = (cashflow_bits.clone(), guess.to_bits());
            if !exact_rows.insert(row_key) {
                continue;
            }
            let id = format!("irr-{split}-{}-{guess_class}", shape.id);
            probes.push(json!({
                "probe": {
                    "id": id,
                    "args": [cashflow_bits, hex(guess)]
                }
            }));
            meta.push_str(&format!(
                "{id},{split},{},{},{},{},{},{guess_class},{}\n",
                shape.id,
                shape.class,
                shape.values.len(),
                shape
                    .values
                    .iter()
                    .map(|x| hex(*x))
                    .collect::<Vec<_>>()
                    .join("|"),
                hex(root),
                hex(guess),
            ));
        }
    }

    let row_id = format!("irr-exact-graph-{split}-20260809");
    let doc = json!({ "function": "IRR", "row_id": row_id, "probes": probes });
    let batch_path = format!("{ROOT}/batch-irr-exact-graph-{split}-20260809.json");
    let meta_path = format!("{ROOT}/meta-irr-exact-graph-{split}-20260809.csv");
    std::fs::write(&batch_path, serde_json::to_vec(&doc).unwrap()).unwrap();
    std::fs::write(&meta_path, meta).unwrap();
    println!(
        "{split}: {} frozen answer-blind probes",
        doc["probes"].as_array().unwrap().len()
    );
    println!("batch={batch_path}");
    println!("meta={meta_path}");
}

fn main() {
    emit("discovery", discovery_shapes());
    emit("heldout", heldout_shapes());
}
