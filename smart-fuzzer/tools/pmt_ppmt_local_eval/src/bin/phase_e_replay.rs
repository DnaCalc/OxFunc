//! W109: replay the W108 Phase-E live-Excel annuity corpus (external
//! `C:\Temp\ExcelExpFunction\finlab\fin_live_*.json`, build 20131) through the
//! CURRENT OxFunc surface dispatch, producing the true per-formula/per-class
//! error surface for catalog row G6-01.
//!
//! Usage: phase_e_replay --out <rollup.json> <fin_live_*.json> ...

use oxfunc_core::functions::surface_dispatch::eval_surface_value_call;
use oxfunc_core::resolver::{
    ReferenceDereferenceRequest, ReferenceResolutionError, ReferenceSystemCapabilities,
    ReferenceSystemProvider,
};
use oxfunc_core::value::{CalcValue, CoreValue};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

struct NoResolver;
impl ReferenceSystemProvider for NoResolver {
    fn capabilities(&self) -> ReferenceSystemCapabilities {
        ReferenceSystemCapabilities::permissive_local()
    }
    fn dereference(
        &self,
        request: &ReferenceDereferenceRequest,
    ) -> Result<CalcValue, ReferenceResolutionError> {
        Err(ReferenceResolutionError::UnresolvedReference {
            target: request.reference.target().to_string(),
        })
    }
}

#[derive(Deserialize)]
struct Corpus {
    rows: Vec<Row>,
}

#[derive(Deserialize)]
struct Row {
    id: String,
    class: String,
    formula: String,
    a_hex: Option<String>,
    b_hex: Option<String>,
    c_hex: Option<String>,
    d_hex: Option<String>,
    e_hex: Option<String>,
    f_hex: Option<String>,
    excel_hex: Option<String>,
    excel_error: Option<String>,
}

#[derive(Default, Serialize, Clone)]
struct Bucket {
    total: u32,
    exact: u32,
    ulp1: u32,
    ulp2_16: u32,
    ulp_gt16: u32,
    error_match: u32,
    kind_mismatch: u32,
    max_ulp: u64,
    worst_id: Option<String>,
}

fn ordered_key(v: f64) -> Option<u64> {
    if v.is_nan() {
        return None;
    }
    let bits = v.to_bits();
    Some(if bits & 0x8000_0000_0000_0000 != 0 {
        !bits
    } else {
        bits | 0x8000_0000_0000_0000
    })
}

