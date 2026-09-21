//! Q-bit piecewise by z-band: Cody 0x74 vs NSWC vs Cephes. Not an identity.
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
fn maybe(x: Ext80, mask: u32, bit: u32) -> Ext80 {
    if bit < 32 && mask & (1u32 << bit) != 0 {
        ef(ext_to_f64(&x, CW))
    } else {
        x
    }
}
fn cody74(y: f64) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, 0x74, 0);
    xden = maybe(xden, 0x74, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D[i]), CW), &ye, CW);
        xnum = maybe(xnum, 0x74, 2 + 2 * i as u32);
        xden = maybe(xden, 0x74, 3 + 2 * i as u32);
    }
    let mut q = ext_div(&ext_add(&xnum, &ef(C[7]), CW), &ext_add(&xden, &ef(D[7]), CW), CW);
    q = maybe(q, 0x74, 16);
    ext_to_f64(&q, CW)
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
        ("cody74 global", |z| qw(z, cody74(z))),
        ("cody74 z<2 else nswc*w", |z| {
            if z < 2.0 {
                qw(z, cody74(z))
            } else {
                qw(z, f::nswc_derfc0(z))
            }
        }),
        ("cody74 z<2 else cephes*w", |z| {
            if z < 2.0 {
                qw(z, cody74(z))
            } else {
                qw(z, f::cephes_f(z))
            }
        }),
        ("nswc z<2 else cody74", |z| {
            if z < 2.0 {
                qw(z, f::nswc_derfc0(z))
            } else {
                qw(z, cody74(z))
            }
        }),
        ("cody74 z<1 else nswc*w", |z| {
            if z < 1.0 {
                qw(z, cody74(z))
            } else {
                qw(z, f::nswc_derfc0(z))
            }
        }),
        ("cody74 z<2 else as714n80*w", |z| {
            if z < 2.0 {
                qw(z, cody74(z))
            } else {
                qw(z, f::cf_as714_x87_n(z, 80))
            }
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
