//! SPECFUN C/D fused loop with 80-bit RN of printed decimals vs f64-wide. Q=w*F. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const C: [f64; 9] = [
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
const C80: [Ext80; 9] = [
    Ext80([0x14, 0xed, 0x81, 0x58, 0x47, 0xa8, 0x6e, 0x90, 0xfe, 0x3f]),
    Ext80([0x0b, 0x0c, 0xe0, 0xc5, 0xad, 0x61, 0x21, 0x8e, 0x02, 0x40]),
    Ext80([0xd4, 0x80, 0x06, 0x21, 0x8e, 0x06, 0x3d, 0x84, 0x05, 0x40]),
    Ext80([0xf4, 0x10, 0x23, 0x5d, 0x35, 0x4c, 0x51, 0x95, 0x07, 0x40]),
    Ext80([0xa3, 0x1a, 0xfd, 0x5c, 0x31, 0xf1, 0x7c, 0xdc, 0x08, 0x40]),
    Ext80([0xb0, 0xb4, 0x46, 0xee, 0x0a, 0x86, 0x01, 0xd6, 0x09, 0x40]),
    Ext80([0x2e, 0x56, 0x7b, 0x1b, 0x09, 0x41, 0x31, 0x80, 0x0a, 0x40]),
    Ext80([0xaa, 0x71, 0xe4, 0x97, 0xfe, 0xdb, 0xca, 0x99, 0x09, 0x40]),
    Ext80([0xe2, 0xaa, 0xc0, 0xfa, 0xe2, 0x81, 0xf3, 0xb8, 0xe5, 0x3f]),
];
const D80: [Ext80; 8] = [
    Ext80([0x68, 0xa4, 0xf6, 0x36, 0xa4, 0x37, 0xeb, 0xfb, 0x02, 0x40]),
    Ext80([0x42, 0x39, 0x6f, 0xfe, 0x87, 0x4d, 0x63, 0xeb, 0x05, 0x40]),
    Ext80([0x4e, 0xee, 0xa4, 0x43, 0x2c, 0x97, 0x4b, 0x86, 0x08, 0x40]),
    Ext80([0x24, 0x35, 0xe9, 0x14, 0x65, 0x77, 0xac, 0xca, 0x09, 0x40]),
    Ext80([0x17, 0x0f, 0x90, 0x68, 0xab, 0xc9, 0xac, 0xcd, 0x0a, 0x40]),
    Ext80([0x18, 0x2e, 0x74, 0x88, 0xe5, 0xf3, 0x54, 0x88, 0x0b, 0x40]),
    Ext80([0x70, 0xe8, 0x6d, 0x48, 0xfe, 0xe1, 0xf5, 0xd6, 0x0a, 0x40]),
    Ext80([0x27, 0x04, 0xfb, 0x9a, 0xfe, 0xdb, 0xca, 0x99, 0x09, 0x40]),
];

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn specfun_f64(y: f64) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C[8]), &ye, CW);
    let mut xden = ye;
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D[i]), CW), &ye, CW);
    }
    ext_to_f64(
        &ext_div(
            &ext_add(&xnum, &ef(C[7]), CW),
            &ext_add(&xden, &ef(D[7]), CW),
            CW,
        ),
        CW,
    )
}
fn specfun_80(y: f64) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&C80[8], &ye, CW);
    let mut xden = ye;
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &C80[i], CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &D80[i], CW), &ye, CW);
    }
    ext_to_f64(
        &ext_div(
            &ext_add(&xnum, &C80[7], CW),
            &ext_add(&xden, &D80[7], CW),
            CW,
        ),
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
    println!(
        "C0 f64-wide {:016x} 80store {:016x}",
        ext_to_f64(&ef(C[0]), CW).to_bits(),
        ext_to_f64(&C80[0], CW).to_bits()
    );
    for (name, ev) in [
        ("C/D f64-wide fused", specfun_f64 as fn(f64) -> f64),
        ("C/D 80-bit decimal fused", specfun_80),
    ] {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut maxu = 0u64;
        let mut dmid = 0usize;
        let mut dn = 0usize;
        let mut tex = 0usize;
        let mut tn = 0usize;
        let mut tmax = 0u64;
        for r in &rows {
            let qg = qw(r.z, ev(r.z));
            if !qg.is_finite() {
                continue;
            }
            let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            if r.z >= 0.5 && r.z < 4.0 {
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
            } else if r.z >= 4.0 {
                tn += 1;
                if d == 0 {
                    tex += 1;
                } else {
                    tmax = tmax.max(d);
                }
            }
        }
        println!("{name:28} mid {ex}/{n} max={maxu} dmid={dmid}/{dn} tail {tex}/{tn} max={tmax}");
    }
}
