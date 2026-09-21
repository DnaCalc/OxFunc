//! Named erf skeletons on ERF.PRECISE z<0.5. Not A[] pokes. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const FDLIBM_PP: [f64; 5] = [
    1.28379167095512558561e-01,
    -3.25042107247001499370e-01,
    -2.84817495755985104766e-02,
    -5.77027029648944159157e-03,
    -2.37630166566501626084e-05,
];
const FDLIBM_QQ: [f64; 5] = [
    3.97917223959155352819e-01,
    6.50222499887672944485e-02,
    5.08130628187576562776e-03,
    1.32494738004321644526e-04,
    -3.96022827877536812320e-06,
];
const CEPHES_T: [f64; 5] = [
    9.60497373987051638749e0,
    9.00260197203842689217e1,
    2.23200534594684319226e3,
    7.00332514112805075473e3,
    5.55923013010394962768e4,
];
const CEPHES_U: [f64; 5] = [
    3.35617141647503099647e1,
    5.21357949780152679795e2,
    4.59432382970980127987e3,
    2.26290000613890934246e4,
    4.92673942608635921086e4,
];
const AA: [f64; 5] = [
    3.16112374387056560,
    113.864154151050156,
    377.485237685302021,
    3209.37758913846947,
    0.185777706184603153,
];
const BB: [f64; 4] = [
    23.6012909523441209,
    244.024637934444173,
    1282.61652607737228,
    2844.23683343917062,
];
const NSWC_A: [f64; 5] = [
    0.771058495001320e-04,
    -0.133733772997339e-02,
    0.323076579225834e-01,
    0.479137145607681e-01,
    0.128379167095513e+00,
];
const NSWC_B: [f64; 3] = [
    0.301048631703895e-02,
    0.538971687740286e-01,
    0.375795757275549e+00,
];

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn horner_lo(cs: &[f64], x: f64) -> f64 {
    let mut a = 0.0;
    for &c in cs.iter().rev() {
        a = a * x + c;
    }
    a
}
fn polevl_hi(cs: &[f64], x: f64) -> f64 {
    let mut a = cs[0];
    for &c in &cs[1..] {
        a = a * x + c;
    }
    a
}
fn p1evl_hi(cs: &[f64], x: f64) -> f64 {
    let mut a = x + cs[0];
    for &c in &cs[1..] {
        a = a * x + c;
    }
    a
}
fn fdlibm_pp(z: f64) -> f64 {
    let zz = z * z;
    let r = horner_lo(&FDLIBM_PP, zz);
    let s = 1.0 + zz * horner_lo(&FDLIBM_QQ, zz);
    z + z * (r / s)
}
fn fdlibm_pp_x87(z: f64) -> f64 {
    let ze = ef(z);
    let zz = ext_mul(&ze, &ze, CW);
    let mut r = ef(0.0);
    for &c in FDLIBM_PP.iter().rev() {
        r = ext_add(&ext_mul(&r, &zz, CW), &ef(c), CW);
    }
    let mut q = ef(0.0);
    for &c in FDLIBM_QQ.iter().rev() {
        q = ext_add(&ext_mul(&q, &zz, CW), &ef(c), CW);
    }
    let s = ext_add(&ef(1.0), &ext_mul(&zz, &q, CW), CW);
    let y = ext_div(&r, &s, CW);
    ext_to_f64(&ext_add(&ze, &ext_mul(&ze, &y, CW), CW), CW)
}
fn cephes_tu(z: f64) -> f64 {
    let zz = z * z;
    z * polevl_hi(&CEPHES_T, zz) / p1evl_hi(&CEPHES_U, zz)
}
fn cody_ab_x87(z: f64) -> f64 {
    let ye = ef(z);
    let ysq = ext_mul(&ye, &ye, CW);
    let mut xnum = ext_mul(&ef(AA[4]), &ysq, CW);
    let mut xden = ysq;
    for i in 0..3 {
        xnum = ext_mul(&ext_add(&xnum, &ef(AA[i]), CW), &ysq, CW);
        xden = ext_mul(&ext_add(&xden, &ef(BB[i]), CW), &ysq, CW);
    }
    ext_to_f64(
        &ext_div(
            &ext_mul(&ye, &ext_add(&xnum, &ef(AA[3]), CW), CW),
            &ext_add(&xden, &ef(BB[3]), CW),
            CW,
        ),
        CW,
    )
}
fn nswc_erfc1(z: f64) -> f64 {
    let t = z * z;
    z * (horner_lo(&NSWC_A, t) + 1.0) / (1.0 + t * horner_lo(&NSWC_B, t))
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
            let Some(e) = parse_bits_hex(&w.expected_bits) else {
                continue;
            };
            if x > 0.0 && x < 0.5 {
                map.entry(x.to_bits()).or_insert(e.to_bits());
            }
        }
    }
    let rows: Vec<_> = map
        .into_iter()
        .map(|(b, e)| (f64::from_bits(b), e))
        .collect();
    println!("P-side rows={}", rows.len());
    let graphs: [(&str, fn(f64) -> f64); 6] = [
        ("fdlibm PP/QQ native x+x*y", fdlibm_pp),
        ("fdlibm PP/QQ x87", fdlibm_pp_x87),
        ("Cephes T/U erf", cephes_tu),
        ("Cody A/B x87", cody_ab_x87),
        ("NSWC erfc1 5-coeff", nswc_erfc1),
        ("libm::erf", libm::erf),
    ];
    for (name, ev) in graphs {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut max = 0u64;
        let mut sum = 0u128;
        for &(z, bits) in &rows {
            let g = ev(z);
            if !g.is_finite() {
                continue;
            }
            let d = ulp_distance(g, f64::from_bits(bits)).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            n += 1;
            if d == 0 {
                ex += 1;
            } else {
                max = max.max(d);
                sum += d as u128;
            }
        }
        println!("{name:28} {ex}/{n} max={max} sum={sum}");
    }
}
