//! Joint keep-descent of NSWC A[21] on ERF.PRECISE z<0.5 from ham2 A0+1 A1+1.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_from_f64, ext_mul, ext_to_f64, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const A0: [f64; 21] = [
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

fn erf_a(z: f64, a: &[f64; 21]) -> f64 {
    let xe = ext_from_f64(z);
    let t = ext_mul(&xe, &xe, CW);
    let mut acc = ext_from_f64(0.0);
    for &c in a.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW), &ext_from_f64(c), CW);
    }
    ext_to_f64(&ext_mul(&xe, &ext_add(&ext_from_f64(1.0), &acc, CW), CW), CW)
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
    let score = |a: &[f64; 21]| {
        let mut ex = 0usize;
        let mut sum = 0u128;
        let mut max = 0u64;
        for &(z, bits) in &rows {
            let d = ulp_distance(erf_a(z, a), f64::from_bits(bits)).unwrap_or(u64::MAX);
            if d == 0 {
                ex += 1;
            } else {
                max = max.max(d);
                sum += d as u128;
            }
        }
        (ex, max, sum)
    };
    let mut a = A0;
    a[0] = a[0].next_up();
    a[1] = a[1].next_up();
    let mut best = score(&a);
    println!("start A0+1 A1+1 {}/{} max={} sum={}", best.0, rows.len(), best.1, best.2);
    let deltas = [1i32, -1, 2, -2, 4, -4];
    loop {
        let mut moved = false;
        for i in 0..21 {
            for &k in &deltas {
                let old = a[i];
                a[i] = poke(old, k);
                let sc = score(&a);
                if sc.0 > best.0 || (sc.0 == best.0 && sc.2 < best.2) {
                    best = sc;
                    moved = true;
                    println!("  keep A[{i}] {k:+} {}/{} max={} sum={}", sc.0, rows.len(), sc.1, sc.2);
                } else {
                    a[i] = old;
                }
            }
        }
        if !moved {
            break;
        }
    }
    println!("joint P-side {}/{} max={} sum={}", best.0, rows.len(), best.1, best.2);
    for i in 0..21 {
        if a[i].to_bits() != A0[i].to_bits() {
            println!("  A[{i}] {:016x} CR {:016x}", a[i].to_bits(), A0[i].to_bits());
        }
    }
}
