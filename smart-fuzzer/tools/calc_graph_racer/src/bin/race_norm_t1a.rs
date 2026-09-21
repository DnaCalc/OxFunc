//! NORMSDIST leftover a≈20.93 vs asymptotic F=1/(z√π). Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const INVSQRT2: f64 = 0.7071067811865476;
const INVSQRTPI: f64 = 0.56418958354775628694807945156077;
const TARGET: f64 = -29.5986386669432804;

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
fn qmul(w: f64, ff: f64) -> f64 {
    let v = w * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let path = format!("{dir}/answers-b24-normref.json");
    let bank: WitnessSet = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let Some(w) = bank.witnesses.iter().find(|ww| {
        let x = match &ww.args[0] {
            WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
            _ => return false,
        };
        (x - TARGET).abs() < 1e-12
    }) else {
        println!("MISSING");
        return;
    };
    let t = parse_bits_hex(&w.expected_bits).unwrap();
    let z = TARGET * INVSQRT2;
    let a = z.abs();
    let w53 = f::w_rn53(a);
    let f_as = INVSQRTPI / a;
    let f_x87 = ext_to_f64(&ext_div(&ef(INVSQRTPI), &ef(a), CW), CW);
    println!("a={a:.16} t={t:e} w53={w53:e} F_as={f_as:.16}");
    for (name, ff) in [("f64", f_as), ("x87", f_x87)] {
        let q = qmul(w53, ff);
        let phi = 0.5 * q;
        print!(
            "  {name} 0.5*w*F={}{}",
            ulp_distance(phi, t).unwrap_or(99),
            if phi < t { "L" } else { "H" }
        );
        for k in 1..=8 {
            if ulp_distance(0.5 * qmul(w53, poke(ff, k)), t).unwrap_or(99) == 0 {
                print!(" F+{k}");
            }
            if ulp_distance(0.5 * qmul(w53, poke(ff, -k)), t).unwrap_or(99) == 0 {
                print!(" F-{k}");
            }
            if ulp_distance(poke(phi, k), t).unwrap_or(99) == 0 {
                print!(" phi+{k}");
            }
            if ulp_distance(poke(phi, -k), t).unwrap_or(99) == 0 {
                print!(" phi-{k}");
            }
            if ulp_distance(0.5 * qmul(poke(w53, k), ff), t).unwrap_or(99) == 0 {
                print!(" w+{k}");
            }
            if ulp_distance(0.5 * qmul(poke(w53, -k), ff), t).unwrap_or(99) == 0 {
                print!(" w-{k}");
            }
        }
        println!(" phi={phi:e}");
    }
    let one_over_zsqrtpi = 1.0 / (a * std::f64::consts::PI.sqrt());
    let phi2 = 0.5 * qmul(w53, one_over_zsqrtpi);
    println!(
        "  1/(z*sqrtpi) 0.5wF={} phi={phi2:e}",
        ulp_distance(phi2, t).unwrap_or(99)
    );
}
