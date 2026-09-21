//! Gautschi erfcx CF: RPINV/(z + (1/2)/(z + 1/(z + 3/2)/(z+...))). Q=w*F.
//! Distinct from A&S 7.1.14 z^2 form. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn gautschi_f64(z: f64, n: u32) -> f64 {
    let mut acc = 0.0;
    for k in (1..=n).rev() {
        acc = (k as f64) * 0.5 / (z + acc);
    }
    f::RPINV / (z + acc)
}
fn gautschi_x87(z: f64, n: u32) -> f64 {
    let ze = ef(z);
    let mut acc = ef(0.0);
    for k in (1..=n).rev() {
        let num = ef(k as f64 * 0.5);
        acc = ext_div(&num, &ext_add(&ze, &acc, CW), CW);
    }
    ext_to_f64(&ext_div(&ef(f::RPINV), &ext_add(&ze, &acc, CW), CW), CW)
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
    println!("## Gautschi CF Q=w*F flush");
    for ncf in [8u32, 12, 16, 24, 32, 48, 80] {
        for (tag, ev) in [
            ("f64", gautschi_f64 as fn(f64, u32) -> f64),
            ("x87", gautschi_x87),
        ] {
            let mut mid = Acc::default();
            let mut tail = Acc::default();
            for r in &rows {
                if r.z < 0.5 {
                    continue;
                }
                let ff = ev(r.z, ncf);
                let mut qg = f::w_rn53(r.z) * ff;
                if qg.abs() < f64::MIN_POSITIVE {
                    qg = 0.0;
                }
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
            println!("n={ncf:2} {tag} mid {} tail {}", fmt(&mid), fmt(&tail));
        }
    }
}
