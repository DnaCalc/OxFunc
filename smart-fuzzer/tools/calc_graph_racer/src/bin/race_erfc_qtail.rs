//! Tail F table on Q bits: Cody P/Q ham2, NSWC CCDD, Cephes. Not C/D pokes.
//! Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const P0: [f64; 6] = [
    0.305326634961232344,
    0.360344899949804439,
    0.125781726111229246,
    0.0160837851487422766,
    6.58749161529837803e-4,
    0.0163153871373020978,
];
const Q0: [f64; 5] = [
    2.56852019228982242,
    1.87295284992346047,
    0.527905102951428412,
    0.0605183413124413191,
    0.00233520497626869185,
];

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn cody_pq(y: f64, p: &[f64; 6], q: &[f64; 5]) -> f64 {
    let ye = ef(y.abs());
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
#[derive(Clone, Copy)]
struct Acc {
    exact: usize,
    n: usize,
    max_ulp: u64,
    sum: u128,
    d: usize,
    dn: usize,
}
fn score_tail(rows: &[f::QRow], eval: impl Fn(f64) -> f64) -> Acc {
    let mut a = Acc {
        exact: 0,
        n: 0,
        max_ulp: 0,
        sum: 0,
        d: 0,
        dn: 0,
    };
    for r in rows {
        if r.z < 4.0 {
            continue;
        }
        let qg = eval(r.z);
        if !qg.is_finite() {
            continue;
        }
        let dist = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
        if dist > ULP_CAP {
            continue;
        }
        a.n += 1;
        if r.direct {
            a.dn += 1;
        }
        if dist == 0 {
            a.exact += 1;
            if r.direct {
                a.d += 1;
            }
        } else {
            a.max_ulp = a.max_ulp.max(dist);
            a.sum += dist as u128;
        }
    }
    a
}
fn fmt(a: &Acc) -> String {
    format!("{}/{} max={} sum={} d={}/{}", a.exact, a.n, a.max_ulp, a.sum, a.d, a.dn)
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);

    println!("## named Q-tail z>=4 flush(w*F)");
    for (name, ev) in [
        ("Cody P/Q x87 CR", (|z| qw(z, cody_pq(z, &P0, &Q0))) as fn(f64) -> f64),
        ("NSWC CCDD native", |z| qw(z, f::nswc_ccdd_f(z))),
        ("NSWC derfc0", |z| qw(z, f::nswc_derfc0(z))),
        ("cephes_f", |z| qw(z, f::cephes_f(z))),
        ("as714 n80", |z| qw(z, f::cf_as714_x87_n(z, 80))),
        ("as714 n24", |z| qw(z, f::cf_as714_x87_n(z, 24))),
    ] {
        println!("{name:22} {}", fmt(&score_tail(&rows, ev)));
    }

    println!("\n## ham2 ±1 Cody P/Q on Q-tail");
    let mut p = P0;
    let mut q = Q0;
    let base = score_tail(&rows, |z| qw(z, cody_pq(z, &p, &q)));
    println!("base {}", fmt(&base));
    let mut best_ex = base.exact;
    let mut best_sum = base.sum;
    let mut best_d = base.d;
    let mut lab = "CR".to_string();
    let ks = [-1i32, 1];
    let n = 6 + 5;
    let get = |p: &[f64; 6], q: &[f64; 5], i: usize| if i < 6 { p[i] } else { q[i - 6] };
    let mut tried = 0u32;
    for i in 0..n {
        for j in (i + 1)..n {
            for &ki in &ks {
                for &kj in &ks {
                    let oi = get(&p, &q, i);
                    let oj = get(&p, &q, j);
                    if i < 6 {
                        p[i] = poke(oi, ki);
                    } else {
                        q[i - 6] = poke(oi, ki);
                    }
                    if j < 6 {
                        p[j] = poke(oj, kj);
                    } else {
                        q[j - 6] = poke(oj, kj);
                    }
                    let sc = score_tail(&rows, |z| qw(z, cody_pq(z, &p, &q)));
                    tried += 1;
                    if sc.exact > best_ex || (sc.exact == best_ex && sc.sum < best_sum) {
                        best_ex = sc.exact;
                        best_sum = sc.sum;
                        best_d = sc.d;
                        lab = format!("i={i}{ki:+} j={j}{kj:+}");
                        println!("HIT {lab} {}", fmt(&sc));
                    }
                    if i < 6 {
                        p[i] = oi;
                    } else {
                        q[i - 6] = oi;
                    }
                    if j < 6 {
                        p[j] = oj;
                    } else {
                        q[j - 6] = oj;
                    }
                }
            }
        }
    }
    println!("tried={tried} best={lab} exact={best_ex} sum={best_sum} d={best_d}");

    println!("\n## joint keep ±1,2 Cody P/Q on Q-tail from CR");
    p = P0;
    q = Q0;
    let mut best = score_tail(&rows, |z| qw(z, cody_pq(z, &p, &q)));
    let deltas = [1i32, -1, 2, -2];
    loop {
        let mut moved = false;
        for i in 0..6 {
            for &k in &deltas {
                let old = p[i];
                p[i] = poke(old, k);
                let sc = score_tail(&rows, |z| qw(z, cody_pq(z, &p, &q)));
                if sc.exact > best.exact || (sc.exact == best.exact && sc.sum < best.sum) {
                    best = sc;
                    moved = true;
                    println!("  keep P[{i}] {k:+} {}", fmt(&sc));
                } else {
                    p[i] = old;
                }
            }
        }
        for i in 0..5 {
            for &k in &deltas {
                let old = q[i];
                q[i] = poke(old, k);
                let sc = score_tail(&rows, |z| qw(z, cody_pq(z, &p, &q)));
                if sc.exact > best.exact || (sc.exact == best.exact && sc.sum < best.sum) {
                    best = sc;
                    moved = true;
                    println!("  keep Q[{i}] {k:+} {}", fmt(&sc));
                } else {
                    q[i] = old;
                }
            }
        }
        if !moved {
            break;
        }
    }
    println!("joint P/Q tail {}", fmt(&best));
}
