//! Direct ERFC.PRECISE only: x87 Cody/Cephes/CDFLIB, piecewise Cody vs
//! x87-NSWC, dump the 63 hard misses. Heldouts unnamed. No landing.
//!
//!   cargo run --release --bin race_erfc_f_direct -- ../../work/w109/G3-01-dist [out-dir]

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
fn x87_polevl(x: Ext80, coef: &[f64]) -> Ext80 {
    let mut ans = ext_from_f64(coef[0]);
    for &c in &coef[1..] {
        ans = ext_add(&ext_mul(&ans, &x, CW_PC64_RN), &ext_from_f64(c), CW_PC64_RN);
    }
    ans
}
fn x87_p1evl(x: Ext80, coef: &[f64]) -> Ext80 {
    let mut ans = ext_add(&x, &ext_from_f64(coef[0]), CW_PC64_RN);
    for &c in &coef[1..] {
        ans = ext_add(&ext_mul(&ans, &x, CW_PC64_RN), &ext_from_f64(c), CW_PC64_RN);
    }
    ans
}

fn pqr_x87(x: f64) -> f64 {
    let xe = ext_from_f64(x);
    let t = ext_div(
        &ext_sub(&xe, &ext_from_f64(3.75), CW_PC64_RN),
        &ext_add(&xe, &ext_from_f64(3.75), CW_PC64_RN),
        CW_PC64_RN,
    );
    let mut acc = ext_div(&x87_horner(&P, xe), &x87_horner(&Q, xe), CW_PC64_RN);
    for &r in R.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW_PC64_RN), &ext_from_f64(r), CW_PC64_RN);
    }
    ext_to_f64(&acc, CW_PC64_RN)
}
fn aabb_x87(x: f64) -> f64 {
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
    let r = ext_div(&x87_horner(&AA, z), &x87_horner(&BB, z), CW_PC64_RN);
    let mut acc = ext_add(&ext_mul(&r, &t, CW_PC64_RN), &ext_from_f64(E2), CW_PC64_RN);
    acc = ext_add(&ext_mul(&acc, &t, CW_PC64_RN), &ext_from_f64(E1), CW_PC64_RN);
    acc = ext_add(&ext_mul(&acc, &t, CW_PC64_RN), &ext_from_f64(E0), CW_PC64_RN);
    ext_to_f64(&ext_div(&acc, &xe, CW_PC64_RN), CW_PC64_RN)
}
fn nswc_x87_15_4(x: f64) -> f64 {
    if x < 1.5 {
        pqr_x87(x)
    } else if x < 4.0 {
        aabb_x87(x)
    } else {
        f::nswc_ccdd_f(x)
    }
}

