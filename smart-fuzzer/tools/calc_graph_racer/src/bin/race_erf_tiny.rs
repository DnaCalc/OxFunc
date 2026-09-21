//! P-side tiny-z hard: 2/sqrt(pi)*x vs 1+A0 vs joint. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    ext_add, ext_div, ext_from_f64, ext_mul, ext_sqrt, ext_to_f64, Ext80, CW_PC64_RN,
};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const PI: f64 = 3.1415926535897932384626433832795;
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
fn two_rsqpi() -> Ext80 {
    ext_div(&ef(2.0), &ext_sqrt(&ef(PI), CW), CW)
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
    let mut a0p = AS0;
    a0p[0] = poke(AS0[0], 4);
    let c = two_rsqpi();
    let c_st = ext_to_f64(&c, CW);
    println!("tiny-z and z<1e-3 P-side (bar joint 866):");
    let evals: [(&str, Box<dyn Fn(f64) -> f64>); 5] = [
        ("joint", Box::new(|z| erf_a(z, &joint))),
        ("A0+4 only", Box::new(|z| erf_a(z, &a0p))),
        ("CR A21", Box::new(|z| erf_a(z, &AS0))),
        (
            "x*2/sqrt(pi) x87",
            Box::new(move |z| ext_to_f64(&ext_mul(&ef(z), &c, CW), CW)),
        ),
        ("x*stored_2rsqpi", Box::new(move |z| z * c_st)),
    ];
    for (name, ev) in &evals {
        let mut tiny_ex = 0usize;
        let mut tiny_n = 0usize;
        let mut sub_ex = 0usize;
        let mut sub_n = 0usize;
        let mut hit_h = 0usize;
        let mut nh = 0usize;
        for &(z, bits) in &rows {
            let d = ulp_distance(ev(z), f64::from_bits(bits)).unwrap_or(99);
            let dj = ulp_distance(erf_a(z, &joint), f64::from_bits(bits)).unwrap_or(99);
            if z < 1e-5 {
                tiny_n += 1;
                if d == 0 {
                    tiny_ex += 1;
                }
                if dj >= 2 {
                    nh += 1;
                    if d == 0 {
                        hit_h += 1;
                        println!("  HIT {name} tiny-hard z={z:.17}");
                    }
                }
            }
            if z < 1e-3 {
                sub_n += 1;
                if d == 0 {
                    sub_ex += 1;
                }
            }
        }
        println!(
            "{name:18} z<1e-5 {tiny_ex}/{tiny_n}  z<1e-3 {sub_ex}/{sub_n} tiny-hard hit={hit_h}/{nh}"
        );
    }
}
