//! Recover a native-vs-x87 rounding selector for NSWC PQR/AABB.
//! Heldouts unnamed. Does not land kernels.
//!
//!   cargo run --release --bin race_erfc_f_selector -- ../../work/w109/G3-01-dist [out-dir]

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

fn pqr_native(x: f64) -> f64 {
    f::nswc_pqr_f(x)
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
fn aabb_native(x: f64) -> f64 {
    let z = 1.0 / (2.5 + x * x);
    let t = 13.0 * z - 1.0;
    let acc = ((f::horner(&AA, z) / f::horner(&BB, z) * t + E2) * t + E1) * t + E0;
    acc / x
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

fn t_bits_pqr(x: f64) -> u64 {
    let t = (x - 3.75) / (x + 3.75);
    t.to_bits()
}
fn t_bits_aabb(x: f64) -> u64 {
    let z = 1.0 / (2.5 + x * x);
    (13.0 * z - 1.0).to_bits()
}

fn main() {
    let dir = std::env::args()
        .nth(1)
        .expect("G3-01-dist (heldout files are not named)");
    assert!(!dir.contains("heldout"));
    let out = PathBuf::from(
        std::env::args()
            .nth(2)
            .unwrap_or_else(|| "../../work/w109/erfc-f-selector".into()),
    );
    fs::create_dir_all(&out).unwrap();
    let tagged = f::load_q_rows_tagged(&dir);
    let mut sf = fs::File::create(out.join("selector.tsv")).unwrap();
    writeln!(sf, "z\tdirect\tband\tsel\tdn\tdx\tzlsb\ttlsb").unwrap();

    // sel: 0 both exact, 1 native-only, 2 x87-only, 3 neither
    let mut pqr = [0usize; 4];
    let mut aabb = [0usize; 4];
    let mut pqr_lsb = [[0usize; 2]; 4];
    let mut aabb_tlsb = [[0usize; 2]; 4];
    let mut n_union_pqr = 0usize;
    let mut n_union_aabb = 0usize;

    for r in &tagged {
        if r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let Some(fo) = f::f_or(r.z, r.qbits) else {
            continue;
        };
        let (dn, dx, tlsb, band) = if r.z < 1.5 {
            (
                ulp_distance(pqr_native(r.z), fo).unwrap_or(u64::MAX),
                ulp_distance(pqr_x87(r.z), fo).unwrap_or(u64::MAX),
                t_bits_pqr(r.z) & 1,
                0u8,
            )
        } else {
            (
                ulp_distance(aabb_native(r.z), fo).unwrap_or(u64::MAX),
                ulp_distance(aabb_x87(r.z), fo).unwrap_or(u64::MAX),
                t_bits_aabb(r.z) & 1,
                1u8,
            )
        };
        let sel = match (dn == 0, dx == 0) {
            (true, true) => 0,
            (true, false) => 1,
            (false, true) => 2,
            (false, false) => 3,
        };
        let zlsb = r.z.to_bits() & 1;
        writeln!(
            sf,
            "{:.16e}\t{}\t{band}\t{sel}\t{dn}\t{dx}\t{zlsb}\t{tlsb}",
            r.z,
            r.direct as u8
        )
        .unwrap();
        if band == 0 {
            pqr[sel] += 1;
            pqr_lsb[sel][zlsb as usize] += 1;
            if sel == 0 || sel == 1 || sel == 2 {
                n_union_pqr += 1;
            }
        } else {
            aabb[sel] += 1;
            aabb_tlsb[sel][tlsb as usize] += 1;
            if sel != 3 {
                n_union_aabb += 1;
            }
        }
    }

    println!("## PQR [0.5,1.5) native vs x87");
    println!(
        "  both={} native-only={} x87-only={} neither={} union={}",
        pqr[0], pqr[1], pqr[2], pqr[3], n_union_pqr
    );
    println!(
        "  native-only vs z lsb: lsb0={} lsb1={}",
        pqr_lsb[1][0], pqr_lsb[1][1]
    );
    println!(
        "  x87-only vs z lsb:    lsb0={} lsb1={}",
        pqr_lsb[2][0], pqr_lsb[2][1]
    );

    println!("## AABB [1.5,4) native vs x87");
    println!(
        "  both={} native-only={} x87-only={} neither={} union={}",
        aabb[0], aabb[1], aabb[2], aabb[3], n_union_aabb
    );
    println!(
        "  native-only vs t lsb: lsb0={} lsb1={}",
        aabb_tlsb[1][0], aabb_tlsb[1][1]
    );
    println!(
        "  x87-only vs t lsb:    lsb0={} lsb1={}",
        aabb_tlsb[2][0], aabb_tlsb[2][1]
    );

    println!("\n## selector vs z decile in PQR (native-only / x87-only / neither)");
    let mut dec = [[0usize; 4]; 10];
    let tagged2 = f::load_q_rows_tagged(&dir);
    for r in &tagged2 {
        if r.z < 0.5 || r.z >= 1.5 {
            continue;
        }
        let Some(fo) = f::f_or(r.z, r.qbits) else {
            continue;
        };
        let dn = ulp_distance(pqr_native(r.z), fo).unwrap_or(u64::MAX);
        let dx = ulp_distance(pqr_x87(r.z), fo).unwrap_or(u64::MAX);
        let sel = match (dn == 0, dx == 0) {
            (true, true) => 0,
            (true, false) => 1,
            (false, true) => 2,
            (false, false) => 3,
        };
        let b = ((r.z - 0.5) / 0.1).floor().clamp(0.0, 9.0) as usize;
        dec[b][sel] += 1;
    }
    for i in 0..10 {
        let lo = 0.5 + i as f64 * 0.1;
        println!(
            "  [{lo:.1},{:.1}) both={} nat={} x87={} neither={}",
            lo + 0.1,
            dec[i][0],
            dec[i][1],
            dec[i][2],
            dec[i][3]
        );
    }

    println!("\n## cheap predicates as selectors (PQR band exact count)");
    let pqr_rows: Vec<(f64, u64)> = tagged
        .iter()
        .filter(|r| r.z >= 0.5 && r.z < 1.5)
        .map(|r| (r.z, r.qbits))
        .collect();
    let preds: [(&str, fn(f64) -> bool); 6] = [
        ("always x87", |_| true),
        ("always native", |_| false),
        ("z lsb == 1 -> x87", |z| z.to_bits() & 1 == 1),
        ("t lsb == 1 -> x87", |z| t_bits_pqr(z) & 1 == 1),
        ("z < 1.0 -> x87", |z| z < 1.0),
        ("(x-3.75) lsb -> x87", |z| (z - 3.75).to_bits() & 1 == 1),
    ];
    for (name, pred) in preds {
        let mut exact = 0usize;
        let mut n = 0usize;
        for &(z, q) in &pqr_rows {
            let Some(fo) = f::f_or(z, q) else {
                continue;
            };
            n += 1;
            let g = if pred(z) { pqr_x87(z) } else { pqr_native(z) };
            if ulp_distance(g, fo).unwrap_or(1) == 0 {
                exact += 1;
            }
        }
        println!("  {name:<28} {exact}/{n}");
    }
    println!("artifacts in {}", out.display());
}
