//! Generate the answer-blind W109 G6-07 CUMPRINC exact-graph gate.
//!
//! The batch is deliberately paired with PMT publications for the same loan
//! inputs.  That makes the upstream stored payment an observed nuisance value
//! rather than an assumption while the CUMPRINC rows discriminate recurrence,
//! boundary, per-period, and fold graphs.  Within each query, four adjacent-PV
//! rows constrain the hidden linear coefficient; exact x1/2 and x2 rows are
//! scale metamers.  Nested prefixes, disjoint partitions, and singleton rows
//! expose the internal addends without relying on standalone PPMT.
//!
//! Generation never reads an oracle result.  Frozen evidence is consulted only
//! for input tuples, and a collision aborts instead of silently changing this
//! gate.  Thus re-running this binary either emits byte-identical files or
//! fails loudly because the frozen-bank contract changed.

use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const ROW_ROOT: &str = "../../work/w109/G6-cumprinc";
const CUM_BATCH_NAME: &str = "batch-cumprinc-exact-discriminator-20260809.json";
const PMT_BATCH_NAME: &str = "batch-pmt-cumprinc-companion-20260809.json";
const META_NAME: &str = "meta-cumprinc-exact-discriminator-20260809.csv";

const W108_INPUTS: &str = "../../runs/w108-b2-financial/excel_out.jsonl";
const OLD_CUM_BATCH: &str = "../../work/w109/G6-solvers/batch-cum-cumprinc.json";
const OLD_PMT_BATCH: &str = "../../work/w109/G6-solvers/batch-cum-pmt.json";
const CATALOG_CASES: &str = "../../runs/catalog-row-recon-20260710-004/cases/cases.jsonl";

const CONTEXT_COUNT: usize = 5;
const TIMINGS: [i32; 2] = [0, 1];
const PV_VARIANTS: usize = 6;

#[derive(Serialize)]
struct Batch {
    function: &'static str,
    row_id: &'static str,
    probes: Vec<ProbeEnvelope>,
}

#[derive(Serialize)]
struct ProbeEnvelope {
    probe: Probe,
}

#[derive(Serialize)]
struct Probe {
    id: String,
    args: Vec<String>,
}

#[derive(Clone, Copy)]
struct Context {
    rate: f64,
    nper: i32,
    pv: f64,
}

#[derive(Clone, Copy)]
struct Shape {
    name: &'static str,
    start: i32,
    end: i32,
    relation: &'static str,
}

fn hex(value: f64) -> String {
    format!("0x{:016x}", value.to_bits())
}

fn parse_hex(text: &str) -> Option<u64> {
    u64::from_str_radix(text.strip_prefix("0x")?, 16).ok()
}

fn xorshift(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}

/// Construct a positive normal with a full deterministic mantissa in
/// `[2^exponent, 2^(exponent+1))`.
fn random_normal(seed: &mut u64, exponent: i32) -> f64 {
    assert!((-1022..=1023).contains(&exponent));
    let fraction = xorshift(seed) & ((1_u64 << 52) - 1);
    f64::from_bits((((exponent + 1023) as u64) << 52) | fraction)
}

fn contexts() -> [Context; CONTEXT_COUNT] {
    let mut seed = 0x6375_6d70_7269_6e63_u64;
    let nper = [17, 31, 67, 127, 257];
    let rate_exponent = [-4, -6, -8, -10, -7];
    let pv_exponent = [10, 13, 16, 8, 12];
    std::array::from_fn(|index| Context {
        rate: random_normal(&mut seed, rate_exponent[index]),
        nper: nper[index],
        pv: random_normal(&mut seed, pv_exponent[index]),
    })
}

fn shapes(nper: i32) -> [Shape; 9] {
    let early = nper / 4;
    let middle = (nper * 3) / 5;
    assert!(1 < early && early < middle && middle < nper);
    [
        Shape {
            name: "full",
            start: 1,
            end: nper,
            relation: "full=prefix_middle+suffix_middle",
        },
        Shape {
            name: "prefix_early",
            start: 1,
            end: early,
            relation: "prefix_middle=prefix_early+interior",
        },
        Shape {
            name: "prefix_middle",
            start: 1,
            end: middle,
            relation: "full=prefix_middle+suffix_middle",
        },
        Shape {
            name: "suffix_middle",
            start: middle + 1,
            end: nper,
            relation: "full=prefix_middle+suffix_middle",
        },
        Shape {
            name: "interior",
            start: early + 1,
            end: middle,
            relation: "prefix_middle=prefix_early+interior",
        },
        Shape {
            name: "singleton_first",
            start: 1,
            end: 1,
            relation: "geometric_singleton_anchor",
        },
        Shape {
            name: "singleton_early",
            start: early,
            end: early,
            relation: "geometric_singleton_early",
        },
        Shape {
            name: "singleton_middle",
            start: middle,
            end: middle,
            relation: "geometric_singleton_middle",
        },
        Shape {
            name: "singleton_last",
            start: nper,
            end: nper,
            relation: "geometric_singleton_terminal",
        },
    ]
}

