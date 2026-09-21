//! Hamming-2 ±1 ULP on Cody C/D, holding NSWC small-series z<1.
//! Joint recovery on the 3008 skeleton. Not an identity.
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
const C0: [f64; 9] = [
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
const D0: [f64; 8] = [
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
fn small_f(z: f64) -> f64 {
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
fn cody_cd(y: f64, c: &[f64; 9], d: &[f64; 8]) -> f64 {
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
fn poke(x: f64, k: i32) -> f64 {
    let mut v = x;
    if k > 0 {
        for _ in 0..k {
            v = v.next_up();
        }
    } else {
        for _ in 0..(-k) {
            v = v.next_down();
        }
    }
    v
}

struct Mid {
    exact: usize,
    sum: u128,
    dmid: usize,
}
fn score_mid(rows: &[f::QRow], c: &[f64; 9], d: &[f64; 8]) -> Mid {
    let mut exact = 0usize;
    let mut sum = 0u128;
    let mut dmid = 0usize;
    for r in rows {
        if r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let Some(fo) = f::f_or(r.z, r.qbits) else {
            continue;
        };
        let fg = if r.z < 1.0 {
            small_f(r.z)
        } else {
            cody_cd(r.z, c, d)
        };
        if !fg.is_finite() {
            continue;
        }
        let dist = ulp_distance(fg, fo).unwrap_or(u64::MAX);
        if dist > ULP_CAP {
            continue;
        }
        if dist == 0 {
            exact += 1;
            if r.direct {
                dmid += 1;
            }
        } else {
            sum += dist as u128;
        }
    }
    Mid { exact, sum, dmid }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mut c = C0;
    let mut d = D0;
    let base = score_mid(&rows, &c, &d);
    println!("base cut=1 CR mid {} sum {} dmid {}", base.exact, base.sum, base.dmid);
    let mut best = base.exact;
    let mut best_sum = base.sum;
    let mut best_d = base.dmid;
    let mut best_lab = "CR".to_string();
    let ks = [-1i32, 1];
    let n = 9 + 8;
    let get = |c: &[f64; 9], d: &[f64; 8], i: usize| -> f64 {
        if i < 9 { c[i] } else { d[i - 9] }
    };
    let set = |c: &mut [f64; 9], d: &mut [f64; 8], i: usize, v: f64| {
        if i < 9 {
            c[i] = v;
        } else {
            d[i - 9] = v;
        }
    };
    let mut tried = 0u32;
    for i in 0..n {
        for j in (i + 1)..n {
            for &ki in &ks {
                for &kj in &ks {
                    let oi = get(&c, &d, i);
                    let oj = get(&c, &d, j);
                    set(&mut c, &mut d, i, poke(oi, ki));
                    set(&mut c, &mut d, j, poke(oj, kj));
                    let sc = score_mid(&rows, &c, &d);
                    tried += 1;
                    if sc.exact > best || (sc.exact == best && sc.sum < best_sum) {
                        best = sc.exact;
                        best_sum = sc.sum;
                        best_d = sc.dmid;
                        best_lab = format!("i={i}{ki:+} j={j}{kj:+}");
                        println!(
                            "HIT {best_lab} mid {} sum {} dmid {}",
                            sc.exact, sc.sum, sc.dmid
                        );
                    }
                    set(&mut c, &mut d, i, oi);
                    set(&mut c, &mut d, j, oj);
                }
            }
        }
    }
    println!("tried={tried} best={best_lab} mid={best} sum={best_sum} dmid={best_d}");
}
