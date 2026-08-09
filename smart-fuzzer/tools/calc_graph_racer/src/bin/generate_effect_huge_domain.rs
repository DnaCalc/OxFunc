//! Generate the W109 EFFECT large-period dispatch and extreme-domain battery.
//!
//! The batch is oracle-free. It brackets 32-bit, 53-bit, signed/unsigned
//! 64-bit, and binary64 magnitude boundaries while pairing each period count
//! with rates that exercise base-rounds-to-one, finite, and overflow outcomes.
//! Capture the generated JSON with `Run-W109BulkBatch.ps1 -NoCache`.

use serde_json::json;

fn bits(value: f64) -> String {
    format!("0x{:016x}", value.to_bits())
}

fn main() {
    let p_1e20 = 1e20_f64;
    let p_1e100 = 1e100_f64;
    let periods = [
        ("2p30-control", 2_f64.powi(30)),
        ("2p31-prev", 2_f64.powi(31) - 1.0),
        ("2p31", 2_f64.powi(31)),
        ("2p31-next", 2_f64.powi(31) + 1.0),
        ("u32-max-minus-2", 2_f64.powi(32) - 3.0),
        ("u32-max-minus-1", 2_f64.powi(32) - 2.0),
        ("frac-trunc-u32-max-minus-1", 2_f64.powi(32) - 1.25),
        ("frac-trunc-u32-max", 2_f64.powi(32) - 0.25),
        ("2p32-prev", 2_f64.powi(32) - 1.0),
        ("2p32", 2_f64.powi(32)),
        ("2p32-next", 2_f64.powi(32) + 1.0),
        ("2p33-control", 2_f64.powi(33)),
        ("frac-2p51-plus-half", f64::from_bits(0x4320_0000_0000_0001)),
        ("frac-2p52-minus-1p5", f64::from_bits(0x432f_ffff_ffff_fffd)),
        (
            "frac-2p52-minus-half",
            f64::from_bits(0x432f_ffff_ffff_ffff),
        ),
        ("2p53-prev", f64::from_bits(0x433f_ffff_ffff_ffff)),
        ("2p53", f64::from_bits(0x4340_0000_0000_0000)),
        ("2p53-next", f64::from_bits(0x4340_0000_0000_0001)),
        ("i64-as-f64-prev", f64::from_bits(0x43df_ffff_ffff_ffff)),
        ("i64-as-f64", f64::from_bits(0x43e0_0000_0000_0000)),
        ("i64-as-f64-next", f64::from_bits(0x43e0_0000_0000_0001)),
        ("u64-as-f64-prev", f64::from_bits(0x43ef_ffff_ffff_ffff)),
        ("u64-as-f64", f64::from_bits(0x43f0_0000_0000_0000)),
        ("u64-as-f64-next", f64::from_bits(0x43f0_0000_0000_0001)),
        ("1e20-prev", f64::from_bits(p_1e20.to_bits() - 1)),
        ("1e20", p_1e20),
        ("1e20-next", f64::from_bits(p_1e20.to_bits() + 1)),
        ("1e100-prev", f64::from_bits(p_1e100.to_bits() - 1)),
        ("1e100", p_1e100),
        ("1e100-next", f64::from_bits(p_1e100.to_bits() + 1)),
        ("f64-max-prev", f64::from_bits(0x7fef_ffff_ffff_fffe)),
        ("f64-max", f64::MAX),
    ];
    let rate_builders: [(&str, fn(f64) -> f64); 5] = [
        ("rate-0p05", |_| 0.05),
        ("rate-n-2m64", |n| n * 2_f64.powi(-64)),
        ("rate-n-2m63", |n| n * 2_f64.powi(-63)),
        ("rate-n-2m53", |n| n * 2_f64.powi(-53)),
        ("rate-n-2m52", |n| n * 2_f64.powi(-52)),
    ];

    let mut probes = Vec::new();
    for (period_label, period) in periods {
        let n = period.trunc();
        for (rate_label, build_rate) in rate_builders {
            let rate = build_rate(n);
            assert!(rate.is_finite() && rate > 0.0);
            probes.push(json!({
                "probe": {
                    "id": format!("effect-huge-{period_label}-{rate_label}"),
                    "args": [bits(rate), bits(period)]
                }
            }));
        }
    }

    let document = json!({
        "function": "EFFECT",
        "row_id": "effect-huge-domain-scratch-build20228",
        "probes": probes
    });
    let root = "../../work/w109/G6-solvers";
    let path = format!("{root}/batch-effect-huge-domain-scratch.json");
    std::fs::write(
        &path,
        serde_json::to_string_pretty(&document).unwrap() + "\n",
    )
    .unwrap();
    println!(
        "wrote {path}: {} probes",
        document["probes"].as_array().unwrap().len()
    );
}
