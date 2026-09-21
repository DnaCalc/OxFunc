//! A&S 7.1.14 Lentz n-sweep as Q=w*F. Not mixed n12 store cube. Not an ID.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use std::env;

const ULP_CAP: u64 = 1 << 20;

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
    println!("## as714 x87 Lentz n, Q=w*F flush");
    for ncf in [8u32, 12, 16, 20, 24, 32, 40, 48, 64, 80] {
        let mut mid = Acc::default();
        let mut tail = Acc::default();
        for r in &rows {
            if r.z < 0.5 {
                continue;
            }
            let ff = f::cf_as714_x87_n(r.z, ncf);
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
        println!("n={ncf:2} mid {} tail {}", fmt(&mid), fmt(&tail));
    }
}