fn parse_hex(h: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(h, 16).expect("hex"))
}

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let mut out_path = None;
    if let Some(pos) = args.iter().position(|a| a == "--out") {
        out_path = Some(args[pos + 1].clone());
        args.drain(pos..=pos + 1);
    }
    assert!(!args.is_empty(), "pass fin_live_*.json corpus files");

    let resolver = NoResolver;
    let mut buckets: BTreeMap<String, Bucket> = BTreeMap::new();
    let mut worst: Vec<(u64, String, String, String, String)> = Vec::new();

    for file in &args {
        let corpus: Corpus =
            serde_json::from_str(&std::fs::read_to_string(file).expect("read")).expect("parse");
        for row in &corpus.rows {
            let function_id = format!("FUNC.{}", row.formula.to_uppercase());
            let arity = match row.formula.as_str() {
                "POWER" => 2,
                "RRI" | "PDURATION" => 3,
                "PMT" | "FV" | "PV" => 5,
                "IPMT" | "PPMT" | "CUMIPMT" | "CUMPRINC" => 6,
                other => {
                    eprintln!("skipping unknown formula {other} (row {})", row.id);
                    continue;
                }
            };
            let hexes = [
                &row.a_hex, &row.b_hex, &row.c_hex, &row.d_hex, &row.e_hex, &row.f_hex,
            ];
            let call_args: Vec<CalcValue> = hexes[..arity]
                .iter()
                .map(|h| CalcValue::number(parse_hex(h.as_ref().expect("arg"))))
                .collect();
            let got = eval_surface_value_call(
                &function_id,
                &call_args,
                &resolver,
                None,
                None,
                None,
                None,
            );
            let key = format!("{}/{}", row.formula, row.class);
            let bucket = buckets.entry(key.clone()).or_default();
            bucket.total += 1;

            match (&row.excel_hex, &row.excel_error) {
                (Some(hex), _) => {
                    let expected = parse_hex(hex);
                    match got {
                        Ok(v) => match v.core() {
                            CoreValue::Number(n) => {
                                if n.to_bits() == expected.to_bits() {
                                    bucket.exact += 1;
                                } else if let (Some(a), Some(b)) =
                                    (ordered_key(*n), ordered_key(expected))
                                {
                                    let d = a.abs_diff(b);
                                    if d == 1 {
                                        bucket.ulp1 += 1;
                                    } else if d <= 16 {
                                        bucket.ulp2_16 += 1;
                                    } else {
                                        bucket.ulp_gt16 += 1;
                                    }
                                    if d > bucket.max_ulp {
                                        bucket.max_ulp = d;
                                        bucket.worst_id = Some(row.id.clone());
                                    }
                                    worst.push((
                                        d,
                                        row.id.clone(),
                                        key.clone(),
                                        format!("0x{:016x}", n.to_bits()),
                                        format!("0x{hex}"),
                                    ));
                                } else {
                                    bucket.kind_mismatch += 1;
                                }
                            }
                            _ => bucket.kind_mismatch += 1,
                        },
                        Err(_) => bucket.kind_mismatch += 1,
                    }
                }
                (None, Some(_err)) => match got {
                    Err(_) => bucket.error_match += 1,
                    Ok(v) => match v.core() {
                        CoreValue::Error(_) => bucket.error_match += 1,
                        _ => bucket.kind_mismatch += 1,
                    },
                },
                (None, None) => bucket.kind_mismatch += 1,
            }
        }
    }

    println!(
        "{:<28} {:>6} {:>6} {:>6} {:>7} {:>7} {:>6} {:>5} {:>9}",
        "formula/class", "total", "exact", "1ulp", "2-16", ">16", "errOK", "kind", "max_ulp"
    );
    let mut grand = Bucket::default();
    for (key, b) in &buckets {
        println!(
            "{:<28} {:>6} {:>6} {:>6} {:>7} {:>7} {:>6} {:>5} {:>9}",
            key, b.total, b.exact, b.ulp1, b.ulp2_16, b.ulp_gt16, b.error_match, b.kind_mismatch,
            b.max_ulp
        );
        grand.total += b.total;
        grand.exact += b.exact;
        grand.ulp1 += b.ulp1;
        grand.ulp2_16 += b.ulp2_16;
        grand.ulp_gt16 += b.ulp_gt16;
        grand.error_match += b.error_match;
        grand.kind_mismatch += b.kind_mismatch;
        grand.max_ulp = grand.max_ulp.max(b.max_ulp);
    }
    println!(
        "{:<28} {:>6} {:>6} {:>6} {:>7} {:>7} {:>6} {:>5} {:>9}",
        "TOTAL",
        grand.total,
        grand.exact,
        grand.ulp1,
        grand.ulp2_16,
        grand.ulp_gt16,
        grand.error_match,
        grand.kind_mismatch,
        grand.max_ulp
    );
    worst.sort_by(|a, b| b.0.cmp(&a.0));
    println!("\nworst 12 rows:");
    for (d, id, key, got, want) in worst.iter().take(12) {
        println!("  {d:>10} ulp  {key:<24} {id:<16} got {got} want {want}");
    }

    if let Some(path) = out_path {
        #[derive(Serialize)]
        struct Rollup {
            schema: &'static str,
            corpus_files: Vec<String>,
            buckets: BTreeMap<String, Bucket>,
        }
        let rollup = Rollup {
            schema: "w109.phase_e_replay.rollup/1",
            corpus_files: args.clone(),
            buckets,
        };
        std::fs::write(&path, serde_json::to_string_pretty(&rollup).unwrap()).unwrap();
        println!("\nrollup written to {path}");
    }
}
