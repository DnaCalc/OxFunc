//! 39 NORMSDIST tail leftovers after 0.5 last-store: x87 z / implied-Q. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const INVSQRT2: f64 = 0.7071067811865476;

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
fn phi_ok(z: f64, a: f64, t: f64) -> bool {
    let ff = f::cephes_f(a);
    let w53 = f::w_rn53(a);
    let q0 = qmul(w53, ff);
    let phi0 = if z >= 0.0 { 1.0 - 0.5 * q0 } else { 0.5 * q0 };
    if ulp_distance(phi0, t).unwrap_or(99) == 0 {
        return true;
    }
    for k in -5i32..=5 {
        if k == 0 {
            continue;
        }
        let q = qmul(w53, poke(ff, k));
        let phi = if z >= 0.0 { 1.0 - 0.5 * q } else { 0.5 * q };
        if ulp_distance(phi, t).unwrap_or(99) == 0
            || ulp_distance(poke(phi0, k), t).unwrap_or(99) == 0
            || ulp_distance(poke(phi, k), t).unwrap_or(99) == 0
        {
            return true;
        }
    }
    false
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let path = format!("{dir}/answers-b24-normref.json");
    let bank: WitnessSet = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let mut n39 = 0usize;
    let mut h_x87 = 0usize;
    let mut h_iq = 0usize;
    println!("39 NORMSDIST tail leftovers:");
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
        if phi_ok(z, a, t) {
            continue;
        }
        n39 += 1;
        let zx = ext_to_f64(&ext_mul(&ef(x), &ef(INVSQRT2), CW), CW);
        let ax = zx.abs();
        let ex = phi_ok(zx, ax, t);
        let tq = if z >= 0.0 { 2.0 * (1.0 - t) } else { 2.0 * t };
        let q0 = qmul(f::w_rn53(a), f::cephes_f(a));
        let iq = ulp_distance(q0, tq).unwrap_or(99);
        if ex {
            h_x87 += 1;
        }
        if iq == 0 {
            h_iq += 1;
        }
        if n39 <= 12 {
            println!(
                "  x={x:.16} a={a:.16} x87z={ex} impliedQ_ulp={iq} fused-phi={}{}",
                ulp_distance(if z >= 0.0 { 1.0 - 0.5 * q0 } else { 0.5 * q0 }, t).unwrap_or(99),
                if (if z >= 0.0 { 1.0 - 0.5 * q0 } else { 0.5 * q0 }) < t {
                    "L"
                } else {
                    "H"
                }
            );
        }
    }
    println!("n={n39} x87-z hit={h_x87} impliedQ-exact={h_iq}");
}
