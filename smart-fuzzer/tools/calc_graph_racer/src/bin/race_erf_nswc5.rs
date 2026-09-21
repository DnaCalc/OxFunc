//! NSWC 5+3 rational (correct fused loop) and Schonfelder t-map association.
//! Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
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
fn nswc5(z: f64) -> f64 {
    let xe = ef(z.abs());
    let t = ext_mul(&xe, &xe, CW);
    let mut xnum = ext_mul(&ef(NSWC_A[4]), &t, CW);
    let mut xden = t;
    for i in 0..3 {
        xnum = ext_mul(&ext_add(&xnum, &ef(NSWC_A[i]), CW), &t, CW);
        xden = ext_mul(&ext_add(&xden, &ef(NSWC_B[i]), CW), &t, CW);
    }
    ext_to_f64(
        &ext_mul(
            &xe,
            &ext_div(&ext_add(&xnum, &ef(NSWC_A[3]), CW), &ext_add(&xden, &ef(1.0), CW), CW),
            CW,
        ),
        CW,
    )
}
fn nswc5_1w(z: f64) -> f64 {
    // erf = x * (1 + Horner([A4,A3,A2,A1,A0], x^2) wait: A[4] is 2/sqrt(pi)-ish
    let xe = ef(z.abs());
    let t = ext_mul(&xe, &xe, CW);
    let mut acc = ef(NSWC_A[0]);
    for &c in &NSWC_A[1..] {
        acc = ext_add(&ext_mul(&acc, &t, CW), &ef(c), CW);
    }
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &acc, CW), CW), CW)
}
fn clenshaw_t(z: f64, t: Ext80) -> f64 {
    let xe = ef(z.abs());
    let two = ef(2.0);
    let mut d1 = ef(0.0);
    let mut d2 = ef(0.0);
    for k in (1..SCH.len()).rev() {
        let dk = ext_add(
            &ext_sub(&ext_mul(&ext_mul(&two, &t, CW), &d1, CW), &d2, CW),
            &ef(SCH[k]),
            CW,
        );
        d2 = d1;
        d1 = dk;
    }
    let y = ext_add(
        &ext_sub(&ext_mul(&t, &d1, CW), &d2, CW),
        &ext_mul(&ef(0.5), &ef(SCH[0]), CW),
        CW,
    );
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
    let graphs: [(&str, fn(f64) -> f64); 6] = [
        ("NSWC 5+3 fused x87", nswc5),
        ("NSWC 5-term 1+w Horner", nswc5_1w),
        ("t=0.5*(x*x)-1", |z| {
            let xe = ef(z.abs());
            let t = ext_sub(&ext_mul(&ef(0.5), &ext_mul(&xe, &xe, CW), CW), &ef(1.0), CW);
            clenshaw_t(z, t)
        }),
        ("t=(0.5*x)*x-1", |z| {
            let xe = ef(z.abs());
            let t = ext_sub(
                &ext_mul(&ext_mul(&ef(0.5), &xe, CW), &xe, CW),
                &ef(1.0),
                CW,
            );
            clenshaw_t(z, t)
        }),
        ("t=0.5*(x*x-2)", |z| {
            let xe = ef(z.abs());
            let t = ext_mul(
                &ef(0.5),
                &ext_sub(&ext_mul(&xe, &xe, CW), &ef(2.0), CW),
                CW,
            );
            clenshaw_t(z, t)
        }),
        ("t=x*(x*0.5)-1", |z| {
            let xe = ef(z.abs());
            let t = ext_sub(&ext_mul(&xe, &ext_mul(&xe, &ef(0.5), CW), CW), &ef(1.0), CW);
            clenshaw_t(z, t)
        }),
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
        println!("{name:28} {ex}/{n} max={maxu}");
    }
}
