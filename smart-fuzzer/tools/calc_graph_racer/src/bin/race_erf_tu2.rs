//! Cephes U[4]−1 extra ±1 and vs joint overlap. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
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
fn erf_a(z: f64, a: &[f64; 21]) -> f64 {
    let xe = ef(z);
    let tt = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &tt, CW), &ef(c), CW);
    }
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &acc, CW), CW), CW)
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
    let mut u4 = U0;
    u4[4] = poke(U0[4], -1);
    let mut joint = AS0;
    joint[0] = poke(AS0[0], 4);
    joint[1] = poke(AS0[1], -2);
    joint[2] = poke(AS0[2], -5);
    joint[3] = poke(AS0[3], 1);
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
    let k0 = keep(&T0, &u4);
    let (h0, _) = hit4(&T0, &u4);
    println!("U[4]-1 keep={k0} hit={h0}/4 (bar CR 380 / joint 866)");
    println!("4 LOW U[4]-1 vs joint:");
    for &lz in &LOWS {
        let Some((_, bits)) = rows.iter().find(|(z, _)| (*z - lz).abs() < 1e-14) else {
            continue;
        };
        let tgt = f64::from_bits(*bits);
        let gu = tu_x87(lz, &T0, &u4);
        let gj = erf_a(lz, &joint);
        let du = ulp_distance(gu, tgt).unwrap_or(99);
        let dj = ulp_distance(gj, tgt).unwrap_or(99);
        let su = if gu < tgt {
            "L"
        } else if gu > tgt {
            "H"
        } else {
            "="
        };
        let sj = if gj < tgt {
            "L"
        } else if gj > tgt {
            "H"
        } else {
            "="
        };
        println!("  z={lz:.16} TU={du}{su} joint={dj}{sj}");
    }
    println!("extra ±1 from U[4]-1 (print hit>0 or keep>=k0):");
    let mut best_h = h0;
    let mut best_k = k0;
    let mut lab = "U[4]-1".to_string();
    for i in 0..5 {
        for k in [-1i32, 1] {
            let mut t = T0;
            t[i] = poke(T0[i], k);
            let kk = keep(&t, &u4);
            let (h, zs) = hit4(&t, &u4);
            if h > 0 || kk >= k0 {
                println!("  T[{i}] {k:+} keep={kk} hit={h}/4 {zs:?}");
            }
            if h > best_h || (h == best_h && kk > best_k) {
                best_h = h;
                best_k = kk;
                lab = format!("U[4]-1 T[{i}] {k:+}");
            }
        }
    }
    for i in 0..4 {
        for k in [-1i32, 1] {
            let mut u = u4;
            u[i] = poke(U0[i], k);
            let kk = keep(&T0, &u);
            let (h, zs) = hit4(&T0, &u);
            if h > 0 || kk >= k0 {
                println!("  U[{i}] {k:+} keep={kk} hit={h}/4 {zs:?}");
            }
            if h > best_h || (h == best_h && kk > best_k) {
                best_h = h;
                best_k = kk;
                lab = format!("U[4]-1 U[{i}] {k:+}");
            }
        }
    }
    println!("best-by-hit {lab} keep={best_k} hit={best_h}");
    let mut both = 0usize;
    let mut only_j = 0usize;
    let mut only_u = 0usize;
    let mut only_u_u1j = 0usize;
    for &(z, bits) in &rows {
        let tgt = f64::from_bits(bits);
        let dj = ulp_distance(erf_a(z, &joint), tgt).unwrap_or(99);
        let du = ulp_distance(tu_x87(z, &T0, &u4), tgt).unwrap_or(99);
        match (dj == 0, du == 0) {
            (true, true) => both += 1,
            (true, false) => only_j += 1,
            (false, true) => {
                only_u += 1;
                if dj == 1 {
                    only_u_u1j += 1;
                }
            }
            _ => {}
        }
    }
    println!(
        "overlap both={both} only_j={only_j} only_U4={only_u} (j ulp1={only_u_u1j}) union={} n={}",
        both + only_j + only_u,
        rows.len()
    );
}
