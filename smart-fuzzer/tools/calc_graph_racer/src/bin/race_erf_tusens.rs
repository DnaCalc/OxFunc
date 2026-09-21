//! Cephes T/U x87 ±1 keep-and-hit vs 4 LOW. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const T0: [f64; 5] = [
    9.60497373987051638749e0,
    9.00260197203842689217e1,
    2.23200534594684319226e3,
    7.00332514112805075473e3,
    5.55923013010394962768e4,
];
const U0: [f64; 5] = [
    3.35617141647503099647e1,
    5.21357949780152679795e2,
    4.59432382970980127987e3,
    2.26290000613890934246e4,
    4.92673942608635921086e4,
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
fn tu_x87(z: f64, t: &[f64; 5], u: &[f64; 5]) -> f64 {
    let ze = ef(z);
    let zz = ext_mul(&ze, &ze, CW);
    let mut num = ef(t[0]);
    for &c in &t[1..] {
        num = ext_add(&ext_mul(&num, &zz, CW), &ef(c), CW);
    }
    let mut den = ext_add(&zz, &ef(u[0]), CW);
    for &c in &u[1..] {
        den = ext_add(&ext_mul(&den, &zz, CW), &ef(c), CW);
    }
    ext_to_f64(&ext_div(&ext_mul(&ze, &num, CW), &den, CW), CW)
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
    let keep = |t: &[f64; 5], u: &[f64; 5]| -> usize {
        rows.iter()
            .filter(|&&(z, bits)| ulp_distance(tu_x87(z, t, u), f64::from_bits(bits)).unwrap_or(99) == 0)
            .count()
    };
    let hit4 = |t: &[f64; 5], u: &[f64; 5]| -> (usize, Vec<f64>) {
        let mut h = 0usize;
        let mut zs = Vec::new();
        for &lz in &LOWS {
            let Some((_, bits)) = rows.iter().find(|(z, _)| (*z - lz).abs() < 1e-14) else {
                continue;
            };
            if ulp_distance(tu_x87(lz, t, u), f64::from_bits(*bits)).unwrap_or(99) == 0 {
                h += 1;
                zs.push(lz);
            }
        }
        (h, zs)
    };
    println!("4 LOW Cephes T/U x87 CR:");
    for &lz in &LOWS {
        let Some((_, bits)) = rows.iter().find(|(z, _)| (*z - lz).abs() < 1e-14) else {
            continue;
        };
        let tgt = f64::from_bits(*bits);
        let g = tu_x87(lz, &T0, &U0);
        let d = ulp_distance(g, tgt).unwrap_or(99);
        let dir = if g < tgt {
            "L"
        } else if g > tgt {
            "H"
        } else {
            "="
        };
        println!("  z={lz:.16} {d}{dir}");
    }
    let k0 = keep(&T0, &U0);
    let (h0, _) = hit4(&T0, &U0);
    println!("T/U x87 CR keep={k0}/{} hit={h0}/4 (bar joint 866)", rows.len());
    println!("±1 (print hit>0 or keep>=k0):");
    let mut best_h = h0;
    let mut best_k = k0;
    let mut lab = "CR".to_string();
    for i in 0..5 {
        for k in [-1i32, 1] {
            let mut t = T0;
            t[i] = poke(T0[i], k);
            let kk = keep(&t, &U0);
            let (h, zs) = hit4(&t, &U0);
            if h > 0 || kk >= k0 {
                println!("  T[{i}] {k:+} keep={kk} hit={h}/4 {zs:?}");
            }
            if h > best_h || (h == best_h && kk > best_k) {
                best_h = h;
                best_k = kk;
                lab = format!("T[{i}] {k:+}");
            }
        }
    }
    for i in 0..5 {
        for k in [-1i32, 1] {
            let mut u = U0;
            u[i] = poke(U0[i], k);
            let kk = keep(&T0, &u);
            let (h, zs) = hit4(&T0, &u);
            if h > 0 || kk >= k0 {
                println!("  U[{i}] {k:+} keep={kk} hit={h}/4 {zs:?}");
            }
            if h > best_h || (h == best_h && kk > best_k) {
                best_h = h;
                best_k = kk;
                lab = format!("U[{i}] {k:+}");
            }
        }
    }
    println!("best-by-hit {lab} keep={best_k} hit={best_h}");
}
