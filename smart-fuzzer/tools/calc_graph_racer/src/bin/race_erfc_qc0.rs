//! C[0] = fitted vs 1/sqrt(pi) vs FORTRAN SQRPI. Q=w*F. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sqrt, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
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
const D: [f64; 8] = [
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
fn cody_cd(y: f64, c: &[f64; 9]) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(c[8]), &ye, CW);
    let mut xden = ye;
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(c[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D[i]), CW), &ye, CW);
    }
    ext_to_f64(
        &ext_div(&ext_add(&xnum, &ef(c[7]), CW), &ext_add(&xden, &ef(D[7]), CW), CW),
        CW,
    )
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let sqrtpi = std::f64::consts::PI.sqrt();
    let inv_sqrtpi = 1.0 / sqrtpi;
    let x87_inv = ext_to_f64(&ext_div(&ef(1.0), &ext_sqrt(&ef(std::f64::consts::PI), CW), CW), CW);
    let sqrpi_f: f64 = 5.6418958354775628695e-1;
    println!(
        "C0={:016x} RPINV={:016x} 1/sqrt(pi)={:016x} x87={:016x} SQRPI={:016x}",
        C0[0].to_bits(),
        f::RPINV.to_bits(),
        inv_sqrtpi.to_bits(),
        x87_inv.to_bits(),
        sqrpi_f.to_bits()
    );
    for (name, c0) in [
        ("fitted C[0]", C0[0]),
        ("RPINV", f::RPINV),
        ("1/sqrt(pi) f64", inv_sqrtpi),
        ("x87 1/sqrt(pi)", x87_inv),
        ("FORTRAN SQRPI", sqrpi_f),
    ] {
        let mut c = C0;
        c[0] = c0;
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut max = 0u64;
        let mut dmid = 0usize;
        for r in &rows {
            if r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            let qg = f::w_rn53(r.z) * cody_cd(r.z, &c);
            if !qg.is_finite() {
                continue;
            }
            let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            n += 1;
            if d == 0 {
                ex += 1;
                if r.direct {
                    dmid += 1;
                }
            } else {
                max = max.max(d);
            }
        }
        println!("{name:18} {ex}/{n} max={max} dmid={dmid}");
    }
}
