//! Q-compose of Cody store-masks 0x74 / 0x210 vs mask-0. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
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
const PP: [f64; 6] = [
    0.305326634961232344,
    0.360344899949804439,
    0.125781726111229246,
    0.0160837851487422766,
    6.58749161529837803e-4,
    0.0163153871373020978,
];
const QQ: [f64; 5] = [
    2.56852019228982242,
    1.87295284992346047,
    0.527905102951428412,
    0.0605183413124413191,
    0.00233520497626869185,
];

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn maybe(x: Ext80, mask: u32, bit: u32) -> Ext80 {
    if (mask >> bit) & 1 == 1 {
        ef(ext_to_f64(&x, CW))
    } else {
        x
    }
}
fn cody_mask(y: f64, mask: u32) -> f64 {
    let ye = ef(y);
    if y <= 4.0 {
        let mut xnum = ext_mul(&ef(C[8]), &ye, CW);
        let mut xden = ye;
        xnum = maybe(xnum, mask, 0);
        xden = maybe(xden, mask, 1);
        for i in 0..7 {
            xnum = ext_mul(&ext_add(&xnum, &ef(C[i]), CW), &ye, CW);
            xden = ext_mul(&ext_add(&xden, &ef(D[i]), CW), &ye, CW);
            xnum = maybe(xnum, mask, 2 + 2 * i as u32);
            xden = maybe(xden, mask, 3 + 2 * i as u32);
        }
        let mut q = ext_div(
            &ext_add(&xnum, &ef(C[7]), CW),
            &ext_add(&xden, &ef(D[7]), CW),
            CW,
        );
        q = maybe(q, mask, 16);
        ext_to_f64(&q, CW)
    } else {
        let ysq = ext_div(&ef(1.0), &ext_mul(&ye, &ye, CW), CW);
        let mut xnum = ext_mul(&ef(PP[5]), &ysq, CW);
        let mut xden = ysq;
        for i in 0..4 {
            xnum = ext_mul(&ext_add(&xnum, &ef(PP[i]), CW), &ysq, CW);
            xden = ext_mul(&ext_add(&xden, &ef(QQ[i]), CW), &ysq, CW);
        }
        let r = ext_div(
            &ext_mul(&ysq, &ext_add(&xnum, &ef(PP[4]), CW), CW),
            &ext_add(&xden, &ef(QQ[4]), CW),
            CW,
        );
        ext_to_f64(&ext_div(&ext_sub(&ef(f::RPINV), &r, CW), &ye, CW), CW)
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
    for (name, mask) in [("mask0", 0u32), ("mask0x74", 0x74), ("mask0x210", 0x210)] {
        let mut mid = Acc::default();
        let mut tail = Acc::default();
        for r in &rows {
            if r.z < 0.5 {
                continue;
            }
            let f64f = cody_mask(r.z, mask);
            let w = f::w_rn53(r.z);
            let qg = w * f64f;
            if !qg.is_finite() {
                continue;
            }
            let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
            if d > 1 << 20 {
                continue;
            }
            if r.z < 4.0 {
                mid.add(d, r.direct);
            } else {
                tail.add(d, r.direct);
            }
        }
        println!("{name:12} Q=w*F64 mid {} tail {}", fmt(&mid), fmt(&tail));
    }
}
