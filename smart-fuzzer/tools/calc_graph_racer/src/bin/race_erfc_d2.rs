//! Joint 2-coeff ±1 of NSWC DERFC0 vs Q [0.5,4). Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use std::env;

const P: [f64; 8] = [
    0.16506148041280876191828601e-03,
    0.15471455377139313353998665e-03,
    0.44852548090298868465196794e-04,
    -0.49177280017226285450486205e-05,
    -0.69353602078656412367801676e-05,
    -0.20508667787746282746857743e-05,
    -0.28982842617824971177267380e-06,
    -0.17272433544836633301127174e-07,
];
const Q: [f64; 8] = [
    1.0,
    0.16272656776533322859856317e+01,
    0.12040996037066026106794322e+01,
    0.52400246352158386907601472e+00,
    0.14497345252798672362384241e+00,
    0.25592517111042546492590736e-01,
    0.26869088293991371028123158e-02,
    0.13133767840925681614496481e-03,
];
const R: [f64; 9] = [
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
const AA: [f64; 9] = [
    -0.45894433406309678202825375e-03,
    -0.12281298722544724287816236e-01,
    -0.91144359512342900801764781e-01,
    -0.28412489223839285652511367e-01,
    0.14083827189977123530129812e+01,
    0.11532175281537044570477189e+01,
    -0.72170903389442152112483632e+01,
    -0.19685597805218214001309225e+01,
    0.93846891504541841150916038e+01,
];
const BB: [f64; 12] = [
    1.0,
    0.25136329960926527692263725e+02,
    0.15349442087145759184067981e+03,
    -0.29971215958498680905476402e+03,
    -0.33876477506888115226730368e+04,
    0.28301829314924804988873701e+04,
    0.22979620942196507068034887e+05,
    -0.24280681522998071562462041e+05,
    -0.36680620673264731899504580e+05,
    0.42278731622295627627042436e+05,
    0.28834257644413614344549790e+03,
    0.70226293775648358646587341e+03,
];
const E0: f64 = 0.540464821348814822409610122136;
const E1: f64 = -0.261515522487415653487049835220e-01;
const E2: f64 = -0.288573438386338758794591212600e-02;

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
fn aabb(x: f64, aa: &[f64; 9], bb: &[f64; 12], e0: f64, e1: f64, e2: f64) -> f64 {
    let z = 1.0 / (2.5 + x * x);
    let t = 13.0 * z - 1.0;
    let acc = ((horner(aa, z) / horner(bb, z) * t + e2) * t + e1) * t + e0;
    acc / x
}
fn derfc0(
    x: f64,
    p: &[f64; 8],
    q: &[f64; 8],
    r: &[f64; 9],
    aa: &[f64; 9],
    bb: &[f64; 12],
    e0: f64,
    e1: f64,
    e2: f64,
) -> f64 {
    if x <= 2.0 {
        pqr(x, p, q, r)
    } else if x <= 4.0 {
        aabb(x, aa, bb, e0, e1, e2)
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

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mid: Vec<&f::QRow> = rows.iter().filter(|r| r.z >= 0.5 && r.z < 4.0).collect();
    let n = mid.len();
    let score = |p: &[f64; 8],
                 q: &[f64; 8],
                 r: &[f64; 9],
                 aa: &[f64; 9],
                 bb: &[f64; 12],
                 e0: f64,
                 e1: f64,
                 e2: f64|
     -> usize {
        mid.iter()
            .filter(|row| {
                let t = f64::from_bits(row.qbits);
                ulp_distance(
                    qmul(f::w_rn53(row.z), derfc0(row.z, p, q, r, aa, bb, e0, e1, e2)),
                    t,
                )
                .unwrap_or(99)
                    == 0
            })
            .count()
    };
    let base = score(&P, &Q, &R, &AA, &BB, E0, E1, E2);
    println!("DERFC0 Q [0.5,4) {base}/{n} (bar 2389)");
    // PQR: 8+7+9 = 24 (skip Q[0]=1)
    let mut pqr_idx = Vec::new();
    for i in 0..8 {
        pqr_idx.push(('P', i));
    }
    for i in 1..8 {
        pqr_idx.push(('Q', i));
    }
    for i in 0..9 {
        pqr_idx.push(('R', i));
    }
    let mut best_p = base;
    let mut desc_p = String::from("base");
    let mut beat_p = 0usize;
    for i in 0..pqr_idx.len() {
        for j in (i + 1)..pqr_idx.len() {
            for &si in &[-1i32, 1] {
                for &sj in &[-1i32, 1] {
                    let mut p = P;
                    let mut q = Q;
                    let mut r = R;
                    let poke_one = |tag, ix, k, p: &mut [f64; 8], q: &mut [f64; 8], r: &mut [f64; 9]| {
                        match tag {
                            'P' => p[ix] = poke(P[ix], k),
                            'Q' => q[ix] = poke(Q[ix], k),
                            _ => r[ix] = poke(R[ix], k),
                        }
                    };
                    poke_one(pqr_idx[i].0, pqr_idx[i].1, si, &mut p, &mut q, &mut r);
                    poke_one(pqr_idx[j].0, pqr_idx[j].1, sj, &mut p, &mut q, &mut r);
                    let s = score(&p, &q, &r, &AA, &BB, E0, E1, E2);
                    if s > base {
                        beat_p += 1;
                    }
                    if s > best_p {
                        best_p = s;
                        desc_p = format!(
                            "{}[{}]{si:+} {}[{}]{sj:+}",
                            pqr_idx[i].0, pqr_idx[i].1, pqr_idx[j].0, pqr_idx[j].1
                        );
                        println!("  PQR new best Q={s} {desc_p}");
                    }
                }
            }
        }
    }
    println!("PQR 2-coeff best={best_p} {desc_p} beat={beat_p} (bar 2389)");
    // AA 9, BB 1..11, E0 E1 E2
    let mut ab_idx = Vec::new();
    for i in 0..9 {
        ab_idx.push(('A', i));
    }
    for i in 1..12 {
        ab_idx.push(('B', i));
    }
    ab_idx.push(('E', 0));
    ab_idx.push(('E', 1));
    ab_idx.push(('E', 2));
    let mut best_a = base;
    let mut desc_a = String::from("base");
    let mut beat_a = 0usize;
    for i in 0..ab_idx.len() {
        for j in (i + 1)..ab_idx.len() {
            for &si in &[-1i32, 1] {
                for &sj in &[-1i32, 1] {
                    let mut aa = AA;
                    let mut bb = BB;
                    let mut e0 = E0;
                    let mut e1 = E1;
                    let mut e2 = E2;
                    let mut poke_ab = |tag, ix, k| {
                        match tag {
                            'A' => aa[ix] = poke(AA[ix], k),
                            'B' => bb[ix] = poke(BB[ix], k),
                            _ if ix == 0 => e0 = poke(E0, k),
                            _ if ix == 1 => e1 = poke(E1, k),
                            _ => e2 = poke(E2, k),
                        }
                    };
                    poke_ab(ab_idx[i].0, ab_idx[i].1, si);
                    poke_ab(ab_idx[j].0, ab_idx[j].1, sj);
                    let s = score(&P, &Q, &R, &aa, &bb, e0, e1, e2);
                    if s > base {
                        beat_a += 1;
                    }
                    if s > best_a {
                        best_a = s;
                        desc_a = format!(
                            "{}[{}]{si:+} {}[{}]{sj:+}",
                            ab_idx[i].0, ab_idx[i].1, ab_idx[j].0, ab_idx[j].1
                        );
                        println!("  AABB new best Q={s} {desc_a}");
                    }
                }
            }
        }
    }
    println!("AABB/E 2-coeff best={best_a} {desc_a} beat={beat_a} (bar 2389)");
}
