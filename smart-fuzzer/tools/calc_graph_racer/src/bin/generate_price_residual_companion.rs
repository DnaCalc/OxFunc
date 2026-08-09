//! Answer-blind PRICE residual companion for the six discovery misses.
//!
//! This generator targets the already-published d1 Actual/365 frequency-4
//! base/exponent pairs at n=7 and n=8.  It perturbs coupon and redemption bits
//! without changing those pairs, then adds n=6/n=9 truncated-ladder controls
//! and exact coupon-cash controls.  It reads input batches only, excludes every
//! heldout path, and never reads oracle answers.

use oxfunc_core::functions::bond_core_family::price_kernel;
use oxfunc_core::locale_format::{WorkbookDateSystem, excel_serial_from_ymd};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_OUT: &str = "../../work/w109/G6-price-duration-exact";
const DEFAULT_BANK_ROOT: &str = "../../work/w109";
const OUTPUT_NAME: &str = "batch-price-residual-companion-discovery-20260809.json";
const META_NAME: &str = "meta-price-residual-companion-discovery-20260809.csv";
const COUPON_BITS: u64 = 0x3fa4_7b19_2e31_a6c5;
const YIELD_BITS: u64 = 0x3fb5_79bd_f246_8ace;

#[derive(Clone, Copy)]
struct Date {
    year: i64,
    month: i64,
    day: i64,
}

#[derive(Serialize)]
struct Batch {
    function: &'static str,
    probes: Vec<Envelope>,
}

#[derive(Serialize)]
struct Envelope {
    probe: Probe,
}

#[derive(Serialize)]
struct Probe {
    id: String,
    args: Vec<String>,
}

fn serial(date: Date) -> f64 {
    excel_serial_from_ymd(
        WorkbookDateSystem::System1900,
        date.year,
        date.month,
        date.day,
    )
    .expect("valid deterministic date")
}

fn add_months(date: Date, months: i64) -> Date {
    let index = date.year * 12 + date.month - 1 + months;
    Date {
        year: index.div_euclid(12),
        month: index.rem_euclid(12) + 1,
        day: date.day,
    }
}

fn hex(value: f64) -> String {
    format!("0x{:016x}", value.to_bits())
}

fn tuple(args: &[String]) -> Vec<u64> {
    args.iter()
        .map(|arg| {
            u64::from_str_radix(arg.strip_prefix("0x").expect("hex argument"), 16)
                .expect("argument bits")
        })
        .collect()
}

fn collect_batches(path: &Path, excluded_output: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            if !name.starts_with("target") && name != "cargo-target" {
                collect_batches(&path, excluded_output, files);
            }
            continue;
        }
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if path == excluded_output
            || name.contains("heldout")
            || !name.starts_with("batch")
            || path.extension().and_then(|ext| ext.to_str()) != Some("json")
        {
            continue;
        }
        files.push(path);
    }
}

fn existing_price_bank(root: &Path, excluded_output: &Path) -> BTreeSet<Vec<u64>> {
    let mut files = Vec::new();
    collect_batches(root, excluded_output, &mut files);
    files.sort();
    let mut bank = BTreeSet::new();
    for path in files {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<Value>(&text) else {
            continue;
        };
        if value.get("function").and_then(Value::as_str) != Some("PRICE") {
            continue;
        }
        let Some(probes) = value.get("probes").and_then(Value::as_array) else {
            continue;
        };
        for envelope in probes {
            let Some(args) = envelope
                .get("probe")
                .and_then(|probe| probe.get("args"))
                .and_then(Value::as_array)
            else {
                continue;
            };
            if args.len() != 7 {
                continue;
            }
            let parsed = args
                .iter()
                .map(|arg| {
                    arg.as_str()
                        .and_then(|text| u64::from_str_radix(text.strip_prefix("0x")?, 16).ok())
                })
                .collect::<Option<Vec<_>>>();
            if let Some(parsed) = parsed {
                bank.insert(parsed);
            }
        }
    }
    bank
}

