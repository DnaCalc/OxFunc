//! Cephes 0x5005 Q-tail leftover fingerprint. Direct vs implied. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
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

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
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
fn cephes_mask(x: f64, mask: u32) -> f64 {
    let ax = x.abs();
    let xe = ef(ax);
    let (num, den, bit0) = if ax < 8.0 {
        let (p, b) = polevl_mask(xe, &CEPHES_P, mask, 0);
        let (q, b) = p1evl_mask(xe, &CEPHES_Q, mask, b);
        (p, q, b)
    } else {
        let (r, b) = polevl_mask(xe, &CEPHES_R, mask, 0);
        let (s, b) = p1evl_mask(xe, &CEPHES_S, mask, b);
        (r, s, b)
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
fn signed_ulp(got: f64, want: f64) -> i64 {
    let d = ulp_distance(got, want).unwrap_or(99) as i64;
    if d == 0 {
        0
    } else if got > want {
        d
    } else {
        -d
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let ev = |z: f64| qw(z, cephes_mask(z, 0x5005));
    let ev0 = |z: f64| qw(z, cephes_mask(z, 0));
    let mut hist_d = BTreeMap::<i64, usize>::new();
    let mut hist_i = BTreeMap::<i64, usize>::new();
    let mut n = 0usize;
    let mut ex = 0usize;
    let mut left_d = 0usize;
    let mut left_i = 0usize;
    let mut m1d = 0usize;
    let mut m1i = 0usize;
    let mut maxu = 0u64;
    let mut maxd = 0u64;
    let mut hit0 = 0usize;
    let mut band_n = [0usize; 6];
    let mut band_left = [0usize; 6];
    for r in &rows {
        if r.z < 4.0 {
            continue;
        }
        let want = f64::from_bits(r.qbits);
        let g = ev(r.z);
        if !g.is_finite() {
            continue;
        }
        let d = ulp_distance(g, want).unwrap_or(u64::MAX);
        if d > ULP_CAP {
            continue;
        }
        n += 1;
        let b = ((r.z - 4.0) / 2.0).floor() as usize;
        let b = b.min(5);
        band_n[b] += 1;
        if d == 0 {
            ex += 1;
            continue;
        }
        maxu = maxu.max(d);
        let s = signed_ulp(g, want);
        band_left[b] += 1;
        if ulp_distance(ev0(r.z), want).unwrap_or(99) == 0 {
            hit0 += 1;
        }
        if r.direct {
            left_d += 1;
            maxd = maxd.max(d);
            *hist_d.entry(s).or_insert(0) += 1;
            if d == 1 {
                m1d += 1;
            }
        } else {
            left_i += 1;
            *hist_i.entry(s).or_insert(0) += 1;
            if d == 1 {
                m1i += 1;
            }
        }
    }
    println!(
        "0x5005 {ex}/{n} leftover={} max={maxu} maxd={maxd} dleft={left_d} m1d={m1d} ileft={left_i} m1i={m1i} mask0-hits-on-left={hit0}",
        left_d + left_i
    );
    for i in 0..6 {
        let lo = 4.0 + i as f64 * 2.0;
        println!(
            "[{:.0},{:.0}) n={} left={}",
            lo,
            lo + 2.0,
            band_n[i],
            band_left[i]
        );
    }
    println!("signed ULP DIRECT leftover:");
    for (k, v) in &hist_d {
        println!("  {k:+} {v}");
    }
    println!("signed ULP implied leftover:");
    for (k, v) in &hist_i {
        println!("  {k:+} {v}");
    }
    let mut hard_n = 0usize;
    println!("HARD direct leftover ulp>=2:");
    for r in &rows {
        if r.z < 4.0 || !r.direct {
            continue;
        }
        let want = f64::from_bits(r.qbits);
        let d = ulp_distance(ev(r.z), want).unwrap_or(99);
        if d < 2 || d > ULP_CAP {
            continue;
        }
        hard_n += 1;
        let d0 = ulp_distance(ev0(r.z), want).unwrap_or(99);
        let dp = ulp_distance(qw(r.z, f::nswc_pqr_f(r.z)), want).unwrap_or(99);
        let dd = ulp_distance(qw(r.z, f::nswc_derfc0(r.z)), want).unwrap_or(99);
        println!(
            "  z={:.16} s={:+} mask0={d0} pqr={dp} derfc0={dd}",
            r.z,
            signed_ulp(ev(r.z), want)
        );
    }
    println!("hard direct n={hard_n}");
    let cuts = [4.05, 4.06, 4.0625, 4.07, 4.08, 4.1, 4.2, 4.5, 4.9, 5.0];
    for &c in &cuts {
        let mut m5 = 0usize;
        let mut ncut = 0usize;
        for r in &rows {
            if r.z < 4.0 {
                continue;
            }
            let want = f64::from_bits(r.qbits);
            let g = if r.z < c { ev0(r.z) } else { ev(r.z) };
            let d = ulp_distance(g, want).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            ncut += 1;
            if d == 0 {
                m5 += 1;
            }
        }
        println!("cut={c:.4} mask0-then-5005={m5}/{ncut} (bar 1572)");
    }
    println!("z>=26 (Cody XBIG=26.543) Q vs 0 vs 0x5005:");
    for r in &rows {
        if r.z < 26.0 {
            continue;
        }
        let want = f64::from_bits(r.qbits);
        let d0 = ulp_distance(0.0, want).unwrap_or(u64::MAX);
        let d5 = ulp_distance(ev(r.z), want).unwrap_or(u64::MAX);
        println!(
            "  z={:.16} q={:016x} ulp0={} ulp5005={} direct={}",
            r.z,
            r.qbits,
            d0,
            d5,
            r.direct
        );
    }
}
