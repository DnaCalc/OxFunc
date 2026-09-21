//! DERFC0 fused ∪ F± last-store on Q [0.5,4). Not an identity.
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
fn qmul(w: f64, ff: f64) -> f64 {
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
    let mut n = 0usize;
    let mut nd = 0usize;
    let mut h0 = 0usize;
    let mut hd0 = 0usize;
    let mut hls = 0usize;
    let mut hdls = 0usize;
    let mut hu = 0usize;
    let mut hdu = 0usize;
    let mut nlo = 0usize;
    let mut hit_lo = 0usize;
    let mut nhi = 0usize;
    let mut hit_hi = 0usize;
    let mut miss = 0usize;
    println!("DERFC0 fused ∪ F± last-store Q [0.5,4):");
    for r in rows.iter().filter(|rr| rr.z >= 0.5 && rr.z < 4.0) {
        n += 1;
        if r.direct {
            nd += 1;
        }
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(r.z);
        let ff = f::nswc_derfc0(r.z);
        let g0 = qmul(w, ff);
        let e0 = ulp_distance(g0, t).unwrap_or(99) == 0;
        let els = (-4i32..=4).filter(|&k| k != 0).any(|k| {
            ulp_distance(qmul(w, poke(ff, k)), t).unwrap_or(99) == 0
        });
        if e0 {
            h0 += 1;
            if r.direct {
                hd0 += 1;
            }
        }
        if els {
            hls += 1;
            if r.direct {
                hdls += 1;
            }
        }
        if e0 || els {
            hu += 1;
            if r.direct {
                hdu += 1;
            }
        } else {
            miss += 1;
            let d = ulp_distance(g0, t).unwrap_or(99);
            println!(
                "  MISS z={:.16} direct={} fused={}{}",
                r.z,
                r.direct,
                d,
                if g0 < t { "L" } else { "H" }
            );
        }
        let d = ulp_distance(g0, t).unwrap_or(99);
        if d >= 2 && g0 < t {
            nlo += 1;
            if els {
                hit_lo += 1;
            } else {
                println!("  LOW-MISS z={:.16} d={d}", r.z);
            }
        }
        if d >= 2 && g0 > t {
            nhi += 1;
            if els {
                hit_hi += 1;
            } else {
                println!("  HIGH-MISS z={:.16} d={d}", r.z);
            }
        }
    }
    println!(
        "Q n={n} fused={h0} F±={hls} union={hu}/{n} leftover-low {hit_lo}/{nlo} leftover-high {hit_hi}/{nhi}"
    );
    println!("DIRECT n={nd} fused={hd0} F±={hdls} union={hdu}/{nd} miss={miss} (Cody unmask∪F± was 226/226)");
}
