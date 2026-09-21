//! NORM.S.DIST vs 0.5*ERFC.PRECISE / 0.5+0.5*ERF.PRECISE on overlapping
//! frozen banks. Direct ERFC bits win. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_from_f64, ext_mul, ext_to_f64, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const SQRT2: f64 = std::f64::consts::SQRT_2;
const INVSQRT2: f64 = std::f64::consts::FRAC_1_SQRT_2;

fn load_bank(path: &str) -> WitnessSet {
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let mut erfc: BTreeMap<u64, (u64, bool)> = BTreeMap::new();
    for r in f::load_q_rows_tagged(&dir) {
        let e = erfc.entry(r.z.to_bits()).or_insert((r.qbits, r.direct));
        if r.direct && !e.1 {
            *e = (r.qbits, true);
        }
    }
    let mut erf: BTreeMap<u64, u64> = BTreeMap::new();
    for name in [
        "answers-b9train.json",
        "answers-erfp.json",
        "answers-erfm.json",
        "answers-b8erf.json",
        "answers-b7erf.json",
        "answers-b11.json",
        "answers-b10.json",
    ] {
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
                erf.entry(x.to_bits()).or_insert(e.to_bits());
            }
        }
    }
    let bank = load_bank(&format!("{dir}/answers-b24-normref.json"));
    let mut norms = Vec::new();
    for w in &bank.witnesses {
        let x = match &w.args[0] {
            WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
            _ => continue,
        };
        let cum = match &w.args[1] {
            WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
            _ => continue,
        };
        if cum != 1.0 {
            continue;
        }
        let Some(e) = parse_bits_hex(&w.expected_bits) else {
            continue;
        };
        norms.push((x, e.to_bits()));
    }
    println!(
        "NORM rows={} ERFC keys={} ERF keys={}",
        norms.len(),
        erfc.len(),
        erf.len()
    );

    let z_fns: [(&str, fn(f64) -> f64); 4] = [
        ("-x*invsqrt2 f64", |x| -x * INVSQRT2),
        ("-x/sqrt2 f64", |x| -x / SQRT2),
        ("x87 (-x)*invsqrt2", |x| {
            ext_to_f64(&ext_mul(&ext_from_f64(-x), &ext_from_f64(INVSQRT2), CW), CW)
        }),
        ("x87 - (x*invsqrt2)", |x| {
            -ext_to_f64(&ext_mul(&ext_from_f64(x), &ext_from_f64(INVSQRT2), CW), CW)
        }),
    ];
    let half_fns: [(&str, fn(f64) -> f64); 3] = [
        ("0.5*q f64", |q| 0.5 * q),
        ("x87 0.5*q", |q| {
            ext_to_f64(&ext_mul(&ext_from_f64(0.5), &ext_from_f64(q), CW), CW)
        }),
        ("q/2 f64", |q| q / 2.0),
    ];

    for (zn, zf) in z_fns {
        for (hn, hf) in half_fns {
            let mut ex = 0usize;
            let mut n = 0usize;
            let mut max = 0u64;
            let mut hit = 0usize;
            for &(x, bits) in &norms {
                let z = zf(x);
                let q = if let Some(&(qb, _)) = erfc.get(&z.to_bits()) {
                    f64::from_bits(qb)
                } else if let Some(&(qb, _)) = erfc.get(&z.abs().to_bits()) {
                    let e = f64::from_bits(qb);
                    if z >= 0.0 {
                        e
                    } else {
                        2.0 - e
                    }
                } else {
                    continue;
                };
                hit += 1;
                let g = hf(q);
                let d = ulp_distance(g, f64::from_bits(bits)).unwrap_or(u64::MAX);
                if d > ULP_CAP {
                    continue;
                }
                n += 1;
                if d == 0 {
                    ex += 1;
                } else {
                    max = max.max(d);
                }
            }
            println!("{zn:24} {hn:14} overlap={hit} exact {ex}/{n} max={max}");
            if zn.starts_with("-x*invsqrt2") && hn.starts_with("0.5*q") && ex + 1 == n {
                for &(x, bits) in &norms {
                    let z = -x * INVSQRT2;
                    let (qb, direct) = if let Some(&p) = erfc.get(&z.to_bits()) {
                        p
                    } else if let Some(&p) = erfc.get(&z.abs().to_bits()) {
                        p
                    } else {
                        continue;
                    };
                    let q = if z >= 0.0 || erfc.contains_key(&z.to_bits()) {
                        f64::from_bits(qb)
                    } else {
                        2.0 - f64::from_bits(qb)
                    };
                    let g = 0.5 * q;
                    let d = ulp_distance(g, f64::from_bits(bits)).unwrap_or(u64::MAX);
                    if d != 0 {
                        println!(
                            "  MISS x={:.17} z={:.17} d={d} erfc_direct={direct} q={:016x} norm={:016x} g={:016x}",
                            x, z, qb, bits, g.to_bits()
                        );
                    }
                }
            }
        }
    }

    println!("\n## 0.5+0.5*ERF.PRECISE(x*invsqrt2)");
    let mut ex = 0usize;
    let mut n = 0usize;
    let mut max = 0u64;
    let mut hit = 0usize;
    for &(x, bits) in &norms {
        let z = (x * INVSQRT2).abs();
        let Some(&eb) = erf.get(&z.to_bits()) else {
            continue;
        };
        hit += 1;
        let e = f64::from_bits(eb);
        let g = if x >= 0.0 {
            0.5 + 0.5 * e
        } else {
            0.5 - 0.5 * e
        };
        let d = ulp_distance(g, f64::from_bits(bits)).unwrap_or(u64::MAX);
        if d > ULP_CAP {
            continue;
        }
        n += 1;
        if d == 0 {
            ex += 1;
        } else {
            max = max.max(d);
        }
    }
    println!("ERF overlap={hit} exact {ex}/{n} max={max}");
}
