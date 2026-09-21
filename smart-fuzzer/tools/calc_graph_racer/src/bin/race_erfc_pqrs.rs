//! NSWC PQR ±1 vs 3 stubborn leftover-low. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use std::env;

const STUB: [f64; 3] = [4.0520833333333330, 5.3333333333333330, 6.0000000000000000];
const P0: [f64; 8] = [
    0.16506148041280876191828601e-03,
    0.15471455377139313353998665e-03,
    0.44852548090298868465196794e-04,
    -0.49177280017226285450486205e-05,
    -0.69353602078656412367801676e-05,
    -0.20508667787746282746857743e-05,
    -0.28982842617824971177267380e-06,
    -0.17272433544836633301127174e-07,
];
const Q0: [f64; 8] = [
    1.0,
    0.16272656776533322859856317e+01,
    0.12040996037066026106794322e+01,
    0.52400246352158386907601472e+00,
    0.14497345252798672362384241e+00,
    0.25592517111042546492590736e-01,
    0.26869088293991371028123158e-02,
    0.13133767840925681614496481e-03,
];
const R0: [f64; 9] = [
    0.145589721275038539045668824025,
    -0.273421931495426482902320421863,
    0.226008066916621506788789064272,
    -0.163571895523923805648814425592,
    0.102604312032193978662297299832,
    -0.548023266949835519254211506880e-01,
    0.241432239725390106956523668160e-01,
    -0.822062115403915116036874169600e-02,
    0.180296241564687154310619200000e-02,
];

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
fn horner(cs: &[f64], x: f64) -> f64 {
    let mut a = 0.0;
    for &c in cs.iter().rev() {
        a = a * x + c;
    }
    a
}
fn pqr(x: f64, p: &[f64; 8], q: &[f64; 8], r: &[f64; 9]) -> f64 {
    let u = horner(p, x);
    let v = horner(q, x);
    let t = (x - 3.75) / (x + 3.75);
    let mut acc = u / v;
    for &c in r.iter().rev() {
        acc = acc * t + c;
    }
    acc
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
    let sc = |p: &[f64; 8], q: &[f64; 8], r: &[f64; 9]| {
        let mut qd = 0usize;
        let mut qe = 0usize;
        let mut hit = 0usize;
        let mut zs = Vec::new();
        for row in rows.iter().filter(|rr| rr.z >= 4.0) {
            let g = qw(row.z, pqr(row.z, p, q, r));
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
            let Some(rr) = tail.iter().find(|x| (x.z - lz).abs() < 1e-12) else {
                continue;
            };
            if ulp_distance(qw(lz, pqr(lz, p, q, r)), f64::from_bits(rr.qbits)).unwrap_or(99) == 0 {
                hit += 1;
                zs.push(lz);
            }
        }
        (qe, qd, hit, zs)
    };
    let (qe0, qd0, h0, _) = sc(&P0, &Q0, &R0);
    println!("PQR CR Q={qe0} d={qd0} hit={h0}/3 (bar 0x5005 1572/53)");
    println!("±1 (print hit>0 or d>=qd0 or Q>=qe0):");
    let mut best_h = h0;
    let mut best_d = qd0;
    let mut lab = "CR".to_string();
    for i in 0..8 {
        for k in [-1i32, 1] {
            let mut p = P0;
            p[i] = poke(P0[i], k);
            let (qe, qd, h, zs) = sc(&p, &Q0, &R0);
            if h > 0 || qd >= qd0 || qe >= qe0 {
                println!("  P[{i}] {k:+} Q={qe} d={qd} hit={h}/3 {zs:?}");
            }
            if h > best_h || (h == best_h && qd > best_d) {
                best_h = h;
                best_d = qd;
                lab = format!("P[{i}] {k:+}");
            }
        }
        for k in [-1i32, 1] {
            let mut q = Q0;
            q[i] = poke(Q0[i], k);
            let (qe, qd, h, zs) = sc(&P0, &q, &R0);
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
    for i in 0..9 {
        for k in [-1i32, 1] {
            let mut r = R0;
            r[i] = poke(R0[i], k);
            let (qe, qd, h, zs) = sc(&P0, &Q0, &r);
            if h > 0 || qd >= qd0 || qe >= qe0 {
                println!("  R[{i}] {k:+} Q={qe} d={qd} hit={h}/3 {zs:?}");
            }
            if h > best_h || (h == best_h && qd > best_d) {
                best_h = h;
                best_d = qd;
                lab = format!("R[{i}] {k:+}");
            }
        }
    }
    println!("best-by-hit {lab} d={best_d} hit={best_h}");
}
