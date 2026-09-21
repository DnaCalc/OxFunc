//! PC64 RU mask0 vs RN 0x5005 Q / F_or overlap. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW_RN: u16 = CW_PC64_RN;
const CW_RU: u16 = CW_PC64_RN | 0x0800;
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
    let mut q_both = 0usize;
    let mut q_ru = 0usize;
    let mut q_rn = 0usize;
    let mut q_ru_u1 = 0usize;
    let mut q_rn_u1 = 0usize;
    let mut d_both = 0usize;
    let mut d_ru = 0usize;
    let mut d_rn = 0usize;
    let mut d_ru_u1 = 0usize;
    let mut d_rn_u1 = 0usize;
    let mut f_both = 0usize;
    let mut f_ru = 0usize;
    let mut f_rn = 0usize;
    let mut n = 0usize;
    let mut shown = 0usize;
    println!("only_RU DIRECT (first 10):");
    for r in &rows {
        if r.z < 4.0 {
            continue;
        }
        n += 1;
        let t = f64::from_bits(r.qbits);
        let gru = qwf(r.z, cephes(r.z, 0, CW_RU));
        let grn = qwf(r.z, cephes(r.z, 0x5005, CW_RN));
        let dru = ulp_distance(gru, t).unwrap_or(99);
        let drn = ulp_distance(grn, t).unwrap_or(99);
        match (dru == 0, drn == 0) {
            (true, true) => q_both += 1,
            (true, false) => {
                q_ru += 1;
                if drn == 1 {
                    q_ru_u1 += 1;
                }
            }
            (false, true) => {
                q_rn += 1;
                if dru == 1 {
                    q_rn_u1 += 1;
                }
            }
            _ => {}
        }
        if r.direct {
            match (dru == 0, drn == 0) {
                (true, true) => d_both += 1,
                (true, false) => {
                    d_ru += 1;
                    if drn == 1 {
                        d_ru_u1 += 1;
                    }
                    if shown < 10 {
                        println!(
                            "  z={:.16} RN_ulp={drn} RU {} excel",
                            r.z,
                            if gru < t { "<" } else { ">" }
                        );
                        shown += 1;
                    }
                }
                (false, true) => {
                    d_rn += 1;
                    if dru == 1 {
                        d_rn_u1 += 1;
                    }
                }
                _ => {}
            }
        }
        if let Some(fo) = f::f_or(r.z, r.qbits) {
            let fru = ulp_distance(cephes(r.z, 0, CW_RU), fo).unwrap_or(99);
            let frn = ulp_distance(cephes(r.z, 0x5005, CW_RN), fo).unwrap_or(99);
            match (fru == 0, frn == 0) {
                (true, true) => f_both += 1,
                (true, false) => f_ru += 1,
                (false, true) => f_rn += 1,
                _ => {}
            }
        }
    }
    println!(
        "Q all n={n} both={q_both} only_RU={q_ru} (u1={q_ru_u1}) only_RN={q_rn} (u1={q_rn_u1}) union={}",
        q_both + q_ru + q_rn
    );
    println!(
        "Q DIRECT both={d_both} only_RU={d_ru} (u1={d_ru_u1}) only_RN={d_rn} (u1={d_rn_u1}) union={}",
        d_both + d_ru + d_rn
    );
    println!(
        "F_or both={f_both} only_RU={f_ru} only_RN={f_rn} union={}",
        f_both + f_ru + f_rn
    );
}
