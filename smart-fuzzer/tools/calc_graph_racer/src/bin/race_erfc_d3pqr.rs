//! DERFC0 PQR 3-coeff ±1 vs fused leftover Q [0.5,4). Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use std::env;

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
fn derfc0(x: f64, p: &[f64; 8], q: &[f64; 8], r: &[f64; 9]) -> f64 {
    if x <= 2.0 {
        pqr(x, p, q, r)
    } else {
        f::nswc_derfc0(x)
    }
}
fn qmul(w: f64, ff: f64) -> f64 {
    let v = w * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}
fn apply(tag: char, ix: usize, k: i32, p: &mut [f64; 8], q: &mut [f64; 8], r: &mut [f64; 9]) {
    match tag {
        'P' => p[ix] = poke(P0[ix], k),
        'Q' => q[ix] = poke(Q0[ix], k),
        _ => r[ix] = poke(R0[ix], k),
    }
}
fn name(tag: char, ix: usize) -> String {
    format!("{tag}[{ix}]")
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mid: Vec<&f::QRow> = rows.iter().filter(|r| r.z >= 0.5 && r.z < 4.0).collect();
    let n = mid.len();
    let score = |p: &[f64; 8], q: &[f64; 8], r: &[f64; 9]| -> usize {
        mid.iter()
            .filter(|row| {
                let t = f64::from_bits(row.qbits);
                ulp_distance(qmul(f::w_rn53(row.z), derfc0(row.z, p, q, r)), t).unwrap_or(99) == 0
            })
            .count()
    };
    let base = score(&P0, &Q0, &R0);
    println!("DERFC0 fused Q [0.5,4) {base}/{n} (bar 2870)");
    let mut idx = Vec::new();
    for i in 0..8 {
        idx.push(('P', i));
    }
    for i in 1..8 {
        idx.push(('Q', i));
    }
    for i in 0..9 {
        idx.push(('R', i));
    }
    let mut best_s = base;
    let mut best_t = (0usize, 0i32, 0usize, 0i32, 0usize, 0i32);
    let mut nbeat = 0usize;
    let mut ngt2870 = 0usize;
    let mut nge2870 = 0usize;
    let mut ncfg = 0usize;
    let mut top: Vec<(usize, usize, i32, usize, i32, usize, i32)> = Vec::new();
    for i in 0..idx.len() {
        for j in (i + 1)..idx.len() {
            for k in (j + 1)..idx.len() {
                for &si in &[-1i32, 1] {
                    for &sj in &[-1i32, 1] {
                        for &sk in &[-1i32, 1] {
                            let mut p = P0;
                            let mut q = Q0;
                            let mut r = R0;
                            apply(idx[i].0, idx[i].1, si, &mut p, &mut q, &mut r);
                            apply(idx[j].0, idx[j].1, sj, &mut p, &mut q, &mut r);
                            apply(idx[k].0, idx[k].1, sk, &mut p, &mut q, &mut r);
                            let s = score(&p, &q, &r);
                            ncfg += 1;
                            if s > base {
                                nbeat += 1;
                            }
                            if s >= 2870 {
                                nge2870 += 1;
                            }
                            if s > 2870 {
                                ngt2870 += 1;
                                top.push((s, i, si, j, sj, k, sk));
                            }
                            if s > best_s {
                                best_s = s;
                                best_t = (i, si, j, sj, k, sk);
                                println!(
                                    "  new best Q={s} {}{si:+} {}{sj:+} {}{sk:+}",
                                    name(idx[i].0, idx[i].1),
                                    name(idx[j].0, idx[j].1),
                                    name(idx[k].0, idx[k].1)
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    top.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    println!("all fused>2870:");
    for t in top.iter().take(8) {
        println!(
            "  Q={} {}{:+} {}{:+} {}{:+}",
            t.0,
            name(idx[t.1].0, idx[t.1].1),
            t.2,
            name(idx[t.3].0, idx[t.3].1),
            t.4,
            name(idx[t.5].0, idx[t.5].1),
            t.6
        );
    }
    println!(
        "PQR 3-coeff n={ncfg} beat_fused={nbeat} best={best_s} {}{:+} {}{:+} {}{:+} ge2870={nge2870} gt2870={ngt2870}",
        name(idx[best_t.0].0, idx[best_t.0].1),
        best_t.1,
        name(idx[best_t.2].0, idx[best_t.2].1),
        best_t.3,
        name(idx[best_t.4].0, idx[best_t.4].1),
        best_t.5
    );
}
