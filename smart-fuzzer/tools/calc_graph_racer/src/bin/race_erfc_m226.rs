//! DIRECT [0.5,4): unmask Cody fused ∪ F± last-store cover. Not an identity.
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
fn hit_ls(z: f64, t: f64, ff: f64) -> bool {
    (-4i32..=4)
        .filter(|&k| k != 0)
        .any(|k| ulp_distance(mul(f::w_rn53(z), poke(ff, k)), t).unwrap_or(99) == 0)
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mut n210 = 0usize;
    let mut n0 = 0usize;
    let mut nls = 0usize;
    let mut n_u = 0usize;
    let mut n_210u = 0usize;
    let mut miss = 0usize;
    let mut n = 0usize;
    println!("DIRECT [0.5,4) unmask fused ∪ F± cover:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        n += 1;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(r.z);
        let f210 = cody_mask(r.z, 0x210);
        let f0 = cody_mask(r.z, 0);
        let e210 = ulp_distance(mul(w, f210), t).unwrap_or(99) == 0;
        let e0 = ulp_distance(mul(w, f0), t).unwrap_or(99) == 0;
        let els = hit_ls(r.z, t, f0);
        if e210 {
            n210 += 1;
        }
        if e0 {
            n0 += 1;
        }
        if els {
            nls += 1;
        }
        if e0 || els {
            n_u += 1;
        }
        if e210 || els {
            n_210u += 1;
        }
        if !(e0 || els) {
            miss += 1;
            let d0 = ulp_distance(mul(w, f0), t).unwrap_or(99);
            let d210 = ulp_distance(mul(w, f210), t).unwrap_or(99);
            println!(
                "  MISS z={:.16} unmask={} 210={} {}",
                r.z,
                d0,
                d210,
                if mul(w, f0) < t { "L" } else { "H" }
            );
        }
    }
    println!(
        "n={n} 0x210={n210} unmask={n0} unmask-F±={nls} unmask∪F±={n_u} 0x210∪unmaskF±={n_210u} miss={miss}"
    );
}
