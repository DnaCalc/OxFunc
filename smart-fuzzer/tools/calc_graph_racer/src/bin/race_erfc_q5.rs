//! 5 non-direct Q-mid misses of unmask ∪ F±. Not an identity.
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
const FIVE: [f64; 5] = [
    1.1842387490730035,
    1.5157442674975992,
    2.0468880508031648,
    2.0646100252689923,
    2.1812794009719783,
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
fn maybe(x: Ext80, mask: u32, bit: u32) -> Ext80 {
    if bit < 32 && mask & (1u32 << bit) != 0 {
        ef(ext_to_f64(&x, CW))
    } else {
        x
    }
}
fn cody_mask(y: f64, mask: u32) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C0[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, mask, 0);
    xden = maybe(xden, mask, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C0[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &ye, CW);
        xnum = maybe(xnum, mask, 2 + 2 * i as u32);
        xden = maybe(xden, mask, 3 + 2 * i as u32);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(C0[7]), CW),
        &ext_add(&xden, &ef(D0[7]), CW),
        CW,
    );
    q = maybe(q, mask, 16);
    ext_to_f64(&q, CW)
}
fn mul(w: f64, ff: f64) -> f64 {
    let v = w * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
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
    let rows = f::load_q_rows_tagged(&dir);
    println!("5 non-direct mid misses:");
    for &lz in &FIVE {
        let Some(r) = rows.iter().find(|rr| (rr.z - lz).abs() < 1e-12) else {
            println!("  z={lz:.16} MISSING");
            continue;
        };
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(lz);
        let f0 = cody_mask(lz, 0);
        let f210 = cody_mask(lz, 0x210);
        let cf = f::cephes_f(lz);
        print!(
            "  z={lz:.16} direct={} unmask={} 210={} cephes={} libm={}",
            r.direct,
            signed(mul(w, f0), t),
            signed(mul(w, f210), t),
            signed(mul(w, cf), t),
            signed(libm::erfc(lz), t)
        );
        for k in 1..=8 {
            if ulp_distance(mul(w, poke(f0, k)), t).unwrap_or(99) == 0
                || ulp_distance(mul(w, poke(f0, -k)), t).unwrap_or(99) == 0
                || ulp_distance(mul(poke(w, k), f0), t).unwrap_or(99) == 0
                || ulp_distance(mul(poke(w, -k), f0), t).unwrap_or(99) == 0
            {
                print!(" hit-k={k}");
            }
        }
        println!();
    }
}
