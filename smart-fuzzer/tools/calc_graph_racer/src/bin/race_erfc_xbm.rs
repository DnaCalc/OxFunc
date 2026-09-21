//! XBIG stubborn pair: x87 last-mul w×F vs f64. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
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
fn f_mask(x: f64, mask: u32) -> f64 {
    let xe = ef(x.abs());
    let (r, b) = polevl_mask(xe, &CEPHES_R, mask, 0);
    let (s, b) = p1evl_mask(xe, &CEPHES_S, mask, b);
    let mut q = ext_div(&r, &s, CW);
    q = maybe(q, mask, b);
    ext_to_f64(&q, CW)
}
fn qw64(z: f64, ff: f64) -> f64 {
    let v = f::w_rn53(z) * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}
fn qw80(z: f64, ff: f64) -> f64 {
    ext_to_f64(&ext_mul(&ef(f::w_rn53(z)), &ef(ff), CW), CW)
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let pair = [26.5300000000000011f64, 26.5429999999999957];
    println!("stubborn XBIG pair last-mul:");
    for r in &rows {
        if pair.iter().all(|z| (r.z - z).abs() > 1e-12) {
            continue;
        }
        let t = f64::from_bits(r.qbits);
        let f5 = f_mask(r.z, 0x5005);
        let fnat = f::cephes_f(r.z);
        println!(
            "  z={:.16} f64*5005={} x87*5005={} f64*nat={} x87*nat={} f64*upF={} x87*upF={}",
            r.z,
            ulp_distance(qw64(r.z, f5), t).unwrap_or(99),
            ulp_distance(qw80(r.z, f5), t).unwrap_or(99),
            ulp_distance(qw64(r.z, fnat), t).unwrap_or(99),
            ulp_distance(qw80(r.z, fnat), t).unwrap_or(99),
            ulp_distance(qw64(r.z, f5.next_up()), t).unwrap_or(99),
            ulp_distance(qw80(r.z, f5.next_up()), t).unwrap_or(99),
        );
    }
    println!("z>=8 keep (bar 0x5005 991):");
    for (name, ev) in [
        ("f64*5005", Box::new(|z: f64| qw64(z, f_mask(z, 0x5005))) as Box<dyn Fn(f64) -> f64>),
        ("x87*5005", Box::new(|z| qw80(z, f_mask(z, 0x5005)))),
        ("f64*nat", Box::new(|z| qw64(z, f::cephes_f(z)))),
        ("x87*nat", Box::new(|z| qw80(z, f::cephes_f(z)))),
        ("x87*up 5005F", Box::new(|z| qw80(z, f_mask(z, 0x5005).next_up()))),
    ] {
        let mut ex = 0usize;
        let mut hit = 0usize;
        for r in &rows {
            if r.z < 8.0 {
                continue;
            }
            if ulp_distance(ev(r.z), f64::from_bits(r.qbits)).unwrap_or(99) == 0 {
                ex += 1;
                if pair.iter().any(|z| (r.z - z).abs() < 1e-12) {
                    hit += 1;
                }
            }
        }
        println!("  {name:14} Q {ex} pair_hit={hit}/2");
    }
}
