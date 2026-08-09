//! Generate the oracle-blind W109 BESSELJ internal-trig held-out battery.
//!
//! The production asymptotic J0 seed uses platform `cos`.  The surviving
//! candidate changes only that one call to Excel's already-identified `fFCOS`
//! chain.  Probe selection uses no Excel answers: it combines fixed structural
//! coverage, adjacent-input ladders around the four current discriminators, and
//! deterministic model-disagreement search.

use oxfunc_core::excel_numeric::research as rx;
use serde_json::json;
use std::collections::BTreeSet;
use std::path::PathBuf;

const ACC: f64 = 40.0;
const BIGNO: f64 = 1.0e10;
const BIGNI: f64 = 1.0e-10;
const NR_2_OVER_PI: f64 = 0.636_619_772;

#[derive(Clone)]
struct Row {
    x: f64,
    order: f64,
    class: &'static str,
    platform_bits: u64,
    candidate_bits: u64,
}

fn hex(value: f64) -> String {
    format!("0x{:016x}", value.to_bits())
}

fn horner(y: f64, coeffs: &[f64]) -> f64 {
    coeffs
        .iter()
        .rev()
        .fold(0.0, |acc, coefficient| acc * y + coefficient)
}

/// Excel `COS`, copied from the established production calculation graph so
/// this oracle-blind generator can evaluate the candidate without worksheet
/// answers.
fn excel_cos_model(x: f64) -> f64 {
    if !x.is_finite() {
        return f64::NAN;
    }
    if x.abs() < f64::from_bits(0x3e50_0000_0000_0000) {
        return 1.0;
    }

    let cw = rx::CW_PC64_RN;
    let minus_one = rx::ext_chs(&rx::ext_one(), cw);
    let pi_half = rx::ext_scale(&rx::ext_pi(), &minus_one, cw);
    let xa = rx::ext_abs(&rx::ext_from_f64(x), cw);
    let (residue, quotient) = rx::ext_prem1_quo(&xa, &pi_half, cw);
    let value = match quotient & 3 {
        0 => rx::ext_cos(&residue, cw),
        1 => rx::ext_chs(&rx::ext_sin(&residue, cw), cw),
        2 => rx::ext_chs(&rx::ext_cos(&residue, cw), cw),
        _ => rx::ext_sin(&residue, cw),
    };
    rx::ext_to_f64(&value, cw)
}

fn bessj0_model(x: f64, candidate_cos: bool) -> f64 {
    let ax = x.abs();
    if ax < 8.0 {
        let y = x * x;
        let numerator = horner(
            y,
            &[
                57_568_490_574.0,
                -13_362_590_354.0,
                651_619_640.7,
                -11_214_424.18,
                77_392.330_17,
                -184.905_245_6,
            ],
        );
        let denominator = horner(
            y,
            &[
                57_568_490_411.0,
                1_029_532_985.0,
                9_494_680.718,
                59_272.648_53,
                267.853_271_2,
                1.0,
            ],
        );
        return numerator / denominator;
    }

    let z = 8.0 / ax;
    let y = z * z;
    let reduced = ax - 0.785_398_164;
    let p = horner(
        y,
        &[
            1.0,
            -0.001_098_628_267,
            0.000_027_345_104_07,
            -0.000_002_073_370_639,
            0.000_000_209_388_721_1,
        ],
    );
    let q = horner(
        y,
        &[
            -0.015_624_999_95,
            0.000_143_048_876_5,
            -0.000_006_911_147_651,
            0.000_000_762_109_516_1,
            -0.000_000_093_493_515_2,
        ],
    );
    let cosine = if candidate_cos {
        excel_cos_model(reduced)
    } else {
        reduced.cos()
    };
    (NR_2_OVER_PI / ax).sqrt() * (cosine * p - z * reduced.sin() * q)
}

fn bessj1_platform(x: f64) -> f64 {
    let ax = x.abs();
    let value = if ax < 8.0 {
        let y = x * x;
        let numerator = x * horner(
            y,
            &[
                72_362_614_232.0,
                -7_895_059_235.0,
                242_396_853.1,
                -2_972_611.439,
                15_704.482_60,
                -30.160_366_06,
            ],
        );
        let denominator = horner(
            y,
            &[
                144_725_228_442.0,
                2_300_535_178.0,
                18_583_304.74,
                99_447.433_94,
                376.999_139_7,
                1.0,
            ],
        );
        numerator / denominator
    } else {
        let z = 8.0 / ax;
        let y = z * z;
        let reduced = ax - 2.356_194_491;
        let p = horner(
            y,
            &[
                1.0,
                0.001_831_05,
                -0.000_035_163_964_96,
                -0.000_035_163_964_96,
                0.000_002_457_520_174,
                -0.000_000_240_337_019,
            ],
        );
        let q = horner(
            y,
            &[
                0.046_874_999_95,
                -0.000_200_269_087_3,
                0.000_008_449_199_096,
                -0.000_000_882_289_87,
                0.000_000_105_787_412,
            ],
        );
        (NR_2_OVER_PI / ax).sqrt() * (reduced.cos() * p - z * reduced.sin() * q)
    };
    if x < 0.0 { -value } else { value }
}

