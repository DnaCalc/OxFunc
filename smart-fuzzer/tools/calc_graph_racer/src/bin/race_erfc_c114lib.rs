//! 0x210-114 vs libm erfc DIRECT [0.5,4). Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const MASK: u32 = 0x210;
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
fn cody(y: f64, c: &[f64; 9]) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(c[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, MASK, 0);
    xden = maybe(xden, MASK, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(c[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &ye, CW);
        xnum = maybe(xnum, MASK, 2 + 2 * i as u32);
        xden = maybe(xden, MASK, 3 + 2 * i as u32);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(c[7]), CW),
        &ext_add(&xden, &ef(D0[7]), CW),
        CW,
    );
    q = maybe(q, MASK, 16);
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
fn k_of(z: f64, t: f64, ff: f64) -> Option<i32> {
    for ak in 0i32..=8 {
        for &s in &[-1i32, 1] {
            let k = if ak == 0 { 0 } else { s * ak };
            if ak == 0 && s < 0 {
                continue;
            }
            if ulp_distance(mul(f::w_rn53(z), poke(ff, k)), t).unwrap_or(99) == 0 {
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
    let rows = f::load_q_rows_tagged(&dir);
    let mut c = C0;
    c[3] = poke(C0[3], -1);
    c[4] = poke(C0[4], 1);
    c[5] = poke(C0[5], -1);
    c[6] = poke(C0[6], 1);
    let mut both = 0usize;
    let mut only_c = 0usize;
    let mut only_l = 0usize;
    let mut nc = 0usize;
    let mut nl = 0usize;
    let mut nls = 0usize;
    let mut n = 0usize;
    println!("DIRECT [0.5,4) 114 vs libm:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        n += 1;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(r.z);
        let ec = ulp_distance(mul(w, cody(r.z, &c)), t).unwrap_or(99) == 0;
        let el = ulp_distance(mul(w, libm::erfc(r.z)), t).unwrap_or(99) == 0;
        let ls = k_of(r.z, t, libm::erfc(r.z)).is_some();
        if ec {
            nc += 1;
        }
        if el {
            nl += 1;
        }
        if ls {
            nls += 1;
        }
        match (ec, el) {
            (true, true) => both += 1,
            (true, false) => only_c += 1,
            (false, true) => only_l += 1,
            _ => {}
        }
    }
    println!(
        "n={n} 114={nc} libm={nl} both={both} only_114={only_c} only_libm={only_l} union={} libm F± {nls}/{n}",
        both + only_c + only_l
    );
}
