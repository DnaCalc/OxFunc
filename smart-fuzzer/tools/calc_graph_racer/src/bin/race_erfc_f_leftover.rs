//! Leftover-row inverse: which named F is exact on x87-NSWC-1.5/4 misses,
//! t-formula histogram, AABB algebraic variants vs pin z=2.125.
//! Heldouts unnamed. Does not land kernels.
//!
//!   cargo run --release --bin race_erfc_f_leftover -- ../../work/w109/G3-01-dist [out-dir]

use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

const P: [f64; 8] = [
    0.16506148041280876191828601e-03,
    0.15471455377139313353998665e-03,
    0.44852548090298868465196794e-04,
    -0.49177280017226285450486205e-05,
    -0.69353602078656412367801676e-05,
    -0.20508667787746282746857743e-05,
    -0.28982842617824971177267380e-06,
    -0.17272433544836633301127174e-07,
];
const Q: [f64; 8] = [
    1.0,
    0.16272656776533322859856317e+01,
    0.12040996037066026106794322e+01,
    0.52400246352158386907601472e+00,
    0.14497345252798672362384241e+00,
    0.25592517111042546492590736e-01,
    0.26869088293991371028123158e-02,
    0.13133767840925681614496481e-03,
];
const R: [f64; 9] = [
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
const AA: [f64; 9] = [
    -0.45894433406309678202825375e-03,
    -0.12281298722544724287816236e-01,
    -0.91144359512342900801764781e-01,
    -0.28412489223839285652511367e-01,
    0.14083827189977123530129812e+01,
    0.11532175281537044570477189e+01,
    -0.72170903389442152112483632e+01,
    -0.19685597805218214001309225e+01,
    0.93846891504541841150916038e+01,
];
const BB: [f64; 12] = [
    1.0,
    0.25136329960926527692263725e+02,
    0.15349442087145759184067981e+03,
    -0.29971215958498680905476402e+03,
    -0.33876477506888115226730368e+04,
    0.28301829314924804988873701e+04,
    0.22979620942196507068034887e+05,
    -0.24280681522998071562462041e+05,
    -0.36680620673264731899504580e+05,
    0.42278731622295627627042436e+05,
    0.28834257644413614344549790e+03,
    0.70226293775648358646587341e+03,
];
const E0: f64 = 0.540464821348814822409610122136;
const E1: f64 = -0.261515522487415653487049835220e-01;
const E2: f64 = -0.288573438386338758794591212600e-02;

fn x87_horner(cs: &[f64], x: Ext80) -> Ext80 {
    let mut acc = ext_from_f64(0.0);
    for &c in cs.iter().rev() {
        acc = ext_add(
            &ext_mul(&acc, &x, CW_PC64_RN),
            &ext_from_f64(c),
            CW_PC64_RN,
        );
    }
    acc
}

fn pqr_x87(x: f64, t_mode: u8) -> f64 {
    let xe = ext_from_f64(x);
    let t = match t_mode {
        1 => {
            let xp = ext_add(&xe, &ext_from_f64(3.75), CW_PC64_RN);
            ext_sub(
                &ext_from_f64(1.0),
                &ext_div(&ext_from_f64(7.5), &xp, CW_PC64_RN),
                CW_PC64_RN,
            )
        }
        2 => {
            let r = ext_div(&ext_from_f64(3.75), &xe, CW_PC64_RN);
            ext_div(
                &ext_sub(&ext_from_f64(1.0), &r, CW_PC64_RN),
                &ext_add(&ext_from_f64(1.0), &r, CW_PC64_RN),
                CW_PC64_RN,
            )
        }
        3 => {
            let u = ext_div(&xe, &ext_from_f64(3.75), CW_PC64_RN);
            ext_div(
                &ext_sub(&u, &ext_from_f64(1.0), CW_PC64_RN),
                &ext_add(&u, &ext_from_f64(1.0), CW_PC64_RN),
                CW_PC64_RN,
            )
        }
        _ => ext_div(
            &ext_sub(&xe, &ext_from_f64(3.75), CW_PC64_RN),
            &ext_add(&xe, &ext_from_f64(3.75), CW_PC64_RN),
            CW_PC64_RN,
        ),
    };
    let mut acc = ext_div(&x87_horner(&P, xe), &x87_horner(&Q, xe), CW_PC64_RN);
    for &r in R.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW_PC64_RN), &ext_from_f64(r), CW_PC64_RN);
    }
    ext_to_f64(&acc, CW_PC64_RN)
}

