//! Freeze an answer-blind FV companion for the RATE one-step discovery rows.
//!
//! RATE's objective is expected to be the requested future value minus the
//! forward FV surface, up to sign.  Capturing FV at the exact guess and every
//! distinct `x+h` admitted by the frozen RATE grammar exposes the objective and
//! forward-difference operands without choosing any new input from RATE answers.

use oxfunc_core::excel_numeric::research as rx;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const ROOT: &str = "../../work/w109/G6-rate";
const FREEZE_ID: &str = "w109-g6-05-rate-fv-companion-v1-20260809";
const SOURCE_BATCH_SHA256: &str =
    "09E7226D5D2DDA3E5AADF6D16DA3C5E87931F3686D4F3FF1AC96F86D0D314B00";
const EXPECTED_RATE_ROWS: usize = 256;
const CW: u16 = rx::CW_PC64_RN;

#[derive(Deserialize)]
struct SourceProbe {
    id: String,
    args: Vec<String>,
}

#[derive(Deserialize)]
struct RankedSourceProbe {
    probe: SourceProbe,
}

#[derive(Deserialize)]
struct SourceBatch {
    function: String,
    row_id: String,
    probes: Vec<RankedSourceProbe>,
}

#[derive(Serialize)]
struct Probe {
    id: String,
    args: [String; 5],
}

#[derive(Serialize)]
struct RankedProbe {
    probe: Probe,
}

#[derive(Serialize)]
struct Batch {
    function: &'static str,
    row_id: &'static str,
    probes: Vec<RankedProbe>,
}

#[derive(Serialize)]
struct Record {
    id: String,
    source_rate_id: String,
    evaluation: String,
    rate_bits: String,
    nper_bits: String,
    pmt_bits: String,
    pv_bits: String,
    type_bits: String,
    requested_fv_bits: String,
    guess_bits: String,
}

fn parse_hex(value: &str) -> f64 {
    let raw = value.strip_prefix("0x").expect("hex prefix");
    assert_eq!(raw.len(), 16);
    f64::from_bits(u64::from_str_radix(raw, 16).unwrap())
}

fn hex(value: f64) -> String {
    format!("0x{:016x}", value.to_bits())
}

fn e(value: f64) -> rx::Ext80 {
    rx::ext_from_f64(value)
}

fn x87_mul(a: f64, b: f64) -> f64 {
    rx::ext_to_f64(&rx::ext_mul(&e(a), &e(b), CW), CW)
}

fn x87_add(a: f64, b: f64) -> f64 {
    rx::ext_to_f64(&rx::ext_add(&e(a), &e(b), CW), CW)
}

fn candidate_next_inputs(x: f64) -> BTreeSet<u64> {
    let h_f64 = 1.0e-6 * x;
    let h_x87 = x87_mul(1.0e-6, x);
    [x + h_f64, x87_add(x, h_f64), x + h_x87, x87_add(x, h_x87)]
        .into_iter()
        .map(f64::to_bits)
        .collect()
}

fn write_frozen(path: &Path, bytes: &[u8]) {
    if let Ok(existing) = std::fs::read(path) {
        assert_eq!(
            existing,
            bytes,
            "refusing to overwrite frozen artifact {} with different bytes",
            path.display()
        );
        println!("verified frozen {}", path.display());
        return;
    }
    std::fs::write(path, bytes).unwrap();
    println!("wrote frozen {}", path.display());
}

fn pretty<T: Serialize>(value: &T) -> Vec<u8> {
    let mut bytes = serde_json::to_vec_pretty(value).unwrap();
    bytes.push(b'\n');
    bytes
}

