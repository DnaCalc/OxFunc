//! 0x210 C[0]=1/√π and C[7]=D[7] palindrome. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sqrt, ext_to_f64, Ext80, CW_PC64_RN};
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
const RPINV: f64 = 0.56418958354775628695;

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
    let rp80 = ext_to_f64(&ext_div(&ef(1.0), &ext_sqrt(&ef(std::f64::consts::PI), CW), CW), CW);
    println!("C[0] pub={:#x} RPINV={:#x} x87_1/sqrtpi={:#x} C7={:#x} D7={:#x}",
        C0[0].to_bits(), RPINV.to_bits(), rp80.to_bits(), C0[7].to_bits(), D0[7].to_bits());
    let mut c_rp = C0;
    c_rp[0] = RPINV;
    let mut c_80 = C0;
    c_80[0] = rp80;
    let mut c_eqd = C0;
    c_eqd[7] = D0[7];
    let mut d_eqc = D0;
    d_eqc[7] = C0[7];
    let mut c_both = C0;
    c_both[0] = RPINV;
    c_both[7] = D0[7];
    let score = |c: &[f64; 9], d: &[f64; 8]| {
        let mut qe = 0usize;
        let mut dmid = 0usize;
        let mut nq = 0usize;
        let mut nd = 0usize;
        let mut mx = 0u64;
        let mut fe = 0usize;
        for r in &rows {
            if r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            nq += 1;
            let ff = cody(r.z, c, d, 0x210);
            let dq = ulp_distance(qwf(r.z, ff), f64::from_bits(r.qbits)).unwrap_or(99);
            if dq == 0 {
                qe += 1;
            } else {
                mx = mx.max(dq);
            }
            if r.direct {
                nd += 1;
                if dq == 0 {
                    dmid += 1;
                }
            }
            if let Some(fo) = f::f_or(r.z, r.qbits) {
                if ulp_distance(ff, fo).unwrap_or(99) == 0 {
                    fe += 1;
                }
            }
        }
        (qe, nq, dmid, nd, mx, fe)
    };
    for (name, c, d) in [
        ("CR 0x210", &C0, &D0),
        ("C0=RPINV", &c_rp, &D0),
        ("C0=x87 1/√π", &c_80, &D0),
        ("C7=D7", &c_eqd, &D0),
        ("D7=C7", &C0, &d_eqc),
        ("C0=RPINV C7=D7", &c_both, &D0),
    ] {
        let (qe, nq, dmid, nd, mx, fe) = score(c, d);
        println!("{name:16} Q {qe}/{nq} dmid={dmid}/{nd} maxd={mx} F_or={fe}");
    }
}
