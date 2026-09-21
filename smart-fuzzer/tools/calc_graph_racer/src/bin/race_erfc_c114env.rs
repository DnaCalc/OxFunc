//! 114 leftover-high and 1-ULP last-mul of unmask. Not an identity.
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
fn cody(y: f64, c: &[f64; 9], mask: u32) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(c[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, mask, 0);
    xden = maybe(xden, mask, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(c[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &ye, CW);
        xnum = maybe(xnum, mask, 2 + 2 * i as u32);
        xden = maybe(xden, mask, 3 + 2 * i as u32);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(c[7]), CW),
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

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mut c114 = C0;
    c114[3] = poke(C0[3], -1);
    c114[4] = poke(C0[4], 1);
    c114[5] = poke(C0[5], -1);
    c114[6] = poke(C0[6], 1);
    let mut nhi = 0usize;
    let mut n1 = 0usize;
    let mut hi_dn = 0usize;
    let mut hi_wm = 0usize;
    let mut u1_up = 0usize;
    let mut u1_dn = 0usize;
    let mut u1_wp = 0usize;
    let mut u1_wm = 0usize;
    println!("114 leftover-high and 1-ULP last-mul of unmask:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let g114 = mul(f::w_rn53(r.z), cody(r.z, &c114, MASK));
        let d114 = ulp_distance(g114, t).unwrap_or(99);
        let ff = cody(r.z, &C0, 0);
        let w = f::w_rn53(r.z);
        let g = mul(w, ff);
        if d114 >= 2 && g114 > t {
            nhi += 1;
            if ulp_distance(g.next_down(), t).unwrap_or(99) == 0 {
                hi_dn += 1;
            }
            if ulp_distance(mul(w.next_down(), ff), t).unwrap_or(99) == 0 {
                hi_wm += 1;
            }
        } else if d114 == 1 {
            n1 += 1;
            if ulp_distance(g.next_up(), t).unwrap_or(99) == 0 {
                u1_up += 1;
            }
            if ulp_distance(g.next_down(), t).unwrap_or(99) == 0 {
                u1_dn += 1;
            }
            if ulp_distance(mul(w.next_up(), ff), t).unwrap_or(99) == 0 {
                u1_wp += 1;
            }
            if ulp_distance(mul(w.next_down(), ff), t).unwrap_or(99) == 0 {
                u1_wm += 1;
            }
        }
    }
    println!(
        "leftover-high n={nhi} last-mul next_down={hi_dn} w-1={hi_wm}"
    );
    println!(
        "1-ULP n={n1} last-mul next_up={u1_up} next_down={u1_dn} w+1={u1_wp} w-1={u1_wm}"
    );
}
