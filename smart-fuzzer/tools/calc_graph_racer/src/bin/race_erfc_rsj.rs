//! Cephes R/S ±1/ham2 on z≥8 Q-tail / XBIG leftover-low. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
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
const XBIG: [f64; 3] = [26.53, 26.542, 26.543];

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
fn polevl_mask(x: Ext80, coef: &[f64], mask: u32, mut bit: u32) -> (Ext80, u32) {
    let mut ans = ef(coef[0]);
    for &c in &coef[1..] {
        ans = ext_add(&ext_mul(&ans, &x, CW), &ef(c), CW);
        ans = maybe(ans, mask, bit);
        bit += 1;
    }
    (ans, bit)
}
fn p1evl_mask(x: Ext80, coef: &[f64], mask: u32, mut bit: u32) -> (Ext80, u32) {
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
fn cephes(x: f64, mask: u32, r: &[f64; 6], s: &[f64; 6]) -> f64 {
    let ax = x.abs();
    let xe = ef(ax);
    let (num, den, bit0) = if ax < 8.0 {
        let (p, b) = polevl_mask(xe, &CEPHES_P, mask, 0);
        let (q, b) = p1evl_mask(xe, &CEPHES_Q, mask, b);
        (p, q, b)
    } else {
        let (rr, b) = polevl_mask(xe, r, mask, 0);
        let (ss, b) = p1evl_mask(xe, s, mask, b);
        (rr, ss, b)
    };
    let mut q = ext_div(&num, &den, CW);
    q = maybe(q, mask, bit0);
    ext_to_f64(&q, CW)
}
fn qw(z: f64, ff: f64) -> f64 {
    let v = f::w_rn53(z) * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}

struct Sc {
    qe: usize,
    qd: usize,
    q8: usize,
    xhit: usize,
}
fn score(rows: &[f::QRow], mask: u32, r: &[f64; 6], s: &[f64; 6]) -> Sc {
    let mut sc = Sc {
        qe: 0,
        qd: 0,
        q8: 0,
        xhit: 0,
    };
    for row in rows {
        if row.z < 4.0 {
            continue;
        }
        let g = qw(row.z, cephes(row.z, mask, r, s));
        let d = ulp_distance(g, f64::from_bits(row.qbits)).unwrap_or(99);
        if d == 0 {
            sc.qe += 1;
            if row.direct {
                sc.qd += 1;
            }
            if row.z >= 8.0 {
                sc.q8 += 1;
            }
        }
        for &xz in &XBIG {
            if (row.z - xz).abs() < 0.0006 && row.direct && d == 0 {
                sc.xhit += 1;
            }
        }
    }
    sc
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    println!("bars 0x5005 Q 1572 d=53; XBIG leftover-low 26.53/26.542/26.543");
    for (lab, m) in [("mask0", 0u32), ("0x24a5", 0x24a5), ("0x5005", 0x5005)] {
        let sc = score(&rows, m, &CEPHES_R, &CEPHES_S);
        println!("  {lab:8} Q {0} d={1} z>=8 {2} xhit={3}/3", sc.qe, sc.qd, sc.q8, sc.xhit);
    }
    let base = score(&rows, 0x5005, &CEPHES_R, &CEPHES_S);
    println!("R/S one-coeff ±1 on 0x5005 (bar Q {}):", base.qe);
    let mut best = base.qe;
    let mut best_lab = "base".to_string();
    let mut best_x = base.xhit;
    for i in 0..6 {
        for k in [-2, -1, 1, 2] {
            let mut r = CEPHES_R;
            r[i] = poke(CEPHES_R[i], k);
            let sc = score(&rows, 0x5005, &r, &CEPHES_S);
            if sc.qe > best || (sc.qe == best && sc.xhit > best_x) || sc.xhit > best_x {
                println!(
                    "  R[{i}] {k:+} Q {} d={} z8={} xhit={}/3",
                    sc.qe, sc.qd, sc.q8, sc.xhit
                );
            }
            if sc.qe > best || (sc.qe == best && sc.xhit > best_x) {
                best = sc.qe;
                best_x = sc.xhit;
                best_lab = format!("R[{i}] {k:+}");
            }
        }
        for k in [-2, -1, 1, 2] {
            let mut s = CEPHES_S;
            s[i] = poke(CEPHES_S[i], k);
            let sc = score(&rows, 0x5005, &CEPHES_R, &s);
            if sc.qe > best || sc.xhit > 0 {
                println!(
                    "  S[{i}] {k:+} Q {} d={} z8={} xhit={}/3",
                    sc.qe, sc.qd, sc.q8, sc.xhit
                );
            }
            if sc.qe > best || (sc.qe == best && sc.xhit > best_x) {
                best = sc.qe;
                best_x = sc.xhit;
                best_lab = format!("S[{i}] {k:+}");
            }
        }
    }
    println!("best one-coeff {best_lab} Q {best} xhit={best_x}");

    println!("R/S ham2 ±1 (print if Q>1572 or xhit>0):");
    let mut hbest = best;
    let mut hlab = best_lab.clone();
    let coeffs = 12usize;
    for a in 0..coeffs {
        for b in (a + 1)..coeffs {
            for sa in [-1, 1] {
                for sb in [-1, 1] {
                    let mut r = CEPHES_R;
                    let mut s = CEPHES_S;
                    if a < 6 {
                        r[a] = poke(CEPHES_R[a], sa);
                    } else {
                        s[a - 6] = poke(CEPHES_S[a - 6], sa);
                    }
                    if b < 6 {
                        r[b] = poke(CEPHES_R[b], sb);
                    } else {
                        s[b - 6] = poke(CEPHES_S[b - 6], sb);
                    }
                    let sc = score(&rows, 0x5005, &r, &s);
                    if sc.qe > 1572 || sc.xhit > 0 {
                        if sc.qe > hbest || sc.xhit > 0 {
                            println!(
                                "  a={a}{sa:+} b={b}{sb:+} Q {} d={} z8={} xhit={}",
                                sc.qe, sc.qd, sc.q8, sc.xhit
                            );
                        }
                    }
                    if sc.qe > hbest {
                        hbest = sc.qe;
                        hlab = format!("ham2 {a}{sa:+} {b}{sb:+}");
                    }
                }
            }
        }
    }
    println!("best ham2 {hlab} Q {hbest}");

    println!("same pokes with mask 0x24a5 (bar 1507):");
    let b24 = score(&rows, 0x24a5, &CEPHES_R, &CEPHES_S);
    println!("  0x24a5 CR Q {} d={} xhit={}", b24.qe, b24.qd, b24.xhit);
    let mut best24 = b24.qe;
    for i in 0..6 {
        for k in [-1, 1] {
            let mut r = CEPHES_R;
            r[i] = poke(CEPHES_R[i], k);
            let sc = score(&rows, 0x24a5, &r, &CEPHES_S);
            if sc.qe > best24 || sc.xhit > b24.xhit {
                println!("  24a5 R[{i}] {k:+} Q {} xhit={}", sc.qe, sc.xhit);
                best24 = best24.max(sc.qe);
            }
            let mut s = CEPHES_S;
            s[i] = poke(CEPHES_S[i], k);
            let sc = score(&rows, 0x24a5, &CEPHES_R, &s);
            if sc.qe > best24 || sc.xhit > b24.xhit {
                println!("  24a5 S[{i}] {k:+} Q {} xhit={}", sc.qe, sc.xhit);
                best24 = best24.max(sc.qe);
            }
        }
    }
}
