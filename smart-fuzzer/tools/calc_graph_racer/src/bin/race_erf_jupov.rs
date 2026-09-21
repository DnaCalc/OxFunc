//! leftover-low/high z-set joint vs unpoked A21. Not an identity.
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
fn k_of(z: f64, t: f64, ff: f64) -> Option<i32> {
    for ak in 0i32..=8 {
        for &s in &[-1i32, 1] {
            let k = if ak == 0 { 0 } else { s * ak };
            if ak == 0 && s < 0 {
                continue;
            }
            if ulp_distance(poke(ff, k), t).unwrap_or(99) == 0 {
                return Some(k);
            }
            if ak == 0 {
                break;
            }
        }
    }
    None
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
    let mut joint = AS0;
    joint[0] = poke(AS0[0], 4);
    joint[1] = poke(AS0[1], -2);
    joint[2] = poke(AS0[2], -5);
    joint[3] = poke(AS0[3], 1);
    let names = ["fused", "1-ULP", "leftover-low", "leftover-high"];
    let idx = |s: &str| match s {
        "fused" => 0,
        "1-ULP" => 1,
        "leftover-low" => 2,
        _ => 3,
    };
    let buck = |g: f64, t: f64| {
        let d = ulp_distance(g, t).unwrap_or(99);
        if d == 0 {
            "fused"
        } else if d == 1 {
            "1-ULP"
        } else if g < t {
            "leftover-low"
        } else {
            "leftover-high"
        }
    };
    let mut ulo = [0usize; 4];
    let mut uhi = [0usize; 4];
    let mut nulo = 0usize;
    let mut nuhi = 0usize;
    let mut jlo = [0usize; 4];
    let mut jhi = [0usize; 4];
    let mut njlo = 0usize;
    let mut njhi = 0usize;
    println!("unpoked leftover as joint / joint leftover as unpoked:");
    for (&zb, &eb) in &map {
        let z = f64::from_bits(zb);
        let t = f64::from_bits(eb);
        let gj = erf_a(z, &joint);
        let gu = erf_a(z, &AS0);
        let du = ulp_distance(gu, t).unwrap_or(99);
        let dj = ulp_distance(gj, t).unwrap_or(99);
        let bj = buck(gj, t);
        let bu = buck(gu, t);
        if du >= 2 && gu < t {
            nulo += 1;
            ulo[idx(bj)] += 1;
            if bj != "leftover-low" {
                println!("  unpoked-LOW z={:.16} d={du} joint={bj}", z);
            }
        } else if du >= 2 && gu > t {
            nuhi += 1;
            uhi[idx(bj)] += 1;
            if bj != "leftover-high" {
                println!("  unpoked-HIGH z={:.16} d={du} joint={bj}", z);
            }
        }
        if dj >= 2 && gj < t {
            njlo += 1;
            jlo[idx(bu)] += 1;
        }
        if dj >= 2 && gj > t {
            njhi += 1;
            jhi[idx(bu)] += 1;
        }
    }
    print!("unpoked leftover-low n={nulo} as joint");
    for (i, c) in ulo.iter().enumerate() {
        if *c > 0 {
            print!(" {}:{}", names[i], c);
        }
    }
    print!("\nunpoked leftover-high n={nuhi} as joint");
    for (i, c) in uhi.iter().enumerate() {
        if *c > 0 {
            print!(" {}:{}", names[i], c);
        }
    }
    print!("\njoint leftover-low n={njlo} as unpoked");
    for (i, c) in jlo.iter().enumerate() {
        if *c > 0 {
            print!(" {}:{}", names[i], c);
        }
    }
    print!("\njoint leftover-high n={njhi} as unpoked");
    for (i, c) in jhi.iter().enumerate() {
        if *c > 0 {
            print!(" {}:{}", names[i], c);
        }
    }
    println!();
}
