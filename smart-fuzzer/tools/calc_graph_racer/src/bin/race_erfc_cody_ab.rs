//! Cody A/B small-erf extended above 0.46875, vs C/D. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const A: [f64; 5] = [
    3.16112374387056560,
    113.864154151050156,
    377.485237685302021,
    3209.37758913846947,
    0.185777706184603153,
];
const B: [f64; 4] = [
    23.6012909523441209,
    244.024637934444173,
    1282.61652607737228,
    2844.23683343917062,
];
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

fn x87_horner(cs: &[f64], x: Ext80) -> Ext80 {
    let mut acc = ext_from_f64(0.0);
    for &c in cs.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &x, CW), &ext_from_f64(c), CW);
    }
    acc
}

fn cody_ab_erfc_x87(z: f64) -> f64 {
    let ye = ext_from_f64(z);
    let ysq = ext_mul(&ye, &ye, CW);
    let mut xnum = ext_mul(&ext_from_f64(A[4]), &ysq, CW);
    let mut xden = ysq;
    for i in 0..3 {
        xnum = ext_mul(&ext_add(&xnum, &ext_from_f64(A[i]), CW), &ysq, CW);
        xden = ext_mul(&ext_add(&xden, &ext_from_f64(B[i]), CW), &ysq, CW);
    }
    let erf = ext_div(
        &ext_mul(
            &ye,
            &ext_add(&xnum, &ext_from_f64(A[3]), CW),
            CW,
        ),
        &ext_add(&xden, &ext_from_f64(B[3]), CW),
        CW,
    );
    ext_to_f64(
        &ext_add(
            &ext_from_f64(0.5),
            &ext_sub(&ext_from_f64(0.5), &erf, CW),
            CW,
        ),
        CW,
    )
}

fn cody_ab_f(z: f64) -> f64 {
    let w = f::w_rn53(z);
    if w == 0.0 {
        f64::NAN
    } else {
        cody_ab_erfc_x87(z) / w
    }
}

fn cody_cd(y: f64) -> f64 {
    let ye = ext_from_f64(y);
    let mut xnum = ext_mul(&ext_from_f64(C[8]), &ye, CW);
    let mut xden = ye;
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ext_from_f64(C[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ext_from_f64(D[i]), CW), &ye, CW);
    }
    ext_to_f64(
        &ext_div(
            &ext_add(&xnum, &ext_from_f64(C[7]), CW),
            &ext_add(&xden, &ext_from_f64(D[7]), CW),
            CW,
        ),
        CW,
    )
}

#[derive(Default)]
struct Acc {
    exact: usize,
    n: usize,
    max_ulp: u64,
    sum_ulp: u128,
}
impl Acc {
    fn add(&mut self, d: u64) {
        self.n += 1;
        if d == 0 {
            self.exact += 1;
        } else {
            self.max_ulp = self.max_ulp.max(d);
            self.sum_ulp += d as u128;
        }
    }
}
fn fmt(a: &Acc) -> String {
    format!("{}/{} max={} sum={}", a.exact, a.n, a.max_ulp, a.sum_ulp)
}
fn score(rows: &[f::QRow], eval: impl Fn(f64) -> f64) -> (Acc, Acc, Acc) {
    let mut mid = Acc::default();
    let mut tail = Acc::default();
    let mut dmid = Acc::default();
    for r in rows {
        if r.z < 0.5 {
            continue;
        }
        let Some(fo) = f::f_or(r.z, r.qbits) else {
            continue;
        };
        let fg = eval(r.z);
        if !fg.is_finite() {
            continue;
        }
        let d = ulp_distance(fg, fo).unwrap_or(u64::MAX);
        if d > ULP_CAP {
            continue;
        }
        if r.z < 4.0 {
            mid.add(d);
            if r.direct {
                dmid.add(d);
            }
        } else {
            tail.add(d);
        }
    }
    (mid, tail, dmid)
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    println!("## Cody A/B small-erf / C/D cut sweep");
    let mut best = 0usize;
    let mut best_c = 0.0;
    for k in 46..=120 {
        let cut = k as f64 / 100.0;
        let (m, t, dm) = score(&rows, |z| {
            if z < cut {
                cody_ab_f(z)
            } else {
                cody_cd(z)
            }
        });
        if m.exact >= best {
            best = m.exact;
            best_c = cut;
            println!(
                "cut={cut:.2} mid {} tail {} dmid {}",
                fmt(&m),
                fmt(&t),
                fmt(&dm)
            );
        }
    }
    println!("best Cody A/B cut={best_c:.2} mid={best}");
    let (m, t, dm) = score(&rows, cody_ab_f);
    println!("Cody A/B global mid {} tail {} dmid {}", fmt(&m), fmt(&t), fmt(&dm));
    let (m, t, dm) = score(&rows, |z| {
        if z < 0.46875 {
            cody_ab_f(z)
        } else {
            cody_cd(z)
        }
    });
    println!(
        "Cody documented 0.46875 A/B else C/D mid {} tail {} dmid {}",
        fmt(&m),
        fmt(&t),
        fmt(&dm)
    );
}