fn besselj_model(x: f64, order: f64, candidate_cos: bool) -> f64 {
    let n = order.trunc() as i32;
    if n == 0 {
        return bessj0_model(x, candidate_cos);
    }
    if n == 1 {
        return bessj1_platform(x);
    }
    if x == 0.0 {
        return 0.0;
    }

    let ax = x.abs();
    let mut answer;
    if ax > f64::from(n) {
        let tox = 2.0 / ax;
        let mut previous = bessj0_model(ax, candidate_cos);
        let mut current = bessj1_platform(ax);
        for j in 1..n {
            let next = f64::from(j) * tox * current - previous;
            previous = current;
            current = next;
        }
        answer = current;
    } else {
        let tox = 2.0 / ax;
        let m = 2 * ((n + (ACC * f64::from(n)).sqrt() as i32) / 2);
        let mut add_to_sum = false;
        let mut sum = 0.0;
        let mut next = 0.0;
        let mut current = 1.0;
        answer = 0.0;
        for j in (1..=m).rev() {
            let previous = f64::from(j) * tox * current - next;
            next = current;
            current = previous;
            if current.abs() > BIGNO {
                current *= BIGNI;
                next *= BIGNI;
                answer *= BIGNI;
                sum *= BIGNI;
            }
            if add_to_sum {
                sum += current;
            }
            add_to_sum = !add_to_sum;
            if j == n {
                answer = next;
            }
        }
        sum = 2.0 * sum - current;
        answer /= sum;
    }
    if x < 0.0 && n % 2 == 1 {
        answer = -answer;
    }
    answer
}

fn push_row(
    rows: &mut Vec<Row>,
    seen: &mut BTreeSet<(u64, u64)>,
    x: f64,
    order: f64,
    class: &'static str,
) {
    if !x.is_finite() || x < 8.0 || order < 0.0 || order.fract() != 0.0 {
        return;
    }
    if !seen.insert((x.to_bits(), order.to_bits())) {
        return;
    }
    let platform = besselj_model(x, order, false);
    let candidate = besselj_model(x, order, true);
    if platform.is_finite() && candidate.is_finite() {
        rows.push(Row {
            x,
            order,
            class,
            platform_bits: platform.to_bits(),
            candidate_bits: candidate.to_bits(),
        });
    }
}

fn adjacent(value: f64, delta: i64) -> f64 {
    f64::from_bits((value.to_bits() as i64 + delta) as u64)
}

fn xorshift(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}

