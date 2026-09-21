//! Cody C/D Fortran association / Horner-in-y² on 0x210. Not an identity.
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
fn maybe(x: Ext80, mask: u32, bit: u32) -> Ext80 {
    if bit < 32 && mask & (1u32 << bit) != 0 {
        ef(ext_to_f64(&x, CW))
    } else {
        x
    }
}
fn specfun(y: f64, mask: u32) -> f64 {
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
fn distrib(y: f64, mask: u32) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C0[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, mask, 0);
    xden = maybe(xden, mask, 1);
    for i in 0..7 {
        let cy = ext_mul(&ef(C0[i]), &ye, CW);
        let dy = ext_mul(&ef(D0[i]), &ye, CW);
        xnum = ext_add(&ext_mul(&xnum, &ye, CW), &cy, CW);
        xden = ext_add(&ext_mul(&xden, &ye, CW), &dy, CW);
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
fn y2_horner(y: f64) -> f64 {
    let ye = ef(y.abs());
    let y2 = ext_mul(&ye, &ye, CW);
    let mut xnum = ef(C0[8]);
    let mut xden = ef(1.0);
    for i in 0..4 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C0[i]), CW), &y2, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &y2, CW);
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
fn f64_specfun(y: f64) -> f64 {
    let y = y.abs();
    let mut xnum = C0[8] * y;
    let mut xden = y;
    for i in 0..7 {
        xnum = (xnum + C0[i]) * y;
        xden = (xden + D0[i]) * y;
    }
    (xnum + C0[7]) / (xden + D0[7])
}
fn qwf(z: f64, ff: f64) -> f64 {
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
    let graphs: [(&str, Box<dyn Fn(f64) -> f64>); 5] = [
        ("SPECFUN 0x210", Box::new(|z| specfun(z, 0x210))),
        ("distrib 0x210", Box::new(|z| distrib(z, 0x210))),
        ("SPECFUN mask0", Box::new(|z| specfun(z, 0))),
        ("f64 SPECFUN", Box::new(f64_specfun)),
        ("Horner y^2", Box::new(y2_horner)),
    ];
    println!("C/D association DIRECT dmid (bar 105):");
    for (name, ev) in &graphs {
        let mut dmid = 0usize;
        let mut nd = 0usize;
        let mut mx = 0u64;
        let mut qe = 0usize;
        let mut nq = 0usize;
        for r in &rows {
            if r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            nq += 1;
            let d = ulp_distance(qwf(r.z, ev(r.z)), f64::from_bits(r.qbits)).unwrap_or(99);
            if d == 0 {
                qe += 1;
            }
            if r.direct {
                nd += 1;
                if d == 0 {
                    dmid += 1;
                } else {
                    mx = mx.max(d);
                }
            }
        }
        println!("  {name:16} Q {qe}/{nq} dmid={dmid}/{nd} maxd={mx}");
    }
}
