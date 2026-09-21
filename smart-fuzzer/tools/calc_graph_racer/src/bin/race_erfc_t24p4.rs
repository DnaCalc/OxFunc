//! 0x24a5 Horner + P/Q 4-coeff ±1 DIRECT z>=4. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const MASK: u32 = 0x24a5;
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
fn maybe(x: Ext80, mask: u32, bit: u32) -> Ext80 {
    if bit < 32 && mask & (1u32 << bit) != 0 {
        ef(ext_to_f64(&x, CW))
    } else {
        x
    }
}
fn polevl(x: Ext80, coef: &[f64], mask: u32, mut bit: u32) -> (Ext80, u32) {
    let mut ans = ef(coef[0]);
    for &c in &coef[1..] {
        ans = ext_add(&ext_mul(&ans, &x, CW), &ef(c), CW);
        ans = maybe(ans, mask, bit);
        bit += 1;
    }
    (ans, bit)
}
fn p1evl(x: Ext80, coef: &[f64], mask: u32, mut bit: u32) -> (Ext80, u32) {
    let mut ans = ext_add(&x, &ef(coef[0]), CW);
    ans = maybe(ans, mask, bit);
    bit += 1;
    for &c in &coef[1..] {
        ans = ext_add(&ext_mul(&ans, &x, CW), &ef(c), CW);
        ans = maybe(ans, mask, bit);
        bit += 1;
    }
    (ans, bit)
}
fn cephes(x: f64, p: &[f64; 9], q: &[f64; 8]) -> f64 {
    let ax = x.abs();
    let xe = ef(ax);
    let (num, den, bit0) = if ax < 8.0 {
        let (pp, b) = polevl(xe, p, MASK, 0);
        let (qq, b) = p1evl(xe, q, MASK, b);
        (pp, qq, b)
    } else {
        let (rr, b) = polevl(xe, &R0, MASK, 0);
        let (ss, b) = p1evl(xe, &S0, MASK, b);
        (rr, ss, b)
    };
    let mut v = ext_div(&num, &den, CW);
    v = maybe(v, MASK, bit0);
    ext_to_f64(&v, CW)
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
    let ntail = tail.len();
    let score = |p: &[f64; 9], q: &[f64; 8]| -> usize {
        tail.iter()
            .filter(|rr| {
                let t = f64::from_bits(rr.qbits);
                ulp_distance(mul(f::w_rn53(rr.z), cephes(rr.z, p, q)), t).unwrap_or(99) == 0
            })
            .count()
    };
    let base = score(&P0, &Q0);
    println!("0x24a5 fused DIRECT z>=4={base}/{ntail}");
    let mut best_s = base;
    let mut best_t = (0usize, 0i32, 0usize, 0i32, 0usize, 0i32, 0usize, 0i32);
    let mut nbeat = 0usize;
    let mut ngt65 = 0usize;
    let mut nge65 = 0usize;
    let mut top: Vec<(usize, usize, i32, usize, i32, usize, i32, usize, i32)> = Vec::new();
    let mut ncfg = 0usize;
    for i in 0..17usize {
        for j in (i + 1)..17 {
            for k in (j + 1)..17 {
                for m in (k + 1)..17 {
                    for &si in &[-1i32, 1] {
                        for &sj in &[-1i32, 1] {
                            for &sk in &[-1i32, 1] {
                                for &sm in &[-1i32, 1] {
                                    let mut p = P0;
                                    let mut q = Q0;
                                    apply_pq(i, si, &mut p, &mut q);
                                    apply_pq(j, sj, &mut p, &mut q);
                                    apply_pq(k, sk, &mut p, &mut q);
                                    apply_pq(m, sm, &mut p, &mut q);
                                    let s = score(&p, &q);
                                    ncfg += 1;
                                    if s > base {
                                        nbeat += 1;
                                    }
                                    if s >= 65 {
                                        nge65 += 1;
                                    }
                                    if s > 65 {
                                        ngt65 += 1;
                                    }
                                    if s > best_s
                                        || (s == best_s
                                            && (i, si, j, sj, k, sk, m, sm)
                                                < (
                                                    best_t.0, best_t.1, best_t.2, best_t.3,
                                                    best_t.4, best_t.5, best_t.6, best_t.7,
                                                ))
                                    {
                                        best_s = s;
                                        best_t = (i, si, j, sj, k, sk, m, sm);
                                    }
                                    if s >= 65 {
                                        top.push((s, i, si, j, sj, k, sk, m, sm));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    top.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    println!("top 12 P/Q 4-coeff ±1 on 0x24a5 (fused>=65):");
    for t in top.iter().take(12) {
        println!(
            "  fused={} {}{:+} {}{:+} {}{:+} {}{:+} Δ={:+}",
            t.0,
            name_pq(t.1),
            t.2,
            name_pq(t.3),
            t.4,
            name_pq(t.5),
            t.6,
            name_pq(t.7),
            t.8,
            t.0 as i32 - base as i32
        );
    }
    println!(
        "4-coeff on 0x24a5 n={ncfg} beat60={nbeat} best={best_s} {}{:+} {}{:+} {}{:+} {}{:+} ge65={nge65} gt65={ngt65}",
        name_pq(best_t.0),
        best_t.1,
        name_pq(best_t.2),
        best_t.3,
        name_pq(best_t.4),
        best_t.5,
        name_pq(best_t.6),
        best_t.7
    );
}