fn push(
    probes: &mut Vec<Envelope>,
    meta: &mut String,
    seen: &mut BTreeSet<Vec<u64>>,
    bank: &BTreeSet<Vec<u64>>,
    n: i64,
    yield_delta: i64,
    variant: &str,
    rate: f64,
    redemption: f64,
) {
    let settlement = serial(Date {
        year: 2028,
        month: 4,
        day: 2,
    });
    let maturity = serial(add_months(
        Date {
            year: 2028,
            month: 5,
            day: 19,
        },
        (n - 1) * 3,
    ));
    let yld = f64::from_bits((i128::from(YIELD_BITS) + i128::from(yield_delta)) as u64);
    let id = format!("pdgc-price-d1-a365-f4-n{n:02}-y{yield_delta:+}-{variant}");
    let args = vec![
        hex(settlement),
        hex(maturity),
        hex(rate),
        hex(yld),
        hex(redemption),
        hex(4.0),
        hex(3.0),
    ];
    let key = tuple(&args);
    assert!(
        !bank.contains(&key),
        "existing non-heldout input collision for {id}"
    );
    assert!(seen.insert(key), "duplicate generated input for {id}");
    let local = price_kernel(settlement, maturity, rate, yld, redemption, 4.0, Some(3.0))
        .unwrap_or_else(|error| panic!("local PRICE validation {id}: {error:?}"));
    assert!(local.is_finite(), "non-finite local PRICE for {id}");
    meta.push_str(&format!(
        "discovery,PRICE,d1-a365-f4,{n},{yield_delta},{variant},{id},{},{},{},{},{},coupon/redemption boundary around exact miss base-exponent pair\n",
        hex(settlement),
        hex(maturity),
        hex(rate),
        hex(yld),
        hex(redemption),
    ));
    probes.push(Envelope {
        probe: Probe { id, args },
    });
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let out = PathBuf::from(args.get(1).map(String::as_str).unwrap_or(DEFAULT_OUT));
    let bank_root = PathBuf::from(args.get(2).map(String::as_str).unwrap_or(DEFAULT_BANK_ROOT));
    fs::create_dir_all(&out).expect("create output directory");
    let output_path = out.join(OUTPUT_NAME);
    let bank = existing_price_bank(&bank_root, &output_path);
    let mut probes = Vec::new();
    let mut seen = BTreeSet::new();
    let mut meta = String::from(
        "split,function,context,n,yield_delta,variant,id,settlement_bits,maturity_bits,coupon_bits,yield_bits,redemption_bits,relation\n",
    );
    let center_coupon = f64::from_bits(COUPON_BITS);
    let red_lo = f64::from_bits(1.0_f64.to_bits() - 1);
    let red_hi = f64::from_bits(1.0_f64.to_bits() + 1);
    let tiny_redemption = f64::from_bits(0x3eb0_0000_0000_0000);
    let boundary_variants = [
        ("rate-1-red1", f64::from_bits(COUPON_BITS - 1), 1.0),
        ("rate+1-red1", f64::from_bits(COUPON_BITS + 1), 1.0),
        ("rate0-red-1", center_coupon, red_lo),
        ("rate0-red+1", center_coupon, red_hi),
        ("rate0-redtiny", center_coupon, tiny_redemption),
        ("rate0-red2", center_coupon, 2.0),
        ("zero-red-1", 0.0, red_lo),
        ("zero-red+1", 0.0, red_hi),
    ];

    for n in [7_i64, 8] {
        for yield_delta in -1_i64..=1 {
            for (variant, rate, redemption) in boundary_variants {
                push(
                    &mut probes,
                    &mut meta,
                    &mut seen,
                    &bank,
                    n,
                    yield_delta,
                    variant,
                    rate,
                    redemption,
                );
            }
        }
    }
    for n in [6_i64, 9] {
        for (variant, rate, redemption) in boundary_variants {
            push(
                &mut probes,
                &mut meta,
                &mut seen,
                &bank,
                n,
                0,
                variant,
                rate,
                redemption,
            );
        }
    }
    for n in [7_i64, 8] {
        for (variant, rate, redemption) in [
            ("cash1-red1", 0.04, 1.0),
            ("cash1-redtiny", 0.04, tiny_redemption),
            ("cash0p5-red1", 0.02, 1.0),
            ("cash0p5-redtiny", 0.02, tiny_redemption),
        ] {
            push(
                &mut probes,
                &mut meta,
                &mut seen,
                &bank,
                n,
                0,
                variant,
                rate,
                redemption,
            );
        }
    }
    assert_eq!(probes.len(), 72);
    let batch = Batch {
        function: "PRICE",
        probes,
    };
    let mut json = serde_json::to_string_pretty(&batch).expect("serialize companion batch");
    json.push('\n');
    fs::write(&output_path, json).expect("write companion batch");
    fs::write(out.join(META_NAME), meta).expect("write companion metadata");
    println!(
        "wrote {} answer-blind PRICE discovery companion probes; non-heldout PRICE bank={} unique",
        batch.probes.len(),
        bank.len()
    );
}
