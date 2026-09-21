//! DIRECT Q row counts by z band. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use std::env;

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mut n0 = 0usize;
    let mut d0 = 0usize;
    let mut n1 = 0usize;
    let mut d1 = 0usize;
    let mut n2 = 0usize;
    let mut d2 = 0usize;
    let mut n3 = 0usize;
    let mut d3 = 0usize;
    for r in &rows {
        if r.z < 0.0 {
            n0 += 1;
            if r.direct {
                d0 += 1;
            }
        } else if r.z < 0.5 {
            n1 += 1;
            if r.direct {
                d1 += 1;
            }
        } else if r.z < 4.0 {
            n2 += 1;
            if r.direct {
                d2 += 1;
            }
        } else {
            n3 += 1;
            if r.direct {
                d3 += 1;
            }
        }
    }
    println!(
        "Q bands z<0 n={n0} d={d0}  [0,0.5) n={n1} d={d1}  [0.5,4) n={n2} d={d2}  z>=4 n={n3} d={d3}  total n={} d={}",
        rows.len(),
        d0 + d1 + d2 + d3
    );
}
