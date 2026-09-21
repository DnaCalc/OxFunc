//! Eval-model / association of Cody C/D and NSWC t-map; ERF P-side
//! 21-term series vs Cody A/B. Not a 24-bit cube. Not an identity.
//!
//!   cargo run --release --bin race_erfc_eval -- G3-01-dist

use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC53_RN, CW_PC64_RN,
};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const ULP_CAP: u64 = 1 << 20;
const C: [f64; 9] = [
    0.564188496988670089,
    8.88314979438837594,
    66.1191906371416295,
    298.635138197400131,
    881.95222124176909,
    1712.04761263407058,
    2051.07837782607147,
    1230.33935479799725,
    2.15311535474403846e-8,
];
const D: [f64; 8] = [
    15.7449261107098347,
    117.693950891312499,
    537.181101862009858,
    1621.38957456669019,
    3290.79923573345963,
    4362.61909014324716,
    3439.36767414372164,
    1230.33935480374942,
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
const PQR_P: [f64; 8] = [
    0.16506148041280876191828601e-03,
    0.15471455377139313353998665e-03,
    0.44852548090298868465196794e-04,
    -0.49177280017226285450486205e-05,
    -0.69353602078656412367801676e-05,
    -0.20508667787746282746857743e-05,
    -0.28982842617824971177267380e-06,
    -0.17272433544836633301127174e-07,
];
const PQR_Q: [f64; 8] = [
    1.0,
    0.16272656776533322859856317e+01,
    0.12040996037066026106794322e+01,
    0.52400246352158386907601472e+00,
    0.14497345252798672362384241e+00,
    0.25592517111042546492590736e-01,
    0.26869088293991371028123158e-02,
    0.13133767840925681614496481e-03,
];
const PQR_R: [f64; 9] = [
    0.145589721275038539045668824025,
    -0.273421931495426482902320421863,
    0.226008066916621506788789064272,
    -0.163571895523923805648814425592,
    0.102604312032193978662297299832,
    -0.548023266949835519254211506880e-01,
    0.241432239725390106956523668160e-01,
    -0.822062115403915116036874169600e-02,
    0.180296241564687154310619200000e-02,
];

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn horner_hi(cs: &[f64], x: Ext80, cw: u16) -> Ext80 {
    let mut acc = ef(0.0);
    for &c in cs.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &x, cw), &ef(c), cw);
    }
    acc
}
fn spill(x: Ext80, cw: u16) -> Ext80 {
    ef(ext_to_f64(&x, cw))
}

fn cody_loop(y: f64, cw: u16, every53: bool) -> f64 {
    let ye = ef(y);
    let mut xnum = ext_mul(&ef(C[8]), &ye, cw);
    let mut xden = ye;
    if every53 {
        xnum = spill(xnum, cw);
        xden = spill(xden, cw);
    }
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C[i]), cw), &ye, cw);
        xden = ext_mul(&ext_add(&xden, &ef(D[i]), cw), &ye, cw);
        if every53 {
            xnum = spill(xnum, cw);
            xden = spill(xden, cw);
        }
    }
    ext_to_f64(
        &ext_div(&ext_add(&xnum, &ef(C[7]), cw), &ext_add(&xden, &ef(D[7]), cw), cw),
        cw,
    )
}
fn cody_dist(y: f64, cw: u16) -> f64 {
    let ye = ef(y);
    let mut xnum = ext_mul(&ef(C[8]), &ye, cw);
    let mut xden = ye;
    for i in 0..7 {
        let ci = ext_mul(&ef(C[i]), &ye, cw);
        let di = ext_mul(&ef(D[i]), &ye, cw);
        xnum = ext_add(&ext_mul(&xnum, &ye, cw), &ci, cw);
        xden = ext_add(&ext_mul(&xden, &ye, cw), &di, cw);
    }
    ext_to_f64(
        &ext_div(&ext_add(&xnum, &ef(C[7]), cw), &ext_add(&xden, &ef(D[7]), cw), cw),
        cw,
    )
}
fn cody_two_horner(y: f64, cw: u16) -> f64 {
    let ye = ef(y);
    let num = horner_hi(&C, ye, cw);
    let den = ext_add(&horner_hi(&D, ye, cw), &ye, cw);
    ext_to_f64(&ext_div(&num, &den, cw), cw)
}
fn cody_native(y: f64) -> f64 {
    let mut xnum = C[8] * y;
    let mut xden = y;
    for i in 0..7 {
        xnum = (xnum + C[i]) * y;
        xden = (xden + D[i]) * y;
    }
    (xnum + C[7]) / (xden + D[7])
}

