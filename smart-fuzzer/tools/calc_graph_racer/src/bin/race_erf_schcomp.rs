//! Compensated Clenshaw of Schonfelder Table 1 as P-side. Also 2/√π transplant.
//! Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
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
const TWO_RSQPI: f64 = 1.1283791670955125738961589031215; // 2/sqrt(pi) = 1+A[0]

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn two_sum(a: f64, b: f64) -> (f64, f64) {
    let s = a + b;
    let v = s - a;
    let e = (a - (s - v)) + (b - v);
    (s, e)
}
fn two_prod(a: f64, b: f64) -> (f64, f64) {
    let p = a * b;
    let e = f64::mul_add(a, b, -p);
    (p, e)
}
fn clenshaw_x87(z: f64, a: &[f64]) -> f64 {
    let xe = ef(z.abs());
    let t = ext_sub(&ext_mul(&ef(0.5), &ext_mul(&xe, &xe, CW), CW), &ef(1.0), CW);
    let two = ef(2.0);
    let mut d1 = ef(0.0);
    let mut d2 = ef(0.0);
    for k in (1..a.len()).rev() {
        let dk = ext_add(
            &ext_sub(&ext_mul(&ext_mul(&two, &t, CW), &d1, CW), &d2, CW),
            &ef(a[k]),
            CW,
        );
        d2 = d1;
        d1 = dk;
    }
    let y = ext_add(
        &ext_sub(&ext_mul(&t, &d1, CW), &d2, CW),
        &ext_mul(&ef(0.5), &ef(a[0]), CW),
        CW,
    );
    ext_to_f64(&ext_mul(&xe, &y, CW), CW)
}
fn clenshaw_comp(z: f64, a: &[f64]) -> f64 {
    let t = 0.5 * z * z - 1.0;
    let mut d1 = 0.0;
    let mut d2 = 0.0;
    let mut e1 = 0.0;
    let mut e2 = 0.0;
    for k in (1..a.len()).rev() {
        let (p, pe) = two_prod(2.0 * t, d1);
        let p = p + (pe + 2.0 * t * e1);
        let (s, se) = two_sum(p, -d2);
        let s = s - e2;
        let (dk, de) = two_sum(s, a[k]);
        e2 = e1;
        d2 = d1;
        d1 = dk;
        e1 = se + de;
    }
    let (p, pe) = two_prod(t, d1);
    let (s, se) = two_sum(p, -d2);
    let (y, ye) = two_sum(s, 0.5 * a[0]);
    let y = y + (pe + se + ye - e2 + t * e1);
    z * y
}
fn a21_horner(z: f64) -> f64 {
    let xe = ef(z.abs());
    let u = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in AS.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &u, CW), &ef(c), CW);
    }
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &acc, CW), CW), CW)
}
fn a21_m0(z: f64) -> f64 {
    let xe = ef(z.abs());
    let u = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in AS[1..].iter().rev() {
        acc = ext_add(&ext_mul(&acc, &u, CW), &ef(c), CW);
    }
    acc = ext_add(&ext_mul(&acc, &u, CW), &ef(TWO_RSQPI - 1.0), CW);
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &acc, CW), CW), CW)
}
fn a21_sch_lead(z: f64) -> f64 {
    let lead = f64::from_bits(0x3ff20dd750429b6e) - 1.0;
    let xe = ef(z.abs());
    let u = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in AS[1..].iter().rev() {
        acc = ext_add(&ext_mul(&acc, &u, CW), &ef(c), CW);
    }
    acc = ext_add(&ext_mul(&acc, &u, CW), &ef(lead), CW);
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &acc, CW), CW), CW)
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
    let graphs: [(&str, fn(f64) -> f64); 5] = [
        ("Clenshaw x87", |z| clenshaw_x87(z, &SCH)),
        ("Clenshaw compensated f64", |z| clenshaw_comp(z, &SCH)),
        ("A21 Horner 1+w", a21_horner),
        ("A21 with 2/sqrt(pi) explicit", a21_m0),
        ("A21 lead = Schon M[0]-1", a21_sch_lead),
    ];
    for (name, ev) in graphs {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut maxu = 0u64;
        let mut hit3 = 0usize;
        for &(z, pbits) in &rows {
            let pg = ev(z);
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
            if (z - 0.224609375).abs() < 1e-15 || (z - 0.45247395833333331).abs() < 1e-14 {
                println!("  {name} z={z:.17} ulp={d}");
                if d == 0 {
                    hit3 += 1;
                }
            }
        }
        println!("{name:32} {ex}/{n} max={maxu} hit3={hit3}");
    }
}
