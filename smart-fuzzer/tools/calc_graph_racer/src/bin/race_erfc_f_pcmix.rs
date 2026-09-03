//! Mixed PC53/PC64 on unspilled NSWC PQR (same family, different op rounding).
//! CDFLIB/Cephes x87 as family controls. Heldouts unnamed. No landing.
//!
//!   cargo run --release --bin race_erfc_f_pcmix -- ../../work/w109/G3-01-dist

use calc_graph_racer::erfc_f_packets as f;
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC53_RN, CW_PC64_RN,
};

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

fn horner(cs: &[f64], x: Ext80, cw: u16) -> Ext80 {
    let mut acc = ext_from_f64(0.0);
    for &c in cs.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &x, cw), &ext_from_f64(c), cw);
    }
    acc
}

fn aabb64(x: f64) -> f64 {
    let xe = ext_from_f64(x);
    let zz = ext_mul(&xe, &xe, CW_PC64_RN);
    let z = ext_div(
        &ext_from_f64(1.0),
        &ext_add(&ext_from_f64(2.5), &zz, CW_PC64_RN),
        CW_PC64_RN,
    );
    let t = ext_sub(
        &ext_mul(&ext_from_f64(13.0), &z, CW_PC64_RN),
        &ext_from_f64(1.0),
        CW_PC64_RN,
    );
    let r = ext_div(&horner(&AA, z, CW_PC64_RN), &horner(&BB, z, CW_PC64_RN), CW_PC64_RN);
    let mut acc = ext_add(&ext_mul(&r, &t, CW_PC64_RN), &ext_from_f64(E2), CW_PC64_RN);
    acc = ext_add(&ext_mul(&acc, &t, CW_PC64_RN), &ext_from_f64(E1), CW_PC64_RN);
    acc = ext_add(&ext_mul(&acc, &t, CW_PC64_RN), &ext_from_f64(E0), CW_PC64_RN);
    ext_to_f64(&ext_div(&acc, &xe, CW_PC64_RN), CW_PC64_RN)
}

fn t_of(xe: &Ext80, cw: u16) -> Ext80 {
    ext_div(
        &ext_sub(xe, &ext_from_f64(3.75), cw),
        &ext_add(xe, &ext_from_f64(3.75), cw),
        cw,
    )
}

fn pqr_mix(x: f64, cw_t: u16, cw_pq: u16, cw_uv: u16, cw_r: u16, rmask: u16) -> f64 {
    let xe = ext_from_f64(x);
    let t = t_of(&xe, cw_t);
    let u = horner(&P, xe, cw_pq);
    let v = horner(&Q, xe, cw_pq);
    let mut acc = ext_div(&u, &v, cw_uv);
    for (i, &r) in R.iter().rev().enumerate() {
        let cw = if rmask & (1 << i) != 0 {
            CW_PC53_RN
        } else {
            cw_r
        };
        acc = ext_add(&ext_mul(&acc, &t, cw), &ext_from_f64(r), cw);
    }
    ext_to_f64(&acc, CW_PC64_RN)
}

fn mid_of(z: f64, pqr: impl Fn(f64) -> f64) -> f64 {
    if z < 1.5 {
        pqr(z)
    } else if z < 4.0 {
        aabb64(z)
    } else {
        f::nswc_ccdd_f(z)
    }
}

fn cdflib_x87(x: f64) -> f64 {
    const CP: [f64; 8] = [
        -1.36864857382717e-7,
        5.64195517478974e-1,
        7.21175825088309e0,
        4.31622272220567e1,
        1.52989285046940e2,
        3.39320816734344e2,
        4.51918953711873e2,
        3.00459261020162e2,
    ];
    const CQ: [f64; 8] = [
        1.0,
        1.27827273196294e1,
        7.70001529352295e1,
        2.77585444743988e2,
        6.38980264465631e2,
        9.31354094850610e2,
        7.90950925327898e2,
        3.00459260956983e2,
    ];
    let xe = ext_from_f64(x);
    ext_to_f64(
        &ext_div(
            &horner(&CP, xe, CW_PC64_RN),
            &horner(&CQ, xe, CW_PC64_RN),
            CW_PC64_RN,
        ),
        CW_PC64_RN,
    )
}

fn cephes_x87(x: f64) -> f64 {
    const CP: [f64; 9] = [
        2.46196981473530512524e-10,
        5.64189564831068821977e-1,
        7.46321056442269912687e0,
        4.86371970985681366614e1,
        1.96520832956077098242e2,
        5.26445194995477358631e2,
        9.34528527171957607540e2,
        1.02755188689515710272e3,
        5.57535335369399327526e2,
    ];
    const CQ: [f64; 8] = [
        1.32281951154744992508e1,
        8.67072140885989742329e1,
        3.54937778887819891062e2,
        9.75708501743205489753e2,
        1.82390916687909736289e3,
        2.24633760818710981792e3,
        1.65666309194161350182e3,
        5.57535340817727675546e2,
    ];
    let xe = ext_from_f64(x);
    let p = horner(&CP, xe, CW_PC64_RN);
    // p1evl: implicit leading 1
    let mut q = xe;
    for &c in &CQ {
        q = ext_add(&ext_mul(&q, &xe, CW_PC64_RN), &ext_from_f64(c), CW_PC64_RN);
    }
    ext_to_f64(&ext_div(&p, &q, CW_PC64_RN), CW_PC64_RN)
}

