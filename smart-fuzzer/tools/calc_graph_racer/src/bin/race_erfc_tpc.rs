//! Cephes 0x5005 Q-tail x87 PC × RC. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC24_RN, CW_PC53_RN, CW_PC64_RN};
use std::env;

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
fn maybe(x: Ext80, mask: u32, bit: u32, cw: u16) -> Ext80 {
    if bit < 32 && mask & (1u32 << bit) != 0 {
        ef(ext_to_f64(&x, cw))
    } else {
        x
    }
}
fn polevl(x: Ext80, coef: &[f64], mask: u32, mut bit: u32, cw: u16) -> (Ext80, u32) {
    let mut ans = ef(coef[0]);
    for &c in &coef[1..] {
        ans = ext_add(&ext_mul(&ans, &x, cw), &ef(c), cw);
        ans = maybe(ans, mask, bit, cw);
        bit += 1;
    }
    (ans, bit)
}
fn p1evl(x: Ext80, coef: &[f64], mask: u32, mut bit: u32, cw: u16) -> (Ext80, u32) {
    let mut ans = ext_add(&x, &ef(coef[0]), cw);
    ans = maybe(ans, mask, bit, cw);
    bit += 1;
    for &c in &coef[1..] {
        ans = ext_add(&ext_mul(&ans, &x, cw), &ef(c), cw);
        ans = maybe(ans, mask, bit, cw);
        bit += 1;
    }
    (ans, bit)
}
fn cephes(x: f64, mask: u32, cw: u16) -> f64 {
    let ax = x.abs();
    let xe = ef(ax);
    let (num, den, bit0) = if ax < 8.0 {
        let (p, b) = polevl(xe, &CEPHES_P, mask, 0, cw);
        let (q, b) = p1evl(xe, &CEPHES_Q, mask, b, cw);
        (p, q, b)
    } else {
        let (r, b) = polevl(xe, &CEPHES_R, mask, 0, cw);
        let (s, b) = p1evl(xe, &CEPHES_S, mask, b, cw);
        (r, s, b)
    };
    let mut q = ext_div(&num, &den, cw);
    q = maybe(q, mask, bit0, cw);
    ext_to_f64(&q, cw)
}
fn qwf(z: f64, ff: f64) -> f64 {
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
    let pcs = [("PC64", CW_PC64_RN), ("PC53", CW_PC53_RN), ("PC24", CW_PC24_RN)];
    let rcs = [("RN", 0u16), ("RD", 0x0400), ("RU", 0x0800), ("RZ", 0x0C00)];
    println!("0x5005 Q-tail PC×RC (bar PC64 RN d=53 / exact 1572):");
    let mut best_e = 0usize;
    let mut best_d = 0usize;
    let mut best_lab = String::new();
    for (pn, pc) in pcs {
        for (rn, rc) in rcs {
            let cw = pc | rc;
            for (mn, mask) in [("m0", 0u32), ("0x5005", 0x5005u32)] {
                let mut ex = 0usize;
                let mut dd = 0usize;
                let mut n = 0usize;
                let mut nd = 0usize;
                for r in &rows {
                    if r.z < 4.0 {
                        continue;
                    }
                    n += 1;
                    let d = ulp_distance(qwf(r.z, cephes(r.z, mask, cw)), f64::from_bits(r.qbits))
                        .unwrap_or(99);
                    if d == 0 {
                        ex += 1;
                        if r.direct {
                            dd += 1;
                        }
                    }
                    if r.direct {
                        nd += 1;
                    }
                }
                let lab = format!("{pn} {rn} {mn}");
                if dd >= 40 || (pn == "PC64" && rn == "RN") || dd > best_d {
                    println!("  {lab:18} Q {ex}/{n} d={dd}/{nd} cw={cw:#06x}");
                }
                if ex > best_e || (ex == best_e && dd > best_d) {
                    best_e = ex;
                    best_d = dd;
                    best_lab = lab;
                }
            }
        }
    }
    println!("BEST Q {best_e} d={best_d} {best_lab}");
}
