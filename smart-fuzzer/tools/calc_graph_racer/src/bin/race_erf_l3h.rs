//! 3 P leftover-HIGH that miss z−1..4 ∪ ow−1..4. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const CW_RD: u16 = CW_PC64_RN | 0x0400;
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
const THREE: [f64; 3] = [0.1865234375, 0.220703125, 0.4485677083333333];

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
fn one_w(z: f64) -> Ext80 {
    let mut a = AS0;
    a[0] = poke(AS0[0], 4);
    a[1] = poke(AS0[1], -2);
    a[2] = poke(AS0[2], -5);
    a[3] = poke(AS0[3], 1);
    let xe = ef(z);
    let t = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW), &ef(c), CW);
    }
    ext_add(&ef(1.0), &acc, CW)
}
fn signed(g: f64, t: f64) -> String {
    let d = ulp_distance(g, t).unwrap_or(99);
    let s = if g < t {
        "L"
    } else if g > t {
        "H"
    } else {
        "="
    };
    format!("{d}{s}")
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
    println!("3 P leftover-HIGH miss last-store:");
    for &lz in &THREE {
        let Some((_, bits)) = rows.iter().find(|(z, _)| (*z - lz).abs() < 1e-14) else {
            println!("  z={lz:.16} MISSING");
            continue;
        };
        let t = f64::from_bits(*bits);
        let ow = one_w(lz);
        let owf = ext_to_f64(&ow, CW);
        let cr = ext_to_f64(&ext_mul(&ef(lz), &ow, CW), CW);
        print!("  z={lz:.16} CR={}", signed(cr, t));
        for k in 5..=8 {
            let g = ext_to_f64(&ext_mul(&ef(poke(lz, -k)), &ow, CW), CW);
            print!(" z-{k}={}", signed(g, t));
        }
        for k in 5..=8 {
            let g = ext_to_f64(&ext_mul(&ef(lz), &ef(poke(owf, -k)), CW), CW);
            print!(" ow-{k}={}", signed(g, t));
        }
        let rd = ext_to_f64(&ext_mul(&ef(lz.next_down()), &ow, CW_RD), CW_RD);
        let tf = lz.next_down() * owf.next_down();
        let add = ext_to_f64(
            &ext_add(&ef(lz), &ext_mul(&ef(lz), &ext_add(&ow, &ef(-1.0), CW), CW), CW),
            CW,
        );
        println!(
            " RD={} dnZ*dnOW={} last-add={}",
            signed(rd, t),
            signed(tf, t),
            signed(add, t)
        );
    }
}
