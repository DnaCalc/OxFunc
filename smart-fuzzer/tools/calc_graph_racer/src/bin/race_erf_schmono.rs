//! Schonfelder Chebyshev converted to monomial Horner vs Clenshaw vs NSWC A21.
//! Same even-power family. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const A1: [f64; 18] = [
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
const M: [f64; 18] = [
    1.12837916709551278061e+00,
    -3.76126389031837260468e-01,
    1.12837916709544267002e-01,
    -2.68661706450561248161e-02,
    5.22397762501597244522e-03,
    -8.54832700872036437001e-04,
    1.20553326442792335714e-04,
    -1.49256449451965359583e-05,
    1.64620513361047982823e-06,
    -1.63652986283818842074e-07,
    1.48036159741914948142e-08,
    -1.22726777461245529123e-09,
    9.35435720862897768209e-11,
    -6.51152298334284814339e-12,
    4.02113254293530697369e-13,
    -2.05380707696652787451e-14,
    7.53603126561993095305e-16,
    -1.44371306114248985554e-17,
];
const AS: [f64; 21] = [
    0.1283791670955125738961589031215,
    -0.3761263890318375246320529677070,
    0.1128379167095512573896158902931,
    -0.2686617064513125175943235372542e-01,
    0.5223977625442187842111812447877e-02,
    -0.8548327023450852832540164081187e-03,
    0.1205533298178966425020717182498e-03,
    -0.1492565035840625090430728526820e-04,
    0.1646211436588924261080723578109e-05,
    -0.1636584469123468757408968429674e-06,
    0.1480719281587021715400818627811e-07,
    -0.1229055530145120140800510155331e-08,
    0.9422759058437197017313055084212e-10,
    -0.6711366740969385085896257227159e-11,
    0.4463222608295664017461758843550e-12,
    -0.2783497395542995487275065856998e-13,
    0.1634095572365337143933023780777e-14,
    -0.9052845786901123985710019387938e-16,
    0.4708274559689744439341671426731e-17,
    -0.2187159356685015949749948252160e-18,
    0.7043407712019701609635599701333e-20,
];

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn horner_u(cs: &[f64], u: Ext80) -> Ext80 {
    let mut acc = ef(0.0);
    for &c in cs.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &u, CW), &ef(c), CW);
    }
    acc
}
fn erf_clenshaw(z: f64) -> f64 {
    let xe = ef(z.abs());
    let t = ext_sub(&ext_mul(&ef(0.5), &ext_mul(&xe, &xe, CW), CW), &ef(1.0), CW);
    let two = ef(2.0);
    let mut d1 = ef(0.0);
    let mut d2 = ef(0.0);
    for k in (1..A1.len()).rev() {
        let dk = ext_add(
            &ext_sub(&ext_mul(&ext_mul(&two, &t, CW), &d1, CW), &d2, CW),
            &ef(A1[k]),
            CW,
        );
        d2 = d1;
        d1 = dk;
    }
    let y = ext_add(
        &ext_sub(&ext_mul(&t, &d1, CW), &d2, CW),
        &ext_mul(&ef(0.5), &ef(A1[0]), CW),
        CW,
    );
    ext_to_f64(&ext_mul(&xe, &y, CW), CW)
}
fn erf_mono(z: f64) -> f64 {
    let xe = ef(z.abs());
    let u = ext_mul(&xe, &xe, CW);
    ext_to_f64(&ext_mul(&xe, &horner_u(&M, u), CW), CW)
}
fn erf_a21(z: f64) -> f64 {
    let xe = ef(z.abs());
    let u = ext_mul(&xe, &xe, CW);
    let w = horner_u(&AS, u);
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &w, CW), CW), CW)
}
fn erf_mono_f64(z: f64) -> f64 {
    let u = z * z;
    let mut acc = 0.0;
    for &c in M.iter().rev() {
        acc = acc * u + c;
    }
    z * acc
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
        ("Clenshaw x87-cont", erf_clenshaw),
        ("monomial M x87 Horner", erf_mono),
        ("monomial M f64 Horner", erf_mono_f64),
        ("NSWC A21 x87 1+w", erf_a21),
    ];
    for (name, ev) in graphs {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut maxu = 0u64;
        let mut sum = 0u128;
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
                sum += d as u128;
            }
        }
        println!("{name:24} {ex}/{n} max={maxu} sum={sum}");
    }
}