fn aabb_x87_var(x: f64, how: u8) -> f64 {
    let xe = ext_from_f64(x);
    let z = match how {
        1 => {
            // 1 / (2.5 + x*x) with native square
            ext_div(
                &ext_from_f64(1.0),
                &ext_from_f64(2.5 + x * x),
                CW_PC64_RN,
            )
        }
        2 => {
            // 1/2.5 * 1/(1 + (x*x)/2.5)
            let zz = ext_mul(&xe, &xe, CW_PC64_RN);
            let u = ext_div(&zz, &ext_from_f64(2.5), CW_PC64_RN);
            ext_mul(
                &ext_from_f64(0.4),
                &ext_div(
                    &ext_from_f64(1.0),
                    &ext_add(&ext_from_f64(1.0), &u, CW_PC64_RN),
                    CW_PC64_RN,
                ),
                CW_PC64_RN,
            )
        }
        _ => {
            let zz = ext_mul(&xe, &xe, CW_PC64_RN);
            ext_div(
                &ext_from_f64(1.0),
                &ext_add(&ext_from_f64(2.5), &zz, CW_PC64_RN),
                CW_PC64_RN,
            )
        }
    };
    let t = match how {
        3 => ext_sub(
            &ext_from_f64(13.0 * ext_to_f64(&z, CW_PC64_RN)),
            &ext_from_f64(1.0),
            CW_PC64_RN,
        ),
        4 => ext_from_f64(13.0 * ext_to_f64(&z, CW_PC64_RN) - 1.0),
        _ => ext_sub(
            &ext_mul(&ext_from_f64(13.0), &z, CW_PC64_RN),
            &ext_from_f64(1.0),
            CW_PC64_RN,
        ),
    };
    let r = ext_div(&x87_horner(&AA, z), &x87_horner(&BB, z), CW_PC64_RN);
    let mut acc = ext_add(&ext_mul(&r, &t, CW_PC64_RN), &ext_from_f64(E2), CW_PC64_RN);
    acc = ext_add(&ext_mul(&acc, &t, CW_PC64_RN), &ext_from_f64(E1), CW_PC64_RN);
    acc = ext_add(&ext_mul(&acc, &t, CW_PC64_RN), &ext_from_f64(E0), CW_PC64_RN);
    ext_to_f64(&ext_div(&acc, &xe, CW_PC64_RN), CW_PC64_RN)
}

fn graph(z: f64) -> f64 {
    if z < 1.5 {
        pqr_x87(z, 0)
    } else if z < 4.0 {
        aabb_x87_var(z, 0)
    } else {
        f::nswc_ccdd_f(z)
    }
}

fn d_of(eval: impl Fn(f64) -> f64, z: f64, fo: f64) -> u64 {
    ulp_distance(eval(z), fo).unwrap_or(u64::MAX)
}

