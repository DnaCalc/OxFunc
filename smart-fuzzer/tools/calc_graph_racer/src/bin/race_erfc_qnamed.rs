//! Q-compose named F graphs. Not an identity.
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
    let graphs: [(&str, fn(f64) -> f64); 6] = [
        ("nswc_derfc0 * w", |z| f::w_rn53(z) * f::nswc_derfc0(z)),
        ("cody_erfcx * w", |z| f::w_rn53(z) * f::cody_erfcx_f(z)),
        ("cephes_f * w", |z| f::w_rn53(z) * f::cephes_f(z)),
        ("as714_x87_n80 * w", |z| f::w_rn53(z) * f::cf_as714_x87_n(z, 80)),
        ("libm::erfc", libm::erfc),
        ("cephes_x87 * w", |z| f::w_rn53(z) * f::cephes_f(z)),
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
        println!("{name:22} mid {} tail {}", fmt(&mid), fmt(&tail));
    }
}
