//! Cody P/Q ±1 and last-constant steps vs 3 stubborn leftover-low. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
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
const STUB: [f64; 3] = [4.0520833333333330, 5.3333333333333330, 6.0000000000000000];

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
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
fn cody_pq(y: f64, p: &[f64; 6], q: &[f64; 5]) -> f64 {
    let ye = ef(y);
    let ysq = ext_div(&ef(1.0), &ext_mul(&ye, &ye, CW), CW);
    let mut xnum = ext_mul(&ef(p[5]), &ysq, CW);
    let mut xden = ysq;
    for i in 0..4 {
        xnum = ext_mul(&ext_add(&xnum, &ef(p[i]), CW), &ysq, CW);
        xden = ext_mul(&ext_add(&xden, &ef(q[i]), CW), &ysq, CW);
    }
    let r = ext_div(
        &ext_mul(&ysq, &ext_add(&xnum, &ef(p[4]), CW), CW),
        &ext_add(&xden, &ef(q[4]), CW),
        CW,
    );
    ext_to_f64(&ext_div(&ext_sub(&ef(f::RPINV), &r, CW), &ye, CW), CW)
}
fn qw(z: f64, ff: f64) -> f64 {
    let v = f::w_rn53(z) * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let tail: Vec<&f::QRow> = rows.iter().filter(|r| r.direct && r.z >= 4.0).collect();
    println!("3 stubborn Cody P/Q CR:");
    for &lz in &STUB {
        let Some(r) = tail.iter().find(|rr| (rr.z - lz).abs() < 1e-12) else {
            println!("  z={lz:.16} MISSING");
            continue;
        };
        let t = f64::from_bits(r.qbits);
        let g = qw(lz, cody_pq(lz, &PP, &QQ));
        let d = ulp_distance(g, t).unwrap_or(99);
        let dir = if g < t {
            "L"
        } else if g > t {
            "H"
        } else {
            "="
        };
        println!("  z={lz:.16} {d}{dir}");
    }
    let sc = |p: &[f64; 6], q: &[f64; 5]| {
        let mut qd = 0usize;
        let mut qe = 0usize;
        let mut hit = 0usize;
        let mut zs = Vec::new();
        for row in rows.iter().filter(|rr| rr.z >= 4.0) {
            let g = qw(row.z, cody_pq(row.z, p, q));
            if !g.is_finite() {
                continue;
            }
            if ulp_distance(g, f64::from_bits(row.qbits)).unwrap_or(99) == 0 {
                qe += 1;
                if row.direct {
                    qd += 1;
                }
            }
        }
        for &lz in &STUB {
            let Some(r) = tail.iter().find(|rr| (rr.z - lz).abs() < 1e-12) else {
                continue;
            };
            if ulp_distance(qw(lz, cody_pq(lz, p, q)), f64::from_bits(r.qbits)).unwrap_or(99) == 0 {
                hit += 1;
                zs.push(lz);
            }
        }
        (qe, qd, hit, zs)
    };
    let (qe0, qd0, h0, _) = sc(&PP, &QQ);
    println!("CR Q={qe0} d={qd0} hit={h0}/3 (bar 0x5005 Q 1572 d=53)");
    println!("±1 (print hit>0 or d>=qd0 or Q>=qe0):");
    let mut best_h = h0;
    let mut best_d = qd0;
    let mut lab = "CR".to_string();
    for i in 0..6 {
        for k in [-1i32, 1] {
            let mut p = PP;
            p[i] = poke(PP[i], k);
            let (qe, qd, h, zs) = sc(&p, &QQ);
            if h > 0 || qd >= qd0 || qe >= qe0 {
                println!("  P[{i}] {k:+} Q={qe} d={qd} hit={h}/3 {zs:?}");
            }
            if h > best_h || (h == best_h && qd > best_d) {
                best_h = h;
                best_d = qd;
                lab = format!("P[{i}] {k:+}");
            }
        }
    }
    for i in 0..5 {
        for k in [-1i32, 1] {
            let mut q = QQ;
            q[i] = poke(QQ[i], k);
            let (qe, qd, h, zs) = sc(&PP, &q);
            if h > 0 || qd >= qd0 || qe >= qe0 {
                println!("  Q[{i}] {k:+} Q={qe} d={qd} hit={h}/3 {zs:?}");
            }
            if h > best_h || (h == best_h && qd > best_d) {
                best_h = h;
                best_d = qd;
                lab = format!("Q[{i}] {k:+}");
            }
        }
    }
    println!("best-by-hit {lab} d={best_d} hit={best_h}");
    println!("P[5] steps:");
    for k in -4i32..=8 {
        let mut p = PP;
        p[5] = poke(PP[5], k);
        let (qe, qd, h, zs) = sc(&p, &QQ);
        println!("  P[5] {k:+} Q={qe} d={qd} hit={h}/3 {zs:?}");
    }
    println!("Q[4] steps:");
    for k in -8i32..=4 {
        let mut q = QQ;
        q[4] = poke(QQ[4], k);
        let (qe, qd, h, zs) = sc(&PP, &q);
        println!("  Q[4] {k:+} Q={qe} d={qd} hit={h}/3 {zs:?}");
    }
}
