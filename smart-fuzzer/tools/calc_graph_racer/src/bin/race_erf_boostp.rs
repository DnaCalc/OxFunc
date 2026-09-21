//! Boost ErfImp An/Ad: erf(z)=z*1.125 + z*P(z)/Q(z) for z∈[0,0.5). Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const AN: [f64; 8] = [
    0.00337916709551257388990745,
    -0.00073695653048167948530905,
    -0.374732337392919607868241,
    0.0817442448733587196071743,
    -0.0421089319936548595203468,
    0.0070165709512095756344528,
    -0.00495091255982435110337458,
    0.000871646599037922480317225,
];
const AD: [f64; 8] = [
    1.0,
    -0.218088218087924645390535,
    0.412542972725442099083918,
    -0.0841891147873106755410271,
    0.0655338856400241519690695,
    -0.0120019604454941768171266,
    0.00408165558926174048329689,
    -0.000615900721557769691924509,
];

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn horner_lo(cs: &[f64], x: Ext80) -> Ext80 {
    let mut acc = ef(0.0);
    for &c in cs.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &x, CW), &ef(c), CW);
    }
    acc
}
fn horner_hi(cs: &[f64], x: Ext80) -> Ext80 {
    let mut acc = ef(cs[0]);
    for &c in &cs[1..] {
        acc = ext_add(&ext_mul(&acc, &x, CW), &ef(c), CW);
    }
    acc
}
fn erf_lo(z: f64) -> f64 {
    let ze = ef(z.abs());
    let r = ext_div(&horner_lo(&AN, ze), &horner_lo(&AD, ze), CW);
    let t = ext_add(&ef(1.125), &r, CW);
    ext_to_f64(&ext_mul(&ze, &t, CW), CW)
}
fn erf_hi(z: f64) -> f64 {
    let ze = ef(z.abs());
    let r = ext_div(&horner_hi(&AN, ze), &horner_hi(&AD, ze), CW);
    let t = ext_add(&ef(1.125), &r, CW);
    ext_to_f64(&ext_mul(&ze, &t, CW), CW)
}
fn erf_f64(z: f64) -> f64 {
    let mut pn = 0.0;
    for &c in AN.iter().rev() {
        pn = pn * z + c;
    }
    let mut pd = 0.0;
    for &c in AD.iter().rev() {
        pd = pd * z + c;
    }
    z * 1.125 + z * (pn / pd)
}
/// Boost C++: Y*z + z*(P/Q) with Y=1.125, not z*(Y+P/Q).
fn erf_yz(z: f64) -> f64 {
    let ze = ef(z.abs());
    let r = ext_div(&horner_lo(&AN, ze), &horner_lo(&AD, ze), CW);
    let a = ext_mul(&ze, &ef(1.125), CW);
    let b = ext_mul(&ze, &r, CW);
    ext_to_f64(&ext_add(&a, &b, CW), CW)
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    const BANKS: [&str; 7] = [
        "answers-b9train.json",
        "answers-erfp.json",
        "answers-erfm.json",
        "answers-b8erf.json",
        "answers-b7erf.json",
        "answers-b11.json",
        "answers-b10.json",
    ];
    let mut map = BTreeMap::new();
    for name in BANKS {
        let path = format!("{dir}/{name}");
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let bank: WitnessSet = serde_json::from_str(&text).unwrap();
        for w in &bank.witnesses {
            let x = match &w.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
                _ => continue,
            };
            if x < 0.0 || x >= 0.5 {
                continue;
            }
            let Some(p) = parse_bits_hex(&w.expected_bits) else {
                continue;
            };
            map.insert(x.to_bits(), (x, p.to_bits()));
        }
    }
    let rows: Vec<_> = map.into_values().collect();
    let graphs: [(&str, fn(f64) -> f64); 4] = [
        ("Boost An/Ad low-first x87", erf_lo),
        ("Boost An/Ad Yz+zPQ x87", erf_yz),
        ("Boost An/Ad high-first x87", erf_hi),
        ("Boost An/Ad f64", erf_f64),
    ];
    for (name, ev) in graphs {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut maxu = 0u64;
        for &(z, pbits) in &rows {
            let pg = ev(z);
            if !pg.is_finite() {
                continue;
            }
            let d = ulp_distance(pg, f64::from_bits(pbits)).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            n += 1;
            if d == 0 {
                ex += 1;
            } else {
                maxu = maxu.max(d);
            }
        }
        println!("{name:32} {ex}/{n} max={maxu}");
    }
}