fn pv_ladder(base: f64) -> [f64; PV_VARIANTS] {
    let bits = base.to_bits();
    let values = [
        base,
        f64::from_bits(bits + 1),
        f64::from_bits(bits + 2),
        f64::from_bits(bits + 3),
        base * 0.5,
        base * 2.0,
    ];
    assert_eq!(values[4].to_bits() + (1_u64 << 52), bits);
    assert_eq!(bits + (1_u64 << 52), values[5].to_bits());
    values
}

fn bits_from_probe(value: &Value) -> Option<Vec<u64>> {
    value["probe"]["args"]
        .as_array()?
        .iter()
        .map(|arg| parse_hex(arg.as_str()?))
        .collect()
}

fn bank_from_batch(path: &Path, arity: usize, bank: &mut BTreeSet<Vec<u64>>) {
    let Ok(text) = std::fs::read_to_string(path) else {
        return;
    };
    let value: Value = serde_json::from_str(&text).expect("parse frozen probe batch");
    for probe in value["probes"].as_array().into_iter().flatten() {
        if let Some(bits) = bits_from_probe(probe) {
            if bits.len() == arity {
                bank.insert(bits);
            }
        }
    }
}

fn bank_from_w108(
    path: &Path,
    cum_bank: &mut BTreeSet<Vec<u64>>,
    pmt_bank: &mut BTreeSet<Vec<u64>>,
) {
    let text = std::fs::read_to_string(path).expect("read frozen W108 financial inputs");
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let value: Value = serde_json::from_str(line).expect("parse frozen W108 row");
        let Some(function) = value["fn"].as_str() else {
            continue;
        };
        let Some(args) = value["arg_bits"].as_array() else {
            continue;
        };
        let Some(bits) = args
            .iter()
            .map(|arg| parse_hex(arg.as_str()?))
            .collect::<Option<Vec<_>>>()
        else {
            continue;
        };
        match function {
            "CUMPRINC" if bits.len() == 6 => {
                cum_bank.insert(bits);
            }
            "PMT" if bits.len() == 5 => {
                pmt_bank.insert(bits);
            }
            _ => {}
        }
    }
}

fn bank_from_catalog(path: &Path, bank: &mut BTreeSet<Vec<u64>>) {
    let text = std::fs::read_to_string(path).expect("read frozen catalog cases");
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let value: Value = serde_json::from_str(line).expect("parse frozen catalog case");
        if value["function_id"].as_str() != Some("FUNC.CUMPRINC") {
            continue;
        }
        let bits = value["args"]
            .as_array()
            .expect("catalog args")
            .iter()
            .map(|arg| {
                arg["value"]
                    .as_f64()
                    .expect("numeric catalog arg")
                    .to_bits()
            })
            .collect::<Vec<_>>();
        if bits.len() == 6 {
            bank.insert(bits);
        }
    }
}

fn write_json(path: &Path, batch: &Batch) {
    let mut text = serde_json::to_string_pretty(batch).expect("serialize probe batch");
    text.push('\n');
    std::fs::write(path, text).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
}

