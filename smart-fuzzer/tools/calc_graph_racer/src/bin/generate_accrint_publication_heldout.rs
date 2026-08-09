//! Generate a frozen, answer-free W109 G6-02 ACCRINT publication held-out.
//!
//! The promoted candidate differs from production at exactly one site: the
//! final multiplication of the stored coupon and stored accrual fraction is
//! x87 PC64 followed by a binary64 store (`RN53(RN64(coupon*a))`).  This
//! generator searches deterministic valid inputs for that rare disagreement,
//! balances across calendar paths, and adds collapse controls.  It writes no
//! Excel answers and therefore can be frozen before the live capture.

use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::bond_core_family::accrint_kernel;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::PathBuf;

const DISAGREEMENT_TARGET: usize = 300;
const CONTROLS_TARGET: usize = 150;
const PER_BUCKET_CAP: usize = 10;
const SEARCH_LIMIT: usize = 4_000_000;

#[derive(Serialize)]
struct ProbeBatch {
    function: &'static str,
    probes: Vec<ProbeEnvelope>,
}

#[derive(Serialize)]
struct ProbeEnvelope {
    probe: Probe,
}

#[derive(Clone, Serialize)]
struct Probe {
    id: String,
    args: Vec<String>,
}

#[derive(Clone)]
struct Row {
    probe: Probe,
    class: &'static str,
    regime: &'static str,
    basis: i32,
    frequency: i32,
    calc_method: bool,
    plain_bits: u64,
    candidate_bits: u64,
}

#[derive(Clone, Copy)]
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0
    }

    fn range(&mut self, lo: u64, hi: u64) -> u64 {
        lo + self.next() % (hi - lo)
    }

    fn unit(&mut self) -> f64 {
        let mantissa = self.next() >> 11;
        mantissa as f64 * (1.0 / ((1_u64 << 53) as f64))
    }
}

fn hx(x: f64) -> String {
    format!("0x{:016x}", x.to_bits())
}

fn evaluate(args: &[f64; 8]) -> Option<(u64, u64)> {
    let accrual = accrint_kernel(
        args[0],
        args[1],
        args[2],
        1.0,
        Some(args[5]),
        args[5],
        Some(args[6]),
        Some(args[7] != 0.0),
    )
    .ok()?;
    let coupon = args[4] * args[3] / args[5];
    let plain = coupon * accrual;
    let candidate = rx::x87_mul(coupon, accrual);
    Some((plain.to_bits(), candidate.to_bits()))
}

fn make_args(rng: &mut Rng, index: usize) -> ([f64; 8], &'static str) {
    let issue = rng.range(42_000, 47_000) as f64;
    let first = issue + rng.range(30, 900) as f64;
    let post_first = (rng.next() & 1) != 0;
    let settlement = if post_first {
        first + rng.range(1, 1_600) as f64
    } else {
        issue + rng.range(1, (first - issue) as u64 + 1) as f64
    };
    let frequency = [1.0, 2.0, 4.0][rng.range(0, 3) as usize];
    let basis = rng.range(0, 5) as f64;
    let calc = if rng.next() & 1 == 0 { 0.0 } else { 1.0 };
    let par = match index % 7 {
        0 => 997.5,
        1 => 1_000.0,
        2 => 100.0,
        _ => 1.0 + rng.unit() * 9_999.0,
    };
    let rate = match index % 11 {
        0 => f64::from_bits(
            0.0615_f64
                .to_bits()
                .wrapping_add((rng.range(0, 129) as i64 - 64) as u64),
        ),
        1 => 0.05,
        2 => 0.037,
        3 => 0.125,
        _ => 1.0e-8 + rng.unit() * 0.35,
    };
    (
        [issue, first, settlement, rate, par, frequency, basis, calc],
        if post_first {
            "post_first"
        } else {
            "pre_first"
        },
    )
}

