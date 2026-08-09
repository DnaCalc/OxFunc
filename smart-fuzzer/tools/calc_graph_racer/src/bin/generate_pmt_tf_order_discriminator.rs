//! Generate a compact, answer-blind PMT general-rate timing-order gate.
//!
//! Each context is a 16-value consecutive-PV ladder captured at both timing
//! values.  The paired ladders let the scorer intersect publication intervals
//! and eliminate the still-unknown annuity helper.  Contexts are selected only
//! from local candidate predictions (never Excel answers), maximizing
//! disagreement among stored reciprocal, stored `r/tf`, tf-before/after-rate,
//! native SSE2, per-op x87-spill, and x87-continuous tails.

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use std::collections::{BTreeMap, BTreeSet};

const ANSWER_DIR: &str = "../../work/w109/G6-solvers";
const ROW_ROOT: &str = "../../work/w109/G6-solvers";
const CONTEXTS: usize = 15;
const LADDER: usize = 16;
const SEARCH_ROWS: usize = 300_000;
const CW: u16 = rx::CW_PC64_RN;

fn hex(value: f64) -> String {
    format!("0x{:016x}", value.to_bits())
}

fn xorshift(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}

fn ext(value: f64) -> rx::Ext80 {
    rx::ext_from_f64(value)
}

fn to_f64(value: &rx::Ext80) -> f64 {
    rx::ext_to_f64(value, CW)
}

fn xmul(left: f64, right: f64) -> f64 {
    to_f64(&rx::ext_mul(&ext(left), &ext(right), CW))
}

fn xdiv(left: f64, right: f64) -> f64 {
    to_f64(&rx::ext_div(&ext(left), &ext(right), CW))
}

fn xadd(left: f64, right: f64) -> f64 {
    to_f64(&rx::ext_add(&ext(left), &ext(right), CW))
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
        // Re-running the generator after its own live capture must reproduce
        // the frozen batch, not treat that batch's answer as an upstream-bank
        // collision and silently select a different gate.
        if name == "answers-pmt-tf-order-discriminator-20260809.json" {
            continue;
        }
        if !name.starts_with("answers-pmt-") || !name.ends_with(".json") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(entry.path()) else {
            continue;
        };
        let Ok(set) = serde_json::from_str::<WitnessSet>(&text) else {
            continue;
        };
        if set.function == "PMT" {
            rows.extend(set.witnesses.iter().filter_map(scalar_key));
        }
    }
    rows
}

fn ordered(bits: u64) -> i128 {
    if bits >> 63 == 0 {
        (bits | (1_u64 << 63)) as i128
    } else {
        (!bits) as i128
    }
}

fn signature(rate: f64, periods: f64, present: f64) -> Option<Vec<u64>> {
    let tau = -(periods * rate.ln_1p());
    let em = tau.exp_m1();
    if em == 0.0 || !em.is_finite() {
        return None;
    }
    let q = present / em;
    if !q.is_finite() {
        return None;
    }
    let tf = 1.0 + rate;
    let xtf = xadd(1.0, rate);
    let reciprocal = 1.0 / tf;
    let xreciprocal = xdiv(1.0, xtf);

    let mut values = vec![
        // Stored reciprocal, before and after the rate publication.
        (q * reciprocal) * rate,
        (q * rate) * reciprocal,
        xmul(xmul(q, xreciprocal), rate),
        xmul(xmul(q, rate), xreciprocal),
        // Stored r/tf (true divide) and r*stored-recip coefficients.
        q * (rate / tf),
        xmul(q, xdiv(rate, xtf)),
        q * (rate * reciprocal),
        xmul(q, xmul(rate, xreciprocal)),
        // True division before/after the rate.
        (q / tf) * rate,
        (q * rate) / tf,
        xmul(xdiv(q, xtf), rate),
        xdiv(xmul(q, rate), xtf),
    ];
    // Fully PC64-continuous q*r/tf and q*recip*r, one final store.
    let qe = ext(q);
    let re = ext(rate);
    let tfe = rx::ext_add(&rx::ext_one(), &re, CW);
    values.push(to_f64(&rx::ext_div(&rx::ext_mul(&qe, &re, CW), &tfe, CW)));
    let rece = rx::ext_div(&rx::ext_one(), &tfe, CW);
    values.push(to_f64(&rx::ext_mul(&rx::ext_mul(&qe, &rece, CW), &re, CW)));
    Some(values.into_iter().map(f64::to_bits).collect())
}

#[derive(Clone)]
struct Candidate {
    score: (usize, u128),
    rate: f64,
    periods: f64,
    present: f64,
}

