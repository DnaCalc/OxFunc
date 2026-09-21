//! Direct-only Q overlap j3432+0x20 vs 0x74 vs mask0. Not an identity.
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
fn cody(y: f64, c: &[f64; 9], d: &[f64; 8], mask: u32) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(c[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, mask, 0);
    xden = maybe(xden, mask, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(c[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(d[i]), CW), &ye, CW);
        xnum = maybe(xnum, mask, 2 + 2 * i as u32);
        xden = maybe(xden, mask, 3 + 2 * i as u32);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(c[7]), CW),
        &ext_add(&xden, &ef(d[7]), CW),
        CW,
    );
    q = maybe(q, mask, 16);
    ext_to_f64(&q, CW)
}
fn qw(z: f64, ff: f64) -> f64 {
    let v = f::w_rn53(z) * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mut jc = C0;
    jc[0] = poke(C0[0], 4);
    jc[1] = poke(C0[1], 1);
    jc[4] = poke(C0[4], -1);
    let mut jd = D0;
    jd[0] = poke(D0[0], -1);
    jd[4] = poke(D0[4], -1);
    let mut both = 0usize;
    let mut only_j = 0usize;
    let mut only_74 = 0usize;
    let mut both_a = 0usize;
    let mut only_j_a = 0usize;
    let mut only_74_a = 0usize;
    let mut nd = 0usize;
    let mut na = 0usize;
    let mut shown = 0usize;
    for r in &rows {
        if r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let t = f64::from_bits(r.qbits);
        let dj = ulp_distance(qw(r.z, cody(r.z, &jc, &jd, 0x20)), t).unwrap_or(99);
        let d7 = ulp_distance(qw(r.z, cody(r.z, &C0, &D0, 0x74)), t).unwrap_or(99);
        let d0 = ulp_distance(qw(r.z, cody(r.z, &jc, &jd, 0)), t).unwrap_or(99);
        na += 1;
        match (dj == 0, d7 == 0) {
            (true, true) => both_a += 1,
            (true, false) => only_j_a += 1,
            (false, true) => only_74_a += 1,
            _ => {}
        }
        if !r.direct {
            continue;
        }
        nd += 1;
        match (dj == 0, d7 == 0) {
            (true, true) => both += 1,
            (true, false) => {
                only_j += 1;
                if shown < 8 {
                    println!(
                        "  only_j20 z={:.16} j0_ulp={} 74_ulp={}",
                        r.z, d0, d7
                    );
                    shown += 1;
                }
            }
            (false, true) => only_74 += 1,
            _ => {}
        }
    }
    println!("Q all both={both_a} only_j20={only_j_a} only_74={only_74_a} union={} n={na}", both_a + only_j_a + only_74_a);
    println!("Q DIRECT both={both} only_j20={only_j} only_74={only_74} union={} n={nd}", both + only_j + only_74);
}
