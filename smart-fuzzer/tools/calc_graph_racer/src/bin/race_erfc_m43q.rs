//! 0x74 F_or-hard 43: Q ulp of same F. Division noise vs F-body miss. Not an identity.
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
    let mut qexact_for_hard = 0usize;
    let mut q1 = 0usize;
    let mut qge2 = 0usize;
    let mut n = 0usize;
    let mut hist = [0usize; 7];
    println!("0x74 F_or-hard DIRECT: Q ulp of 0x74 / j3432 / j+0x20 (Q=w*F):");
    for r in &rows {
        if !r.direct || r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let Some(fo) = f::f_or(r.z, r.qbits) else {
            continue;
        };
        let f74 = cody(r.z, &C0, &D0, 0x74);
        let dfor = ulp_distance(f74, fo).unwrap_or(99);
        if dfor < 2 {
            continue;
        }
        n += 1;
        let t = f64::from_bits(r.qbits);
        let dq74 = ulp_distance(qw(r.z, f74), t).unwrap_or(99);
        let dqj = ulp_distance(qw(r.z, cody(r.z, &jc, &jd, 0)), t).unwrap_or(99);
        let dq20 = ulp_distance(qw(r.z, cody(r.z, &jc, &jd, 0x20)), t).unwrap_or(99);
        let b = dq74.min(6) as usize;
        hist[b] += 1;
        if dq74 == 0 {
            qexact_for_hard += 1;
        } else if dq74 == 1 {
            q1 += 1;
        } else {
            qge2 += 1;
        }
        if dq74 == 0 || dqj == 0 || dq20 == 0 {
            println!(
                "  Q-exact z={:.16} For_ulp={dfor} Q74={dq74} Qj={dqj} Qj20={dq20}",
                r.z
            );
        }
    }
    println!("n={n} Q74 exact={qexact_for_hard} ulp1={q1} ulp>=2={qge2} hist0..6+={hist:?}");
    println!("(Q-exact + F_or-hard => extra division is the miss, F-body is right)");
}
