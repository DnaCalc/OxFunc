//! cephes_f fused ∪ F±8 DIRECT [0.5,4). Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use std::env;

fn poke(x: f64, k: i32) -> f64 {
    let mut v = x;
    if k > 0 {
        for _ in 0..k {
            v = v.next_up();
        }
    } else {
        for _ in 0..(-k) {
            v = v.next_down();
        }
    }
    v
}
fn mul(w: f64, ff: f64) -> f64 {
    let v = w * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    for win in [4i32, 5, 6, 8] {
        let mut h0 = 0usize;
        let mut hu = 0usize;
        let mut n = 0usize;
        for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
            n += 1;
            let t = f64::from_bits(r.qbits);
            let w = f::w_rn53(r.z);
            let ff = f::cephes_f(r.z);
            let e0 = ulp_distance(mul(w, ff), t).unwrap_or(99) == 0;
            let els = (-win..=win)
                .filter(|&k| k != 0)
                .any(|k| ulp_distance(mul(w, poke(ff, k)), t).unwrap_or(99) == 0);
            if e0 {
                h0 += 1;
            }
            if e0 || els {
                hu += 1;
            }
        }
        println!("cephes F±{win} fused={h0} union={hu}/{n}");
    }
}
