//! 0x5005 leftover-low: w*up(F) analog of 1+w-up. Not an identity.
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
fn cephes(x: f64) -> f64 {
    let ax = x.abs();
    let xe = ef(ax);
    let mask = 0x5005u32;
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
fn qwf(w: f64, ff: f64) -> f64 {
    let v = w * ff;
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
    println!("0x5005 DIRECT leftover-low (ulp>=2 graph<excel):");
    let mut n = 0usize;
    let mut hit_uf = 0usize;
    let mut hit_uw = 0usize;
    let mut hit_uq = 0usize;
    for r in &rows {
        if !r.direct || r.z < 4.0 {
            continue;
        }
        let t = f64::from_bits(r.qbits);
        let ff = cephes(r.z);
        let w = f::w_rn53(r.z);
        let g = qwf(w, ff);
        let d = ulp_distance(g, t).unwrap_or(99);
        if d < 2 || g >= t {
            continue;
        }
        n += 1;
        let duf = ulp_distance(qwf(w, ff.next_up()), t).unwrap_or(99);
        let duw = ulp_distance(qwf(w.next_up(), ff), t).unwrap_or(99);
        let duq = ulp_distance(g.next_up(), t).unwrap_or(99);
        if duf == 0 {
            hit_uf += 1;
        }
        if duw == 0 {
            hit_uw += 1;
        }
        if duq == 0 {
            hit_uq += 1;
        }
        println!(
            "  z={:.16} kind={} ulp={d} w*upF={duf} upw*F={duw} upQ={duq}",
            r.z,
            kind(r.z)
        );
    }
    println!("n={n} w*upF hit={hit_uf} upw*F hit={hit_uw} upQ hit={hit_uq}");
    let mut k0 = 0usize;
    let mut kuf = 0usize;
    let mut nd = 0usize;
    for r in &rows {
        if !r.direct || r.z < 4.0 {
            continue;
        }
        nd += 1;
        let t = f64::from_bits(r.qbits);
        let ff = cephes(r.z);
        let w = f::w_rn53(r.z);
        if ulp_distance(qwf(w, ff), t).unwrap_or(99) == 0 {
            k0 += 1;
        }
        if ulp_distance(qwf(w, ff.next_up()), t).unwrap_or(99) == 0 {
            kuf += 1;
        }
    }
    println!("DIRECT keep base={k0}/{nd} w*upF={kuf}/{nd} (bar 53)");
}
