//! XBIG leftover a=20.93: last-store of z / z² in excel_exp. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research::{excel_exp, x87_mul};
use std::env;
use std::fs;

const INVSQRT2: f64 = 0.7071067811865476;
const TARGET: f64 = -29.5986386669432804;

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
    let z0 = TARGET * INVSQRT2;
    let a0 = z0.abs();
    let ff0 = f::cephes_f(a0);
    println!("a0={a0:.16} t={t:e} fused={}H", {
        let p = 0.5 * qmul(f::w_rn53(a0), ff0);
        ulp_distance(p, t).unwrap_or(99)
    });
    print!("  z±k 0.5*w(z)*F(a0):");
    for k in -16i32..=16 {
        if k == 0 {
            continue;
        }
        let a = poke(a0, k);
        let p = 0.5 * qmul(f::w_rn53(a), ff0);
        if ulp_distance(p, t).unwrap_or(99) == 0 {
            print!(" z{k:+}");
        }
        let p2 = 0.5 * qmul(f::w_rn53(a), f::cephes_f(a));
        if ulp_distance(p2, t).unwrap_or(99) == 0 {
            print!(" zF{k:+}");
        }
    }
    println!();
    print!("  x±k then *invsqrt2:");
    for k in -16i32..=16 {
        if k == 0 {
            continue;
        }
        let a = (poke(TARGET, k) * INVSQRT2).abs();
        let p = 0.5 * qmul(f::w_rn53(a), ff0);
        if ulp_distance(p, t).unwrap_or(99) == 0 {
            print!(" x{k:+}");
        }
    }
    println!();
    let tt0 = x87_mul(a0, a0);
    print!("  t=z*z ±k excel_exp(-t):");
    for k in -16i32..=16 {
        if k == 0 {
            continue;
        }
        let w = excel_exp(-poke(tt0, k));
        let p = 0.5 * qmul(w, ff0);
        if ulp_distance(p, t).unwrap_or(99) == 0 {
            print!(" t{k:+}");
        }
    }
    println!();
    print!("  exp arg ±k:");
    let arg = -tt0;
    for k in -16i32..=16 {
        if k == 0 {
            continue;
        }
        let w = excel_exp(poke(arg, k));
        let p = 0.5 * qmul(w, ff0);
        if ulp_distance(p, t).unwrap_or(99) == 0 {
            print!(" e{k:+}");
        }
    }
    println!();
    print!("  w±k:");
    let w0 = f::w_rn53(a0);
    for k in -16i32..=16 {
        if k == 0 {
            continue;
        }
        let p = 0.5 * qmul(poke(w0, k), ff0);
        if ulp_distance(p, t).unwrap_or(99) == 0 {
            print!(" w{k:+}");
        }
    }
    println!();
    // nearest k for z via ulp of phi
    println!("  z-scan ulp of 0.5*w(z)*F(a0):");
    for k in -8i32..=8 {
        let a = poke(a0, k);
        let p = 0.5 * qmul(f::w_rn53(a), ff0);
        let d = ulp_distance(p, t).unwrap_or(99);
        print!(" {k:+}={d}");
    }
    println!();
    let rows = f::load_q_rows_tagged(&dir);
    let mut near = 0usize;
    for r in rows.iter().filter(|rr| (rr.z - a0).abs() < 1e-6) {
        near += 1;
        println!(
            "  Q-row z={:.16} direct={} q={:e}",
            r.z,
            r.direct,
            f64::from_bits(r.qbits)
        );
    }
    println!("Q-rows near a0: {near}");
}