fn main() {
    let banked = banked_rows();
    let periods_bank = [1.0, 2.0, 3.0, 6.0, 12.0, 37.0, 360.0, 4095.5];
    let mut seed = 0x706d_745f_6f72_6465_u64;
    let mut pool = Vec::new();

    for index in 0..SEARCH_ROWS {
        let z0 = xorshift(&mut seed);
        let z1 = xorshift(&mut seed);
        let z2 = xorshift(&mut seed);
        // Positive, non-dyadic-looking rates over the main financial range.
        let exponent = -30 + (z0 % 29) as i32;
        let mantissa = 1.0 + (z1 & ((1_u64 << 52) - 1)) as f64 / (1_u64 << 52) as f64;
        let rate = mantissa * 2.0_f64.powi(exponent);
        if rate.to_bits() & ((1_u64 << 52) - 1) == 0 {
            continue;
        }
        let periods = periods_bank[index % periods_bank.len()];
        let pv_exponent = ((z2 >> 58) % 17) as i32 - 8;
        let pv_mantissa = 1.0 + (z2 & ((1_u64 << 52) - 1)) as f64 / (1_u64 << 52) as f64;
        let present = pv_mantissa * 2.0_f64.powi(pv_exponent);
        let Some(signature) = signature(rate, periods, present) else {
            continue;
        };
        let distinct = signature.iter().copied().collect::<BTreeSet<_>>().len();
        if distinct < 4 {
            continue;
        }
        let min = signature.iter().map(|bits| ordered(*bits)).min().unwrap();
        let max = signature.iter().map(|bits| ordered(*bits)).max().unwrap();
        let spread = (max - min).unsigned_abs();
        pool.push(Candidate {
            score: (distinct, spread),
            rate,
            periods,
            present,
        });
    }
    pool.sort_by(|left, right| right.score.cmp(&left.score));

    let mut selected = Vec::new();
    let mut rates = BTreeSet::new();
    let mut period_counts = BTreeMap::new();
    for candidate in pool {
        if selected.len() == CONTEXTS {
            break;
        }
        // Avoid one spectacular rate or period monopolizing the gate.
        if rates.contains(&candidate.rate.to_bits()) {
            continue;
        }
        if *period_counts
            .get(&candidate.periods.to_bits())
            .unwrap_or(&0usize)
            >= 3
        {
            continue;
        }
        let mut collision = false;
        for offset in 0..LADDER {
            let present_bits = candidate.present.to_bits() + offset as u64;
            for timing in [0.0_f64, 1.0_f64] {
                let key = [
                    candidate.rate.to_bits(),
                    candidate.periods.to_bits(),
                    present_bits,
                    0.0_f64.to_bits(),
                    timing.to_bits(),
                ];
                collision |= banked.contains(&key);
            }
        }
        if collision {
            continue;
        }
        rates.insert(candidate.rate.to_bits());
        *period_counts
            .entry(candidate.periods.to_bits())
            .or_insert(0usize) += 1;
        selected.push(candidate);
    }
    assert_eq!(
        selected.len(),
        CONTEXTS,
        "not enough discriminator contexts"
    );

    let mut batch = String::from(
        "{\"function\":\"PMT\",\"row_id\":\"pmt-tf-order-discriminator-20260809\",\"probes\":[",
    );
    let mut meta = String::from(
        "context,pair_id,type0_id,type1_id,rate_bits,nper_bits,pv_bits,distinct_candidates,signature_span_ulp,candidate_bits\n",
    );
    let mut first = true;
    for (context, candidate) in selected.iter().enumerate() {
        for offset in 0..LADDER {
            let pair = context * LADDER + offset;
            let present = f64::from_bits(candidate.present.to_bits() + offset as u64);
            let row_signature = signature(candidate.rate, candidate.periods, present)
                .expect("selected row keeps a finite local signature");
            let distinct = row_signature.iter().copied().collect::<BTreeSet<_>>().len();
            let min = row_signature
                .iter()
                .map(|bits| ordered(*bits))
                .min()
                .unwrap();
            let max = row_signature
                .iter()
                .map(|bits| ordered(*bits))
                .max()
                .unwrap();
            let spread = (max - min).unsigned_abs();
            let candidate_bits = row_signature
                .iter()
                .map(|bits| format!("{bits:016x}"))
                .collect::<Vec<_>>()
                .join("|");
            let id0 = format!("pmt-tf-order-{pair:04}-t0");
            let id1 = format!("pmt-tf-order-{pair:04}-t1");
            for (id, timing) in [(&id0, 0.0_f64), (&id1, 1.0_f64)] {
                if !first {
                    batch.push(',');
                }
                first = false;
                batch.push_str(&format!(
                    "{{\"probe\":{{\"id\":\"{id}\",\"args\":[\"{}\",\"{}\",\"{}\",\"0x0000000000000000\",\"{}\"]}}}}",
                    hex(candidate.rate),
                    hex(candidate.periods),
                    hex(present),
                    hex(timing),
                ));
            }
            meta.push_str(&format!(
                "{context:02},{pair:04},{id0},{id1},{},{},{},{},{},{}\n",
                hex(candidate.rate),
                hex(candidate.periods),
                hex(present),
                distinct,
                spread,
                candidate_bits,
            ));
        }
    }
    batch.push_str("]}");

    let batch_path = format!("{ROW_ROOT}/batch-pmt-tf-order-discriminator-20260809.json");
    let meta_path = format!("{ROW_ROOT}/meta-pmt-tf-order-discriminator-20260809.csv");
    std::fs::write(&batch_path, batch).expect("write PMT tf-order batch");
    std::fs::write(&meta_path, meta).expect("write PMT tf-order metadata");
    println!(
        "wrote {CONTEXTS} contexts x {LADDER} PVs x 2 timings = {} answer-blind, bank-disjoint calls",
        CONTEXTS * LADDER * 2
    );
    println!("batch={batch_path}");
    println!("meta={meta_path}");
}
