//! NSWC PQR 80-bit printed decimals as Q=w*F. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const P: [f64; 8] = [
    0.16506148041280876191828601e-03,
    0.15471455377139313353998665e-03,
    0.44852548090298868465196794e-04,
    -0.49177280017226285450486205e-05,
    -0.69353602078656412367801676e-05,
    -0.20508667787746282746857743e-05,
    -0.28982842617824971177267380e-06,
    -0.17272433544836633301127174e-07,
];
const Q: [f64; 8] = [
    1.0,
    0.16272656776533322859856317e+01,
    0.12040996037066026106794322e+01,
    0.52400246352158386907601472e+00,
    0.14497345252798672362384241e+00,
    0.25592517111042546492590736e-01,
    0.26869088293991371028123158e-02,
    0.13133767840925681614496481e-03,
];
const R: [f64; 9] = [
    0.145589721275038539045668824025,
    -0.273421931495426482902320421863,
    0.226008066916621506788789064272,
    -0.163571895523923805648814425592,
    0.102604312032193978662297299832,
    -0.548023266949835519254211506880e-01,
    0.241432239725390106956523668160e-01,
    -0.822062115403915116036874169600e-02,
    0.180296241564687154310619200000e-02,
];
const PP80: [Ext80; 8] = [
    Ext80([0x33, 0x14, 0x59, 0x30, 0x90, 0x5a, 0x14, 0xad, 0xf2, 0x3f]),
    Ext80([0xb7, 0x80, 0xa3, 0xb9, 0x2d, 0xdf, 0x3a, 0xa2, 0xf2, 0x3f]),
    Ext80([0x65, 0x54, 0x59, 0x48, 0x8a, 0x0e, 0x20, 0xbc, 0xf0, 0x3f]),
    Ext80([0xd8, 0xd3, 0x28, 0x82, 0x3d, 0xf6, 0x02, 0xa5, 0xed, 0xbf]),
    Ext80([0x7e, 0x1f, 0x5d, 0xfa, 0x61, 0x4a, 0xb6, 0xe8, 0xed, 0xbf]),
    Ext80([0x4c, 0xb0, 0x42, 0x2b, 0x7b, 0x9f, 0xa1, 0x89, 0xec, 0xbf]),
    Ext80([0x66, 0x36, 0x86, 0x46, 0x30, 0xb7, 0x99, 0x9b, 0xe9, 0xbf]),
    Ext80([0x6b, 0xb3, 0x83, 0xd7, 0xa8, 0x7b, 0x5e, 0x94, 0xe5, 0xbf]),
];
const QQ80: [Ext80; 8] = [
    Ext80([0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0xff, 0x3f]),
    Ext80([0x8b, 0xc7, 0x50, 0xb6, 0xe1, 0x3d, 0x4a, 0xd0, 0xff, 0x3f]),
    Ext80([0x60, 0x0b, 0xf1, 0x85, 0x91, 0xef, 0x1f, 0x9a, 0xff, 0x3f]),
    Ext80([0x93, 0x30, 0x40, 0xd9, 0x83, 0x06, 0x25, 0x86, 0xfe, 0x3f]),
    Ext80([0x31, 0xd3, 0x69, 0x95, 0xb5, 0xeb, 0x73, 0x94, 0xfc, 0x3f]),
    Ext80([0xa8, 0x82, 0xb8, 0x74, 0x00, 0x66, 0xa7, 0xd1, 0xf9, 0x3f]),
    Ext80([0x01, 0x31, 0xc7, 0xb2, 0x8c, 0xd9, 0x16, 0xb0, 0xf6, 0x3f]),
    Ext80([0xa8, 0x91, 0xa2, 0x37, 0x89, 0xb0, 0xb7, 0x89, 0xf2, 0x3f]),
];
const RR80: [Ext80; 9] = [
    Ext80([0x0c, 0x7c, 0x49, 0x0a, 0xce, 0x78, 0x15, 0x95, 0xfc, 0x3f]),
    Ext80([0xc3, 0x7f, 0x63, 0x90, 0x9b, 0xf5, 0xfd, 0x8b, 0xfd, 0xbf]),
    Ext80([0x8b, 0xaf, 0x01, 0x28, 0xa0, 0xa8, 0x6e, 0xe7, 0xfc, 0x3f]),
    Ext80([0x09, 0x3d, 0xa9, 0x47, 0x17, 0x64, 0x7f, 0xa7, 0xfc, 0xbf]),
    Ext80([0x64, 0x73, 0xd5, 0xda, 0xa4, 0x35, 0x22, 0xd2, 0xfb, 0x3f]),
    Ext80([0x2c, 0x1c, 0x04, 0x65, 0x8e, 0x67, 0x78, 0xe0, 0xfa, 0xbf]),
    Ext80([0xbd, 0x3a, 0xd2, 0x39, 0xac, 0x02, 0xc8, 0xc5, 0xf9, 0x3f]),
    Ext80([0xad, 0xf3, 0x12, 0x9a, 0xc0, 0xc8, 0xaf, 0x86, 0xf8, 0xbf]),
    Ext80([0xfd, 0x51, 0xc8, 0xe4, 0x38, 0x61, 0x51, 0xec, 0xf5, 0x3f]),
];

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn horner_f(cs: &[f64], x: Ext80) -> Ext80 {
    let mut acc = ef(0.0);
    for &c in cs.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &x, CW), &ef(c), CW);
    }
    acc
}
fn horner_80(cs: &[Ext80], x: Ext80) -> Ext80 {
    let mut acc = ef(0.0);
    for c in cs.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &x, CW), c, CW);
    }
    acc
}
fn pqr_f64(x: f64) -> f64 {
    let xe = ef(x.abs());
    let u = horner_f(&P, xe);
    let v = horner_f(&Q, xe);
    let t = ext_div(
        &ext_sub(&xe, &ef(3.75), CW),
        &ext_add(&xe, &ef(3.75), CW),
        CW,
    );
    let mut acc = ext_div(&u, &v, CW);
    for &r in R.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW), &ef(r), CW);
    }
    ext_to_f64(&acc, CW)
}
fn pqr_80(x: f64) -> f64 {
    let xe = ef(x.abs());
    let u = horner_80(&PP80, xe);
    let v = horner_80(&QQ80, xe);
    let t = ext_div(
        &ext_sub(&xe, &ef(3.75), CW),
        &ext_add(&xe, &ef(3.75), CW),
        CW,
    );
    let mut acc = ext_div(&u, &v, CW);
    for r in RR80.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW), r, CW);
    }
    ext_to_f64(&acc, CW)
}
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
fn specfun_cd(y: f64) -> f64 {
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
    for (name, ev) in [
        ("PQR f64-wide x87", pqr_f64 as fn(f64) -> f64),
        ("PQR 80-bit decimals", pqr_80),
        ("PQR f64 native packet", f::nswc_pqr_f),
    ] {
        let mut mex = 0usize;
        let mut mn = 0usize;
        let mut mmax = 0u64;
        let mut tex = 0usize;
        let mut tn = 0usize;
        let mut tmax = 0u64;
        let mut td = 0usize;
        let mut tdn = 0usize;
        for r in &rows {
            if r.z < 0.5 {
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
            if r.z < 4.0 {
                mn += 1;
                if d == 0 {
                    mex += 1;
                } else {
                    mmax = mmax.max(d);
                }
            } else {
                tn += 1;
                if r.direct {
                    tdn += 1;
                }
                if d == 0 {
                    tex += 1;
                    if r.direct {
                        td += 1;
                    }
                } else {
                    tmax = tmax.max(d);
                }
            }
        }
        println!("{name:24} mid {mex}/{mn} max={mmax}  tail {tex}/{tn} max={tmax} d={td}/{tdn}");
    }
    let mut both = 0usize;
    let mut only_p = 0usize;
    let mut only_c = 0usize;
    let mut band_p = [0usize; 8];
    let mut band_c = [0usize; 8];
    let mut band_n = [0usize; 8];
    for r in &rows {
        if r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let want = f64::from_bits(r.qbits);
        let dp = ulp_distance(qw(r.z, pqr_80(r.z)), want).unwrap_or(99);
        let dc = ulp_distance(qw(r.z, specfun_cd(r.z)), want).unwrap_or(99);
        let b = ((r.z - 0.5) / 0.5).floor() as usize;
        let b = b.min(7);
        band_n[b] += 1;
        if dp == 0 {
            band_p[b] += 1;
        }
        if dc == 0 {
            band_c[b] += 1;
        }
        match (dp == 0, dc == 0) {
            (true, true) => both += 1,
            (true, false) => only_p += 1,
            (false, true) => only_c += 1,
            _ => {}
        }
    }
    println!(
        "overlap PQR80 vs C/D both={} only_pqr={} only_cd={} union={}",
        both,
        only_p,
        only_c,
        both + only_p + only_c
    );
    for i in 0..8 {
        let lo = 0.5 + i as f64 * 0.5;
        println!(
            "[{:.1},{:.1}) n={} pqr80={} cd={}",
            lo,
            lo + 0.5,
            band_n[i],
            band_p[i],
            band_c[i]
        );
    }
    let cuts = [
        0.75, 1.0, 1.25, 1.4, 1.45, 1.5, 1.55, 1.6, 1.75, 2.0, 2.5, 3.0, 3.5,
    ];
    for &c in &cuts {
        let mut pc = 0usize;
        let mut cp = 0usize;
        for r in &rows {
            if r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            let want = f64::from_bits(r.qbits);
            let g = if r.z < c {
                qw(r.z, pqr_80(r.z))
            } else {
                qw(r.z, specfun_cd(r.z))
            };
            let h = if r.z < c {
                qw(r.z, specfun_cd(r.z))
            } else {
                qw(r.z, pqr_80(r.z))
            };
            if ulp_distance(g, want).unwrap_or(99) == 0 {
                pc += 1;
            }
            if ulp_distance(h, want).unwrap_or(99) == 0 {
                cp += 1;
            }
        }
        println!("cut={c:.2} PQR80-then-CD={pc} CD-then-PQR80={cp}");
    }
}
