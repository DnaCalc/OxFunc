//! Estrin / pairwise vs SPECFUN fused C/D loop. Q=w*F. Not an identity.
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
fn specfun(y: f64) -> f64 {
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
const NUM_A: [f64; 9] = [C[7], C[6], C[5], C[4], C[3], C[2], C[1], C[0], C[8]];
const DEN_A: [f64; 9] = [D[7], D[6], D[5], D[4], D[3], D[2], D[1], D[0], 1.0];
fn horner_a80(a: &[f64; 9], y: f64) -> Ext80 {
    let ye = ef(y.abs());
    let mut acc = ef(a[8]);
    for i in (0..8).rev() {
        acc = ext_add(&ext_mul(&acc, &ye, CW), &ef(a[i]), CW);
    }
    acc
}
fn horner_a(a: &[f64; 9], y: f64) -> f64 {
    ext_to_f64(&horner_a80(a, y), CW)
}
fn estrin_a80(a: &[f64; 9], y: f64) -> Ext80 {
    let ye = ef(y.abs());
    let y2 = ext_mul(&ye, &ye, CW);
    let y4 = ext_mul(&y2, &y2, CW);
    let y8 = ext_mul(&y4, &y4, CW);
    let c0 = ext_add(&ef(a[0]), &ext_mul(&ef(a[1]), &ye, CW), CW);
    let c1 = ext_add(&ef(a[2]), &ext_mul(&ef(a[3]), &ye, CW), CW);
    let c2 = ext_add(&ef(a[4]), &ext_mul(&ef(a[5]), &ye, CW), CW);
    let c3 = ext_add(&ef(a[6]), &ext_mul(&ef(a[7]), &ye, CW), CW);
    let d0 = ext_add(&c0, &ext_mul(&c1, &y2, CW), CW);
    let d1 = ext_add(&c2, &ext_mul(&c3, &y2, CW), CW);
    let e0 = ext_add(&d0, &ext_mul(&d1, &y4, CW), CW);
    ext_add(&e0, &ext_mul(&ef(a[8]), &y8, CW), CW)
}
fn estrin_a(a: &[f64; 9], y: f64) -> f64 {
    let ye = ef(y.abs());
    let y2 = ext_mul(&ye, &ye, CW);
    let y4 = ext_mul(&y2, &y2, CW);
    let y8 = ext_mul(&y4, &y4, CW);
    let c0 = ext_add(&ef(a[0]), &ext_mul(&ef(a[1]), &ye, CW), CW);
    let c1 = ext_add(&ef(a[2]), &ext_mul(&ef(a[3]), &ye, CW), CW);
    let c2 = ext_add(&ef(a[4]), &ext_mul(&ef(a[5]), &ye, CW), CW);
    let c3 = ext_add(&ef(a[6]), &ext_mul(&ef(a[7]), &ye, CW), CW);
    let d0 = ext_add(&c0, &ext_mul(&c1, &y2, CW), CW);
    let d1 = ext_add(&c2, &ext_mul(&c3, &y2, CW), CW);
    let e0 = ext_add(&d0, &ext_mul(&d1, &y4, CW), CW);
    ext_to_f64(&ext_add(&e0, &ext_mul(&ef(a[8]), &y8, CW), CW), CW)
}
fn pairwise_a(a: &[f64; 9], y: f64) -> f64 {
    let ye = ef(y.abs());
    let y2 = ext_mul(&ye, &ye, CW);
    let mut even = ef(a[8]);
    even = ext_add(&ext_mul(&even, &y2, CW), &ef(a[6]), CW);
    even = ext_add(&ext_mul(&even, &y2, CW), &ef(a[4]), CW);
    even = ext_add(&ext_mul(&even, &y2, CW), &ef(a[2]), CW);
    even = ext_add(&ext_mul(&even, &y2, CW), &ef(a[0]), CW);
    let mut odd = ef(a[7]);
    odd = ext_add(&ext_mul(&odd, &y2, CW), &ef(a[5]), CW);
    odd = ext_add(&ext_mul(&odd, &y2, CW), &ef(a[3]), CW);
    odd = ext_add(&ext_mul(&odd, &y2, CW), &ef(a[1]), CW);
    ext_to_f64(&ext_add(&even, &ext_mul(&odd, &ye, CW), CW), CW)
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
        }
    }
}
fn fmt(a: &Acc) -> String {
    format!("{}/{} max={} d={}/{}", a.exact, a.n, a.max_ulp, a.dmid, a.dn)
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let graphs: [(&str, fn(f64) -> f64); 6] = [
        ("SPECFUN fused loop", specfun),
        ("Horner of SPECFUN monomials", |z| {
            horner_a(&NUM_A, z) / horner_a(&DEN_A, z)
        }),
        ("Estrin of SPECFUN monomials", |z| {
            estrin_a(&NUM_A, z) / estrin_a(&DEN_A, z)
        }),
        ("pairwise even/odd", |z| {
            pairwise_a(&NUM_A, z) / pairwise_a(&DEN_A, z)
        }),
        ("Horner 80-bit then x87 div", |z| {
            ext_to_f64(&ext_div(&horner_a80(&NUM_A, z), &horner_a80(&DEN_A, z), CW), CW)
        }),
        ("Estrin 80-bit then x87 div", |z| {
            ext_to_f64(&ext_div(&estrin_a80(&NUM_A, z), &estrin_a80(&DEN_A, z), CW), CW)
        }),
    ];
    for (name, ev) in graphs {
        let mut mid = Acc::default();
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
            mid.add(d, r.direct);
        }
        println!("{name:32} {}", fmt(&mid));
    }
}
