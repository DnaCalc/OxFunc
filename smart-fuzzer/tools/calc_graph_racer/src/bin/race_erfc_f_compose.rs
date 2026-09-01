//! Compose the two live F constraints: x87 NSWC retuned cuts + x87 CF.
//! Dump the even-odd x87 extra-exact tail rows. Heldouts unnamed. No landing.
//!
//!   cargo run --release --bin race_erfc_f_compose -- ../../work/w109/G3-01-dist [out-dir]

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
const CC: [f64; 9] = [
    -0.7040906288250128001000086e-04,
    -0.3858822461760510359506941e-02,
    -0.7708202127512212359395078e-01,
    -0.6713655014557429480440263e+00,
    -0.2081992124162995545731882e+01,
    0.2898831421475282558867888e+01,
    0.2199509380600429331650192e+02,
    0.2907064664404115316722996e+01,
    -0.4766208741588182425380950e+02,
];
const DD: [f64; 10] = [
    1.0,
    0.5238852785508439144747174e+02,
    0.9646843357714742409535148e+03,
    0.7007152775135939601804416e+04,
    0.8515386792259821780601162e+04,
    -0.1002360095177164564992134e+06,
    -0.2065250031331232815791912e+06,
    0.5695324805290370358175984e+06,
    0.6589752493461331195697873e+06,
    -0.1192930193156561957631462e+07,
];
const E0: f64 = 0.540464821348814822409610122136;
const E1: f64 = -0.261515522487415653487049835220e-01;
const E2: f64 = -0.288573438386338758794591212600e-02;
const E3: f64 = -0.529353396945788057720258856000e-03;

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

fn nswc_pqr_x87(x: f64) -> f64 {
    let xe = ext_from_f64(x);
    let xm = ext_sub(&xe, &ext_from_f64(3.75), CW_PC64_RN);
    let xp = ext_add(&xe, &ext_from_f64(3.75), CW_PC64_RN);
    let t = ext_div(&xm, &xp, CW_PC64_RN);
    let u = x87_horner(&P, xe);
    let v = x87_horner(&Q, xe);
    let mut acc = ext_div(&u, &v, CW_PC64_RN);
    for &r in R.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW_PC64_RN), &ext_from_f64(r), CW_PC64_RN);
    }
    ext_to_f64(&acc, CW_PC64_RN)
}

