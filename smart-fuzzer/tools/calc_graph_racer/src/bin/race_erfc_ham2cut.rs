//! Ham2 winner C[6]+1 D[4]+1 at cuts; small-series 1-erf vs 0.5+(0.5-erf).
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
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

fn x87_horner(cs: &[f64], x: Ext80) -> Ext80 {
    let mut acc = ext_from_f64(0.0);
    for &c in cs.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &x, CW), &ext_from_f64(c), CW);
    }
    acc
}
fn horner(cs: &[f64], x: f64) -> f64 {
    let mut a = 0.0;
    for &c in cs.iter().rev() {
        a = a * x + c;
    }
    a
}
fn small_x87_cancel(z: f64) -> f64 {
    let w = f::w_rn53(z);
    if w == 0.0 {
        return f64::NAN;
    }
    let xe = ext_from_f64(z);
    let t = ext_mul(&xe, &xe, CW);
    let ww = x87_horner(&AS, t);
    let inner = ext_mul(&xe, &ext_add(&ext_from_f64(1.0), &ww, CW), CW);
    ext_to_f64(
        &ext_add(
            &ext_from_f64(0.5),
            &ext_sub(&ext_from_f64(0.5), &inner, CW),
            CW,
        ),
        CW,
    ) / w
}
fn small_x87_one_minus(z: f64) -> f64 {
    let w = f::w_rn53(z);
    if w == 0.0 {
        return f64::NAN;
    }
    let xe = ext_from_f64(z);
    let t = ext_mul(&xe, &xe, CW);
    let ww = x87_horner(&AS, t);
    let erf = ext_mul(&xe, &ext_add(&ext_from_f64(1.0), &ww, CW), CW);
    ext_to_f64(&ext_sub(&ext_from_f64(1.0), &erf, CW), CW) / w
}
fn small_native_cancel(z: f64) -> f64 {
    let w = f::w_rn53(z);
    if w == 0.0 {
        return f64::NAN;
    }
    let ww = horner(&AS, z * z);
    (0.5 + (0.5 - z * (1.0 + ww))) / w
}
fn cody_cd(y: f64, c6: f64, d4: f64) -> f64 {
    let mut c = C;
    let mut d = D;
    c[6] = c6;
    d[4] = d4;
    let ye = ext_from_f64(y);
    let mut xnum = ext_mul(&ext_from_f64(c[8]), &ye, CW);
    let mut xden = ye;
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ext_from_f64(c[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ext_from_f64(d[i]), CW), &ye, CW);
    }
    ext_to_f64(
        &ext_div(
            &ext_add(&xnum, &ext_from_f64(c[7]), CW),
            &ext_add(&xden, &ext_from_f64(d[7]), CW),
            CW,
        ),
        CW,
    )
}

fn score(rows: &[f::QRow], eval: impl Fn(f64) -> f64) -> (usize, u128, usize) {
    let mut ex = 0usize;
    let mut sum = 0u128;
    let mut dm = 0usize;
    for r in rows {
        if r.z < 0.5 || r.z >= 4.0 {
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
        if d == 0 {
            ex += 1;
            if r.direct {
                dm += 1;
            }
        } else {
            sum += d as u128;
        }
    }
    (ex, sum, dm)
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let c6p = C[6].next_up();
    let d4p = D[4].next_up();
    println!("## association at cut=1 CR Cody");
    for (name, sm) in [
        ("x87 0.5+(0.5-erf)", small_x87_cancel as fn(f64) -> f64),
        ("x87 1-erf", small_x87_one_minus),
        ("native 0.5+(0.5-erf)", small_native_cancel),
    ] {
        let (ex, sum, dm) = score(&rows, |z| {
            if z < 1.0 {
                sm(z)
            } else {
                cody_cd(z, C[6], D[4])
            }
        });
        println!("{name:24} mid {ex} sum {sum} dmid {dm}");
    }
    println!("## ham2 C6+1 D4+1 at cuts");
    for cut in [1.0, 1.32, 1.347, 1.35, 1.36] {
        let (ex, sum, dm) = score(&rows, |z| {
            if z < cut {
                small_x87_cancel(z)
            } else {
                cody_cd(z, c6p, d4p)
            }
        });
        println!("cut={cut:.3} ham2 mid {ex} sum {sum} dmid {dm}");
    }
}