fn main() {
    let root = PathBuf::from(ROOT);
    let source: SourceBatch = serde_json::from_str(
        &std::fs::read_to_string(root.join("batch-rate-one-step-discovery-v2.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(source.function, "RATE");
    assert_eq!(source.probes.len(), EXPECTED_RATE_ROWS);
    assert_eq!(
        source
            .probes
            .iter()
            .map(|entry| entry.probe.id.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        EXPECTED_RATE_ROWS
    );

    let mut probes = Vec::new();
    let mut records = Vec::new();
    let mut per_row_next_input_counts = Vec::new();
    for (row, entry) in source.probes.iter().enumerate() {
        assert_eq!(entry.probe.args.len(), 6);
        let nper = parse_hex(&entry.probe.args[0]);
        let pmt = parse_hex(&entry.probe.args[1]);
        let pv = parse_hex(&entry.probe.args[2]);
        let requested_fv = parse_hex(&entry.probe.args[3]);
        let ty = parse_hex(&entry.probe.args[4]);
        let guess = parse_hex(&entry.probe.args[5]);
        assert!(nper.is_finite() && pmt.is_finite() && pv.is_finite());
        assert!(requested_fv.is_finite() && (ty == 0.0 || ty == 1.0));
        assert!(guess.is_finite() && guess != 0.0 && guess > -1.0);

        let mut rates = vec![("guess".to_owned(), guess)];
        let next_inputs = candidate_next_inputs(guess);
        per_row_next_input_counts.push(next_inputs.len());
        for (index, bits) in next_inputs.into_iter().enumerate() {
            rates.push((format!("x-plus-h-{index:02}"), f64::from_bits(bits)));
        }
        for (evaluation, rate) in rates {
            let id = format!("rate-fv-companion-v1-{row:04}-{evaluation}");
            let args = [hex(rate), hex(nper), hex(pmt), hex(pv), hex(ty)];
            probes.push(RankedProbe {
                probe: Probe {
                    id: id.clone(),
                    args,
                },
            });
            records.push(Record {
                id,
                source_rate_id: entry.probe.id.clone(),
                evaluation,
                rate_bits: hex(rate),
                nper_bits: hex(nper),
                pmt_bits: hex(pmt),
                pv_bits: hex(pv),
                type_bits: hex(ty),
                requested_fv_bits: hex(requested_fv),
                guess_bits: hex(guess),
            });
        }
    }
    let count_histogram = per_row_next_input_counts.iter().fold(
        std::collections::BTreeMap::<usize, usize>::new(),
        |mut map, &count| {
            *map.entry(count).or_default() += 1;
            map
        },
    );
    let manifest = json!({
        "schema_version": "oxfunc.w109.rate_fv_companion_manifest.v1",
        "freeze_id": FREEZE_ID,
        "answer_blind": true,
        "clean_room": true,
        "source": {
            "batch": "batch-rate-one-step-discovery-v2.json",
            "batch_sha256": SOURCE_BATCH_SHA256,
            "row_id": source.row_id,
            "rate_rows": EXPECTED_RATE_ROWS,
        },
        "transform": {
            "function": "FV",
            "arguments": "FV(rate, nper, pmt, pv, type)",
            "evaluations_per_rate_row": "exact guess plus every distinct x+h bit pattern in the frozen RATE grammar",
            "h_graphs": [
                "h=f64(1e-6*x); xh=f64(x+h)",
                "h=f64(1e-6*x); xh=RN53(RN64(x+h))",
                "h=RN53(RN64(1e-6*x)); xh=f64(x+h)",
                "h=RN53(RN64(1e-6*x)); xh=RN53(RN64(x+h))"
            ],
            "unique_x_plus_h_count_histogram": count_histogram,
            "fv_call_count": probes.len(),
        },
        "heldout": "not opened, transformed, or captured",
    });
    let metadata = json!({
        "schema_version": "oxfunc.w109.rate_fv_companion_dataset_bank.v1",
        "freeze_id": FREEZE_ID,
        "answer_blind": true,
        "records": records,
    });
    let batch = Batch {
        function: "FV",
        row_id: "G6-05-rate-fv-companion-discovery-v1-20260809",
        probes,
    };
    for (name, bytes) in [
        (
            "candidate-manifest-rate-fv-companion-v1.json",
            pretty(&manifest),
        ),
        (
            "meta-rate-fv-companion-discovery-v1.json",
            pretty(&metadata),
        ),
        ("batch-rate-fv-companion-discovery-v1.json", pretty(&batch)),
    ] {
        write_frozen(&root.join(name), &bytes);
    }
    println!(
        "freeze_id={FREEZE_ID} source_rate_rows={} fv_calls={} xh_count_histogram={count_histogram:?}",
        EXPECTED_RATE_ROWS,
        batch.probes.len(),
    );
}
