//! Cephes 0x24a5 P/Q ±1 vs 6 leftover-low. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const MASK: u32 = 0x24a5;
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
fn kind(z: f64) -> &'static str {
    let m = z.to_bits() & ((1u64 << 52) - 1);
    if m.trailing_zeros() >= 20 {
        "dyad"
    } else if (z.to_bits() & 0xfff) == 0x555 {
        "555"
    } else {
        "other"
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let tail: Vec<&f::QRow> = rows.iter().filter(|r| r.direct && r.z >= 4.0).collect();
    println!("6 leftover-low CR 0x5005:");
    for &lz in &SIX {
        let Some(r) = tail.iter().find(|rr| (rr.z - lz).abs() < 1e-12) else {
            println!("  z={lz:.16} MISSING");
            continue;
        };
        let t = f64::from_bits(r.qbits);
        let g = qw(lz, cephes(lz, &CEPHES_P, &CEPHES_Q));
        let d = ulp_distance(g, t).unwrap_or(99);
        let dir = if g < t {
            "L"
        } else if g > t {
            "H"
        } else {
            "="
        };
        println!("  z={lz:.16} kind={} {d}{dir}", kind(lz));
    }
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
    let (qe0, qd0, h0, _) = sc(&CEPHES_P, &CEPHES_Q);
    println!("0x24a5 CR Q={qe0} d={qd0} hit={h0}/6 (bar 0x5005 1572/53)");
    println!("±1 (print hit>0 or d>=qd0 or Q>=qe0):");
    let mut best_h = h0;
    let mut best_d = qd0;
    let mut lab = "CR".to_string();
    for i in 0..9 {
        for k in [-1i32, 1] {
            let mut p = CEPHES_P;
            p[i] = poke(CEPHES_P[i], k);
            let (qe, qd, h, zs) = sc(&p, &CEPHES_Q);
            if h > 0 || qd >= qd0 || qe >= qe0 {
                println!("  P[{i}] {k:+} Q={qe} d={qd} hit={h}/6 {zs:?}");
            }
            if h > best_h || (h == best_h && qd > best_d) {
                best_h = h;
                best_d = qd;
                lab = format!("P[{i}] {k:+}");
            }
        }
    }
    for i in 0..8 {
        for k in [-1i32, 1] {
            let mut q = CEPHES_Q;
            q[i] = poke(CEPHES_Q[i], k);
            let (qe, qd, h, zs) = sc(&CEPHES_P, &q);
            if h > 0 || qd >= qd0 || qe >= qe0 {
                println!("  Q[{i}] {k:+} Q={qe} d={qd} hit={h}/6 {zs:?}");
            }
            if h > best_h || (h == best_h && qd > best_d) {
                best_h = h;
                best_d = qd;
                lab = format!("Q[{i}] {k:+}");
            }
        }
    }
    println!("best-by-hit {lab} d={best_d} hit={best_h}");
    println!("P[8] steps:");
    for k in -4i32..=8 {
        let mut p = CEPHES_P;
        p[8] = poke(CEPHES_P[8], k);
        let (qe, qd, h, zs) = sc(&p, &CEPHES_Q);
        println!("  P[8] {k:+} Q={qe} d={qd} hit={h}/6 {zs:?}");
    }
    println!("Q[7] steps:");
    for k in -8i32..=4 {
        let mut q = CEPHES_Q;
        q[7] = poke(CEPHES_Q[7], k);
        let (qe, qd, h, zs) = sc(&CEPHES_P, &q);
        println!("  Q[7] {k:+} Q={qe} d={qd} hit={h}/6 {zs:?}");
    }
}
