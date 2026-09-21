//! FMA Horner of SPECFUN C/D as Q=w*F, plus leftover miss-1 fingerprint.
//! Not an identity.
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

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn cody_x87(y: f64) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C[8]), &ye, CW);
    let mut xden = ye;
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D[i]), CW), &ye, CW);
    }
    ext_to_f64(
        &ext_div(&ext_add(&xnum, &ef(C[7]), CW), &ext_add(&xden, &ef(D[7]), CW), CW),
        CW,
    )
}
fn cody_f64(y: f64) -> f64 {
    let ye = y.abs();
    let mut xnum = C[8] * ye;
    let mut xden = ye;
    for i in 0..7 {
        xnum = (xnum + C[i]) * ye;
        xden = (xden + D[i]) * ye;
    }
    (xnum + C[7]) / (xden + D[7])
}
fn cody_fma_addmul(y: f64) -> f64 {
    let ye = y.abs();
    let mut xnum = C[8] * ye;
    let mut xden = ye;
    for i in 0..7 {
        xnum = f64::mul_add(xnum + C[i], ye, 0.0);
        xden = f64::mul_add(xden + D[i], ye, 0.0);
    }
    (xnum + C[7]) / (xden + D[7])
}
fn cody_fma_inner(y: f64) -> f64 {
    let ye = y.abs();
    let mut xnum = C[8] * ye;
    let mut xden = ye;
    for i in 0..7 {
        xnum = f64::mul_add(xnum, ye, C[i] * ye);
        xden = f64::mul_add(xden, ye, D[i] * ye);
    }
    (xnum + C[7]) / (xden + D[7])
}
fn cody_fma_ci(y: f64) -> f64 {
    let ye = y.abs();
    let mut xnum = C[8] * ye;
    let mut xden = ye;
    for i in 0..7 {
        xnum = f64::mul_add(C[i], ye, xnum * ye);
        xden = f64::mul_add(D[i], ye, xden * ye);
    }
    (xnum + C[7]) / (xden + D[7])
}
fn qw(z: f64, ff: f64) -> f64 {
    let v = f::w_rn53(z) * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}

#[derive(Default)]
struct Acc {
    exact: usize,
    n: usize,
    max_ulp: u64,
    dmid: usize,
    dn: usize,
    miss1: usize,
}
impl Acc {
    fn add(&mut self, d: u64, direct: bool) {
        self.n += 1;
        if direct {
            self.dn += 1;
        }
        if d == 0 {
            self.exact += 1;
            if direct {
                self.dmid += 1;
            }
        } else {
            self.max_ulp = self.max_ulp.max(d);
            if d == 1 {
                self.miss1 += 1;
            }
        }
    }
}
fn fmt(a: &Acc) -> String {
    format!(
        "{}/{} max={} d={}/{} m1={}",
        a.exact, a.n, a.max_ulp, a.dmid, a.dn, a.miss1
    )
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let graphs: [(&str, fn(f64) -> f64); 5] = [
        ("x87 SPECFUN", |z| qw(z, cody_x87(z))),
        ("f64 (x+c)*y", |z| qw(z, cody_f64(z))),
        ("fma((x+c),y,0)", |z| qw(z, cody_fma_addmul(z))),
        ("fma(x,y,c*y)", |z| qw(z, cody_fma_inner(z))),
        ("fma(c,y,x*y)", |z| qw(z, cody_fma_ci(z))),
    ];
    for (name, ev) in graphs {
        let mut mid = Acc::default();
        for r in &rows {
            if r.z < 0.5 || r.z >= 4.0 {
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
            mid.add(d, r.direct);
        }
        println!("{name:22} {}", fmt(&mid));
    }

    println!("## leftover miss-1 of x87 w*F");
    let mut n_m1 = 0usize;
    let mut n_m1_dir = 0usize;
    let mut n_m1_imp = 0usize;
    let mut pos = 0usize;
    let mut neg = 0usize;
    let mut f_only = 0usize;
    let mut w_only = 0usize;
    let mut both = 0usize;
    let mut mul_only = 0usize;
    let mut max_dir = 0u64;
    let mut max_imp = 0u64;
    let mut hist = [0usize; 8];
    for r in &rows {
        if r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let ff = cody_x87(r.z);
        let w = f::w_rn53(r.z);
        let qg = qw(r.z, ff);
        let qo = f64::from_bits(r.qbits);
        let d = ulp_distance(qg, qo).unwrap_or(u64::MAX);
        if d > ULP_CAP {
            continue;
        }
        if r.direct {
            max_dir = max_dir.max(d);
        } else {
            max_imp = max_imp.max(d);
        }
        let bucket = d.min(7) as usize;
        hist[bucket] += 1;
        if d != 1 {
            continue;
        }
        n_m1 += 1;
        if r.direct {
            n_m1_dir += 1;
        } else {
            n_m1_imp += 1;
        }
        if qg > qo {
            pos += 1;
        } else {
            neg += 1;
        }
        let fo = qo / w;
        let wo = qo / ff;
        let df = ulp_distance(ff, fo).unwrap_or(99);
        let dw = ulp_distance(w, wo).unwrap_or(99);
        if dw <= 1 && df > 1 {
            w_only += 1;
        } else if df <= 1 && dw > 1 {
            f_only += 1;
        } else if df <= 1 && dw <= 1 {
            mul_only += 1;
        } else {
            both += 1;
        }
    }
    println!(
        "miss1 {n_m1} direct={n_m1_dir} implied={n_m1_imp} sign +{pos} -{neg}"
    );
    println!("  w_only={w_only} f_only={f_only} mul_only={mul_only} both={both}");
    println!("  maxULP direct={max_dir} implied={max_imp}");
    println!(
        "  hist ulp0..7 = {} {} {} {} {} {} {} {}",
        hist[0], hist[1], hist[2], hist[3], hist[4], hist[5], hist[6], hist[7]
    );
}
