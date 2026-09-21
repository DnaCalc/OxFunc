//! P-side Chebyshev of erf/x on t=8x²−1 (interval [0,0.5]). Generated n=10. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const T8: [f64; 10] = [
    2.167764410981226023422160,
    -4.367779108737509726597131e-2,
    8.071118294131840042020829e-4,
    -1.191317951889317940040507e-5,
    1.440278733915555962296588e-7,
    -1.467716223645495639234165e-9,
    1.290184916287942751736550e-11,
    -9.911673484638174006654611e-14,
    9.244259604038638252181434e-16,
    7.813431418946882659220572e-16,
];
const SCH: [f64; 18] = [
    1.4831105640848035818894480790578,
    -0.3010710733865949424707310463118,
    0.0689948306898315662466031807188,
    -0.0139162712647221876825465256678,
    0.0024207995224334636628916782398,
    -0.0003658639685848086446493825778,
    4.86209844323190482828875688e-5,
    -5.7492565580356848350542158e-6,
    6.113243578434764697067588e-7,
    -5.89910153129584343908468e-8,
    5.2070090920686482404558e-9,
    -4.232975879965543268108e-10,
    3.18811350664917497488e-11,
    -2.2361550188326842738e-12,
    1.467329847991084928e-13,
    -9.0440019853817478e-15,
    5.254813715470928e-16,
    -2.88742612228498e-17,
];

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn dcsevl(t: Ext80, cs: &[f64]) -> Ext80 {
    let twox = ext_mul(&ef(2.0), &t, CW);
    let mut b0 = ef(0.0);
    let mut b1 = ef(0.0);
    let mut b2 = ef(0.0);
    for i in (0..cs.len()).rev() {
        b2 = b1;
        b1 = b0;
        b0 = ext_add(
            &ext_sub(&ext_mul(&twox, &b1, CW), &b2, CW),
            &ef(cs[i]),
            CW,
        );
    }
    ext_mul(&ef(0.5), &ext_sub(&b0, &b2, CW), CW)
}
fn erf_t8(z: f64) -> f64 {
    let xe = ef(z.abs());
    let t = ext_sub(
        &ext_mul(&ef(8.0), &ext_mul(&xe, &xe, CW), CW),
        &ef(1.0),
        CW,
    );
    let y = dcsevl(t, &T8);
    ext_to_f64(&ext_mul(&xe, &y, CW), CW)
}
fn erf_sch_t8(z: f64) -> f64 {
    let xe = ef(z.abs());
    let t = ext_sub(
        &ext_mul(&ef(8.0), &ext_mul(&xe, &xe, CW), CW),
        &ef(1.0),
        CW,
    );
    let y = dcsevl(t, &SCH);
    ext_to_f64(&ext_mul(&xe, &y, CW), CW)
}
fn erf_t8_halfx2(z: f64) -> f64 {
    let xe = ef(z.abs());
    let t = ext_sub(&ext_mul(&ef(0.5), &ext_mul(&xe, &xe, CW), CW), &ef(1.0), CW);
    let y = dcsevl(t, &T8);
    ext_to_f64(&ext_mul(&xe, &y, CW), CW)
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
        ("t=8x2-1 n=10 DCSEVL", erf_t8),
        ("t=8x2-1 Schonfelder coeffs", erf_sch_t8),
        ("T8 coeffs on t=x2/2-1", erf_t8_halfx2),
        ("libm::erf", libm::erf),
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
        println!("{name:32} {ex}/{n} max={maxu}");
    }
}
