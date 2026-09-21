//! fdlibm P[0]+1 (keep 643) plus one ±1 vs 4 LOW. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const PP: [f64; 5] = [
    1.28379167095512558561e-01,
    -3.25042107247001499370e-01,
    -2.84817495755985104766e-02,
    -5.77027029648944159157e-03,
    -2.37630166566501626084e-05,
];
const QQ: [f64; 5] = [
    3.97917223959155352819e-01,
    6.50222499887672944485e-02,
    5.08130628187576562776e-03,
    1.32494738004321644526e-04,
    -3.96022827877536812320e-06,
];
const LOWS: [f64; 4] = [
    0.23046875,
    0.37109375,
    0.4524739583333333,
    0.4716796875,
];

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn poke(x: f64, k: i32) -> f64 {
    let mut v = x;
    if k > 0 {
        for _ in 0..k {
            v = v.next_up();
        }
    } else {
        for _ in 0..(-k) {
            v = v.next_down();
        }
    }
    v
}
fn fd_x87(z: f64, p: &[f64; 5], q: &[f64; 5]) -> f64 {
    let ze = ef(z);
    let zz = ext_mul(&ze, &ze, CW);
    let mut r = ef(0.0);
    for &c in p.iter().rev() {
        r = ext_add(&ext_mul(&r, &zz, CW), &ef(c), CW);
    }
    let mut s = ef(0.0);
    for &c in q.iter().rev() {
        s = ext_add(&ext_mul(&s, &zz, CW), &ef(c), CW);
    }
    let den = ext_add(&ef(1.0), &ext_mul(&zz, &s, CW), CW);
    let y = ext_div(&r, &den, CW);
    ext_to_f64(&ext_add(&ze, &ext_mul(&ze, &y, CW), CW), CW)
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    const BANKS: [&str; 7] = [
        "answers-b9train.json",
        "answers-erfp.json",
        "answers-erfm.json",
        "answers-b8erf.json",
        "answers-b7erf.json",
        "answers-b11.json",
        "answers-b10.json",
    ];
    let mut map = BTreeMap::new();
    for name in BANKS {
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
            if x > 0.0 && x < 0.5 {
                map.entry(x.to_bits()).or_insert(e.to_bits());
            }
        }
    }
    let rows: Vec<_> = map
        .into_iter()
        .map(|(b, e)| (f64::from_bits(b), e))
        .collect();
    let mut p0 = PP;
    p0[0] = poke(PP[0], 1);
    let keep = |p: &[f64; 5], q: &[f64; 5]| -> usize {
        rows.iter()
            .filter(|&&(z, bits)| ulp_distance(fd_x87(z, p, q), f64::from_bits(bits)).unwrap_or(99) == 0)
            .count()
    };
    let hit4 = |p: &[f64; 5], q: &[f64; 5]| -> (usize, Vec<f64>) {
        let mut h = 0usize;
        let mut zs = Vec::new();
        for &lz in &LOWS {
            let Some((_, bits)) = rows.iter().find(|(z, _)| (*z - lz).abs() < 1e-14) else {
                continue;
            };
            if ulp_distance(fd_x87(lz, p, q), f64::from_bits(*bits)).unwrap_or(99) == 0 {
                h += 1;
                zs.push(lz);
            }
        }
        (h, zs)
    };
    let k0 = keep(&p0, &QQ);
    let (h0, _) = hit4(&p0, &QQ);
    println!("P[0]+1 keep={k0} hit={h0}/4 (bar CR 544 / joint 866)");
    println!("extra ±1 (print hit>0 or keep>=k0):");
    let mut best_h = h0;
    let mut best_k = k0;
    let mut lab = "P[0]+1".to_string();
    for i in 1..5 {
        for k in [-1i32, 1] {
            let mut p = p0;
            p[i] = poke(PP[i], k);
            let kk = keep(&p, &QQ);
            let (h, zs) = hit4(&p, &QQ);
            if h > 0 || kk >= k0 {
                println!("  P[{i}] {k:+} keep={kk} hit={h}/4 {zs:?}");
            }
            if h > best_h || (h == best_h && kk > best_k) {
                best_h = h;
                best_k = kk;
                lab = format!("P[0]+1 P[{i}] {k:+}");
            }
        }
    }
    for i in 0..5 {
        for k in [-1i32, 1] {
            let mut q = QQ;
            q[i] = poke(QQ[i], k);
            let kk = keep(&p0, &q);
            let (h, zs) = hit4(&p0, &q);
            if h > 0 || kk >= k0 {
                println!("  Q[{i}] {k:+} keep={kk} hit={h}/4 {zs:?}");
            }
            if h > best_h || (h == best_h && kk > best_k) {
                best_h = h;
                best_k = kk;
                lab = format!("P[0]+1 Q[{i}] {k:+}");
            }
        }
    }
    // also P[0]+2..+8 like A21 A[0] joint
    println!("P[0] extra steps from CR:");
    for k in 2..=8 {
        let mut p = PP;
        p[0] = poke(PP[0], k);
        let kk = keep(&p, &QQ);
        let (h, zs) = hit4(&p, &QQ);
        println!("  P[0] {k:+} keep={kk} hit={h}/4 {zs:?}");
        if h > best_h || (h == best_h && kk > best_k) {
            best_h = h;
            best_k = kk;
            lab = format!("P[0] {k:+}");
        }
    }
    println!("best-by-hit {lab} keep={best_k} hit={best_h}");
}