fn main() {
    let mut rows = Vec::new();
    let mut seen = BTreeSet::new();

    // Known discriminators are anchors, not held-out evidence.  Keeping them in
    // the same batch makes every replay prove that the lane still separates.
    for &(x, order) in &[(50.0, 0.0), (150.0, 0.0), (50.0, 2.0), (150.0, 2.0)] {
        push_row(&mut rows, &mut seen, x, order, "anchor");
    }

    // Broad deterministic affected-branch coverage.  Direct J1 and a few
    // downward-recurrence rows are deliberate collapse controls.
    let broad_x = [
        8.0,
        8.25,
        8.75,
        9.0,
        9.5,
        10.0,
        12.0,
        13.0,
        16.0,
        20.0,
        25.0,
        31.0,
        32.0,
        40.0,
        48.0,
        64.0,
        75.0,
        96.0,
        100.0,
        127.0,
        128.0,
        192.0,
        200.0,
        255.0,
        256.0,
        400.0,
        512.0,
        1_024.0,
        4_096.0,
        65_536.0,
        1_000_000.0,
    ];
    let broad_orders = [0.0, 1.0, 2.0, 3.0, 5.0, 7.0, 10.0, 16.0, 31.0, 63.0];
    for x in broad_x {
        for order in broad_orders {
            let class = if order >= x {
                "downward-control"
            } else if order == 1.0 {
                "j1-control"
            } else {
                "structured"
            };
            push_row(&mut rows, &mut seen, x, order, class);
        }
    }

    // Adjacent-input ladders around both discovered cosine-sensitive x values.
    for center in [50.0, 150.0] {
        for delta in [-64_i64, -16, -4, -2, -1, 0, 1, 2, 4, 16, 64] {
            let x = adjacent(center, delta);
            for order in [0.0, 2.0, 3.0, 5.0, 10.0, 16.0] {
                if order < x {
                    push_row(&mut rows, &mut seen, x, order, "adjacent-ladder");
                }
            }
        }
    }

    // Deterministic active selection: retain candidate-disagreement rows and a
    // smaller collapse-control sample.  No oracle result participates.
    let mut seed = 0xb355_e1c0_2026_0809_u64;
    let search_orders = [0.0, 2.0, 3.0, 5.0, 7.0, 10.0, 16.0, 31.0, 63.0];
    let mut disagreements = 0usize;
    let mut controls = 0usize;
    for _ in 0..2_000_000 {
        let z = xorshift(&mut seed);
        let exponent = 3 + ((z >> 7) % 17) as i32;
        let fraction = ((z >> 24) & 0x00ff_ffff) as f64 / 16_777_216.0;
        let x = 2.0_f64.powi(exponent) * (1.0 + fraction);
        let order = search_orders[((z >> 3) as usize) % search_orders.len()];
        if order >= x {
            continue;
        }
        let platform = besselj_model(x, order, false).to_bits();
        let candidate = besselj_model(x, order, true).to_bits();
        if platform != candidate && disagreements < 256 {
            let before = rows.len();
            push_row(&mut rows, &mut seen, x, order, "model-disagreement");
            disagreements += usize::from(rows.len() != before);
        } else if platform == candidate && controls < 96 {
            let before = rows.len();
            push_row(&mut rows, &mut seen, x, order, "model-collapse-control");
            controls += usize::from(rows.len() != before);
        }
        if disagreements == 256 && controls == 96 {
            break;
        }
    }

    let probes: Vec<_> = rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            json!({
                "probe": {
                    "id": format!("besj-ho-{index:04}"),
                    "args": [hex(row.x), hex(row.order)]
                }
            })
        })
        .collect();
    let batch = json!({
        "function": "BESSELJ",
        "row_id": "besselj-internal-trig-heldout-20260809",
        "probes": probes
    });

    let mut meta =
        String::from("id,class,x_bits,order_bits,platform_bits,j0_excel_cos_bits,discriminates\n");
    for (index, row) in rows.iter().enumerate() {
        meta.push_str(&format!(
            "besj-ho-{index:04},{},{},{},0x{:016x},0x{:016x},{}\n",
            row.class,
            hex(row.x),
            hex(row.order),
            row.platform_bits,
            row.candidate_bits,
            row.platform_bits != row.candidate_bits
        ));
    }

    let class_counts = rows
        .iter()
        .fold(std::collections::BTreeMap::new(), |mut map, row| {
            *map.entry(row.class).or_insert(0usize) += 1;
            map
        });
    let manifest = json!({
        "schema_version": "oxfunc.w109.besselj_internal_trig_generator.v0",
        "generated_date": "2026-08-09",
        "oracle_blind": true,
        "seed_hex": "0xb355e1c020260809",
        "selection": "anchors + structural grid + adjacent ladders + deterministic model-disagreement search",
        "candidate": "bessj0 x>=8: replace platform cos(xx) with established Excel fFCOS chain; retain platform sin and J1",
        "row_count": rows.len(),
        "model_disagreement_count": rows.iter().filter(|row| row.platform_bits != row.candidate_bits).count(),
        "class_counts": class_counts
    });

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../work/w109/G4-besselj");
    std::fs::create_dir_all(&root).expect("create G4-besselj work directory");
    let batch_path = root.join("batch-besselj-internal-trig-heldout-20260809.json");
    let meta_path = root.join("batch-besselj-internal-trig-heldout-20260809-meta.csv");
    let manifest_path = root.join("batch-besselj-internal-trig-heldout-20260809-manifest.json");
    std::fs::write(&batch_path, serde_json::to_string_pretty(&batch).unwrap())
        .expect("write BESSELJ batch");
    std::fs::write(&meta_path, meta).expect("write BESSELJ metadata");
    std::fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .expect("write BESSELJ manifest");

    println!("wrote BESSELJ held-out: {} rows", rows.len());
    println!(
        "candidate disagreements: {}",
        rows.iter()
            .filter(|row| row.platform_bits != row.candidate_bits)
            .count()
    );
    println!("batch: {}", batch_path.display());
    println!("meta: {}", meta_path.display());
    println!("manifest: {}", manifest_path.display());
}
