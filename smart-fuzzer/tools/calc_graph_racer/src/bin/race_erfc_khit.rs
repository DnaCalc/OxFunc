//! Keep-and-hit R/S on XBIG +2 pair; 0x5005 ulp==2 fingerprint. Not an identity.
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
const PAIR: [f64; 2] = [26.5300000000000011, 26.5429999999999957];

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

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mut hist = [0usize; 6];
    let mut d2_low = 0usize;
    let mut d2_high = 0usize;
    let mut d2_z20 = 0usize;
    let mut shown = 0usize;
    println!("0x5005 ulp==2 LOW z>=4 (first 16) / hist:");
    for r in &rows {
        if r.z < 4.0 {
            continue;
        }
        let t = f64::from_bits(r.qbits);
        let g = qw(r.z, cephes(r.z, 0x5005, &CEPHES_R, &CEPHES_S));
        let d = ulp_distance(g, t).unwrap_or(99).min(5) as usize;
        hist[d] += 1;
        if d == 2 {
            if g < t {
                d2_low += 1;
                if shown < 16 {
                    println!(
                        "  LOW z={:.16} dir={} z>=20={}",
                        r.z,
                        r.direct,
                        r.z >= 20.0
                    );
                    shown += 1;
                }
            } else {
                d2_high += 1;
            }
            if r.z >= 20.0 {
                d2_z20 += 1;
            }
        }
    }
    println!(
        "ulp hist 0..5+ = {} {} {} {} {} {}  +2 LOW={d2_low} HIGH={d2_high} z>=20 +2={d2_z20}",
        hist[0], hist[1], hist[2], hist[3], hist[4], hist[5]
    );

    let pair_hit = |r: &[f64; 6], s: &[f64; 6]| -> usize {
        let mut h = 0usize;
        for row in &rows {
            if PAIR.iter().all(|&z| (row.z - z).abs() > 1e-13) {
                continue;
            }
            if ulp_distance(
                qw(row.z, cephes(row.z, 0x5005, r, s)),
                f64::from_bits(row.qbits),
            )
            .unwrap_or(99)
                == 0
            {
                h += 1;
            }
        }
        h
    };
    let keep_tail = |r: &[f64; 6], s: &[f64; 6]| -> usize {
        rows.iter()
            .filter(|row| {
                row.z >= 4.0
                    && ulp_distance(
                        qw(row.z, cephes(row.z, 0x5005, r, s)),
                        f64::from_bits(row.qbits),
                    )
                    .unwrap_or(99)
                        == 0
            })
            .count()
    };
    println!(
        "base keep={} pair_hit={}",
        keep_tail(&CEPHES_R, &CEPHES_S),
        pair_hit(&CEPHES_R, &CEPHES_S)
    );

    println!("R/S one-coeff ±1..4 (print pair_hit>0 or keep>=1572):");
    let mut best_h = 0usize;
    let mut best_k = 1572usize;
    let mut best_lab = "base".to_string();
    for i in 0..6 {
        for k in -4i32..=4 {
            if k == 0 {
                continue;
            }
            let mut r = CEPHES_R;
            r[i] = poke(CEPHES_R[i], k);
            let kk = keep_tail(&r, &CEPHES_S);
            let h = pair_hit(&r, &CEPHES_S);
            if h > 0 || kk >= 1572 {
                if h > 0 || kk > 1572 {
                    println!("  R[{i}] {k:+} keep={kk} pair={h}/2");
                }
            }
            if h > best_h || (h == best_h && kk > best_k) {
                best_h = h;
                best_k = kk;
                best_lab = format!("R[{i}] {k:+}");
            }
            let mut s = CEPHES_S;
            s[i] = poke(CEPHES_S[i], k);
            let kk = keep_tail(&CEPHES_R, &s);
            let h = pair_hit(&CEPHES_R, &s);
            if h > 0 || kk > 1572 {
                println!("  S[{i}] {k:+} keep={kk} pair={h}/2");
            }
            if h > best_h || (h == best_h && kk > best_k) {
                best_h = h;
                best_k = kk;
                best_lab = format!("S[{i}] {k:+}");
            }
        }
    }
    println!("best-by-pair {best_lab} keep={best_k} pair={best_h}");

    println!("R/S ham2 ±1 (print pair>0 or keep>1578):");
    for a in 0..12 {
        for b in (a + 1)..12 {
            for sa in [-1i32, 1] {
                for sb in [-1i32, 1] {
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
                    let h = pair_hit(&r, &s);
                    let kk = if h > 0 || true {
                        keep_tail(&r, &s)
                    } else {
                        0
                    };
                    if h > 0 || kk > 1578 {
                        println!("  a={a}{sa:+} b={b}{sb:+} keep={kk} pair={h}/2");
                    }
                    if h > best_h || (h == best_h && kk > best_k) {
                        best_h = h;
                        best_k = kk;
                        best_lab = format!("ham2 {a}{sa:+} {b}{sb:+}");
                    }
                }
            }
        }
    }
    println!("ham2 best-by-pair {best_lab} keep={best_k} pair={best_h}");
}
