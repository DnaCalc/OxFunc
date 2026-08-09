//! Generate a fresh, answer-blind PMT timing-factor metamer gate.
//!
//! For a power-of-two rate and `fv=0`, the final multiplication by `rate`
//! is an exact exponent shift.  Paired type-0/type-1 worksheet results thus
//! distinguish a true division by `1+rate` from multiplication by the stored
//! reciprocal without modelling PMT's still-open annuity helper.

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use std::collections::BTreeSet;

const ANSWER_DIR: &str = "../../work/w109/G6-solvers";
const ROW_ROOT: &str = "../../work/w109/G6-solvers";
const PAIR_COUNT: usize = 768;

fn hex(value: f64) -> String {
    format!("0x{:016x}", value.to_bits())
}

fn xorshift(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}

fn scalar_key(witness: &calc_graph_racer::score::Witness) -> Option<[u64; 5]> {
    let values = witness
        .args
        .iter()
        .filter_map(|arg| match arg {
            WitnessArg::Scalar(text) => parse_bits_hex(text).map(f64::to_bits),
            _ => None,
        })
        .collect::<Vec<_>>();
    (values.len() == 5).then(|| values.try_into().unwrap())
}

fn banked_rows() -> BTreeSet<[u64; 5]> {
    let mut rows = BTreeSet::new();
    for entry in std::fs::read_dir(ANSWER_DIR).expect("read PMT evidence directory") {
        let entry = entry.expect("read evidence entry");
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with("answers-pmt-") || !name.ends_with(".json") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(entry.path()) else {
            continue;
        };
        let Ok(set) = serde_json::from_str::<WitnessSet>(&text) else {
            continue;
        };
        if set.function != "PMT" {
            continue;
        }
        rows.extend(set.witnesses.iter().filter_map(scalar_key));
    }
    rows
}

fn main() {
    let banked = banked_rows();
    let mut seed = 0x706d_745f_7466_0809_u64;
    let mut tuples = Vec::with_capacity(PAIR_COUNT);
    let mut seen = BTreeSet::new();

    while tuples.len() < PAIR_COUNT {
        let z0 = xorshift(&mut seed);
        let z1 = xorshift(&mut seed);
        let z2 = xorshift(&mut seed);

        // Keep `1+rate` distinct from one and cover several reciprocal shapes.
        let rate_exp = 2 + (z0 % 39) as i32;
        let rate = 2.0_f64.powi(-rate_exp);

        // Mix integer and non-integer periods while staying far from overflow.
        let integer = 2 + (z1 % 4093) as u64;
        let periods = if tuples.len() & 1 == 0 {
            integer as f64
        } else {
            integer as f64 + ((z1 >> 16) & 0xffff) as f64 / 65536.0
        };

        // Moderate, full-mantissa PV values make the publication rounding
        // discriminating without introducing domain or overflow confounders.
        let pv_exp = ((z2 >> 53) % 25) as i32 - 8;
        let mantissa = 1.0 + (z2 & ((1_u64 << 52) - 1)) as f64 / (1_u64 << 52) as f64;
        let present = mantissa * 2.0_f64.powi(pv_exp);
        let future = 0.0_f64;
        let tuple = [
            rate.to_bits(),
            periods.to_bits(),
            present.to_bits(),
            future.to_bits(),
        ];
        if !seen.insert(tuple) {
            continue;
        }
        let type0 = [tuple[0], tuple[1], tuple[2], tuple[3], 0.0_f64.to_bits()];
        let type1 = [tuple[0], tuple[1], tuple[2], tuple[3], 1.0_f64.to_bits()];
        if banked.contains(&type0) || banked.contains(&type1) {
            continue;
        }
        tuples.push((rate, periods, present));
    }

    let mut batch = String::from(
        "{\"function\":\"PMT\",\"row_id\":\"pmt-tf-metamer-heldout-20260809\",\"probes\":[",
    );
    let mut meta = String::from("pair_id,type0_id,type1_id,rate_bits,nper_bits,pv_bits\n");
    for (pair, (rate, periods, present)) in tuples.iter().enumerate() {
        let id0 = format!("pmt-tf-ho-{pair:04}-t0");
        let id1 = format!("pmt-tf-ho-{pair:04}-t1");
        if pair > 0 {
            batch.push(',');
        }
        batch.push_str(&format!(
            "{{\"probe\":{{\"id\":\"{id0}\",\"args\":[\"{}\",\"{}\",\"{}\",\"0x0000000000000000\",\"0x0000000000000000\"]}}}},",
            hex(*rate),
            hex(*periods),
            hex(*present),
        ));
        batch.push_str(&format!(
            "{{\"probe\":{{\"id\":\"{id1}\",\"args\":[\"{}\",\"{}\",\"{}\",\"0x0000000000000000\",\"0x3ff0000000000000\"]}}}}",
            hex(*rate),
            hex(*periods),
            hex(*present),
        ));
        meta.push_str(&format!(
            "{pair:04},{id0},{id1},{},{},{}\n",
            hex(*rate),
            hex(*periods),
            hex(*present),
        ));
    }
    batch.push_str("]}");

    std::fs::write(
        format!("{ROW_ROOT}/batch-pmt-tf-metamer-heldout-20260809.json"),
        batch,
    )
    .expect("write PMT timing metamer batch");
    std::fs::write(
        format!("{ROW_ROOT}/meta-pmt-tf-metamer-heldout-20260809.csv"),
        meta,
    )
    .expect("write PMT timing metamer metadata");
    println!(
        "wrote {} answer-blind, bank-disjoint PMT timing-factor pairs ({} calls)",
        tuples.len(),
        tuples.len() * 2
    );
}