fn main() {
    let dir = std::env::args()
        .nth(1)
        .expect("G3-01-dist (heldout files are not named)");
    assert!(!dir.contains("heldout"));
    let out = PathBuf::from(
        std::env::args()
            .nth(2)
            .unwrap_or_else(|| "../../work/w109/erfc-f-leftover".into()),
    );
    fs::create_dir_all(&out).unwrap();
    let tagged = f::load_q_rows_tagged(&dir);
    let rows: Vec<(f64, u64)> = tagged.iter().map(|r| (r.z, r.qbits)).collect();

    let named: [(&str, fn(f64) -> f64); 6] = [
        ("nswc_native", f::nswc_derfc0),
        ("cody", f::cody_erfcx_f),
        ("cephes", f::cephes_f),
        ("cdflib", f::cdflib_erfc1_f),
        ("cf_as714", f::cf_as714_f),
        ("cf_gaut", f::cf_gautschi_f),
    ];

    println!("## leftover mid vs named packets (graph = x87 PQR@1.5/AABB@4)");
    let mut win = [0usize; 7];
    let mut none = 0usize;
    let mut wf = fs::File::create(out.join("leftover-winner.tsv")).unwrap();
    writeln!(wf, "z\tdirect\tulp_graph\twinner\twin_ulp").unwrap();
    let mut leftover_pqr = Vec::new();
    for r in &tagged {
        if r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let Some(fo) = f::f_or(r.z, r.qbits) else {
            continue;
        };
        let dg = d_of(graph, r.z, fo);
        if dg == 0 {
            continue;
        }
        if r.z < 1.5 {
            leftover_pqr.push((r.z, r.qbits, r.direct, fo));
        }
        let mut best = ("none", u64::MAX);
        for (i, (name, eval)) in named.iter().enumerate() {
            let d = d_of(*eval, r.z, fo);
            if d < best.1 {
                best = (*name, d);
            }
            if d == 0 {
                win[i] += 1;
            }
        }
        if best.1 != 0 {
            none += 1;
        }
        writeln!(
            wf,
            "{:.16e}\t{}\t{dg}\t{}\t{}",
            r.z,
            r.direct as u8,
            best.0,
            best.1
        )
        .unwrap();
    }
    for (i, (name, _)) in named.iter().enumerate() {
        println!("  leftover rows exact on {name}: {}", win[i]);
    }
    println!("  leftover with no named exact: {none}");

    println!("\n## t-formula on leftover PQR [0.5,1.5): exact counts + per-row winner");
    let mut hist = [0usize; 5];
    let mut n_multi = 0usize;
    let mut n_none = 0usize;
    for &(z, _, _, fo) in &leftover_pqr {
        let mut hits = 0u8;
        let mut first = 4usize;
        for tm in 0u8..4 {
            if d_of(|x| pqr_x87(x, tm), z, fo) == 0 {
                hits += 1;
                if first == 4 {
                    first = tm as usize;
                }
            }
        }
        if hits == 0 {
            n_none += 1;
            hist[4] += 1;
        } else {
            hist[first] += 1;
            if hits > 1 {
                n_multi += 1;
            }
        }
    }
    println!(
        "  leftover PQR n={}  t0={} t1={} t2={} t3={} none={} multi-hit={}",
        leftover_pqr.len(),
        hist[0],
        hist[1],
        hist[2],
        hist[3],
        n_none,
        n_multi
    );

    println!("\n## AABB algebraic variants (pin z=2.125 + mid band)");
    let pin = 2.125;
    let Some(pr) = tagged.iter().find(|r| r.z == pin) else {
        panic!("pin 2.125 missing");
    };
    let fo_pin = f::f_or(pin, pr.qbits).unwrap();
    let aabb_band: Vec<(f64, u64)> = rows
        .iter()
        .copied()
        .filter(|(z, _)| *z >= 1.5 && *z < 4.0)
        .collect();
    for how in 0u8..=4 {
        let dp = d_of(|x| aabb_x87_var(x, how), pin, fo_pin);
        let (m, _) = f::score_f(&aabb_band, |z| aabb_x87_var(z, how));
        println!(
            "  how={how} pin_ulp={dp} AABB-band {}",
            f::fmt_acc(&m)
        );
    }

    println!("\n## t-mode as a piecewise on PQR band (global t_mode exact)");
    let pqr_band: Vec<(f64, u64)> = rows
        .iter()
        .copied()
        .filter(|(z, _)| *z >= 0.5 && *z < 1.5)
        .collect();
    for tm in 0u8..4 {
        let (m, _) = f::score_f(&pqr_band, |z| pqr_x87(z, tm));
        println!("  t_mode={tm} PQR-band {}", f::fmt_acc(&m));
    }
    println!("artifacts in {}", out.display());
}
