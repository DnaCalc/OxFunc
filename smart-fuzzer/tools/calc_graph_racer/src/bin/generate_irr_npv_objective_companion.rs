//! Freeze the answer-blind worksheet-NPV companion for the W109 IRR discovery set.
//!
//! This reads only the already-frozen 300-row IRR discovery inputs.  It emits
//! three worksheet-NPV evaluation points per row: the original guess and the
//! two binary32-widened +/-0.001 perturbations in discount-factor space
//! `v = 1 / (1 + rate)`.  No IRR or NPV oracle answer participates in the
//! construction, and the sealed IRR held-out batch is never opened.

use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

const ROOT: &str = "../../work/w109/G6-solvers";
const SOURCE_FILE: &str = "batch-irr-exact-graph-discovery-20260809.json";
const SOURCE_SHA256: &str = "93E340A1A571799519DA9D38B26996C8BBA439B7BF646C9185D3966874B55A98";
const OUT_BATCH: &str = "batch-irr-npv-objective-companion-discovery-20260809.json";
const OUT_META: &str = "meta-irr-npv-objective-companion-discovery-20260809.csv";

fn hex(value: f64) -> String {
    format!("0x{:016x}", value.to_bits())
}

fn from_hex(raw: &str) -> f64 {
    assert_eq!(raw.len(), 18, "invalid binary64 hex: {raw}");
    assert!(raw.starts_with("0x"), "invalid binary64 hex: {raw}");
    let bits = u64::from_str_radix(&raw[2..], 16).expect("invalid binary64 hex digits");
    f64::from_bits(bits)
}

fn required_str<'a>(value: &'a Value, field: &str) -> &'a str {
    value[field]
        .as_str()
        .unwrap_or_else(|| panic!("missing string field {field}"))
}

