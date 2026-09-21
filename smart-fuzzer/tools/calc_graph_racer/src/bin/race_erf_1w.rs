//! P-side joint 1+w inner-store nudge vs leftover-low 13. Not an identity.
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
fn w_x87(z: f64, a: &[f64; 21]) -> Ext80 {
    let t = ext_mul(&ef(z), &ef(z), CW);
    let mut acc = ef(0.0);
    for &c in a.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW), &ef(c), CW);
    }
    acc
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
    let erf_cont = |z: f64| {
        let w = w_x87(z, &joint);
        ext_to_f64(&ext_mul(&ef(z), &ext_add(&ef(1.0), &w, CW), CW), CW)
    };
    let evals: [(&str, Box<dyn Fn(f64) -> f64>); 7] = [
        ("x*(1+w) cont", Box::new(erf_cont)),
        (
            "x*next_up(1+w)",
            Box::new(|z| {
                let w = w_x87(z, &joint);
                let s = ext_to_f64(&ext_add(&ef(1.0), &w, CW), CW).next_up();
                z * s
            }),
        ),
        (
            "x*next_down(1+w)",
            Box::new(|z| {
                let w = w_x87(z, &joint);
                let s = ext_to_f64(&ext_add(&ef(1.0), &w, CW), CW).next_down();
                z * s
            }),
        ),
        (
            "next_up(w) then 1+",
            Box::new(|z| {
                let w = ext_to_f64(&w_x87(z, &joint), CW).next_up();
                ext_to_f64(&ext_mul(&ef(z), &ext_add(&ef(1.0), &ef(w), CW), CW), CW)
            }),
        ),
        (
            "store 1+w then *x x87",
            Box::new(|z| {
                let w = w_x87(z, &joint);
                let s = ef(ext_to_f64(&ext_add(&ef(1.0), &w, CW), CW));
                ext_to_f64(&ext_mul(&ef(z), &s, CW), CW)
            }),
        ),
        (
            "x + x*w",
            Box::new(|z| {
                let w = w_x87(z, &joint);
                ext_to_f64(&ext_add(&ef(z), &ext_mul(&ef(z), &w, CW), CW), CW)
            }),
        ),
        (
            "x*next_up2(1+w)",
            Box::new(|z| {
                let w = w_x87(z, &joint);
                let s = ext_to_f64(&ext_add(&ef(1.0), &w, CW), CW)
                    .next_up()
                    .next_up();
                z * s
            }),
        ),
    ];
    let lows: Vec<(f64, u64)> = rows
        .iter()
        .copied()
        .filter(|&(z, bits)| {
            let t = f64::from_bits(bits);
            let g = erf_cont(z);
            ulp_distance(g, t).unwrap_or(99) >= 2 && g < t
        })
        .collect();
    let highs: Vec<(f64, u64)> = rows
        .iter()
        .copied()
        .filter(|&(z, bits)| {
            let t = f64::from_bits(bits);
            let g = erf_cont(z);
            ulp_distance(g, t).unwrap_or(99) >= 2 && g > t
        })
        .collect();
    println!(
        "inner 1+w store nudge (bar 866 / leftover-low {} / leftover-high {}):",
        lows.len(),
        highs.len()
    );
    for (name, ev) in &evals {
        let mut ex = 0usize;
        let mut hit = 0usize;
        let mut keep = 0usize;
        let mut mx = 0u64;
        for &(z, bits) in &rows {
            let t = f64::from_bits(bits);
            let d = ulp_distance(ev(z), t).unwrap_or(99);
            let dj = ulp_distance(erf_cont(z), t).unwrap_or(99);
            if d == 0 {
                ex += 1;
                if dj == 0 {
                    keep += 1;
                }
            } else {
                mx = mx.max(d);
            }
        }
        let mut hit_h = 0usize;
        for &(z, bits) in &lows {
            if ulp_distance(ev(z), f64::from_bits(bits)).unwrap_or(99) == 0 {
                hit += 1;
            }
        }
        for &(z, bits) in &highs {
            if ulp_distance(ev(z), f64::from_bits(bits)).unwrap_or(99) == 0 {
                hit_h += 1;
            }
        }
        println!(
            "{name:24} {ex}/1508 max={mx} keep_j={keep}/866 low_hit={hit}/{} high_hit={hit_h}/{}",
            lows.len(),
            highs.len()
        );
    }
}
