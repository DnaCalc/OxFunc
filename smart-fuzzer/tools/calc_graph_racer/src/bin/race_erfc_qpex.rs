//! Excel P vs Q complement identity on shared z<0.5 and z>=0.5. Not a kernel land.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use std::collections::BTreeMap;
use std::env;
use std::fs;

fn main() {
    let dir = env::args().nth(1).expect("dir");
    const PBANKS: [&str; 7] = [
        "answers-b9train.json",
        "answers-erfp.json",
        "answers-erfm.json",
        "answers-b8erf.json",
        "answers-b7erf.json",
        "answers-b11.json",
        "answers-b10.json",
    ];
    let mut pmap = BTreeMap::new();
    for name in PBANKS {
        let path = format!("{dir}/{name}");
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let bank: WitnessSet = serde_json::from_str(&text).unwrap();
        for w in &bank.witnesses {
            let x = match &w.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
                _ => continue,
            };
            let Some(e) = parse_bits_hex(&w.expected_bits) else {
                continue;
            };
            if x > 0.0 {
                pmap.entry(x.to_bits()).or_insert(e.to_bits());
            }
        }
    }
    let qrows = f::load_q_rows_tagged(&dir);
    let mut n_lo = 0usize;
    let mut n_hi = 0usize;
    let mut q_eq_1p_lo = 0usize;
    let mut q_eq_1p_hi = 0usize;
    let mut p_eq_1q_lo = 0usize;
    let mut p_eq_1q_hi = 0usize;
    let mut q_d1_lo = 0usize;
    let mut q_d1_hi = 0usize;
    let mut shown = 0usize;
    for r in &qrows {
        let Some(&pbits) = pmap.get(&r.z.to_bits()) else {
            continue;
        };
        let p = f64::from_bits(pbits);
        let q = f64::from_bits(r.qbits);
        let one_p = 1.0 - p;
        let one_q = 1.0 - q;
        let dq = ulp_distance(one_p, q).unwrap_or(99);
        let dp = ulp_distance(one_q, p).unwrap_or(99);
        if r.z < 0.5 {
            n_lo += 1;
            if dq == 0 {
                q_eq_1p_lo += 1;
            } else if dq == 1 {
                q_d1_lo += 1;
            }
            if dp == 0 {
                p_eq_1q_lo += 1;
            }
            if dq >= 2 && shown < 8 {
                println!("  z<0.5 Q vs 1-P ulp={dq} z={:.16} direct={}", r.z, r.direct);
                shown += 1;
            }
        } else {
            n_hi += 1;
            if dq == 0 {
                q_eq_1p_hi += 1;
            } else if dq == 1 {
                q_d1_hi += 1;
            }
            if dp == 0 {
                p_eq_1q_hi += 1;
            }
        }
    }
    println!("shared z<0.5 n={n_lo}  Q==1-P {q_eq_1p_lo}  Q 1ulp {q_d1_lo}  P==1-Q {p_eq_1q_lo}");
    println!("shared z>=0.5 n={n_hi}  Q==1-P {q_eq_1p_hi}  Q 1ulp {q_d1_hi}  P==1-Q {p_eq_1q_hi}");
    println!(
        "  (identity: z<0.5 Q=RN(1-P); z>=0.5 P=RN(1-Q))"
    );
}
