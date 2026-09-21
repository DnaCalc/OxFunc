//! Joint 3-coeff ±1 of unmask Cephes P/Q DIRECT z>=4. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const P0: [f64; 9] = [
    2.46196981473530512524e-10,
    5.64189564831068821977e-1,
    7.46321056442269912687e0,
    4.86371970985681366614e1,
    1.96520832956077098242e2,
    5.26445194995477358631e2,
    9.34528527171957607540e2,
    1.02755188689515710272e3,
    5.57535335369399327526e2,
];
const Q0: [f64; 8] = [
    1.32281951154744992508e1,
    8.67072140885989742329e1,
    3.54937778887819891062e2,
    9.75708501743205489753e2,
    1.82390916687909736289e3,
    2.24633760818710981792e3,
    1.65666309194161350182e3,
    5.57535340817727675546e2,
];
const R0: [f64; 6] = [
    5.64189583547755073984e-1,
    1.27536670759978104416e0,
    5.01905042251180477414e0,
    6.16021097993053585195e0,
    7.40974269950448939160e0,
    2.97886665372100240670e0,
];
const S0: [f64; 6] = [
    2.26052863220117276590e0,
    9.39603524938001434673e0,
    1.20489539808096656605e1,
    1.70814450747565897222e1,
    9.60896809063285878198e0,
    3.36907645100081516050e0,
];

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
fn polevl(x: Ext80, coef: &[f64]) -> Ext80 {
    let mut ans = ef(coef[0]);
    for &c in &coef[1..] {
        ans = ext_add(&ext_mul(&ans, &x, CW), &ef(c), CW);
    }
    ans
}
fn p1evl(x: Ext80, coef: &[f64]) -> Ext80 {
    let mut ans = ext_add(&x, &ef(coef[0]), CW);
    for &c in &coef[1..] {
        ans = ext_add(&ext_mul(&ans, &x, CW), &ef(c), CW);
    }
    ans
}
fn cephes(x: f64, p: &[f64; 9], q: &[f64; 8], r: &[f64; 6], s: &[f64; 6]) -> f64 {
    let ax = x.abs();
    let xe = ef(ax);
    let (num, den) = if ax < 8.0 {
        (polevl(xe, p), p1evl(xe, q))
    } else {
        (polevl(xe, r), p1evl(xe, s))
    };
    ext_to_f64(&ext_div(&num, &den, CW), CW)
}
fn mul(w: f64, ff: f64) -> f64 {
    let v = w * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}
fn apply_pq(idx: usize, k: i32, p: &mut [f64; 9], q: &mut [f64; 8]) {
    if idx < 9 {
        p[idx] = poke(P0[idx], k);
    } else {
        q[idx - 9] = poke(Q0[idx - 9], k);
    }
}
fn name_pq(idx: usize) -> String {
    if idx < 9 {
        format!("P[{idx}]")
    } else {
        format!("Q[{}]", idx - 9)
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let tail: Vec<&f::QRow> = rows.iter().filter(|r| r.direct && r.z >= 4.0).collect();
    let n = tail.len();
    let score = |p: &[f64; 9], q: &[f64; 8]| -> usize {
        tail.iter()
            .filter(|rr| {
                let t = f64::from_bits(rr.qbits);
                ulp_distance(mul(f::w_rn53(rr.z), cephes(rr.z, p, q, &R0, &S0)), t).unwrap_or(99)
                    == 0
            })
            .count()
    };
    let base = score(&P0, &Q0);
    println!("unmask fused DIRECT z>=4={base}/{n} (bar store-mask 60)");
    let mut best = Vec::new();
    for i in 0..17usize {
        for j in (i + 1)..17 {
            for k in (j + 1)..17 {
                for &si in &[-1i32, 1] {
                    for &sj in &[-1i32, 1] {
                        for &sk in &[-1i32, 1] {
                            let mut p = P0;
                            let mut q = Q0;
                            apply_pq(i, si, &mut p, &mut q);
                            apply_pq(j, sj, &mut p, &mut q);
                            apply_pq(k, sk, &mut p, &mut q);
                            let s = score(&p, &q);
                            best.push((s, i, si, j, sj, k, sk));
                        }
                    }
                }
            }
        }
    }
    best.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    println!("top 12 P/Q 3-coeff ±1:");
    for (s, i, si, j, sj, k, sk) in best.iter().take(12) {
        println!(
            "  fused={s} {}{si:+} {}{sj:+} {}{sk:+} Δ={:+}",
            name_pq(*i),
            name_pq(*j),
            name_pq(*k),
            *s as i32 - base as i32
        );
    }
    let nbeat = best.iter().filter(|t| t.0 > base).count();
    let nge60 = best.iter().filter(|t| t.0 >= 60).count();
    println!(
        "P/Q triples*signs={} beat_unmask={nbeat} best={} ge60={nge60} (bar 60)",
        best.len(),
        best[0].0
    );
}
