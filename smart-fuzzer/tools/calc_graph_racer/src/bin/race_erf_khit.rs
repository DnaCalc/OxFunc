//! Keep-and-hit on A21-joint vs P-side +2 ULP leftover. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
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

    let mut hist = [0usize; 6];
    let mut d2_low: Vec<(f64, u64)> = Vec::new();
    let mut d2_high: Vec<(f64, u64)> = Vec::new();
    let mut keep0 = 0usize;
    for &(z, bits) in &rows {
        let t = f64::from_bits(bits);
        let g = erf_a(z, &joint);
        let d = ulp_distance(g, t).unwrap_or(99).min(5) as usize;
        hist[d] += 1;
        if d == 0 {
            keep0 += 1;
        } else if d == 2 {
            if g < t {
                d2_low.push((z, bits));
            } else {
                d2_high.push((z, bits));
            }
        }
    }
    println!(
        "joint ulp hist 0..5+ = {} {} {} {} {} {}  n={} keep={keep0}",
        hist[0], hist[1], hist[2], hist[3], hist[4], hist[5], rows.len()
    );
    println!("joint +2 LOW n={} HIGH n={}", d2_low.len(), d2_high.len());
    for &(z, bits) in &d2_low {
        println!(
            "  LOW z={z:.16} zbits={:#x} excel={bits:#x} dyadic={}",
            z.to_bits(),
            z.to_bits() & ((1u64 << 20) - 1) == 0
        );
    }
    println!("HIGH +2 (first 8):");
    for &(z, _) in d2_low.iter().take(0).chain(d2_high.iter().take(8)) {
        println!("  HIGH z={z:.16}");
    }

    let lows = [
        0.23046875f64,
        0.37109375,
        0.4524739583333333,
        0.4716796875,
    ];
    let hit4 = |a: &[f64; 21]| -> usize {
        let mut h = 0usize;
        for &(z, bits) in &rows {
            if lows.iter().all(|&t| (z - t).abs() > 1e-14) {
                continue;
            }
            if ulp_distance(erf_a(z, a), f64::from_bits(bits)).unwrap_or(99) == 0 {
                h += 1;
            }
        }
        h
    };
    let keep = |a: &[f64; 21]| -> usize {
        rows.iter()
            .filter(|&&(z, bits)| ulp_distance(erf_a(z, a), f64::from_bits(bits)).unwrap_or(99) == 0)
            .count()
    };
    println!("base keep={} hit4={}", keep(&joint), hit4(&joint));

    println!("one extra coeff ±1..8 from joint (print keep>=860 or hit>0):");
    let mut best_keep = keep0;
    let mut best_hit = 0usize;
    let mut best_lab = "base".to_string();
    for i in 0..21 {
        for k in -8i32..=8 {
            if k == 0 {
                continue;
            }
            let mut a = joint;
            a[i] = poke(joint[i], k);
            let kk = keep(&a);
            let h = hit4(&a);
            if h > 0 || kk >= 860 {
                if h > 0 || kk > keep0 {
                    println!("  A[{i}] {k:+} keep={kk} hit={h}/4");
                }
            }
            if h > best_hit || (h == best_hit && kk > best_keep) {
                best_hit = h;
                best_keep = kk;
                best_lab = format!("A[{i}] {k:+}");
            }
        }
    }
    println!("best-by-hit {best_lab} keep={best_keep} hit={best_hit}");

    println!("ham2 extra ±1 from joint (print hit>0 or keep>866):");
    let mut hbest_hit = best_hit;
    let mut hbest_keep = best_keep;
    let mut hlab = best_lab.clone();
    for i in 0..21 {
        for j in (i + 1)..21 {
            for si in [-1i32, 1] {
                for sj in [-1i32, 1] {
                    let mut a = joint;
                    a[i] = poke(joint[i], si);
                    a[j] = poke(joint[j], sj);
                    let kk = keep(&a);
                    let h = hit4(&a);
                    if h > 0 || kk > keep0 {
                        println!("  A[{i}]{si:+} A[{j}]{sj:+} keep={kk} hit={h}/4");
                    }
                    if h > hbest_hit || (h == hbest_hit && kk > hbest_keep) {
                        hbest_hit = h;
                        hbest_keep = kk;
                        hlab = format!("A[{i}]{si:+} A[{j}]{sj:+}");
                    }
                }
            }
        }
    }
    println!("ham2 best-by-hit {hlab} keep={hbest_keep} hit={hbest_hit}");
}