fn report(label: &str, rows: &[(f64, u64)], eval: impl Fn(f64) -> f64) {
    let (m, t) = f::score_f(rows, eval);
    println!("  {label:<44} mid {}  tail {}", f::fmt_acc(&m), f::fmt_acc(&t));
}

fn main() {
    let dir = std::env::args()
        .nth(1)
        .expect("G3-01-dist (heldout files are not named)");
    assert!(!dir.contains("heldout"));
    let tagged = f::load_q_rows_tagged(&dir);
    let rows: Vec<(f64, u64)> = tagged.iter().map(|r| (r.z, r.qbits)).collect();
    let c64 = CW_PC64_RN;
    let c53 = CW_PC53_RN;

    println!("## named PC mixes, PQR cut 1.5 + AABB64 + native CCDD");
    let mixes: [(&str, u16, u16, u16, u16); 10] = [
        ("all PC64 (bar)", c64, c64, c64, c64),
        ("all PC53", c53, c53, c53, c53),
        ("t PC53, rest PC64", c53, c64, c64, c64),
        ("P/Q PC53, rest PC64", c64, c53, c64, c64),
        ("uv-div PC53, rest PC64", c64, c64, c53, c64),
        ("R PC53, rest PC64", c64, c64, c64, c53),
        ("t+R PC53, P/Q PC64", c53, c64, c64, c53),
        ("P/Q+uv PC53, t+R PC64", c64, c53, c53, c64),
        ("t PC64, everything else PC53", c64, c53, c53, c53),
        ("R PC64, everything else PC53", c53, c53, c53, c64),
    ];
    for (name, ct, cpq, cuv, cr) in mixes {
        report(name, &rows, |z| {
            mid_of(z, |x| pqr_mix(x, ct, cpq, cuv, cr, 0))
        });
    }

    println!("\n## 9-bit R-step PC53 mask (rest PC64), keep if beats bar");
    let mut best = (0u16, 0usize);
    for mask in 0u16..512 {
        let (m, _) = f::score_f(&rows, |z| {
            mid_of(z, |x| pqr_mix(x, c64, c64, c64, c64, mask))
        });
        if m.exact > best.1 {
            best = (mask, m.exact);
            println!("  Rmask=0x{mask:03x} mid {}", f::fmt_acc(&m));
        }
    }
    println!("  best R PC53 mask=0x{:03x} exact={}", best.0, best.1);

    println!("\n## 3-bit t-op PC53 (sub, add, div independently) rest PC64");
    // 0: sub, 1: add, 2: div
    let mut best_t = (0u8, 0usize);
    for mask in 0u8..8 {
        let eval = |x: f64| {
            let xe = ext_from_f64(x);
            let csub = if mask & 1 != 0 { c53 } else { c64 };
            let cadd = if mask & 2 != 0 { c53 } else { c64 };
            let cdiv = if mask & 4 != 0 { c53 } else { c64 };
            let num = ext_sub(&xe, &ext_from_f64(3.75), csub);
            let den = ext_add(&xe, &ext_from_f64(3.75), cadd);
            let t = ext_div(&num, &den, cdiv);
            let mut acc = ext_div(&horner(&P, xe, c64), &horner(&Q, xe, c64), c64);
            for &r in R.iter().rev() {
                acc = ext_add(&ext_mul(&acc, &t, c64), &ext_from_f64(r), c64);
            }
            ext_to_f64(&acc, c64)
        };
        let (m, _) = f::score_f(&rows, |z| mid_of(z, eval));
        if m.exact >= best_t.1 {
            best_t = (mask, m.exact);
            println!("  tmask=0x{mask:x} mid {}", f::fmt_acc(&m));
        }
    }
    println!("  best t-op PC53 mask=0x{:x} exact={}", best_t.0, best_t.1);

    println!("\n## other-family x87 on mid (cut 1.5 still AABB/CCDD for far)");
    report("CDFLIB x87 P/Q all z as mid F", &rows, |z| {
        if z < 4.0 {
            cdflib_x87(z)
        } else {
            f::nswc_ccdd_f(z)
        }
    });
    report("CDFLIB x87 until 1.5 then AABB", &rows, |z| {
        if z < 1.5 {
            cdflib_x87(z)
        } else if z < 4.0 {
            aabb64(z)
        } else {
            f::nswc_ccdd_f(z)
        }
    });
    report("Cephes x87 P/Q [0.5,8) + CCDD", &rows, |z| {
        if z < 8.0 {
            cephes_x87(z)
        } else {
            f::nswc_ccdd_f(z)
        }
    });
    report("Cody x87 all mid + CCDD", &rows, |z| {
        if z < 4.0 {
            // reuse packet native cody for the >4 branch inside; x87 Cody is
            // the published C/D Horner — packet already has cody_erfcx_f.
            f::cody_erfcx_f(z)
        } else {
            f::nswc_ccdd_f(z)
        }
    });
}
