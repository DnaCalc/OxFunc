//! Q-association leftover: w*(num/den) vs (w*num)/den vs num*(w/den).
//! x87-cont and last-op RN. Not an identity.
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
fn numden(y: f64) -> (Ext80, Ext80) {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C[8]), &ye, CW);
    let mut xden = ye;
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D[i]), CW), &ye, CW);
    }
    (
        ext_add(&xnum, &ef(C[7]), CW),
        ext_add(&xden, &ef(D[7]), CW),
    )
}
fn flush(v: f64) -> f64 {
    if v.abs() < f64::MIN_POSITIVE { 0.0 } else { v }
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
    type Ev = fn(f64) -> f64;
    let graphs: [(&str, Ev); 8] = [
        ("w * (num/den) f64", |z| {
            let (n, d) = numden(z);
            let ff = ext_to_f64(&ext_div(&n, &d, CW), CW);
            flush(f::w_rn53(z) * ff)
        }),
        ("(w*num)/den x87 then st", |z| {
            let (n, d) = numden(z);
            let w = ef(f::w_rn53(z));
            flush(ext_to_f64(&ext_div(&ext_mul(&w, &n, CW), &d, CW), CW))
        }),
        ("num*(w/den) x87 then st", |z| {
            let (n, d) = numden(z);
            let w = ef(f::w_rn53(z));
            flush(ext_to_f64(&ext_mul(&n, &ext_div(&w, &d, CW), CW), CW))
        }),
        ("(w/den)*num x87 then st", |z| {
            let (n, d) = numden(z);
            let w = ef(f::w_rn53(z));
            flush(ext_to_f64(&ext_mul(&ext_div(&w, &d, CW), &n, CW), CW))
        }),
        ("w*(num/den) x87 mul then st", |z| {
            let (n, d) = numden(z);
            let ff = ext_div(&n, &d, CW);
            flush(ext_to_f64(&ext_mul(&ef(f::w_rn53(z)), &ff, CW), CW))
        }),
        ("f64 (w*num64)/den64", |z| {
            let (n, d) = numden(z);
            let nn = ext_to_f64(&n, CW);
            let dd = ext_to_f64(&d, CW);
            flush((f::w_rn53(z) * nn) / dd)
        }),
        ("f64 num64*(w/den64)", |z| {
            let (n, d) = numden(z);
            let nn = ext_to_f64(&n, CW);
            let dd = ext_to_f64(&d, CW);
            flush(nn * (f::w_rn53(z) / dd))
        }),
        ("fma(w, num/den, 0)", |z| {
            let (n, d) = numden(z);
            let ff = ext_to_f64(&ext_div(&n, &d, CW), CW);
            flush(f64::mul_add(f::w_rn53(z), ff, 0.0))
        }),
    ];
    for (name, ev) in graphs {
        let mut mid = Acc::default();
        let mut tail = Acc::default();
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
                mid.add(d, r.direct);
            } else {
                tail.add(d, r.direct);
            }
        }
        println!("{name:32} mid {} tail {}", fmt(&mid), fmt(&tail));
    }
}
