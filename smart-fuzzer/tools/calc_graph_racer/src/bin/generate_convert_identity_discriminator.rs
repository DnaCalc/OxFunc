//! Generate the tiny typed CONVERT identity/adjacent-bit discriminator.
//!
//! This batch closes the one discovery-design gap left by the main 7,026-row
//! battery: adjacent inputs were exercised for non-identity direct pairs and
//! prefixed identities, but not for plain direct-unit identities.  Those rows
//! distinguish argument-cell mutation from arithmetic staging.

use serde::Serialize;
use serde_json::json;
use std::path::PathBuf;

#[derive(Serialize)]
struct Probe {
    id: String,
    args: [String; 3],
}

#[derive(Serialize)]
struct Envelope {
    probe: Probe,
}

#[derive(Serialize)]
struct Batch {
    schema_version: &'static str,
    function: &'static str,
    row_id: &'static str,
    arg_encoding: serde_json::Value,
    probes: Vec<Envelope>,
}

fn next_down(value: f64) -> f64 {
    f64::from_bits(value.to_bits() - 1)
}

fn next_up(value: f64) -> f64 {
    f64::from_bits(value.to_bits() + 1)
}

fn main() {
    let mut probes = Vec::new();
    let mut sequence = 0;
    for exponent in [0] {
        let power = 2.0_f64.powi(exponent);
        for number in [next_down(power), power, next_up(power)] {
            for (from, to) in [
                ("m", "m"),
                ("ft", "ft"),
                ("sec", "sec"),
                ("l", "l"),
                ("m", "ft"),
                ("ft", "yd"),
            ] {
                probes.push(Envelope {
                    probe: Probe {
                        id: format!("conv-identity-{sequence:03}"),
                        args: [
                            format!("0x{:016x}", number.to_bits()),
                            from.to_string(),
                            to.to_string(),
                        ],
                    },
                });
                sequence += 1;
            }
        }
    }
    let batch = Batch {
        schema_version: "w109.convert.mixed_scalar_probe_batch.v1",
        function: "CONVERT",
        row_id: "convert-identity-discriminator-20260809",
        arg_encoding: json!({
            "number": "0x followed by exactly 16 IEEE-754 binary64 hex digits",
            "text": "verbatim non-hex JSON string",
            "purpose": "direct identity adjacent-bit discriminator"
        }),
        probes,
    };
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../work/w109/G4-convert/batch-convert-identity-discriminator-20260809.json");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_vec_pretty(&batch).unwrap()).unwrap();
    println!("wrote {} probes -> {}", batch.probes.len(), path.display());
}
