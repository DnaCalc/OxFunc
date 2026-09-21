//! DERFC0 R 3-coeff ±1 vs fused leftover Q [0.5,4). Not an identity.
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
fn pqr(x: f64, r: &[f64; 9]) -> f64 {
    let u = horner(&P, x);
    let v = horner(&Q, x);
    let t = (x - 3.75) / (x + 3.75);
    let mut acc = u / v;
    for &c in r.iter().rev() {
        acc = acc * t + c;
    }
    acc
}
fn derfc0(x: f64, r: &[f64; 9]) -> f64 {
    if x <= 2.0 {
        pqr(x, r)
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
    let score = |r: &[f64; 9]| -> usize {
        mid.iter()
            .filter(|row| {
                let t = f64::from_bits(row.qbits);
                ulp_distance(qmul(f::w_rn53(row.z), derfc0(row.z, r)), t).unwrap_or(99) == 0
            })
            .count()
    };
    let base = score(&R0);
    println!("DERFC0 fused Q [0.5,4) {base}/{n} (bar 2869 R[1]+1 R[5]+1)");
    let mut best = Vec::new();
    for i in 0..9usize {
        for j in (i + 1)..9 {
            for k in (j + 1)..9 {
                for &si in &[-1i32, 1] {
                    for &sj in &[-1i32, 1] {
                        for &sk in &[-1i32, 1] {
                            let mut r = R0;
                            r[i] = poke(R0[i], si);
                            r[j] = poke(R0[j], sj);
                            r[k] = poke(R0[k], sk);
                            let s = score(&r);
                            best.push((s, i, si, j, sj, k, sk));
                        }
                    }
                }
            }
        }
    }
    best.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    println!("top 8 R 3-coeff ±1:");
    for (s, i, si, j, sj, k, sk) in best.iter().take(8) {
        println!(
            "  Q={s} R[{i}]{si:+} R[{j}]{sj:+} R[{k}]{sk:+} Δ={:+}",
            *s as i32 - base as i32
        );
    }
    let nbeat = best.iter().filter(|t| t.0 > base).count();
    let nge2869 = best.iter().filter(|t| t.0 >= 2869).count();
    let ngt2869 = best.iter().filter(|t| t.0 > 2869).count();
    println!(
        "R 3-coeff n={} beat_fused={nbeat} best={} ge2869={nge2869} gt2869={ngt2869}",
        best.len(),
        best[0].0
    );
    let mut r15 = R0;
    r15[1] = poke(R0[1], 1);
    r15[5] = poke(R0[5], 1);
    let s15 = score(&r15);
    println!("R[1]+1 R[5]+1 Q={s15}");
    let mut rb = R0;
    let (bi, bsi, bj, bsj, bk, bsk) = (
        best[0].1, best[0].2, best[0].3, best[0].4, best[0].5, best[0].6,
    );
    rb[bi] = poke(R0[bi], bsi);
    rb[bj] = poke(R0[bj], bsj);
    rb[bk] = poke(R0[bk], bsk);
    let mut nlo = 0usize;
    let mut hit_lo = 0usize;
    let mut nd = 0usize;
    let mut ndb = 0usize;
    for row in &mid {
        let t = f64::from_bits(row.qbits);
        let gf = qmul(f::w_rn53(row.z), derfc0(row.z, &R0));
        let gp = qmul(f::w_rn53(row.z), derfc0(row.z, &rb));
        let d = ulp_distance(gf, t).unwrap_or(99);
        if d >= 2 && gf < t {
            nlo += 1;
            if ulp_distance(gp, t).unwrap_or(99) == 0 {
                hit_lo += 1;
            }
        }
        if row.direct {
            nd += 1;
            if ulp_distance(gp, t).unwrap_or(99) == 0 {
                ndb += 1;
            }
        }
    }
    println!(
        "best leftover-low {hit_lo}/{nlo} DIRECT {ndb}/{nd}"
    );
}
