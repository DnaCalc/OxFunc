//! Schonfelder 1978 Table 3: erfcx Chebyshev in t=(x-3.75)/(x+3.75) as Q=w*F.
//! Coeffs from Chebyshev fit (paper a3 was OCR'd 8 vs 6). Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const K: f64 = 3.75;
/// numpy chebfit c0 + c1 T1 + ... so primed a0 = 2*c0
const C: [f64; 20] = [
    3.05071540961588005825e-01,
    -4.34841272712597426420e-01,
    1.76351193643602105476e-01,
    -6.07107956092359304923e-02,
    1.77120689957185779850e-02,
    -4.32111938554379220595e-03,
    8.54216676898531131722e-04,
    -1.27155090615212786878e-04,
    1.12481672235697892646e-05,
    3.13063860786722558710e-07,
    -2.70988085467857227288e-07,
    3.07376214241329369041e-08,
    2.51563512523473473695e-09,
    -1.02890660591442623657e-09,
    2.99647464214812850309e-11,
    2.60600654601287319338e-11,
    -2.64258899190125898097e-12,
    -6.62959460955010113204e-13,
    9.10600488202903572621e-14,
    6.17906690469500054552e-15,
];
const CODY_C: [f64; 9] = [
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
const CODY_D: [f64; 8] = [
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
fn clenshaw_np(t: Ext80, c: &[f64]) -> Ext80 {
    // numpy: y = c0 + c1 T1 + ... = t*d1 - d2 + c0  (c0 already unprimed)
    let two = ef(2.0);
    let mut d1 = ef(0.0);
    let mut d2 = ef(0.0);
    for k in (1..c.len()).rev() {
        let dk = ext_add(
            &ext_sub(&ext_mul(&ext_mul(&two, &t, CW), &d1, CW), &d2, CW),
            &ef(c[k]),
            CW,
        );
        d2 = d1;
        d1 = dk;
    }
    ext_add(&ext_sub(&ext_mul(&t, &d1, CW), &d2, CW), &ef(c[0]), CW)
}
fn tmap(x: f64) -> Ext80 {
    let xe = ef(x.abs());
    let k = ef(K);
    ext_div(&ext_sub(&xe, &k, CW), &ext_add(&xe, &k, CW), CW)
}
fn t3_f(z: f64) -> f64 {
    ext_to_f64(&clenshaw_np(tmap(z), &C), CW)
}
fn t3_f64(z: f64) -> f64 {
    let t = (z.abs() - K) / (z.abs() + K);
    let mut d1 = 0.0;
    let mut d2 = 0.0;
    for k in (1..C.len()).rev() {
        let dk = 2.0 * t * d1 - d2 + C[k];
        d2 = d1;
        d1 = dk;
    }
    t * d1 - d2 + C[0]
}
fn cody_cd(z: f64) -> f64 {
    let ye = ef(z.abs());
    let mut xnum = ext_mul(&ef(CODY_C[8]), &ye, CW);
    let mut xden = ye;
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(CODY_C[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(CODY_D[i]), CW), &ye, CW);
    }
    ext_to_f64(
        &ext_div(
            &ext_add(&xnum, &ef(CODY_C[7]), CW),
            &ext_add(&xden, &ef(CODY_D[7]), CW),
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
    type Ev = fn(f64) -> f64;
    let graphs: [(&str, Ev); 3] = [
        ("T3 x87 Clenshaw w*F", |z| qw(z, t3_f(z))),
        ("T3 f64 Clenshaw w*F", |z| qw(z, t3_f64(z))),
        ("Cody C/D x87 w*F", |z| qw(z, cody_cd(z))),
    ];
    for (name, ev) in graphs {
        let mut mid_ex = 0usize;
        let mut mid_n = 0usize;
        let mut dmid = 0usize;
        let mut dn = 0usize;
        let mut max_d = 0u64;
        let mut tail_ex = 0usize;
        let mut tail_n = 0usize;
        let mut dtail = 0usize;
        let mut tdn = 0usize;
        let mut max_t = 0u64;
        for r in &rows {
            if r.z < 0.5 {
                continue;
            }
            let qg = ev(r.z);
            if !qg.is_finite() {
                continue;
            }
            let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            if r.z < 4.0 {
                mid_n += 1;
                if d == 0 {
                    mid_ex += 1;
                }
                if r.direct {
                    dn += 1;
                    max_d = max_d.max(d);
                    if d == 0 {
                        dmid += 1;
                    }
                }
            } else {
                tail_n += 1;
                if d == 0 {
                    tail_ex += 1;
                }
                if r.direct {
                    tdn += 1;
                    max_t = max_t.max(d);
                    if d == 0 {
                        dtail += 1;
                    }
                }
            }
        }
        println!(
            "{name:22} mid {mid_ex}/{mid_n} dmid={dmid}/{dn} maxd={max_d}  tail {tail_ex}/{tail_n} dtail={dtail}/{tdn} maxt={max_t}"
        );
    }
}
