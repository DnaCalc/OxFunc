//! Modern Boost erf_imp 53/64 even-poly in z² and Boost 1.35 P(z)/Q(z). P-side. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;

/// Boost develop 53-bit: z*(Y + P(z²)/Q(z²)), Y is a float.
const Y53: f64 = 1.044948577880859375;
const P53: [f64; 5] = [
    0.0834305892146531832907,
    -0.338165134459360935041,
    -0.0509990735146777432841,
    -0.00772758345802133288487,
    -0.000322780120964605683831,
];
const Q53: [f64; 5] = [
    1.0,
    0.455004033050794024546,
    0.0875222600142252549554,
    0.00858571925074406212772,
    0.000370900071787748000569,
];

/// Boost 64-bit long-double table, rounded to f64.
const Y64: f64 = 1.044948577880859375;
const P64: [f64; 6] = [
    0.0834305892146531988966,
    -0.338097283075565413695,
    -0.0509602734406067204596,
    -0.00904906346158537794396,
    -0.000489468651464798669181,
    -0.200305626366151877759e-4,
];
const Q64: [f64; 6] = [
    1.0,
    0.455817300515875172439,
    0.0916537354356241792007,
    0.0102722652675910031202,
    0.000650511752687851548735,
    0.189532519105655496778e-4,
];

/// Boost 1.35 53-bit: z*1.125 + z*P(z)/Q(z), 7+7 in z not z².
const N135: [f64; 7] = [
    0.00337916709551257778174,
    -0.000147024115786688745475,
    -0.37463022236812520164,
    0.0163061594494816999803,
    -0.0534354147807331748737,
    0.00161898096813581982844,
    -0.0059528010489182840404,
];
const D135: [f64; 7] = [
    1.0,
    -0.0435089806536379531594,
    0.442761965043509204727,
    -0.017375974533016704678,
    0.0772756490303260060769,
    -0.00210552465858669941879,
    0.00544772980263244037286,
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
fn even_x87(z: f64, y: f64, p: &[f64], q: &[f64]) -> f64 {
    let ze = ef(z.abs());
    let zz = ext_mul(&ze, &ze, CW);
    let r = ext_div(&horner_lo(p, zz), &horner_lo(q, zz), CW);
    ext_to_f64(&ext_mul(&ze, &ext_add(&ef(y), &r, CW), CW), CW)
}
fn even_f64(z: f64, y: f64, p: &[f64], q: &[f64]) -> f64 {
    let zz = z * z;
    let mut pn = 0.0;
    for &c in p.iter().rev() {
        pn = pn * zz + c;
    }
    let mut pd = 0.0;
    for &c in q.iter().rev() {
        pd = pd * zz + c;
    }
    z * (y + pn / pd)
}
fn boost53_x87(z: f64) -> f64 {
    even_x87(z, Y53, &P53, &Q53)
}
fn boost53_f64(z: f64) -> f64 {
    even_f64(z, Y53, &P53, &Q53)
}
fn boost64_x87(z: f64) -> f64 {
    even_x87(z, Y64, &P64, &Q64)
}
fn boost64_f64(z: f64) -> f64 {
    even_f64(z, Y64, &P64, &Q64)
}
fn boost135_x87(z: f64) -> f64 {
    let ze = ef(z.abs());
    let r = ext_div(&horner_lo(&N135, ze), &horner_lo(&D135, ze), CW);
    ext_to_f64(&ext_mul(&ze, &ext_add(&ef(1.125), &r, CW), CW), CW)
}
fn boost135_f64(z: f64) -> f64 {
    let mut pn = 0.0;
    for &c in N135.iter().rev() {
        pn = pn * z + c;
    }
    let mut pd = 0.0;
    for &c in D135.iter().rev() {
        pd = pd * z + c;
    }
    z * 1.125 + z * (pn / pd)
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
    let graphs: [(&str, fn(f64) -> f64); 6] = [
        ("Boost53 even z2 x87", boost53_x87),
        ("Boost53 even z2 f64", boost53_f64),
        ("Boost64 even z2 x87", boost64_x87),
        ("Boost64 even z2 f64", boost64_f64),
        ("Boost1.35 P(z)/Q(z) x87", boost135_x87),
        ("Boost1.35 P(z)/Q(z) f64", boost135_f64),
    ];
    println!("P-side rows={}", rows.len());
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
        println!("{name:28} {ex}/{n} max={maxu}");
    }
}