fn cody_x87(y: f64) -> f64 {
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
    const PP: [f64; 6] = [
        0.305326634961232344,
        0.360344899949804439,
        0.125781726111229246,
        0.0160837851487422766,
        6.58749161529837803e-4,
        0.0163153871373020978,
    ];
    const QQ: [f64; 5] = [
        2.56852019228982242,
        1.87295284992346047,
        0.527905102951428412,
        0.0605183413124413191,
        0.00233520497626869185,
    ];
    let ye = ext_from_f64(y);
    if y <= 4.0 {
        let mut xnum = ext_mul(&ext_from_f64(C[8]), &ye, CW_PC64_RN);
        let mut xden = ye;
        for i in 0..7 {
            xnum = ext_mul(&ext_add(&xnum, &ext_from_f64(C[i]), CW_PC64_RN), &ye, CW_PC64_RN);
            xden = ext_mul(&ext_add(&xden, &ext_from_f64(D[i]), CW_PC64_RN), &ye, CW_PC64_RN);
        }
        ext_to_f64(
            &ext_div(
                &ext_add(&xnum, &ext_from_f64(C[7]), CW_PC64_RN),
                &ext_add(&xden, &ext_from_f64(D[7]), CW_PC64_RN),
                CW_PC64_RN,
            ),
            CW_PC64_RN,
        )
    } else {
        let ysq = ext_div(&ext_from_f64(1.0), &ext_mul(&ye, &ye, CW_PC64_RN), CW_PC64_RN);
        let mut xnum = ext_mul(&ext_from_f64(PP[5]), &ysq, CW_PC64_RN);
        let mut xden = ysq;
        for i in 0..4 {
            xnum = ext_mul(&ext_add(&xnum, &ext_from_f64(PP[i]), CW_PC64_RN), &ysq, CW_PC64_RN);
            xden = ext_mul(&ext_add(&xden, &ext_from_f64(QQ[i]), CW_PC64_RN), &ysq, CW_PC64_RN);
        }
        let r = ext_div(
            &ext_mul(&ysq, &ext_add(&xnum, &ext_from_f64(PP[4]), CW_PC64_RN), CW_PC64_RN),
            &ext_add(&xden, &ext_from_f64(QQ[4]), CW_PC64_RN),
            CW_PC64_RN,
        );
        ext_to_f64(
            &ext_div(
                &ext_sub(&ext_from_f64(f::RPINV), &r, CW_PC64_RN),
                &ye,
                CW_PC64_RN,
            ),
            CW_PC64_RN,
        )
    }
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
    const CR: [f64; 6] = [
        5.64189583547755073984e-1,
        1.27536670759978104416e0,
        5.01905042251180477414e0,
        6.16021097993053585195e0,
        7.40974269950448939160e0,
        2.97886665372100240670e0,
    ];
    const CS: [f64; 6] = [
        2.26052863220117276590e0,
        9.39603524938001434673e0,
        1.20489539808096656605e1,
        1.70814450747565897222e1,
        9.60896809063285878198e0,
        3.36907645100081516050e0,
    ];
    let ax = x.abs();
    let xe = ext_from_f64(ax);
    if ax < 8.0 {
        ext_to_f64(&ext_div(&x87_polevl(xe, &CP), &x87_p1evl(xe, &CQ), CW_PC64_RN), CW_PC64_RN)
    } else {
        ext_to_f64(&ext_div(&x87_polevl(xe, &CR), &x87_p1evl(xe, &CS), CW_PC64_RN), CW_PC64_RN)
    }
}

fn report(label: &str, rows: &[(f64, u64)], eval: impl Fn(f64) -> f64) {
    let (m, t) = f::score_f(rows, eval);
    println!("  {label:<32} mid {}  tail {}", f::fmt_acc(&m), f::fmt_acc(&t));
}