fn main() {
    let source_path = format!("{ROOT}/{SOURCE_FILE}");
    let source: Value = serde_json::from_slice(
        &std::fs::read(&source_path).expect("read frozen IRR discovery batch"),
    )
    .expect("parse frozen IRR discovery batch");
    assert_eq!(required_str(&source, "function"), "IRR");
    assert_eq!(
        required_str(&source, "row_id"),
        "irr-exact-graph-discovery-20260809"
    );
    let source_probes = source["probes"].as_array().expect("source probes array");
    assert_eq!(source_probes.len(), 300);

    let h_magnitude = f32::from_bits(0x3a83_126f) as f64;
    assert_eq!(hex(h_magnitude), "0x3f50624de0000000");
    let points = [
        ("base", 0.0_f64),
        ("v_h_neg", -h_magnitude),
        ("v_h_pos", h_magnitude),
    ];

    let mut probes = Vec::with_capacity(source_probes.len() * points.len());
    let mut ids = BTreeSet::new();
    let mut source_counts = BTreeMap::<String, usize>::new();
    let mut meta = String::from(
        "id,source_irr_id,point_class,cashflow_count,c0_bits,tail_bits,guess_bits,v0_bits,h_bits,evaluation_v_bits,rate_bits\n",
    );

    for wrapper in source_probes {
        let probe = &wrapper["probe"];
        let source_id = required_str(probe, "id");
        let args = probe["args"].as_array().expect("source args array");
        assert_eq!(args.len(), 2, "{source_id}: expected IRR values and guess");
        let cashflow_json = args[0].as_array().expect("IRR cash-flow array");
        assert!(
            (2..=8).contains(&cashflow_json.len()),
            "{source_id}: unexpected cash-flow count"
        );
        let cashflow_bits = cashflow_json
            .iter()
            .map(|item| item.as_str().expect("cash-flow bit string").to_owned())
            .collect::<Vec<_>>();
        let cashflows = cashflow_bits
            .iter()
            .map(|raw| from_hex(raw))
            .collect::<Vec<_>>();
        assert!(cashflows.iter().all(|value| value.is_finite()));
        let c0_bits = cashflow_bits[0].clone();
        let tail_bits = cashflow_bits[1..].to_vec();
        let guess_bits = args[1].as_str().expect("guess bit string");
        let guess = from_hex(guess_bits);
        assert!(guess.is_finite() && guess > -1.0);

        // Freeze the candidate variable/publication graph explicitly.  Each
        // local is a binary64 staging boundary under Rust's strict IEEE mode.
        let one_plus_guess = 1.0 + guess;
        let v0 = 1.0 / one_plus_guess;
        assert!(v0.is_finite() && v0 > h_magnitude);

        for (point_class, h) in points {
            let evaluation_v = v0 + h;
            assert!(evaluation_v.is_finite() && evaluation_v > 0.0);
            let reciprocal = 1.0 / evaluation_v;
            let derived_rate = reciprocal - 1.0;
            let rate = if point_class == "base" {
                // Preserve the exact source guess at the base point.  The
                // derived graph is retained separately in the metadata.
                guess
            } else {
                derived_rate
            };
            assert!(rate.is_finite() && rate > -1.0);

            let id = format!("npv-objective-{source_id}-{point_class}");
            assert!(ids.insert(id.clone()), "duplicate companion id {id}");
            *source_counts.entry(source_id.to_owned()).or_default() += 1;
            probes.push(json!({
                "probe": {
                    "id": id,
                    "source_irr_id": source_id,
                    "point_class": point_class,
                    "c0_bits": c0_bits,
                    "tail_bits": tail_bits,
                    "guess_bits": guess_bits,
                    "v0_bits": hex(v0),
                    "h_bits": hex(h),
                    "evaluation_v_bits": hex(evaluation_v),
                    "derived_rate_bits": hex(derived_rate),
                    "rate_bits": hex(rate)
                }
            }));
            meta.push_str(&format!(
                "{id},{source_id},{point_class},{},{},{},{},{},{},{},{}\n",
                cashflow_bits.len(),
                c0_bits,
                tail_bits.join("|"),
                guess_bits,
                hex(v0),
                hex(h),
                hex(evaluation_v),
                hex(rate),
            ));
        }
    }

    assert_eq!(probes.len(), 900);
    assert_eq!(ids.len(), 900);
    assert_eq!(source_counts.len(), 300);
    assert!(source_counts.values().all(|count| *count == 3));

    let document = json!({
        "schema_version": "w109.irr.npv_objective_companion.batch.v1",
        "function": "NPV",
        "row_id": "irr-npv-objective-companion-discovery-20260809",
        "source_discovery": {
            "function": "IRR",
            "row_id": "irr-exact-graph-discovery-20260809",
            "probe_count": 300,
            "path": format!("smart-fuzzer/work/w109/G6-solvers/{SOURCE_FILE}"),
            "sha256": SOURCE_SHA256
        },
        "point_specification": {
            "space": "v=1/(1+rate)",
            "h_origin": "binary32(0.001) widened exactly to binary64",
            "h_magnitude_bits": hex(h_magnitude),
            "point_classes": ["base", "v_h_neg", "v_h_pos"],
            "derived_rate_graph": "v0=1/(1+guess); evaluation_v=v0+h; reciprocal=1/evaluation_v; rate=reciprocal-1",
            "base_rate_rule": "use exact source guess bits"
        },
        "capture_contract": {
            "raw": "NPV(rate,c1..cn)",
            "direct_composed": "NPV(rate,c1..cn)+c0 in one worksheet formula",
            "cell_composed": "raw_npv_cell+c0 in a separate worksheet formula",
            "input_plumbing": "exact binary64 Range.Value2 matrix and R1C1 cell references"
        },
        "probes": probes
    });

    let batch_path = format!("{ROOT}/{OUT_BATCH}");
    let meta_path = format!("{ROOT}/{OUT_META}");
    std::fs::write(&batch_path, serde_json::to_vec(&document).unwrap())
        .expect("write companion batch");
    std::fs::write(&meta_path, meta).expect("write companion metadata");
    println!("frozen answer-blind companion probes=900 source_rows=300 points_per_source=3");
    println!("batch={batch_path}");
    println!("meta={meta_path}");
}
