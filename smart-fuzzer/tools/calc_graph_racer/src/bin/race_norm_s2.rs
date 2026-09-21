//! 2 NORMSDIST |z|<0.5 leftovers vs 0.5 last-store / x87 z. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const INVSQRT2: f64 = 0.7071067811865476;
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
const TWO: [f64; 2] = [-0.5630434782608696, -0.4804347826086955];

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
fn a21(z: f64, a: &[f64; 21]) -> f64 {
    let xe = ef(z.abs());
    let t = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW), &ef(c), CW);
    }
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &acc, CW), CW), CW)
}
fn joint_a() -> [f64; 21] {
    let mut a = AS0;
    a[0] = poke(AS0[0], 4);
    a[1] = poke(AS0[1], -2);
    a[2] = poke(AS0[2], -5);
    a[3] = poke(AS0[3], 1);
    a
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let path = format!("{dir}/answers-b24-normref.json");
    let bank: WitnessSet = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    println!("2 NORMSDIST |z|<0.5 leftovers:");
    let ja = joint_a();
    for &lx in &TWO {
        let Some(w) = bank.witnesses.iter().find(|ww| {
            let x = match &ww.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
                _ => return false,
            };
            (x - lx).abs() < 1e-12
        }) else {
            println!("  x={lx:.16} MISSING");
            continue;
        };
        let t = parse_bits_hex(&w.expected_bits).unwrap();
        let zf = lx * INVSQRT2;
        let zx = ext_to_f64(&ext_mul(&ef(lx), &ef(INVSQRT2), CW), CW);
        for (name, z) in [("f64", zf.abs()), ("x87", zx.abs())] {
            let p = a21(z, &ja);
            let q = 1.0 - p;
            let phi = 0.5 * q;
            let dphi = ulp_distance(phi, t).unwrap_or(99);
            let mut hit = dphi == 0;
            for k in 1..=4 {
                if ulp_distance(poke(phi, k), t).unwrap_or(99) == 0
                    || ulp_distance(poke(phi, -k), t).unwrap_or(99) == 0
                {
                    hit = true;
                    print!("  {name} phi±{k}");
                }
            }
            println!(
                "  x={lx:.16} {name} z={z:.16} 0.5*(1-P)={}{} hit={hit}",
                dphi,
                if phi < t { "L" } else { "H" }
            );
        }
    }
}
