//! DIRECT z>=4: mask0 cephes fused ∪ F± last-store cover. Not an identity.
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
fn cephes_mask(x: f64, mask: u32) -> f64 {
    let ax = x.abs();
    let xe = ef(ax);
    let (num, den, bit0) = if ax < 8.0 {
        let (p, b) = polevl(xe, &CEPHES_P, mask, 0);
        let (q, b) = p1evl(xe, &CEPHES_Q, mask, b);
        (p, q, b)
    } else {
        let (r, b) = polevl(xe, &CEPHES_R, mask, 0);
        let (s, b) = p1evl(xe, &CEPHES_S, mask, b);
        (r, s, b)
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
fn hit_ls(z: f64, t: f64, ff: f64) -> bool {
    (-5i32..=5)
        .filter(|&k| k != 0)
        .any(|k| ulp_distance(mul(f::w_rn53(z), poke(ff, k)), t).unwrap_or(99) == 0)
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mut n5 = 0usize;
    let mut n0 = 0usize;
    let mut nls = 0usize;
    let mut n_u = 0usize;
    let mut n_5u = 0usize;
    let mut ncf = 0usize;
    let mut n_cfu = 0usize;
    let mut miss = 0usize;
    let mut n = 0usize;
    println!("DIRECT z>=4 mask0 fused ∪ F± cover:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 4.0) {
        n += 1;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(r.z);
        let f5 = cephes_mask(r.z, 0x5005);
        let f0 = cephes_mask(r.z, 0);
        let cf = f::cephes_f(r.z);
        let e5 = ulp_distance(mul(w, f5), t).unwrap_or(99) == 0;
        let e0 = ulp_distance(mul(w, f0), t).unwrap_or(99) == 0;
        let ecf = ulp_distance(mul(w, cf), t).unwrap_or(99) == 0;
        let els0 = hit_ls(r.z, t, f0);
        let elscf = hit_ls(r.z, t, cf);
        if e5 {
            n5 += 1;
        }
        if e0 {
            n0 += 1;
        }
        if els0 {
            nls += 1;
        }
        if ecf {
            ncf += 1;
        }
        if e0 || els0 {
            n_u += 1;
        }
        if e5 || els0 {
            n_5u += 1;
        }
        if ecf || elscf {
            n_cfu += 1;
        }
        if !(e0 || els0 || ecf || elscf) {
            miss += 1;
            let d0 = ulp_distance(mul(w, f0), t).unwrap_or(99);
            let d5 = ulp_distance(mul(w, f5), t).unwrap_or(99);
            println!(
                "  MISS z={:.16} mask0={} 5005={} {}",
                r.z,
                d0,
                d5,
                if mul(w, f0) < t { "L" } else { "H" }
            );
        }
    }
    println!(
        "n={n} 0x5005={n5} mask0={n0} mask0-F±={nls} mask0∪F±={n_u} 5005∪mask0F±={n_5u} cephes_f={ncf} cephes_f∪F±={n_cfu} miss={miss}"
    );
}
