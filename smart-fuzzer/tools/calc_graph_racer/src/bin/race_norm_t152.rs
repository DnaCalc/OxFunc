//! NORMSDIST |z|>=4 leftover vs 0.5 last-store and cephes F±. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use std::env;
use std::fs;

const INVSQRT2: f64 = 0.7071067811865476;

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
    let mut n = 0usize;
    let mut h_f = 0usize;
    let mut h_lsf = 0usize;
    let mut h_half = 0usize;
    let mut h_u = 0usize;
    let mut miss = 0usize;
    println!("NORMSDIST |z|>=4 0.5 last-store / F±:");
    for w in &bank.witnesses {
        let x = match &w.args[0] {
            WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
            _ => continue,
        };
        let cum = match &w.args[1] {
            WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
            _ => continue,
        };
        if cum != 1.0 {
            continue;
        }
        let Some(e) = parse_bits_hex(&w.expected_bits) else {
            continue;
        };
        let t = e;
        let z = x * INVSQRT2;
        let a = z.abs();
        if a < 4.0 {
            continue;
        }
        n += 1;
        let ff = f::cephes_f(a);
        let w53 = f::w_rn53(a);
        let q0 = qmul(w53, ff);
        let phi0 = if z >= 0.0 { 1.0 - 0.5 * q0 } else { 0.5 * q0 };
        let efused = ulp_distance(phi0, t).unwrap_or(99) == 0;
        let mut elsf = false;
        let mut ehalf = ulp_distance(phi0, t).unwrap_or(99) == 0;
        for k in -5i32..=5 {
            if k == 0 {
                continue;
            }
            let q = qmul(w53, poke(ff, k));
            let phi = if z >= 0.0 { 1.0 - 0.5 * q } else { 0.5 * q };
            if ulp_distance(phi, t).unwrap_or(99) == 0 {
                elsf = true;
            }
            if ulp_distance(poke(phi0, k), t).unwrap_or(99) == 0 {
                ehalf = true;
            }
            if ulp_distance(poke(phi, k), t).unwrap_or(99) == 0 {
                ehalf = true;
            }
        }
        if efused {
            h_f += 1;
        }
        if elsf {
            h_lsf += 1;
        }
        if ehalf {
            h_half += 1;
        }
        if efused || elsf || ehalf {
            h_u += 1;
        } else {
            miss += 1;
            if miss <= 8 {
                println!(
                    "  MISS x={x:.16} a={a:.16} fused={}{}",
                    ulp_distance(phi0, t).unwrap_or(99),
                    if phi0 < t { "L" } else { "H" }
                );
            }
        }
    }
    println!(
        "n={n} fused-phi={h_f} F±-phi={h_lsf} 0.5-last-store={h_half} union={h_u}/{n} miss={miss}"
    );
}
