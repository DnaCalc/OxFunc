//! A21 joint HW=1 stores vs P-side hard 46 hits. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const NSITE: u32 = 25;
const AS0: [f64; 21] = [
    0.1283791670955125738961589031215,
    -0.3761263890318375246320529677070,
    0.1128379167095512573896158902931,
    -0.2686617064513125175943235372542e-01,
    0.5223977625442187842111812447877e-02,
    -0.8548327023450852832540164081187e-03,
    0.1205533298178966425020717182498e-03,
    -0.1492565035840625090430728526820e-04,
    0.1646211436588924261080723578109e-05,
    -0.1636584469123468757408968429674e-06,
    0.1480719281587021715400818627811e-07,
    -0.1229055530145120140800510155331e-08,
    0.9422759058437197017313055084212e-10,
    -0.6711366740969385085896257227159e-11,
    0.4463222608295664017461758843550e-12,
    -0.2783497395542995487275065856998e-13,
    0.1634095572365337143933023780777e-14,
    -0.9052845786901123985710019387938e-16,
    0.4708274559689744439341671426731e-17,
    -0.2187159356685015949749948252160e-18,
    0.7043407712019701609635599701333e-20,
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
fn joint_a() -> [f64; 21] {
    let mut a = AS0;
    a[0] = poke(AS0[0], 4);
    a[1] = poke(AS0[1], -2);
    a[2] = poke(AS0[2], -5);
    a[3] = poke(AS0[3], 1);
    a
}
fn maybe(x: Ext80, mask: u32, bit: u32) -> Ext80 {
    if bit < 32 && mask & (1u32 << bit) != 0 {
        ef(ext_to_f64(&x, CW))
    } else {
        x
    }
}
fn erf_j(z: f64, a: &[f64; 21], mask: u32) -> f64 {
    let xe = ef(z.abs());
    let mut u = ext_mul(&xe, &xe, CW);
    u = maybe(u, mask, 0);
    let mut acc = ef(0.0);
    acc = maybe(acc, mask, 1);
    for (i, &c) in a.iter().rev().enumerate() {
        acc = ext_add(&ext_mul(&acc, &u, CW), &ef(c), CW);
        acc = maybe(acc, mask, 2 + i as u32);
    }
    let mut s = ext_add(&ef(1.0), &acc, CW);
    s = maybe(s, mask, 23);
    let mut p = ext_mul(&xe, &s, CW);
    p = maybe(p, mask, 24);
    ext_to_f64(&p, CW)
}

#[derive(Clone, Copy, Default)]
struct Sc {
    exact: usize,
    n: usize,
    max_ulp: u64,
}
fn score(rows: &[(f64, u64)], a: &[f64; 21], mask: u32) -> Sc {
    let mut s = Sc::default();
    for &(z, pbits) in rows {
        let pg = erf_j(z, a, mask);
        if !pg.is_finite() {
            continue;
        }
        let d = ulp_distance(pg, f64::from_bits(pbits)).unwrap_or(u64::MAX);
        if d > ULP_CAP {
            continue;
        }
        s.n += 1;
        if d == 0 {
            s.exact += 1;
        } else {
            s.max_ulp = s.max_ulp.max(d);
        }
    }
    s
}
fn better(a: Sc, b: Sc) -> bool {
    a.exact > b.exact || (a.exact == b.exact && a.max_ulp < b.max_ulp)
}
fn fmt(s: Sc) -> String {
    format!("{}/{} max={}", s.exact, s.n, s.max_ulp)
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
            if x < 0.0 || x >= 0.5 {
                continue;
            }
            let Some(p) = parse_bits_hex(&w.expected_bits) else {
                continue;
            };
            map.insert(x.to_bits(), (x, p.to_bits()));
        }
    }
    let rows: Vec<_> = map.into_values().collect();
    let a = joint_a();
    let base = score(&rows, &a, 0);
    println!("joint mask0 {}", fmt(base));
    println!("## ulp>=2 of joint mask0");
    for &(z, pbits) in &rows {
        let pg = erf_j(z, &a, 0);
        let d = ulp_distance(pg, f64::from_bits(pbits)).unwrap_or(99);
        if d >= 2 {
            println!("  z={:.17} ulp={d}", z);
        }
    }
    let hard: Vec<(f64, u64)> = rows
        .iter()
        .copied()
        .filter(|&(z, pbits)| {
            ulp_distance(erf_j(z, &a, 0), f64::from_bits(pbits)).unwrap_or(99) >= 2
        })
        .collect();
    println!("hard n={}", hard.len());
    println!("## HW=1 exact / hard_hit (bar 866 / 0/46)");
    let mut best_h = 0usize;
    let mut best_m = 0u32;
    for b in 0..NSITE {
        let m = 1u32 << b;
        let sc = score(&rows, &a, m);
        let mut hit = 0usize;
        for &(z, pbits) in &hard {
            if ulp_distance(erf_j(z, &a, m), f64::from_bits(pbits)).unwrap_or(99) == 0 {
                hit += 1;
            }
        }
        if hit > best_h {
            best_h = hit;
            best_m = m;
        }
        if hit > 0 || sc.exact >= 866 {
            println!("  bit={b:2} mask={m:#07x} {} hard_hit={hit}/{}", fmt(sc), hard.len());
        }
    }
    println!("best hard_hit HW1 {best_h} {best_m:#07x}");
    let _ = better;
}
