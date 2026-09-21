//! unpoked vs unpoked-z+1 overlap and z-cuts. Not an identity.
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
fn joint_a() -> [f64; 21] {
    let mut a = AS0;
    a[0] = poke(AS0[0], 4);
    a[1] = poke(AS0[1], -2);
    a[2] = poke(AS0[2], -5);
    a[3] = poke(AS0[3], 1);
    a
}
fn a21(z: f64, a: &[f64; 21]) -> f64 {
    let xe = ef(z);
    let t = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW), &ef(c), CW);
    }
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &acc, CW), CW), CW)
}
fn z1(z: f64) -> f64 {
    let xe = ef(z);
    let t = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in AS0.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW), &ef(c), CW);
    }
    ext_to_f64(
        &ext_mul(&ef(z.next_up()), &ext_add(&ef(1.0), &acc, CW), CW),
        CW,
    )
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
    let ja = joint_a();
    let mut both = 0usize;
    let mut only_c = 0usize;
    let mut only_z = 0usize;
    let mut bj = 0usize;
    let mut oj = 0usize;
    let mut oz = 0usize;
    for &(z, bits) in &rows {
        let t = f64::from_bits(bits);
        let ec = ulp_distance(a21(z, &AS0), t).unwrap_or(99) == 0;
        let ez = ulp_distance(z1(z), t).unwrap_or(99) == 0;
        let ej = ulp_distance(a21(z, &ja), t).unwrap_or(99) == 0;
        match (ec, ez) {
            (true, true) => both += 1,
            (true, false) => only_c += 1,
            (false, true) => only_z += 1,
            _ => {}
        }
        match (ej, ez) {
            (true, true) => bj += 1,
            (true, false) => oj += 1,
            (false, true) => oz += 1,
            _ => {}
        }
    }
    println!(
        "unpoked vs z+1 both={both} only_cr={only_c} only_z1={only_z} union={}",
        both + only_c + only_z
    );
    println!(
        "joint vs unpoked-z+1 both={bj} only_j={oj} only_z1={oz} union={}",
        bj + oj + oz
    );
    let cuts = [
        0.001, 0.01, 0.05, 0.1, 0.15, 0.2, 0.25, 0.3, 0.35, 0.4, 0.45,
    ];
    println!("z-cut unpoked-then-z+1 / z+1-then-unpoked:");
    for &c in &cuts {
        let mut a = 0usize;
        let mut b = 0usize;
        for &(z, bits) in &rows {
            let t = f64::from_bits(bits);
            let ec = ulp_distance(a21(z, &AS0), t).unwrap_or(99) == 0;
            let ez = ulp_distance(z1(z), t).unwrap_or(99) == 0;
            if z < c {
                if ec {
                    a += 1;
                }
                if ez {
                    b += 1;
                }
            } else {
                if ez {
                    a += 1;
                }
                if ec {
                    b += 1;
                }
            }
        }
        println!("  cut={c} cr-then-z1={a} z1-then-cr={b} (bar 866)");
    }
}