fn main() {
    let dir = std::env::args()
        .nth(1)
        .expect("G3-01-dist (heldout files are not named)");
    assert!(!dir.contains("heldout"));
    let out = PathBuf::from(
        std::env::args()
            .nth(2)
            .unwrap_or_else(|| "../../work/w109/erfc-f-direct".into()),
    );
    fs::create_dir_all(&out).unwrap();
    let tagged = f::load_q_rows_tagged(&dir);
    let direct: Vec<(f64, u64)> = tagged
        .iter()
        .filter(|r| r.direct)
        .map(|r| (r.z, r.qbits))
        .collect();
    println!("direct rows={} (heldout absent)", direct.len());

    println!("\n## named + x87 on direct ERFC.PRECISE");
    report("nswc_native", &direct, f::nswc_derfc0);
    report("nswc_x87_1.5/4", &direct, nswc_x87_15_4);
    report("cody_native", &direct, f::cody_erfcx_f);
    report("cody_x87", &direct, cody_x87);
    report("cephes_native", &direct, f::cephes_f);
    report("cephes_x87", &direct, cephes_x87);
    report("cdflib_native", &direct, f::cdflib_erfc1_f);
    report("cf_as714_x87_n24", &direct, |z| f::cf_as714_x87_n(z, 24));

    println!("\n## piecewise Cody native / x87-NSWC 1.5/4 on direct");
    let mut best = (0.0, 0usize);
    for k in 5..=40 {
        let cut = k as f64 * 0.1;
        let (m, t) = f::score_f(&direct, |z| {
            if z < cut {
                f::cody_erfcx_f(z)
            } else {
                nswc_x87_15_4(z)
            }
        });
        let all = m.exact + t.exact;
        if all > best.1 {
            best = (cut, all);
            println!(
                "  cut={cut:.1} mid {} tail {} all={all}",
                f::fmt_acc(&m),
                f::fmt_acc(&t)
            );
        }
    }
    println!("  best Cody-then-x87NSWC cut={:.1} all_exact={}", best.0, best.1);

    println!("\n## piecewise Cody x87 / x87-NSWC 1.5/4 on direct");
    let mut best2 = (0.0, 0usize);
    for k in 5..=40 {
        let cut = k as f64 * 0.1;
        let (m, t) = f::score_f(&direct, |z| {
            if z < cut {
                cody_x87(z)
            } else {
                nswc_x87_15_4(z)
            }
        });
        let all = m.exact + t.exact;
        if all > best2.1 {
            best2 = (cut, all);
            println!(
                "  cut={cut:.1} mid {} tail {} all={all}",
                f::fmt_acc(&m),
                f::fmt_acc(&t)
            );
        }
    }
    println!(
        "  best Codyx87-then-x87NSWC cut={:.1} all_exact={}",
        best2.0, best2.1
    );

    println!("\n## union exact (per-row better of Cody x87 vs x87-NSWC) direct mid");
    let mut u_both = 0usize;
    let mut u_cody = 0usize;
    let mut u_nswc = 0usize;
    let mut u_neither = 0usize;
    let mut n_mid = 0usize;
    for &(z, q) in &direct {
        if z < 0.5 || z >= 4.0 {
            continue;
        }
        let Some(fo) = f::f_or(z, q) else {
            continue;
        };
        n_mid += 1;
        let dc = ulp_distance(cody_x87(z), fo).unwrap_or(u64::MAX);
        let dn = ulp_distance(nswc_x87_15_4(z), fo).unwrap_or(u64::MAX);
        match (dc == 0, dn == 0) {
            (true, true) => u_both += 1,
            (true, false) => u_cody += 1,
            (false, true) => u_nswc += 1,
            (false, false) => u_neither += 1,
        }
    }
    println!(
        "  both={u_both} cody-only={u_cody} nswc-only={u_nswc} neither={u_neither} n={n_mid} union={}",
        u_both + u_cody + u_nswc
    );

    println!("\n## hard direct mid misses vs x87-NSWC 1.5/4 with no named exact");
    let named: [(&str, fn(f64) -> f64); 6] = [
        ("nswc", f::nswc_derfc0),
        ("cody", f::cody_erfcx_f),
        ("cephes", f::cephes_f),
        ("cdflib", f::cdflib_erfc1_f),
        ("cf80", f::cf_as714_f),
        ("cody_x87", cody_x87),
    ];
    let mut hf = fs::File::create(out.join("hard-direct-mid.tsv")).unwrap();
    writeln!(hf, "z\tulp_x87\tbest\tbest_ulp").unwrap();
    let mut n_hard = 0usize;
    for r in tagged.iter().filter(|r| r.direct && r.z >= 0.5 && r.z < 4.0) {
        let Some(fo) = f::f_or(r.z, r.qbits) else {
            continue;
        };
        let dg = ulp_distance(nswc_x87_15_4(r.z), fo).unwrap_or(u64::MAX);
        if dg == 0 {
            continue;
        }
        let mut best_n = "none";
        let mut best_d = u64::MAX;
        for (name, eval) in named {
            let d = ulp_distance(eval(r.z), fo).unwrap_or(u64::MAX);
            if d < best_d {
                best_d = d;
                best_n = name;
            }
        }
        if best_d == 0 {
            continue;
        }
        n_hard += 1;
        writeln!(hf, "{:.16e}\t{dg}\t{best_n}\t{best_d}", r.z).unwrap();
        println!(
            "  z={:.16e} x87ulp={dg} closest={best_n} ulp={best_d}",
            r.z
        );
    }
    println!("  hard direct mid n={n_hard}");

    println!("\n## pins (direct)");
    for &z in &f::PIN_Z {
        let Some(r) = tagged.iter().find(|rr| rr.z == z && rr.direct) else {
            println!("  z={z} not in direct banks");
            continue;
        };
        let Some(fo) = f::f_or(z, r.qbits) else {
            continue;
        };
        print!("  z={z}:");
        for (name, eval) in [
            ("nswc", f::nswc_derfc0 as fn(f64) -> f64),
            ("x87cut", nswc_x87_15_4),
            ("cody", f::cody_erfcx_f),
            ("cody_x87", cody_x87),
            ("cephes_x87", cephes_x87),
            ("cdflib", f::cdflib_erfc1_f),
        ] {
            print!(
                " {name}={}",
                ulp_distance(eval(z), fo).unwrap_or(u64::MAX)
            );
        }
        println!();
    }
    println!("artifacts in {}", out.display());
}
