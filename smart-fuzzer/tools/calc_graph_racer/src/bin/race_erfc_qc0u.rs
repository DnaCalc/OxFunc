//! Unstored x87 1/sqrt(pi) as C[0] in SPECFUN C/D fused loop. Q=w*F. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sqrt, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const PI: f64 = 3.1415926535897932384626433832795;
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
const SQRPI: f64 = 5.6418958354775628695e-1;

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn cody(y: f64, c0: Ext80) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C0[8]), &ye, CW);
    let mut xden = ye;
    for i in 0..7 {
        let ci = if i == 0 { c0 } else { ef(C0[i]) };
        xnum = ext_mul(&ext_add(&xnum, &ci, CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &ye, CW);
    }
    ext_to_f64(
        &ext_div(&ext_add(&xnum, &ef(C0[7]), CW), &ext_add(&xden, &ef(D0[7]), CW), CW),
        CW,
    )
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
    let x87_inv = ext_div(&ef(1.0), &ext_sqrt(&ef(PI), CW), CW);
    let half_two = ext_mul(&ef(0.5), &ext_div(&ef(2.0), &ext_sqrt(&ef(PI), CW), CW), CW);
    println!(
        "C0={:016x} 1/sqrt(pi)f64={:016x} x87_store={:016x} SQRPI={:016x} 0.5*2/sqrt(pi)_store={:016x}",
        C0[0].to_bits(),
        (1.0 / PI.sqrt()).to_bits(),
        ext_to_f64(&x87_inv, CW).to_bits(),
        SQRPI.to_bits(),
        ext_to_f64(&half_two, CW).to_bits()
    );
    let graphs: [(&str, Ext80); 4] = [
        ("fitted C[0]", ef(C0[0])),
        ("x87 1/sqrt(pi) unstored", x87_inv),
        ("f64 1/sqrt(pi) as C0", ef(1.0 / PI.sqrt())),
        ("0.5*x87 2/sqrt(pi) unstored", half_two),
    ];
    for (name, c0) in graphs {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut maxu = 0u64;
        let mut dmid = 0usize;
        let mut dn = 0usize;
        for r in &rows {
            if r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            let qg = qw(r.z, cody(r.z, c0));
            if !qg.is_finite() {
                continue;
            }
            let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            n += 1;
            if r.direct {
                dn += 1;
            }
            if d == 0 {
                ex += 1;
                if r.direct {
                    dmid += 1;
                }
            } else {
                maxu = maxu.max(d);
            }
        }
        println!("{name:32} {ex}/{n} max={maxu} dmid={dmid}/{dn}");
    }
    let assoc: [(&str, fn(f64) -> f64); 2] = [
        ("C[0]+C[8]*y first", |y| {
            let ye = ef(y.abs());
            let mut xnum = ext_add(&ef(C0[0]), &ext_mul(&ef(C0[8]), &ye, CW), CW);
            let mut xden = ye;
            xnum = ext_mul(&xnum, &ye, CW);
            xden = ext_mul(&ext_add(&xden, &ef(D0[0]), CW), &ye, CW);
            for i in 1..7 {
                xnum = ext_mul(&ext_add(&xnum, &ef(C0[i]), CW), &ye, CW);
                xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &ye, CW);
            }
            ext_to_f64(
                &ext_div(&ext_add(&xnum, &ef(C0[7]), CW), &ext_add(&xden, &ef(D0[7]), CW), CW),
                CW,
            )
        }),
        ("y*(C[8]*y+C[0]) first", |y| {
            let ye = ef(y.abs());
            let mut xnum = ext_mul(
                &ye,
                &ext_add(&ext_mul(&ef(C0[8]), &ye, CW), &ef(C0[0]), CW),
                CW,
            );
            let mut xden = ye;
            xden = ext_mul(&ext_add(&xden, &ef(D0[0]), CW), &ye, CW);
            for i in 1..7 {
                xnum = ext_mul(&ext_add(&xnum, &ef(C0[i]), CW), &ye, CW);
                xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &ye, CW);
            }
            ext_to_f64(
                &ext_div(&ext_add(&xnum, &ef(C0[7]), CW), &ext_add(&xden, &ef(D0[7]), CW), CW),
                CW,
            )
        }),
    ];
    for (name, ev) in assoc {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut maxu = 0u64;
        let mut dmid = 0usize;
        let mut dn = 0usize;
        for r in &rows {
            if r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            let qg = qw(r.z, ev(r.z));
            if !qg.is_finite() {
                continue;
            }
            let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            n += 1;
            if r.direct {
                dn += 1;
            }
            if d == 0 {
                ex += 1;
                if r.direct {
                    dmid += 1;
                }
            } else {
                maxu = maxu.max(d);
            }
        }
        println!("{name:32} {ex}/{n} max={maxu} dmid={dmid}/{dn}");
    }
}
