//! Pin-first ±1 ULP on AABB published constants (E0/E1/E2, 2.5, 13) and
//! PQR 3.75, under x87 F_or. Heldouts unnamed. No landing.
//!
//!   cargo run --release --bin race_erfc_f_aabb_ulp -- ../../work/w109/G3-01-dist

use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};

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

#[derive(Clone, Copy)]
struct C {
    e0: f64,
    e1: f64,
    e2: f64,
    two5: f64,
    thirteen: f64,
    c375: f64,
}

fn pqr(x: f64, c: &C) -> f64 {
    let xe = ext_from_f64(x);
    let k = ext_from_f64(c.c375);
    let t = ext_div(
        &ext_sub(&xe, &k, CW_PC64_RN),
        &ext_add(&xe, &k, CW_PC64_RN),
        CW_PC64_RN,
    );
    let mut acc = ext_div(&x87_horner(&P, xe), &x87_horner(&Q, xe), CW_PC64_RN);
    for &r in R.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW_PC64_RN), &ext_from_f64(r), CW_PC64_RN);
    }
    ext_to_f64(&acc, CW_PC64_RN)
}

fn aabb(x: f64, c: &C) -> f64 {
    let xe = ext_from_f64(x);
    let zz = ext_mul(&xe, &xe, CW_PC64_RN);
    let z = ext_div(
        &ext_from_f64(1.0),
        &ext_add(&ext_from_f64(c.two5), &zz, CW_PC64_RN),
        CW_PC64_RN,
    );
    let t = ext_sub(
        &ext_mul(&ext_from_f64(c.thirteen), &z, CW_PC64_RN),
        &ext_from_f64(1.0),
        CW_PC64_RN,
    );
    let r = ext_div(&x87_horner(&AA, z), &x87_horner(&BB, z), CW_PC64_RN);
    let mut acc = ext_add(&ext_mul(&r, &t, CW_PC64_RN), &ext_from_f64(c.e2), CW_PC64_RN);
    acc = ext_add(&ext_mul(&acc, &t, CW_PC64_RN), &ext_from_f64(c.e1), CW_PC64_RN);
    acc = ext_add(&ext_mul(&acc, &t, CW_PC64_RN), &ext_from_f64(c.e0), CW_PC64_RN);
    ext_to_f64(&ext_div(&acc, &xe, CW_PC64_RN), CW_PC64_RN)
}

fn eval(x: f64, c: &C) -> f64 {
    if x < 1.5 {
        pqr(x, c)
    } else if x < 4.0 {
        aabb(x, c)
    } else {
        f::nswc_ccdd_f(x)
    }
}

fn poke(x: f64, s: i32) -> f64 {
    if s > 0 {
        x.next_up()
    } else {
        x.next_down()
    }
}

fn main() {
    let dir = std::env::args()
        .nth(1)
        .expect("G3-01-dist (heldout files are not named)");
    assert!(!dir.contains("heldout"));
    let tagged = f::load_q_rows_tagged(&dir);
    let direct: Vec<(f64, u64)> = tagged
        .iter()
        .filter(|r| r.direct)
        .map(|r| (r.z, r.qbits))
        .collect();
    let base = C {
        e0: 0.540464821348814822409610122136,
        e1: -0.261515522487415653487049835220e-01,
        e2: -0.288573438386338758794591212600e-02,
        two5: 2.5,
        thirteen: 13.0,
        c375: 3.75,
    };
    let pins = f::PIN_Z;
    let pin_fo: Vec<(f64, f64)> = pins
        .iter()
        .filter_map(|&z| {
            let r = tagged.iter().find(|rr| rr.z == z)?;
            f::f_or(z, r.qbits).map(|fo| (z, fo))
        })
        .collect();

    let pin_ulp = |c: &C| -> Vec<(f64, u64)> {
        pin_fo
            .iter()
            .map(|&(z, fo)| (z, ulp_distance(eval(z, c), fo).unwrap_or(u64::MAX)))
            .collect()
    };

    println!("baseline pins {:?}", pin_ulp(&base));
    let (bm, bt) = f::score_f(&direct, |z| eval(z, &base));
    println!("baseline direct mid {} tail {}", f::fmt_acc(&bm), f::fmt_acc(&bt));

    let names = ["E0", "E1", "E2", "2.5", "13", "3.75"];
    for (i, name) in names.iter().enumerate() {
        for &s in &[-1i32, 1] {
            let mut c = C { ..base };
            match i {
                0 => c.e0 = poke(base.e0, s),
                1 => c.e1 = poke(base.e1, s),
                2 => c.e2 = poke(base.e2, s),
                3 => c.two5 = poke(base.two5, s),
                4 => c.thirteen = poke(base.thirteen, s),
                _ => c.c375 = poke(base.c375, s),
            }
            let pu = pin_ulp(&c);
            let pin_improved = pu.iter().any(|&(z, d)| {
                let b = pin_ulp(&base)
                    .iter()
                    .find(|&&(zz, _)| zz == z)
                    .map(|p| p.1)
                    .unwrap_or(u64::MAX);
                d < b
            });
            let pin_hit = pu.iter().any(|&(_, d)| d == 0);
            if pin_improved || pin_hit {
                let (m, t) = f::score_f(&direct, |z| eval(z, &c));
                println!(
                    "  {name} {s:+} pins {pu:?}  direct mid {} tail {}",
                    f::fmt_acc(&m),
                    f::fmt_acc(&t)
                );
            } else {
                println!("  {name} {s:+} pins {pu:?}  (no pin improvement)");
            }
        }
    }
}