fn pqr(x: f64, cw: u16, tmode: u8) -> f64 {
    let xe = ef(x);
    let t = match tmode {
        0 => {
            let c = ef(3.75);
            ext_div(&ext_sub(&xe, &c, cw), &ext_add(&xe, &c, cw), cw)
        }
        1 => {
            let c = ext_div(&ef(15.0), &ef(4.0), cw);
            ext_div(&ext_sub(&xe, &c, cw), &ext_add(&xe, &c, cw), cw)
        }
        2 => {
            let c = ext_add(&ef(3.0), &ef(0.75), cw);
            ext_div(&ext_sub(&xe, &c, cw), &ext_add(&xe, &c, cw), cw)
        }
        3 => {
            let den = ext_add(&xe, &ef(3.75), cw);
            ext_sub(&ef(1.0), &ext_div(&ef(7.5), &den, cw), cw)
        }
        _ => unreachable!(),
    };
    let mut acc = ext_div(&horner_hi(&PQR_P, xe, cw), &horner_hi(&PQR_Q, xe, cw), cw);
    for &ri in PQR_R.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, cw), &ef(ri), cw);
    }
    ext_to_f64(&acc, cw)
}

fn small_erf(z: f64, cw: u16) -> f64 {
    let xe = ef(z);
    let t = ext_mul(&xe, &xe, cw);
    let w = horner_hi(&AS, t, cw);
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &w, cw), cw), cw)
}
fn small_erfc_f(z: f64, cw: u16) -> f64 {
    let w = f::w_rn53(z);
    if w == 0.0 {
        return f64::NAN;
    }
    let erf = small_erf(z, cw);
    let erfc = ext_to_f64(
        &ext_add(&ef(0.5), &ext_sub(&ef(0.5), &ef(erf), cw), cw),
        cw,
    );
    erfc / w
}
fn cody_ab_erf(z: f64, cw: u16) -> f64 {
    let ye = ef(z);
    let ysq = ext_mul(&ye, &ye, cw);
    let mut xnum = ext_mul(&ef(AA[4]), &ysq, cw);
    let mut xden = ysq;
    for i in 0..3 {
        xnum = ext_mul(&ext_add(&xnum, &ef(AA[i]), cw), &ysq, cw);
        xden = ext_mul(&ext_add(&xden, &ef(BB[i]), cw), &ysq, cw);
    }
    ext_to_f64(
        &ext_div(
            &ext_mul(&ye, &ext_add(&xnum, &ef(AA[3]), cw), cw),
            &ext_add(&xden, &ef(BB[3]), cw),
            cw,
        ),
        cw,
    )
}

#[derive(Default, Clone, Copy)]
struct Acc {
    exact: usize,
    n: usize,
    max_ulp: u64,
    sum_ulp: u128,
    dmid: usize,
}
impl Acc {
    fn add(&mut self, d: u64, direct: bool) {
        self.n += 1;
        if d == 0 {
            self.exact += 1;
            if direct {
                self.dmid += 1;
            }
        } else {
            self.max_ulp = self.max_ulp.max(d);
            self.sum_ulp += d as u128;
        }
    }
}
fn fmt(a: &Acc) -> String {
    format!(
        "{}/{} max={} sum={} dmid={}",
        a.exact, a.n, a.max_ulp, a.sum_ulp, a.dmid
    )
}

fn score_f(rows: &[f::QRow], eval: impl Fn(f64) -> f64) -> (Acc, Acc) {
    let mut mid = Acc::default();
    let mut tail = Acc::default();
    for r in rows {
        if r.z < 0.5 {
            continue;
        }
        let Some(fo) = f::f_or(r.z, r.qbits) else {
            continue;
        };
        let fg = eval(r.z);
        if !fg.is_finite() {
            continue;
        }
        let d = ulp_distance(fg, fo).unwrap_or(u64::MAX);
        if d > ULP_CAP {
            continue;
        }
        if r.z < 4.0 {
            mid.add(d, r.direct);
        } else {
            tail.add(d, r.direct);
        }
    }
    (mid, tail)
}

fn load_erf(dir: &str) -> Vec<(f64, u64)> {
    const BANKS: [&str; 7] = [
        "answers-b9train.json",
        "answers-erfp.json",
        "answers-erfm.json",
        "answers-b8erf.json",
        "answers-b7erf.json",
        "answers-b11.json",
        "answers-b10.json",
    ];
    let mut rows = BTreeMap::new();
    for name in BANKS {
        let path = format!("{dir}/{name}");
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let bank: WitnessSet = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{path}: {e}"));
        for w in &bank.witnesses {
            let x = match &w.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).expect("x"),
                _ => continue,
            };
            let Some(e) = parse_bits_hex(&w.expected_bits) else {
                continue;
            };
            if x > 0.0 && x.is_finite() {
                rows.entry(x.to_bits()).or_insert(e.to_bits());
            }
        }
    }
    rows.into_iter()
        .map(|(b, e)| (f64::from_bits(b), e))
        .collect()
}

