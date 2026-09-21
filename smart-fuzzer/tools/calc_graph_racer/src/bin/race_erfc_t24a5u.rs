//! 0x24a5 leftover ± and 1-ULP vs unmask cephes F last-store. Not an identity.
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
fn cephes(x: f64, p: &[f64; 9], q: &[f64; 8], mask: u32) -> f64 {
    let ax = x.abs();
    let xe = ef(ax);
    let (num, den, bit0) = if ax < 8.0 {
        let (pp, b) = polevl(xe, p, mask, 0);
        let (qq, b) = p1evl(xe, q, mask, b);
        (pp, qq, b)
    } else {
        let (rr, b) = polevl(xe, &R0, mask, 0);
        let (ss, b) = p1evl(xe, &S0, mask, b);
        (rr, ss, b)
    };
    let mut v = ext_div(&num, &den, CW);
    v = maybe(v, mask, bit0);
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
fn signed_k(z: f64, t: f64, p: &[f64; 9], q: &[f64; 8], mask: u32) -> Option<i32> {
    let ff = cephes(z, p, q, mask);
    for ak in 0i32..=8 {
        for &s in &[-1i32, 1] {
            let k = if ak == 0 { 0 } else { s * ak };
            if ak == 0 && s < 0 {
                continue;
            }
            if ulp_distance(mul(f::w_rn53(z), poke(ff, k)), t).unwrap_or(99) == 0 {
                return Some(k);
            }
            if ak == 0 {
                break;
            }
        }
    }
    None
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let tail: Vec<&f::QRow> = rows.iter().filter(|r| r.direct && r.z >= 4.0).collect();
    let n = tail.len();
    let mut fused = 0usize;
    let mut nlo = 0usize;
    let mut lo_cov = 0usize;
    let mut nhi = 0usize;
    let mut hi_cov = 0usize;
    let mut n1 = 0usize;
    let mut u1_cov = 0usize;
    let mut hi_h = [0usize; 9];
    let mut u1_h = [0usize; 9];
    println!("0x24a5 leftover ± and 1-ULP vs unmask F DIRECT z>=4 n={n}:");
    for r in &tail {
        let t = f64::from_bits(r.qbits);
        let g = mul(f::w_rn53(r.z), cephes(r.z, &P0, &Q0, MASK));
        let d = ulp_distance(g, t).unwrap_or(99);
        let k0 = signed_k(r.z, t, &P0, &Q0, 0);
        if d == 0 {
            fused += 1;
            continue;
        }
        if d == 1 {
            n1 += 1;
            match k0 {
                Some(kk) => {
                    let ak = kk.unsigned_abs() as usize;
                    if ak < 9 {
                        u1_h[ak] += 1;
                    }
                    u1_cov += 1;
                }
                None => println!("  1-ULP MISS z={:.16}", r.z),
            }
            continue;
        }
        if g < t {
            nlo += 1;
            if k0.is_some() {
                lo_cov += 1;
            }
        } else {
            nhi += 1;
            match k0 {
                Some(kk) => {
                    let ak = kk.unsigned_abs() as usize;
                    if ak < 9 {
                        hi_h[ak] += 1;
                    }
                    hi_cov += 1;
                }
                None => println!("  leftover-high MISS z={:.16} d={d}", r.z),
            }
        }
    }
    print!("fused={fused} leftover-low {lo_cov}/{nlo} leftover-high {hi_cov}/{nhi} F± |k|");
    for (i, c) in hi_h.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    print!(" 1-ULP {u1_cov}/{n1} |k|");
    for (i, c) in u1_h.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    println!();
    println!(
        "0x24a5 envelope vs unmask last-store cover={}",
        fused + lo_cov + hi_cov + u1_cov
    );
}