fn nswc_aabb_x87(x: f64) -> f64 {
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

fn nswc_ccdd_x87(x: f64) -> f64 {
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
    let r = ext_div(&x87_horner(&CC, z), &x87_horner(&DD, z), CW_PC64_RN);
    let mut acc = ext_add(&ext_mul(&r, &t, CW_PC64_RN), &ext_from_f64(E3), CW_PC64_RN);
    acc = ext_add(&ext_mul(&acc, &t, CW_PC64_RN), &ext_from_f64(E2), CW_PC64_RN);
    acc = ext_add(&ext_mul(&acc, &t, CW_PC64_RN), &ext_from_f64(E1), CW_PC64_RN);
    acc = ext_add(&ext_mul(&acc, &t, CW_PC64_RN), &ext_from_f64(E0), CW_PC64_RN);
    ext_to_f64(&ext_div(&acc, &xe, CW_PC64_RN), CW_PC64_RN)
}

fn nswc_x87_cut(x: f64, mid: f64, far: f64) -> f64 {
    if x < mid {
        nswc_pqr_x87(x)
    } else if x < far {
        nswc_aabb_x87(x)
    } else {
        nswc_ccdd_x87(x)
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
            .unwrap_or_else(|| "../../work/w109/erfc-f-compose".into()),
    );
    fs::create_dir_all(&out).unwrap();
    let tagged = f::load_q_rows_tagged(&dir);
    let rows: Vec<(f64, u64)> = tagged.iter().map(|r| (r.z, r.qbits)).collect();
    let direct: Vec<(f64, u64)> = tagged
        .iter()
        .filter(|r| r.direct)
        .map(|r| (r.z, r.qbits))
        .collect();
    println!(
        "rows={} direct={}  F_or w_rn53; heldout absent",
        rows.len(),
        direct.len()
    );

    println!("\n## x87 CF n-sweep vs native (the +5 tail question)");
    println!(
        "  {:>4} {:>22} {:>22} {:>22} {:>22} {:>22} {:>22}",
        "n", "nat as714", "x87 as714", "nat gaut", "x87 gaut", "nat eodd", "x87 eodd"
    );
    let mut ntab = String::from("n\tnat_as\tx87_as\tnat_gaut\tx87_gaut\tnat_eodd\tx87_eodd\n");
    for n in [8u32, 12, 16, 21, 24, 32, 40, 80] {
        let (am, at) = f::score_f(&rows, |z| f::cf_as714_n(z, n));
        let (axm, axt) = f::score_f(&rows, |z| f::cf_as714_x87_n(z, n));
        let (gm, gt) = f::score_f(&rows, |z| f::cf_gautschi_n(z, n));
        let (gxm, gxt) = f::score_f(&rows, |z| f::cf_gautschi_x87_n(z, n));
        let pairs = n / 2;
        let (em, et) = f::score_f(&rows, |z| f::cf_evenodd_as714_n(z, pairs));
        let (exm, ext) = f::score_f(&rows, |z| f::cf_evenodd_as714_x87_n(z, pairs));
        println!(
            "  {n:>4} {:>22} {:>22} {:>22} {:>22} {:>22} {:>22}",
            f::fmt_acc(&at),
            f::fmt_acc(&axt),
            f::fmt_acc(&gt),
            f::fmt_acc(&gxt),
            f::fmt_acc(&et),
            f::fmt_acc(&ext)
        );
        let _ = (am, axm, gm, gxm, em, exm);
        ntab.push_str(&format!(
            "{n}\t{}/{}\t{}/{}\t{}/{}\t{}/{}\t{}/{}\t{}/{}\n",
            at.exact, at.n, axt.exact, axt.n, gt.exact, gt.n, gxt.exact, gxt.n, et.exact, et.n, ext.exact, ext.n
        ));
    }
    fs::write(out.join("cf-x87-n.tsv"), ntab).unwrap();

    println!("\n## extra-exact tail rows: x87 even-odd n12 pairs vs native");
    let mut extra = fs::File::create(out.join("evenodd-x87-extra-tail.tsv")).unwrap();
    writeln!(extra, "z\tdirect\tnative_ulp\tx87_ulp\tf_or").unwrap();
    let mut n_extra = 0usize;
    let mut n_lost = 0usize;
    for r in &tagged {
        if r.z < 4.0 {
            continue;
        }
        let Some(fo) = f::f_or(r.z, r.qbits) else {
            continue;
        };
        let dn = ulp_distance(f::cf_evenodd_as714_n(r.z, 12), fo).unwrap_or(u64::MAX);
        let dx = ulp_distance(f::cf_evenodd_as714_x87_n(r.z, 12), fo).unwrap_or(u64::MAX);
        if dx == 0 && dn != 0 {
            n_extra += 1;
            writeln!(
                extra,
                "{:.16e}\t{}\t{dn}\t{dx}\t{}",
                r.z,
                r.direct as u8,
                fo.to_bits()
            )
            .unwrap();
        }
        if dn == 0 && dx != 0 {
            n_lost += 1;
        }
    }
    println!("  x87-extra-exact={n_extra}  x87-lost-native-exact={n_lost}");

    println!("\n## x87 NSWC cuts + x87 CCDD + compose with x87 even-odd");
    report("nswc_native", &rows, f::nswc_derfc0);
    report("nswc_x87_pub_cuts", &rows, |z| nswc_x87_cut(z, 2.0, 4.0));
    report("nswc_x87_1.5/4 + native CCDD skip", &rows, |z| {
        if z < 1.5 {
            nswc_pqr_x87(z)
        } else if z < 4.0 {
            nswc_aabb_x87(z)
        } else {
            f::nswc_ccdd_f(z)
        }
    });
    report("nswc_x87_1.5/4 + x87 CCDD", &rows, |z| nswc_x87_cut(z, 1.5, 4.0));
    report("x87 CCDD all z>=0.5", &rows, nswc_ccdd_x87);
    report("compose 1.5/4 x87NSWC else x87 eodd12", &rows, |z| {
        if z < 4.0 {
            nswc_x87_cut(z, 1.5, 4.0)
        } else {
            f::cf_evenodd_as714_x87_n(z, 12)
        }
    });
    report("compose 1.5/4 x87NSWC else x87 as714 n21", &rows, |z| {
        if z < 4.0 {
            nswc_x87_cut(z, 1.5, 4.0)
        } else {
            f::cf_as714_x87_n(z, 21)
        }
    });
    report("compose 1.5 x87PQR else x87 eodd12", &rows, |z| {
        if z < 1.5 {
            nswc_pqr_x87(z)
        } else {
            f::cf_evenodd_as714_x87_n(z, 12)
        }
    });

    println!("\n## compose cut: x87 NSWC(1.5 / min(cut,4)) then x87 even-odd n12");
    let mut best_c = (0.0, 0usize);
    for k in 16..=70 {
        let cut = k as f64 * 0.1;
        let (m, t) = f::score_f(&rows, |z| {
            if z < cut {
                nswc_x87_cut(z, 1.5, cut.min(4.0))
            } else {
                f::cf_evenodd_as714_x87_n(z, 12)
            }
        });
        let all = m.exact + t.exact;
        if all > best_c.1 {
            best_c = (cut, all);
            println!(
                "  cut={cut:.1} mid {} tail {} all={all}",
                f::fmt_acc(&m),
                f::fmt_acc(&t)
            );
        }
    }
    println!("  best compose cut={:.1} all_exact={}", best_c.0, best_c.1);

    println!("\n## same graphs on direct-only ERFC.PRECISE");
    report("direct nswc_native", &direct, f::nswc_derfc0);
    report("direct nswc_x87_1.5/4", &direct, |z| nswc_x87_cut(z, 1.5, 4.0));
    report("direct compose eodd12", &direct, |z| {
        if z < 4.0 {
            nswc_x87_cut(z, 1.5, 4.0)
        } else {
            f::cf_evenodd_as714_x87_n(z, 12)
        }
    });

    println!("\n## x87 1.5/4 residual hist (mid)");
    let mut hist = [0usize; 10];
    for &(z, qbits) in &rows {
        if z < 0.5 || z >= 4.0 {
            continue;
        }
        let Some(fo) = f::f_or(z, qbits) else {
            continue;
        };
        let d = ulp_distance(nswc_x87_cut(z, 1.5, 4.0), fo).unwrap_or(u64::MAX);
        if d > (1 << 20) {
            continue;
        }
        let b = match d {
            0 => 0,
            1 => 1,
            2 => 2,
            3 => 3,
            4 => 4,
            5 => 5,
            6 => 6,
            7 => 7,
            8..=15 => 8,
            _ => 9,
        };
        hist[b] += 1;
    }
    println!("  ulp hist 0..7,8-15,16+: {hist:?}");

    println!("\n## pins");
    for &z in &f::PIN_Z {
        let Some(r) = tagged.iter().find(|rr| rr.z == z) else {
            continue;
        };
        let Some(fo) = f::f_or(z, r.qbits) else {
            continue;
        };
        print!("  z={z} direct={}:", r.direct);
        for (name, eval) in [
            ("nswc", f::nswc_derfc0 as fn(f64) -> f64),
            ("x87_1.5/4", |x| nswc_x87_cut(x, 1.5, 4.0)),
            ("eodd_x87_12", |x| f::cf_evenodd_as714_x87_n(x, 12)),
            ("as714_x87_21", |x| f::cf_as714_x87_n(x, 21)),
            ("ccdd_x87", nswc_ccdd_x87),
        ] {
            print!(
                " {name}={}",
                ulp_distance(eval(z), fo).unwrap_or(u64::MAX)
            );
        }
        println!();
    }
    println!("\nartifacts in {}", out.display());
}
