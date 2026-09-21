//! 7 cephes last-store DIRECT misses vs Cody/DERFC0/libm F±. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const C0: [f64; 9] = [
    0.564188496988670089,
    8.88314979438837594,
    66.1191906371416295,
    298.635138197400131,
    881.95222124176909,
    1712.04761263407058,
    2051.07837782607147,
    1230.33935479799725,
    2.15311535474403846e-8,
];
const D0: [f64; 8] = [
    15.7449261107098347,
    117.693950891312499,
    537.181101862009858,
    1621.38957456669019,
    3290.79923573345963,
    4362.61909014324716,
    3439.36767414372164,
    1230.33935480374942,
];
const SEVEN: [f64; 7] = [
    0.8437499999999999,
    0.8437500000000001,
    0.9375,
    0.9583333333333333,
    0.9895833333333333,
    2.125,
    2.4270833333333335,
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
fn cody0(y: f64) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C0[8]), &ye, CW);
    let mut xden = ye;
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C0[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &ye, CW);
    }
    ext_to_f64(
        &ext_div(
            &ext_add(&xnum, &ef(C0[7]), CW),
            &ext_add(&xden, &ef(D0[7]), CW),
            CW,
        ),
        CW,
    )
}
fn mul(w: f64, ff: f64) -> f64 {
    let v = w * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}
fn k_hit(z: f64, t: f64, ff: f64) -> Option<i32> {
    for k in -8i32..=8 {
        if ulp_distance(mul(f::w_rn53(z), poke(ff, k)), t).unwrap_or(99) == 0 {
            return Some(k);
        }
    }
    None
}
fn libm_f(z: f64) -> f64 {
    let w = f::w_rn53(z);
    if w == 0.0 {
        0.0
    } else {
        libm::erfc(z) / w
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    println!("7 cephes DIRECT misses last-store k:");
    let mut n0 = 0usize;
    let mut nd = 0usize;
    let mut nl = 0usize;
    let mut nls = 0usize;
    let mut hls = 0usize;
    let mut hu = 0usize;
    for &lz in &SEVEN {
        let Some(r) = rows
            .iter()
            .find(|rr| rr.direct && (rr.z - lz).abs() < 1e-15)
        else {
            println!("  z={lz:.16} MISSING");
            continue;
        };
        let t = f64::from_bits(r.qbits);
        let kc = k_hit(lz, t, cody0(lz));
        let kd = k_hit(lz, t, f::nswc_derfc0(lz));
        let kl = k_hit(lz, t, libm_f(lz));
        let kce = k_hit(lz, t, f::cephes_f(lz));
        if kc.is_some() {
            n0 += 1;
        }
        if kd.is_some() {
            nd += 1;
        }
        if kl.is_some() {
            nl += 1;
        }
        println!(
            "  z={lz:.16} Cody={:?} DERFC0={:?} libm={:?} cephes={:?}",
            kc, kd, kl, kce
        );
    }
    println!("7: Cody={n0} DERFC0={nd} libm={nl}");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        nls += 1;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(r.z);
        let ff = libm_f(r.z);
        let e0 = ulp_distance(mul(w, ff), t).unwrap_or(99) == 0;
        let els = (-4i32..=4)
            .filter(|&k| k != 0)
            .any(|k| ulp_distance(mul(w, poke(ff, k)), t).unwrap_or(99) == 0);
        if e0 || els {
            hls += 1;
            hu += 1;
        } else if e0 {
            hu += 1;
        }
    }
    let mut h0 = 0usize;
    let mut hu2 = 0usize;
    let mut miss = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(r.z);
        let ff = libm_f(r.z);
        let e0 = ulp_distance(mul(w, ff), t).unwrap_or(99) == 0;
        let els = (-4i32..=4)
            .filter(|&k| k != 0)
            .any(|k| ulp_distance(mul(w, poke(ff, k)), t).unwrap_or(99) == 0);
        if e0 {
            h0 += 1;
        }
        if e0 || els {
            hu2 += 1;
        } else {
            miss += 1;
        }
    }
    println!(
        "libm F fused ∪ F± DIRECT [0.5,4) fused={h0} union={hu2}/{nls} miss={miss}"
    );
    let _ = (hls, hu);
}
