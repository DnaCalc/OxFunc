//! leftover-low 6 of 0x5005: RU last-mul ∪ up(w)×up(F). Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const CW_RU: u16 = CW_PC64_RN | 0x0800;
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
    let (num, den, bit0) = if ax < 8.0 {
        let (p, b) = polevl(xe, &CEPHES_P, MASK, 0);
        let (q, b) = p1evl(xe, &CEPHES_Q, MASK, b);
        (p, q, b)
    } else {
        let (r, b) = polevl(xe, &CEPHES_R, MASK, 0);
        let (s, b) = p1evl(xe, &CEPHES_S, MASK, b);
        (r, s, b)
    };
    let mut v = ext_div(&num, &den, CW);
    v = maybe(v, MASK, bit0);
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
fn ru_mul(w: f64, ff: f64) -> f64 {
    let v = ext_to_f64(&ext_mul(&ef(w), &ef(ff), CW_RU), CW_RU);
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
    println!("6 leftover-low 5005 two-mode:");
    let mut hit_ru = 0usize;
    let mut hit_tf = 0usize;
    let mut hit_u = 0usize;
    for &lz in &SIX {
        let Some(r) = tail.iter().find(|rr| (rr.z - lz).abs() < 1e-12) else {
            println!("  z={lz:.16} MISSING");
            continue;
        };
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(lz);
        let ff = cephes(lz);
        let dc = ulp_distance(mul(w, ff), t).unwrap_or(99);
        let eru = ulp_distance(ru_mul(w.next_up(), ff), t).unwrap_or(99) == 0;
        let etf = ulp_distance(mul(w.next_up(), ff.next_up()), t).unwrap_or(99) == 0;
        let ew = ulp_distance(mul(w.next_up(), ff), t).unwrap_or(99);
        let efup = ulp_distance(mul(w, ff.next_up()), t).unwrap_or(99);
        if eru {
            hit_ru += 1;
        }
        if etf {
            hit_tf += 1;
        }
        if eru || etf {
            hit_u += 1;
        }
        println!(
            "  z={lz:.16} CR={dc} RU={eru} TF={etf} upw={ew} upF={efup} any={}",
            eru || etf
        );
    }
    println!("leftover RU={hit_ru}/6 TF={hit_tf}/6 union={hit_u}/6");
    let mut qe_c = 0usize;
    let mut qd_c = 0usize;
    let mut qe_u = 0usize;
    let mut qd_u = 0usize;
    for row in rows.iter().filter(|rr| rr.z >= 4.0) {
        let t = f64::from_bits(row.qbits);
        let w = f::w_rn53(row.z);
        let ff = cephes(row.z);
        let c = ulp_distance(mul(w, ff), t).unwrap_or(99) == 0;
        let u = ulp_distance(ru_mul(w.next_up(), ff), t).unwrap_or(99) == 0
            || ulp_distance(mul(w.next_up(), ff.next_up()), t).unwrap_or(99) == 0;
        if c {
            qe_c += 1;
            if row.direct {
                qd_c += 1;
            }
        }
        if u {
            qe_u += 1;
            if row.direct {
                qd_u += 1;
            }
        }
    }
    println!("Q z>=4 5005={qe_c} d={qd_c} two-mode={qe_u} d={qd_u} (bar 1572/53)");
}
