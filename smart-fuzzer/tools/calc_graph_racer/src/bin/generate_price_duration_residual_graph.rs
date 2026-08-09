//! Deterministic, answer-blind PRICE/DURATION residual-graph batteries.
//!
//! The discovery and heldout splits use disjoint bond contexts.  Every context
//! publishes an adjacent-yield triplet over a ladder of consecutively truncated
//! maturities.  PRICE additionally publishes a 2x2 coupon/redemption grid so
//! coupon-prefix and terminal-redemption effects can be cancelled after capture;
//! DURATION publishes the matching coupon and zero-coupon rows.
//!
//! This generator reads existing *input batches* only.  It never reads oracle
//! answers.  The heldout files are frozen here but must not be captured until a
//! coherent discovery survivor has been fixed.

use oxfunc_core::functions::bond_core_family::{duration_kernel, price_kernel};
use oxfunc_core::locale_format::{WorkbookDateSystem, excel_serial_from_ymd};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_OUT: &str = "../../work/w109/G6-price-duration-exact";
const DEFAULT_BANK_ROOT: &str = "../../work/w109";

#[derive(Clone, Copy)]
struct Date {
    year: i64,
    month: i64,
    day: i64,
}

#[derive(Clone, Copy)]
struct Context {
    tag: &'static str,
    settlement: Date,
    first_next: Date,
    coupon_bits: u64,
    yield_bits: u64,
    frequency: i64,
    basis: i64,
}

const DISCOVERY: [Context; 4] = [
    Context {
        tag: "d0-a360-f2",
        settlement: Date {
            year: 2027,
            month: 3,
            day: 17,
        },
        first_next: Date {
            year: 2027,
            month: 7,
            day: 11,
        },
        coupon_bits: 0x3fae_b4c2_13a5_79d1,
        yield_bits: 0x3fc9_abcd_ef01_2345,
        frequency: 2,
        basis: 2,
    },
    Context {
        tag: "d1-a365-f4",
        settlement: Date {
            year: 2028,
            month: 4,
            day: 2,
        },
        first_next: Date {
            year: 2028,
            month: 5,
            day: 19,
        },
        coupon_bits: 0x3fa4_7b19_2e31_a6c5,
        yield_bits: 0x3fb5_79bd_f246_8ace,
        frequency: 4,
        basis: 3,
    },
    Context {
        tag: "d2-a360-f1",
        settlement: Date {
            year: 2029,
            month: 4,
            day: 19,
        },
        first_next: Date {
            year: 2029,
            month: 11,
            day: 7,
        },
        coupon_bits: 0x3f98_53a1_c6d2_e47b,
        yield_bits: 0x3fa2_6bd1_359a_ce71,
        frequency: 1,
        basis: 2,
    },
    Context {
        tag: "d3-a365-f4",
        settlement: Date {
            year: 2030,
            month: 11,
            day: 2,
        },
        first_next: Date {
            year: 2030,
            month: 12,
            day: 23,
        },
        coupon_bits: 0x3fb0_5d73_a91c_e2b7,
        yield_bits: 0x3fd0_2468_ace1_3579,
        frequency: 4,
        basis: 3,
    },
];

const HELDOUT: [Context; 3] = [
    Context {
        tag: "h0-a365-f1",
        settlement: Date {
            year: 2033,
            month: 2,
            day: 14,
        },
        first_next: Date {
            year: 2033,
            month: 9,
            day: 29,
        },
        coupon_bits: 0x3fa9_73c5_e102_b4d6,
        yield_bits: 0x3fc4_1357_9bdf_0246,
        frequency: 1,
        basis: 3,
    },
    Context {
        tag: "h1-a360-f4",
        settlement: Date {
            year: 2034,
            month: 6,
            day: 8,
        },
        first_next: Date {
            year: 2034,
            month: 7,
            day: 21,
        },
        coupon_bits: 0x3f91_b6e4_2d70_a35c,
        yield_bits: 0x3fa7_eca8_6420_b975,
        frequency: 4,
        basis: 2,
    },
    Context {
        tag: "h2-a365-f2",
        settlement: Date {
            year: 2035,
            month: 10,
            day: 5,
        },
        first_next: Date {
            year: 2035,
            month: 12,
            day: 17,
        },
        coupon_bits: 0x3fb2_8f5c_1a69_d3e7,
        yield_bits: 0x3fba_d036_9cf2_58b1,
        frequency: 2,
        basis: 3,
    },
];

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
    let index = date.year * 12 + (date.month - 1) + months;
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