fn row_id(prefix: &str, n: usize) -> String {
    format!("accrint-pub-ho-{prefix}-{n:04}")
}

fn main() {
    let out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../work/w109/G6-b2b3");
    std::fs::create_dir_all(&out_dir).expect("create work dir");
    let batch_path = out_dir.join("batch-accrint-publication-heldout-20260809.json");
    let meta_path = out_dir.join("meta-accrint-publication-heldout-20260809.csv");

    let mut rng = Rng(0xa663_1a7e_d35c_2026);
    let mut rows = Vec::<Row>::new();
    let mut bucket_counts = BTreeMap::<String, usize>::new();
    let mut seen = BTreeSet::<Vec<u64>>::new();
    let mut disagreement_count = 0usize;
    let mut controls_seen = 0usize;

    for index in 0..SEARCH_LIMIT {
        if disagreement_count >= DISAGREEMENT_TARGET && controls_seen >= CONTROLS_TARGET {
            break;
        }
        let (args, regime) = make_args(&mut rng, index);
        let Some((plain_bits, candidate_bits)) = evaluate(&args) else {
            continue;
        };
        let key: Vec<u64> = args.iter().map(|v| v.to_bits()).collect();
        if !seen.insert(key) {
            continue;
        }
        let disagreement = plain_bits != candidate_bits;
        let basis = args[6] as i32;
        let frequency = args[5] as i32;
        let calc_method = args[7] != 0.0;
        let bucket = format!("{regime}-b{basis}-f{frequency}-c{}", calc_method as u8);
        let class = if disagreement {
            let count = bucket_counts.entry(bucket).or_default();
            if *count >= PER_BUCKET_CAP || disagreement_count >= DISAGREEMENT_TARGET {
                continue;
            }
            *count += 1;
            disagreement_count += 1;
            "disagreement"
        } else {
            // Deterministic sparse controls, separate from the searched
            // disagreement surface.
            if controls_seen >= CONTROLS_TARGET || index % 1_997 != 0 {
                continue;
            }
            controls_seen += 1;
            "collapse_control"
        };
        let ordinal = rows.len();
        rows.push(Row {
            probe: Probe {
                id: row_id(if disagreement { "d" } else { "c" }, ordinal),
                args: args.iter().map(|&v| hx(v)).collect(),
            },
            class,
            regime,
            basis,
            frequency,
            calc_method,
            plain_bits,
            candidate_bits,
        });
    }

    let disagreements = rows.iter().filter(|r| r.class == "disagreement").count();
    let controls = rows.len() - disagreements;
    assert_eq!(
        disagreements, DISAGREEMENT_TARGET,
        "search did not fill disagreement set"
    );
    assert_eq!(controls, CONTROLS_TARGET, "search did not fill controls");
    assert!(bucket_counts.len() >= 30, "insufficient path diversity");

    let batch = ProbeBatch {
        function: "ACCRINT",
        probes: rows
            .iter()
            .cloned()
            .map(|r| ProbeEnvelope { probe: r.probe })
            .collect(),
    };
    std::fs::write(
        &batch_path,
        serde_json::to_vec_pretty(&batch).expect("serialize batch"),
    )
    .expect("write batch");

    let mut meta =
        String::from("id,class,regime,basis,frequency,calc_method,plain_bits,candidate_bits\n");
    for row in &rows {
        writeln!(
            meta,
            "{},{},{},{},{},{},0x{:016x},0x{:016x}",
            row.probe.id,
            row.class,
            row.regime,
            row.basis,
            row.frequency,
            row.calc_method,
            row.plain_bits,
            row.candidate_bits
        )
        .unwrap();
    }
    std::fs::write(&meta_path, meta).expect("write meta");
    println!(
        "wrote {} rows ({disagreements} disagreements + {controls} controls; {} buckets)\n{}\n{}",
        rows.len(),
        bucket_counts.len(),
        batch_path.display(),
        meta_path.display()
    );
}