fn score_erf(rows: &[(f64, u64)], lo: f64, hi: f64, eval: impl Fn(f64) -> f64) -> Acc {
    let mut a = Acc::default();
    for &(z, bits) in rows {
        if z < lo || z >= hi {
            continue;
        }
        let or = f64::from_bits(bits);
        let g = eval(z);
        if !g.is_finite() {
            continue;
        }
        let d = ulp_distance(g, or).unwrap_or(u64::MAX);
        if d > ULP_CAP {
            continue;
        }
        a.add(d, true);
    }
    a
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let q = f::load_q_rows_tagged(&dir);
    println!("## Cody C/D eval-model (F_or mid z in [0.5,4))");
    for (name, ev) in [
        ("PC64 SPECFUN loop", (|y| cody_loop(y, CW_PC64_RN, false)) as fn(f64) -> f64),
        ("PC64 every-op store53", |y| cody_loop(y, CW_PC64_RN, true)),
        ("PC53 SPECFUN loop", |y| cody_loop(y, CW_PC53_RN, false)),
        ("PC53 every-op store", |y| cody_loop(y, CW_PC53_RN, true)),
        ("PC64 distributed xnum*y+C*y", |y| cody_dist(y, CW_PC64_RN)),
        ("PC64 two-Horner C / (Horner D + y)", |y| cody_two_horner(y, CW_PC64_RN)),
        ("native f64 SPECFUN loop", cody_native),
    ] {
        let (m, t) = score_f(&q, ev);
        println!("{name:40} mid {} tail {}", fmt(&m), fmt(&t));
    }

    println!("\n## NSWC PQR t-map (x87 PC64), F on z<=2 else nswc_derfc0");
    for (name, mode) in [
        ("t=(x-3.75)/(x+3.75)", 0u8),
        ("t=(x-15/4)/(x+15/4) int", 1),
        ("t=(x-(3+0.75))/(x+(3+0.75))", 2),
        ("t=1-7.5/(x+3.75)", 3),
    ] {
        let (m, t) = score_f(&q, |z| {
            if z <= 2.0 {
                pqr(z, CW_PC64_RN, mode)
            } else {
                f::nswc_derfc0(z)
            }
        });
        println!("{name:40} mid {} tail {}", fmt(&m), fmt(&t));
    }

    println!("\n## small-series eval-model + Cody PC64, cut=1");
    for (name, sm) in [
        ("small PC64 + Cody PC64", (|z| small_erfc_f(z, CW_PC64_RN)) as fn(f64) -> f64),
        ("small PC53 + Cody PC64", |z| small_erfc_f(z, CW_PC53_RN)),
        ("small PC53 + Cody PC53", |z| small_erfc_f(z, CW_PC53_RN)),
    ] {
        let (m, t) = score_f(&q, |z| {
            if z < 1.0 {
                sm(z)
            } else if matches!(name, "small PC53 + Cody PC53") {
                cody_loop(z, CW_PC53_RN, false)
            } else {
                cody_loop(z, CW_PC64_RN, false)
            }
        });
        println!("{name:40} mid {} tail {}", fmt(&m), fmt(&t));
    }

    let erf = load_erf(&dir);
    let n05 = erf.iter().filter(|(z, _)| *z < 0.5).count();
    println!("\n## ERF.PRECISE P-side ({} rows z<0.5, {} all +ve)", n05, erf.len());
    for (name, ev) in [
        ("NSWC A21 erf PC64", (|z| small_erf(z, CW_PC64_RN)) as fn(f64) -> f64),
        ("NSWC A21 erf PC53", |z| small_erf(z, CW_PC53_RN)),
        ("NSWC A21 erf native", |z| {
            let mut w = 0.0;
            let t = z * z;
            for &c in AS.iter().rev() {
                w = w * t + c;
            }
            z * (1.0 + w)
        }),
        ("Cody A/B erf PC64", |z| cody_ab_erf(z, CW_PC64_RN)),
        ("Cody A/B erf PC53", |z| cody_ab_erf(z, CW_PC53_RN)),
        ("libm erf", libm::erf),
    ] {
        let a = score_erf(&erf, 0.0, 0.5, ev);
        let b = score_erf(&erf, 0.0, 0.46875, ev);
        println!("{name:28} z<0.5 {}  z<0.46875 {}", fmt(&a), fmt(&b));
    }
}