fn collect_json(path: &Path, excluded_root: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path == excluded_root {
                continue;
            }
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            if !name.starts_with("target") && name != "cargo-target" {
                collect_json(&path, excluded_root, files);
            }
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("json")
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("batch"))
        {
            files.push(path);
        }
    }
}

fn existing_bank(
    root: &Path,
    excluded_root: &Path,
    function: &str,
    arity: usize,
) -> BTreeSet<Vec<u64>> {
    let mut files = Vec::new();
    collect_json(root, excluded_root, &mut files);
    files.sort();
    let mut bank = BTreeSet::new();
    for path in files {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<Value>(&text) else {
            continue;
        };
        if value.get("function").and_then(Value::as_str) != Some(function) {
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
            if args.len() != arity {
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

fn push_probe(
    probes: &mut Vec<Envelope>,
    seen: &mut BTreeSet<Vec<u64>>,
    bank: &BTreeSet<Vec<u64>>,
    id: String,
    args: Vec<String>,
) {
    let bits = tuple(&args);
    assert!(!bank.contains(&bits), "existing input collision for {id}");
    assert!(seen.insert(bits), "duplicate generated input for {id}");
    probes.push(Envelope {
        probe: Probe { id, args },
    });
}

fn build_split(
    split: &str,
    contexts: &[Context],
    price_bank: &BTreeSet<Vec<u64>>,
    duration_bank: &BTreeSet<Vec<u64>>,
) -> (Batch, Batch, String, BTreeSet<Vec<u64>>, BTreeSet<Vec<u64>>) {
    let mut price = Vec::new();
    let mut duration = Vec::new();
    let mut price_seen = BTreeSet::new();
    let mut duration_seen = BTreeSet::new();
    let mut meta = String::from(
        "split,function,context,basis,frequency,n,yield_delta,mode,id,settlement_bits,maturity_bits,coupon_bits,yield_bits,redemption_bits,relation\n",
    );

    for context in contexts {
        let settlement = serial(context.settlement);
        let coupon = f64::from_bits(context.coupon_bits);
        let months_per_coupon = 12 / context.frequency;
        for n in 2_i64..=12 {
            let maturity = serial(add_months(context.first_next, (n - 1) * months_per_coupon));
            assert!(settlement < maturity);
            for yield_delta in -1_i64..=1 {
                let yield_bits = (i128::from(context.yield_bits) + i128::from(yield_delta)) as u64;
                let yld = f64::from_bits(yield_bits);
                assert!(yld.is_finite() && yld > 0.0);

                for (mode, rate, redemption) in [
                    ("c100", coupon, 100.0),
                    ("z100", 0.0, 100.0),
                    ("c1", coupon, 1.0),
                    ("z1", 0.0, 1.0),
                ] {
                    let id = format!(
                        "pdg-{split}-price-{}-n{n:02}-y{yield_delta:+}-{mode}",
                        context.tag
                    );
                    let args = vec![
                        hex(settlement),
                        hex(maturity),
                        hex(rate),
                        hex(yld),
                        hex(redemption),
                        hex(context.frequency as f64),
                        hex(context.basis as f64),
                    ];
                    push_probe(&mut price, &mut price_seen, price_bank, id.clone(), args);
                    meta.push_str(&format!(
                        "{split},PRICE,{},{},{},{n},{yield_delta},{mode},{id},{},{},{},{},{},coupon/redemption cancellation and truncated-prefix ladder\n",
                        context.tag,
                        context.basis,
                        context.frequency,
                        hex(settlement),
                        hex(maturity),
                        hex(rate),
                        hex(yld),
                        hex(redemption),
                    ));
                }

                for (mode, rate) in [("coupon", coupon), ("zero", 0.0)] {
                    let id = format!(
                        "pdg-{split}-duration-{}-n{n:02}-y{yield_delta:+}-{mode}",
                        context.tag
                    );
                    let args = vec![
                        hex(settlement),
                        hex(maturity),
                        hex(rate),
                        hex(yld),
                        hex(context.frequency as f64),
                        hex(context.basis as f64),
                    ];
                    push_probe(
                        &mut duration,
                        &mut duration_seen,
                        duration_bank,
                        id.clone(),
                        args,
                    );
                    meta.push_str(&format!(
                        "{split},DURATION,{},{},{},{n},{yield_delta},{mode},{id},{},{},{},{},{},zero-coupon control and PRICE-denominator companion\n",
                        context.tag,
                        context.basis,
                        context.frequency,
                        hex(settlement),
                        hex(maturity),
                        hex(rate),
                        hex(yld),
                        hex(100.0),
                    ));
                }
            }
        }
    }

    (
        Batch {
            function: "PRICE",
            probes: price,
        },
        Batch {
            function: "DURATION",
            probes: duration,
        },
        meta,
        price_seen,
        duration_seen,
    )
}

fn write_json(path: &Path, batch: &Batch) {
    let mut text = serde_json::to_string_pretty(batch).expect("serialize deterministic batch");
    text.push('\n');
    fs::write(path, text).unwrap_or_else(|error| panic!("write {}: {error}", path.display()));
}

fn validate_locally(batch: &Batch) {
    for envelope in &batch.probes {
        let args = envelope
            .probe
            .args
            .iter()
            .map(|arg| f64::from_bits(tuple(std::slice::from_ref(arg))[0]))
            .collect::<Vec<_>>();
        let value = match batch.function {
            "PRICE" => price_kernel(
                args[0],
                args[1],
                args[2],
                args[3],
                args[4],
                args[5],
                Some(args[6]),
            )
            .unwrap_or_else(|error| panic!("local PRICE {}: {error:?}", envelope.probe.id)),
            "DURATION" => {
                duration_kernel(args[0], args[1], args[2], args[3], args[4], Some(args[5]))
                    .unwrap_or_else(|error| {
                        panic!("local DURATION {}: {error:?}", envelope.probe.id)
                    })
            }
            function => panic!("unexpected function {function}"),
        };
        assert!(
            value.is_finite(),
            "non-finite local result for {}",
            envelope.probe.id
        );
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let out = PathBuf::from(args.get(1).map(String::as_str).unwrap_or(DEFAULT_OUT));
    let bank_root = PathBuf::from(args.get(2).map(String::as_str).unwrap_or(DEFAULT_BANK_ROOT));
    fs::create_dir_all(&out).unwrap_or_else(|error| panic!("create {}: {error}", out.display()));
    let out = fs::canonicalize(&out).expect("canonical output root");
    let bank_root = fs::canonicalize(&bank_root).expect("canonical input-bank root");

    let price_bank = existing_bank(&bank_root, &out, "PRICE", 7);
    let duration_bank = existing_bank(&bank_root, &out, "DURATION", 6);
    let (
        discovery_price,
        discovery_duration,
        discovery_meta,
        discovery_price_set,
        discovery_duration_set,
    ) = build_split("discovery", &DISCOVERY, &price_bank, &duration_bank);
    let (heldout_price, heldout_duration, heldout_meta, heldout_price_set, heldout_duration_set) =
        build_split("heldout", &HELDOUT, &price_bank, &duration_bank);

    assert!(discovery_price_set.is_disjoint(&heldout_price_set));
    assert!(discovery_duration_set.is_disjoint(&heldout_duration_set));
    assert_eq!(discovery_price.probes.len(), 528);
    assert_eq!(discovery_duration.probes.len(), 264);
    assert_eq!(heldout_price.probes.len(), 396);
    assert_eq!(heldout_duration.probes.len(), 198);
    validate_locally(&discovery_price);
    validate_locally(&discovery_duration);
    validate_locally(&heldout_price);
    validate_locally(&heldout_duration);

    write_json(
        &out.join("batch-price-residual-graph-discovery-20260809.json"),
        &discovery_price,
    );
    write_json(
        &out.join("batch-duration-residual-graph-discovery-20260809.json"),
        &discovery_duration,
    );
    fs::write(
        out.join("meta-price-duration-residual-graph-discovery-20260809.csv"),
        discovery_meta,
    )
    .expect("write discovery metadata");

    write_json(
        &out.join("batch-price-residual-graph-heldout-20260809.json"),
        &heldout_price,
    );
    write_json(
        &out.join("batch-duration-residual-graph-heldout-20260809.json"),
        &heldout_duration,
    );
    fs::write(
        out.join("meta-price-duration-residual-graph-heldout-20260809.csv"),
        heldout_meta,
    )
    .expect("write heldout metadata");

    eprintln!(
        "froze discovery PRICE={} DURATION={} and heldout PRICE={} DURATION={}; input banks PRICE={} DURATION={}",
        discovery_price.probes.len(),
        discovery_duration.probes.len(),
        heldout_price.probes.len(),
        heldout_duration.probes.len(),
        price_bank.len(),
        duration_bank.len(),
    );
}
