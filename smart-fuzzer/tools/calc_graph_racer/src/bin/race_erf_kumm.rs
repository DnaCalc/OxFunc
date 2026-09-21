//! P-side Kummer M(1,3/2,z²) with w_rn53 vs libm exp; overlap vs A21-joint. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sqrt, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
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
const TWOSQPI: f64 = 1.1283791670955125739;

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
fn erf_a(z: f64, a: &[f64; 21]) -> f64 {
    let xe = ef(z);
    let t = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW), &ef(c), CW);
    }
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &acc, CW), CW), CW)
}
fn m132(z2: f64, n: usize) -> f64 {
    let mut term = 1.0;
    let mut m = 1.0;
    for k in 1..n {
        term *= z2 / (k as f64 + 0.5);
        m += term;
        if term.abs() < 1e-22 {
            break;
        }
    }
    m
}
fn m132_80(z2: Ext80, n: usize) -> Ext80 {
    let mut term = ef(1.0);
    let mut m = ef(1.0);
    for k in 1..n {
        term = ext_mul(&term, &ext_from_f64(1.0 / (k as f64 + 0.5)), CW);
        term = ext_mul(&term, &z2, CW);
        m = ext_add(&m, &term, CW);
    }
    m
}
fn kumm_libm(z: f64, n: usize) -> f64 {
    TWOSQPI * z * (-z * z).exp() * m132(z * z, n)
}
fn kumm_w(z: f64, n: usize) -> f64 {
    TWOSQPI * z * f::w_rn53(z) * m132(z * z, n)
}
fn kumm_w80(z: f64, n: usize) -> f64 {
    let ze = ef(z);
    let z2 = ext_mul(&ze, &ze, CW);
    let m = m132_80(z2, n);
    let mut acc = ext_mul(&ef(TWOSQPI), &ze, CW);
    acc = ext_mul(&acc, &ef(f::w_rn53(z)), CW);
    acc = ext_mul(&acc, &m, CW);
    ext_to_f64(&acc, CW)
}
fn kumm_lead80(z: f64, n: usize) -> f64 {
    let ze = ef(z);
    let z2 = ext_mul(&ze, &ze, CW);
    let m = m132_80(z2, n);
    let two = ext_div(&ef(2.0), &ext_sqrt(&ef(std::f64::consts::PI), CW), CW);
    let mut acc = ext_mul(&two, &ze, CW);
    acc = ext_mul(&acc, &ef(f::w_rn53(z)), CW);
    acc = ext_mul(&acc, &m, CW);
    ext_to_f64(&acc, CW)
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
    let mut joint = AS0;
    joint[0] = poke(AS0[0], 4);
    joint[1] = poke(AS0[1], -2);
    joint[2] = poke(AS0[2], -5);
    joint[3] = poke(AS0[3], 1);
    let lows = [
        0.23046875f64,
        0.37109375,
        0.4524739583333333,
        0.4716796875,
    ];

    let graphs: [(&str, Box<dyn Fn(f64) -> f64>); 8] = [
        ("joint", Box::new(|z| erf_a(z, &joint))),
        ("k libm24", Box::new(|z| kumm_libm(z, 24))),
        ("k w12", Box::new(|z| kumm_w(z, 12))),
        ("k w24", Box::new(|z| kumm_w(z, 24))),
        ("k w40", Box::new(|z| kumm_w(z, 40))),
        ("k w80-24", Box::new(|z| kumm_w80(z, 24))),
        ("k lead80", Box::new(|z| kumm_lead80(z, 24))),
        ("k w*stM", Box::new(|z| TWOSQPI * z * f::w_rn53(z) * m132(z * z, 24))),
    ];
    println!("P-side n={}", rows.len());
    for (name, ev) in &graphs {
        let mut keep = 0usize;
        let mut mx = 0u64;
        let mut hit = 0usize;
        let mut u1 = 0usize;
        for &(z, bits) in &rows {
            let d = ulp_distance(ev(z), f64::from_bits(bits)).unwrap_or(99);
            if d == 0 {
                keep += 1;
            } else {
                mx = mx.max(d);
            }
        }
        for &z in &lows {
            let t = rows.iter().find(|r| (r.0 - z).abs() < 1e-15).map(|r| f64::from_bits(r.1));
            let Some(t) = t else { continue };
            let d = ulp_distance(ev(z), t).unwrap_or(99);
            if d == 0 {
                hit += 1;
                println!("  HIT {name} z={z:.16}");
            } else if d == 1 {
                u1 += 1;
            }
        }
        println!("{name:10} keep={keep}/{} max={mx} LOW hit={hit}/4 ulp1={u1}", rows.len());
    }

    let j = |z: f64| erf_a(z, &joint);
    let kw = |z: f64| kumm_w(z, 24);
    let mut both = 0usize;
    let mut only_j = 0usize;
    let mut only_k = 0usize;
    let mut only_k_u1j = 0usize;
    let mut shown = 0usize;
    for &(z, bits) in &rows {
        let t = f64::from_bits(bits);
        let dj = ulp_distance(j(z), t).unwrap_or(99);
        let dk = ulp_distance(kw(z), t).unwrap_or(99);
        match (dj == 0, dk == 0) {
            (true, true) => both += 1,
            (true, false) => only_j += 1,
            (false, true) => {
                only_k += 1;
                if dj == 1 {
                    only_k_u1j += 1;
                }
                if shown < 12 {
                    println!("  only_kumm z={z:.16} j_ulp={dj} k{}excel", if kw(z) < t { "<" } else { ">" });
                    shown += 1;
                }
            }
            _ => {}
        }
    }
    println!(
        "overlap joint vs kumm_w24 both={both} only_j={only_j} only_k={only_k} (j ulp1={only_k_u1j}) union={}",
        both + only_j + only_k
    );

    println!("kumm-then-joint / joint-then-kumm cuts (bar 866):");
    let cuts: [(&str, Box<dyn Fn(f64) -> f64>, Box<dyn Fn(f64) -> f64>); 2] = [
        ("k-then-j", Box::new(kw), Box::new(j)),
        ("j-then-k", Box::new(j), Box::new(kw)),
    ];
    for (tag, lo, hi) in &cuts {
        let mut best = 0usize;
        let mut best_c = 0.0;
        let mut best_hit = 0usize;
        for k in 1..50 {
            let c = k as f64 * 0.01;
            let mut ex = 0usize;
            let mut hit = 0usize;
            for &(z, bits) in &rows {
                let g = if z < c { lo(z) } else { hi(z) };
                if ulp_distance(g, f64::from_bits(bits)).unwrap_or(99) == 0 {
                    ex += 1;
                }
            }
            for &z in &lows {
                let t = rows
                    .iter()
                    .find(|r| (r.0 - z).abs() < 1e-15)
                    .map(|r| f64::from_bits(r.1));
                let Some(t) = t else { continue };
                let g = if z < c { lo(z) } else { hi(z) };
                if ulp_distance(g, t).unwrap_or(99) == 0 {
                    hit += 1;
                }
            }
            if ex > best || (ex == best && hit > best_hit) {
                best = ex;
                best_c = c;
                best_hit = hit;
            }
        }
        println!("  {tag} best {best} @ {best_c:.2} LOW hit={best_hit}/4");
    }
}