fn main() {
    let out_root = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(ROW_ROOT));
    std::fs::create_dir_all(&out_root)
        .unwrap_or_else(|e| panic!("create {}: {e}", out_root.display()));

    let mut cum_bank = BTreeSet::new();
    let mut pmt_bank = BTreeSet::new();
    bank_from_w108(Path::new(W108_INPUTS), &mut cum_bank, &mut pmt_bank);
    bank_from_batch(Path::new(OLD_CUM_BATCH), 6, &mut cum_bank);
    bank_from_batch(Path::new(OLD_PMT_BATCH), 5, &mut pmt_bank);
    bank_from_catalog(Path::new(CATALOG_CASES), &mut cum_bank);

    let mut cum_probes = Vec::new();
    let mut pmt_probes = Vec::new();
    let mut generated_cum = BTreeSet::new();
    let mut generated_pmt = BTreeSet::new();
    let mut meta = String::from(
        "context,timing,shape,pv_variant,cumprinc_id,pmt_id,rate_bits,nper_bits,pv_bits,start_bits,end_bits,relation\n",
    );

    for (context_index, context) in contexts().iter().enumerate() {
        let ladder = pv_ladder(context.pv);
        for timing in TIMINGS {
            for (pv_variant, pv) in ladder.iter().copied().enumerate() {
                let pmt_id = format!("cumx-c{context_index:02}-t{timing}-v{pv_variant:02}-pmt");
                let pmt_bits = vec![
                    context.rate.to_bits(),
                    (context.nper as f64).to_bits(),
                    pv.to_bits(),
                    0.0_f64.to_bits(),
                    (timing as f64).to_bits(),
                ];
                assert!(
                    !pmt_bank.contains(&pmt_bits),
                    "frozen PMT bank collision for {pmt_id}"
                );
                assert!(generated_pmt.insert(pmt_bits), "duplicate {pmt_id}");
                pmt_probes.push(ProbeEnvelope {
                    probe: Probe {
                        id: pmt_id.clone(),
                        args: vec![
                            hex(context.rate),
                            hex(context.nper as f64),
                            hex(pv),
                            hex(0.0),
                            hex(timing as f64),
                        ],
                    },
                });

                for shape in shapes(context.nper) {
                    let id = format!(
                        "cumx-c{context_index:02}-t{timing}-{}-v{pv_variant:02}",
                        shape.name
                    );
                    let cum_bits = vec![
                        context.rate.to_bits(),
                        (context.nper as f64).to_bits(),
                        pv.to_bits(),
                        (shape.start as f64).to_bits(),
                        (shape.end as f64).to_bits(),
                        (timing as f64).to_bits(),
                    ];
                    assert!(
                        !cum_bank.contains(&cum_bits),
                        "frozen CUMPRINC bank collision for {id}"
                    );
                    assert!(generated_cum.insert(cum_bits), "duplicate {id}");
                    cum_probes.push(ProbeEnvelope {
                        probe: Probe {
                            id: id.clone(),
                            args: vec![
                                hex(context.rate),
                                hex(context.nper as f64),
                                hex(pv),
                                hex(shape.start as f64),
                                hex(shape.end as f64),
                                hex(timing as f64),
                            ],
                        },
                    });
                    meta.push_str(&format!(
                        "{context_index:02},{timing},{},{pv_variant:02},{id},{pmt_id},{},{},{},{},{},{}\n",
                        shape.name,
                        hex(context.rate),
                        hex(context.nper as f64),
                        hex(pv),
                        hex(shape.start as f64),
                        hex(shape.end as f64),
                        shape.relation,
                    ));
                }
            }
        }
    }

    assert_eq!(cum_probes.len(), CONTEXT_COUNT * 2 * PV_VARIANTS * 9);
    assert_eq!(pmt_probes.len(), CONTEXT_COUNT * 2 * PV_VARIANTS);

    let cum_batch = Batch {
        function: "CUMPRINC",
        row_id: "cumprinc-exact-discriminator-20260809",
        probes: cum_probes,
    };
    let pmt_batch = Batch {
        function: "PMT",
        row_id: "pmt-cumprinc-companion-20260809",
        probes: pmt_probes,
    };
    let cum_path = out_root.join(CUM_BATCH_NAME);
    let pmt_path = out_root.join(PMT_BATCH_NAME);
    let meta_path = out_root.join(META_NAME);
    write_json(&cum_path, &cum_batch);
    write_json(&pmt_path, &pmt_batch);
    std::fs::write(&meta_path, meta)
        .unwrap_or_else(|e| panic!("write {}: {e}", meta_path.display()));

    println!(
        "wrote {} answer-blind, bank-disjoint CUMPRINC calls and {} paired PMT publications",
        cum_batch.probes.len(),
        pmt_batch.probes.len()
    );
    println!(
        "frozen banks: CUMPRINC={} PMT={}",
        cum_bank.len(),
        pmt_bank.len()
    );
    println!("cum_batch={}", cum_path.display());
    println!("pmt_batch={}", pmt_path.display());
    println!("meta={}", meta_path.display());
}
