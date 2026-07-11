//! W109 G3-02: does Excel's positive GAMMA equal exp-chain(published GAMMALN)?
//! Tests the x87 EXP of the published GAMMALN value per point.

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet, ulp_distance};
use oxfunc_core::excel_numeric::research as rx;
use std::collections::HashMap;

fn main() {
    let dir = std::env::args().nth(1).expect("work dir");
    let gamma: WitnessSet =
        serde_json::from_str(&std::fs::read_to_string(format!("{dir}/answers-r0.json")).unwrap())
            .unwrap();
    let gammaln: WitnessSet = serde_json::from_str(
        &std::fs::read_to_string(format!("{dir}/answers-gammaln.json")).unwrap(),
    )
    .unwrap();
    let gl: HashMap<String, f64> = gammaln
        .witnesses
        .iter()
        .filter_map(|w| {
            Some((
                w.id.clone()?.trim_start_matches("gl-").to_string(),
                parse_bits_hex(&w.expected_bits)?,
            ))
        })
        .collect();
    let (mut exact, mut total, mut max_ulp) = (0u32, 0u32, 0u64);
    let mut worst: Vec<(u64, String, f64)> = Vec::new();
    for w in &gamma.witnesses {
        let Some(id) = &w.id else { continue };
        if !id.starts_with("pos-") && !id.starts_with("fix-pos") {
            continue;
        }
        let x = match &w.args[0] {
            WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
            _ => continue,
        };
        let (Some(expected), Some(g)) = (parse_bits_hex(&w.expected_bits), gl.get(id)) else {
            continue;
        };
        total += 1;
        let v = rx::excel_exp(*g);
        if v.to_bits() == expected.to_bits() {
            exact += 1;
        } else {
            let d = ulp_distance(v, expected).unwrap_or(u64::MAX);
            max_ulp = max_ulp.max(d);
            worst.push((d, id.clone(), x));
        }
    }
    println!("exp(x87) of published GAMMALN: {exact}/{total} exact, max_ulp {max_ulp}");
    worst.sort_by(|a, b| b.0.cmp(&a.0));
    for (d, id, x) in worst.iter().take(8) {
        println!("  {d:>8} ulp {id} x={x:+.9e}");
    }
}
