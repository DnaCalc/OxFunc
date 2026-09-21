//! Extra ±1 from Q[0]−1 and Q[1]−1 vs 6 leftover-low. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const MASK: u32 = 0x5005;
const CEPHES_P: [f64; 9] = [
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
const CEPHES_Q: [f64; 8] = [
    1.32281951154744992508e1,
    8.67072140885989742329e1,
    3.54937778887819891062e2,
    9.75708501743205489753e2,
    1.82390916687909736289e3,
    2.24633760818710981792e3,
    1.65666309194161350182e3,
    5.57535340817727675546e2,
];
const CEPHES_R: [f64; 6] = [
    5.64189583547755073984e-1,
    1.27536670759978104416e0,
    5.01905042251180477414e0,
    6.16021097993053585195e0,
    7.40974269950448939160e0,
    2.97886665372100240670e0,
];
const CEPHES_S: [f64; 6] = [
    2.26052863220117276590e0,
    9.39603524938001434673e0,
    1.20489539808096656605e1,
    1.70814450747565897222e1,
    9.60896809063285878198e0,
    3.36907645100081516050e0,
];
const SIX: [f64; 6] = [
    4.0520833333333330,
    4.0625000000000000,
    5.2395833333333330,
    5.2708333333333330,
    5.3333333333333330,
    6.0000000000000000,
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
        let (rr, b) = polevl(xe, &CEPHES_R, MASK, 0);
        let (ss, b) = p1evl(xe, &CEPHES_S, MASK, b);
        (rr, ss, b)
    };
    let mut v = ext_div(&num, &den, CW);
    v = maybe(v, MASK, bit0);
    ext_to_f64(&v, CW)
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
    let sc = |p: &[f64; 9], q: &[f64; 8]| {
        let mut qd = 0usize;
        let mut qe = 0usize;
        let mut hit = 0usize;
        let mut zs = Vec::new();
        for row in rows.iter().filter(|rr| rr.z >= 4.0) {
            let g = qw(row.z, cephes(row.z, p, q));
            if ulp_distance(g, f64::from_bits(row.qbits)).unwrap_or(99) == 0 {
                qe += 1;
                if row.direct {
                    qd += 1;
                }
            }
        }
        for &lz in &SIX {
            let Some(r) = tail.iter().find(|rr| (rr.z - lz).abs() < 1e-12) else {
                continue;
            };
            if ulp_distance(qw(lz, cephes(lz, p, q)), f64::from_bits(r.qbits)).unwrap_or(99) == 0 {
                hit += 1;
                zs.push(lz);
            }
        }
        (qe, qd, hit, zs)
    };
    let extra = |tag: &str, p0: [f64; 9], q0: [f64; 8], skip_q: Option<usize>| {
        let (qe0, qd0, h0, _) = sc(&p0, &q0);
        println!("{tag} Q={qe0} d={qd0} hit={h0}/6");
        println!("extra ±1 (print hit>h0 or d>=qd0 or Q>=qe0):");
        let mut best_h = h0;
        let mut best_d = qd0;
        let mut lab = tag.to_string();
        for i in 0..9 {
            for k in [-1i32, 1] {
                let mut p = p0;
                p[i] = poke(CEPHES_P[i], k);
                let (qe, qd, h, zs) = sc(&p, &q0);
                if h > h0 || qd >= qd0 || qe >= qe0 {
                    println!("  P[{i}] {k:+} Q={qe} d={qd} hit={h}/6 {zs:?}");
                }
                if h > best_h || (h == best_h && qd > best_d) {
                    best_h = h;
                    best_d = qd;
                    lab = format!("{tag} P[{i}] {k:+}");
                }
            }
        }
        for i in 0..8 {
            if skip_q == Some(i) {
                continue;
            }
            for k in [-1i32, 1] {
                let mut q = q0;
                q[i] = poke(CEPHES_Q[i], k);
                let (qe, qd, h, zs) = sc(&p0, &q);
                if h > h0 || qd >= qd0 || qe >= qe0 {
                    println!("  Q[{i}] {k:+} Q={qe} d={qd} hit={h}/6 {zs:?}");
                }
                if h > best_h || (h == best_h && qd > best_d) {
                    best_h = h;
                    best_d = qd;
                    lab = format!("{tag} Q[{i}] {k:+}");
                }
            }
        }
        println!("best-by-hit {lab} d={best_d} hit={best_h}");
    };
    let mut q0m = CEPHES_Q;
    q0m[0] = poke(CEPHES_Q[0], -1);
    extra("Q[0]-1", CEPHES_P, q0m, Some(0));
    let mut q1m = CEPHES_Q;
    q1m[1] = poke(CEPHES_Q[1], -1);
    extra("Q[1]-1", CEPHES_P, q1m, Some(1));
}
