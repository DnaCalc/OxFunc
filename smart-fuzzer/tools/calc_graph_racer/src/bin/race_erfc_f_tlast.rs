//! Implied-t last-bit inverse: is leftover PQR a 1-ULP t object?
//! Unspilled x87 t, integer (4x-15)/(4x+15), PC53 vs PC64, AABB t-invert.
//! Heldouts unnamed. No landing.
//!
//!   cargo run --release --bin race_erfc_f_tlast -- ../../work/w109/G3-01-dist [out-dir]

use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC53_RN, CW_PC64_RN,
};
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

fn x87_horner(cs: &[f64], x: Ext80, cw: u16) -> Ext80 {
    let mut acc = ext_from_f64(0.0);
    for &c in cs.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &x, cw), &ext_from_f64(c), cw);
    }
    acc
}

fn pqr_x87_t(x: f64, t: f64) -> f64 {
    pqr_x87_text(x, ext_from_f64(t), CW_PC64_RN)
}

fn pqr_x87_text(x: f64, t: Ext80, cw: u16) -> f64 {
    let xe = ext_from_f64(x);
    let mut acc = ext_div(&x87_horner(&P, xe, cw), &x87_horner(&Q, xe, cw), cw);
    for &r in R.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, cw), &ext_from_f64(r), cw);
    }
    ext_to_f64(&acc, cw)
}

fn aabb_x87(x: f64) -> f64 {
    aabb_ck(x, 2.5, 13.0)
}

fn aabb_ck(x: f64, c: f64, k: f64) -> f64 {
    let xe = ext_from_f64(x);
    let zz = ext_mul(&xe, &xe, CW_PC64_RN);
    let z = ext_div(
        &ext_from_f64(1.0),
        &ext_add(&ext_from_f64(c), &zz, CW_PC64_RN),
        CW_PC64_RN,
    );
    let t = ext_sub(
        &ext_mul(&ext_from_f64(k), &z, CW_PC64_RN),
        &ext_from_f64(1.0),
        CW_PC64_RN,
    );
    let r = ext_div(
        &x87_horner(&AA, z, CW_PC64_RN),
        &x87_horner(&BB, z, CW_PC64_RN),
        CW_PC64_RN,
    );
    let mut acc = ext_add(&ext_mul(&r, &t, CW_PC64_RN), &ext_from_f64(E2), CW_PC64_RN);
    acc = ext_add(&ext_mul(&acc, &t, CW_PC64_RN), &ext_from_f64(E1), CW_PC64_RN);
    acc = ext_add(&ext_mul(&acc, &t, CW_PC64_RN), &ext_from_f64(E0), CW_PC64_RN);
    ext_to_f64(&ext_div(&acc, &xe, CW_PC64_RN), CW_PC64_RN)
}

fn invert_t(f_or: f64, uv: f64, t0: f64) -> Option<f64> {
    let mut t = t0;
    for _ in 0..16 {
        let mut acc = uv;
        let mut dacc = 0.0;
        for &r in R.iter().rev() {
            dacc = dacc * t + acc;
            acc = acc * t + r;
        }
        let err = acc - f_or;
        if err.abs() == 0.0 {
            return Some(t);
        }
        if dacc == 0.0 || !dacc.is_finite() {
            return None;
        }
        let t1 = t - err / dacc;
        if !t1.is_finite() {
            return None;
        }
        if (t1 - t).abs() <= (t.abs() * 2.0f64.powi(-52)).max(2.0f64.powi(-60)) {
            return Some(t1);
        }
        t = t1;
    }
    Some(t)
}

fn invert_aabb_t(fx: f64, r: f64, t0: f64) -> Option<f64> {
    // target poly(t) = r*t^3 + E2*t^2 + E1*t + E0  equals fx = F_or * x
    let mut t = t0;
    for _ in 0..16 {
        let acc = ((r * t + E2) * t + E1) * t + E0;
        let dacc = (3.0 * r * t + 2.0 * E2) * t + E1;
        let err = acc - fx;
        if err.abs() == 0.0 {
            return Some(t);
        }
        if dacc == 0.0 || !dacc.is_finite() {
            return None;
        }
        let t1 = t - err / dacc;
        if !t1.is_finite() {
            return None;
        }
        if (t1 - t).abs() <= (t.abs() * 2.0f64.powi(-52)).max(2.0f64.powi(-60)) {
            return Some(t1);
        }
        t = t1;
    }
    Some(t)
}

fn t_pub(x: f64) -> f64 {
    (x - 3.75) / (x + 3.75)
}
fn t_oneminus(x: f64) -> f64 {
    1.0 - 7.5 / (x + 3.75)
}
fn t_divfirst(x: f64) -> f64 {
    let r = 3.75 / x;
    (1.0 - r) / (1.0 + r)
}
fn t_scaled(x: f64) -> f64 {
    let u = x / 3.75;
    (u - 1.0) / (u + 1.0)
}
fn t_4x15(x: f64) -> f64 {
    let x4 = 4.0 * x;
    (x4 - 15.0) / (x4 + 15.0)
}
fn t_2x75(x: f64) -> f64 {
    let x2 = 2.0 * x;
    (x2 - 7.5) / (x2 + 7.5)
}
fn t_8x30(x: f64) -> f64 {
    let x8 = 8.0 * x;
    (x8 - 30.0) / (x8 + 30.0)
}
fn t_16x60(x: f64) -> f64 {
    let x16 = 16.0 * x;
    (x16 - 60.0) / (x16 + 60.0)
}

fn t_ext(x: f64, cw: u16, kind: u8) -> Ext80 {
    let xe = ext_from_f64(x);
    match kind {
        0 => {
            let num = ext_sub(&xe, &ext_from_f64(3.75), cw);
            let den = ext_add(&xe, &ext_from_f64(3.75), cw);
            ext_div(&num, &den, cw)
        }
        1 => {
            let den = ext_add(&xe, &ext_from_f64(3.75), cw);
            let q = ext_div(&ext_from_f64(7.5), &den, cw);
            ext_sub(&ext_from_f64(1.0), &q, cw)
        }
        2 => {
            let r = ext_div(&ext_from_f64(3.75), &xe, cw);
            let num = ext_sub(&ext_from_f64(1.0), &r, cw);
            let den = ext_add(&ext_from_f64(1.0), &r, cw);
            ext_div(&num, &den, cw)
        }
        3 => {
            let x4 = ext_mul(&xe, &ext_from_f64(4.0), cw);
            let num = ext_sub(&x4, &ext_from_f64(15.0), cw);
            let den = ext_add(&x4, &ext_from_f64(15.0), cw);
            ext_div(&num, &den, cw)
        }
        4 => {
            let x2 = ext_mul(&xe, &ext_from_f64(2.0), cw);
            let num = ext_sub(&x2, &ext_from_f64(7.5), cw);
            let den = ext_add(&x2, &ext_from_f64(7.5), cw);
            ext_div(&num, &den, cw)
        }
        5 => {
            let x4 = ext_mul(&ext_from_f64(4.0), &xe, cw);
            let num = ext_sub(&x4, &ext_from_f64(15.0), cw);
            let den = ext_add(&x4, &ext_from_f64(15.0), cw);
            ext_div(&num, &den, cw)
        }
        6 => {
            let fifteen = ext_div(&ext_from_f64(15.0), &ext_from_f64(4.0), cw);
            let num = ext_sub(&xe, &fifteen, cw);
            let den = ext_add(&xe, &fifteen, cw);
            ext_div(&num, &den, cw)
        }
        _ => t_ext(x, cw, 0),
    }
}

fn mid_compose(z: f64, t: f64) -> f64 {
    if z < 1.5 {
        pqr_x87_t(z, t)
    } else if z < 4.0 {
        aabb_x87(z)
    } else {
        f::nswc_ccdd_f(z)
    }
}

fn mid_compose_text(z: f64, t: Ext80, cw: u16) -> f64 {
    if z < 1.5 {
        pqr_x87_text(z, t, cw)
    } else if z < 4.0 {
        aabb_x87(z)
    } else {
        f::nswc_ccdd_f(z)
    }
}

fn report(label: &str, rows: &[(f64, u64)], eval: impl Fn(f64) -> f64) {
    let (m, t) = f::score_f(rows, eval);
    println!("  {label:<44} mid {}  tail {}", f::fmt_acc(&m), f::fmt_acc(&t));
}

fn lsb(x: f64) -> u64 {
    x.to_bits() & 1
}

fn main() {
    let dir = std::env::args()
        .nth(1)
        .expect("G3-01-dist (heldout files are not named)");
    assert!(!dir.contains("heldout"));
    let out = PathBuf::from(
        std::env::args()
            .nth(2)
            .unwrap_or_else(|| "../../work/w109/erfc-f-tlast".into()),
    );
    fs::create_dir_all(&out).unwrap();
    let tagged = f::load_q_rows_tagged(&dir);
    let rows: Vec<(f64, u64)> = tagged.iter().map(|r| (r.z, r.qbits)).collect();

    println!("## leftover PQR: does 1-ULP t make F exact?");
    let mut n = 0usize;
    let mut n_fail = 0usize;
    let mut exact_t_imp_native = 0usize;
    let mut exact_t_imp_x87 = 0usize;
    let mut exact_step_native = 0usize;
    let mut exact_step_x87 = 0usize;
    let mut exact_best3_x87 = 0usize;
    let mut exact_best3_native = 0usize;
    let mut hist_dt = [0usize; 8];
    let mut gt = 0usize;
    let mut lt = 0usize;
    let mut n_dt1 = 0usize;
    let mut sel_zlsb_n = [0usize; 2];
    let mut sel_zlsb_up = [0usize; 2];
    let mut sel_tlsb_n = [0usize; 2];
    let mut sel_tlsb_up = [0usize; 2];
    let mut sel_direct_n = [0usize; 2];
    let mut sel_direct_up = [0usize; 2];
    let mut sel_zlt1_n = [0usize; 2];
    let mut sel_zlt1_up = [0usize; 2];
    let mut match_graph = [0usize; 12];
    let mut exact_graph_x87 = [0usize; 12];
    let graph_name = [
        "t_pub",
        "t_oneminus",
        "t_divfirst",
        "t_scaled",
        "t_4x15",
        "t_2x75",
        "t_8x30",
        "t_16x60",
        "next_up",
        "next_down",
        "t_pc64_spill",
        "t_4x15_pc64_spill",
    ];
    let mut tf = fs::File::create(out.join("tlast-leftover.tsv")).unwrap();
    writeln!(
        tf,
        "z\tdirect\tulp_pub\tdt\tt_pub\tt_imp\tstep_x87_exact\tbest3_x87"
    )
    .unwrap();

    for r in &tagged {
        if r.z < 0.5 || r.z >= 1.5 {
            continue;
        }
        let Some(fo) = f::f_or(r.z, r.qbits) else {
            continue;
        };
        let tp = t_pub(r.z);
        let fg0 = pqr_x87_t(r.z, tp);
        let d0 = ulp_distance(fg0, fo).unwrap_or(u64::MAX);
        if d0 == 0 {
            continue;
        }
        n += 1;
        let uv = f::horner(&P, r.z) / f::horner(&Q, r.z);
        let Some(ti) = invert_t(fo, uv, tp) else {
            n_fail += 1;
            continue;
        };
        let dt = ulp_distance(ti, tp).unwrap_or(u64::MAX);
        let bucket = match dt {
            0 => 0,
            1 => 1,
            2 => 2,
            3..=7 => 3,
            8..=63 => 4,
            64..=1023 => 5,
            1024..=1048576 => 6,
            _ => 7,
        };
        hist_dt[bucket] += 1;
        if ti > tp {
            gt += 1;
        } else if ti < tp {
            lt += 1;
        }
        if dt == 1 {
            n_dt1 += 1;
            let toward_up = ti > tp;
            let iz = lsb(r.z) as usize;
            sel_zlsb_n[iz] += 1;
            sel_zlsb_up[iz] += toward_up as usize;
            let it = lsb(tp) as usize;
            sel_tlsb_n[it] += 1;
            sel_tlsb_up[it] += toward_up as usize;
            let id = r.direct as usize;
            sel_direct_n[id] += 1;
            sel_direct_up[id] += toward_up as usize;
            let i1 = (r.z < 1.0) as usize;
            sel_zlt1_n[i1] += 1;
            sel_zlt1_up[i1] += toward_up as usize;
        }
        let step = if ti > tp { tp.next_up() } else { tp.next_down() };
        let d_imp_n = ulp_distance(f::nswc_pqr_t(r.z, ti), fo).unwrap_or(u64::MAX);
        let d_imp_x = ulp_distance(pqr_x87_t(r.z, ti), fo).unwrap_or(u64::MAX);
        let d_step_n = ulp_distance(f::nswc_pqr_t(r.z, step), fo).unwrap_or(u64::MAX);
        let d_step_x = ulp_distance(pqr_x87_t(r.z, step), fo).unwrap_or(u64::MAX);
        if d_imp_n == 0 {
            exact_t_imp_native += 1;
        }
        if d_imp_x == 0 {
            exact_t_imp_x87 += 1;
        }
        if d_step_n == 0 {
            exact_step_native += 1;
        }
        if d_step_x == 0 {
            exact_step_x87 += 1;
        }
        let cands = [tp, tp.next_up(), tp.next_down()];
        if cands
            .iter()
            .any(|&t| ulp_distance(pqr_x87_t(r.z, t), fo).unwrap_or(1) == 0)
        {
            exact_best3_x87 += 1;
        }
        if cands
            .iter()
            .any(|&t| ulp_distance(f::nswc_pqr_t(r.z, t), fo).unwrap_or(1) == 0)
        {
            exact_best3_native += 1;
        }
        let graphs = [
            tp,
            t_oneminus(r.z),
            t_divfirst(r.z),
            t_scaled(r.z),
            t_4x15(r.z),
            t_2x75(r.z),
            t_8x30(r.z),
            t_16x60(r.z),
            tp.next_up(),
            tp.next_down(),
            ext_to_f64(&t_ext(r.z, CW_PC64_RN, 0), CW_PC64_RN),
            ext_to_f64(&t_ext(r.z, CW_PC64_RN, 3), CW_PC64_RN),
        ];
        for (i, &tg) in graphs.iter().enumerate() {
            if tg.to_bits() == ti.to_bits() || ulp_distance(tg, ti).unwrap_or(1) == 0 {
                match_graph[i] += 1;
            }
            if ulp_distance(pqr_x87_t(r.z, tg), fo).unwrap_or(1) == 0 {
                exact_graph_x87[i] += 1;
            }
        }
        writeln!(
            tf,
            "{:.16e}\t{}\t{d0}\t{dt}\t{}\t{}\t{}\t{}",
            r.z,
            r.direct as u8,
            tp.to_bits(),
            ti.to_bits(),
            (d_step_x == 0) as u8,
            cands
                .iter()
                .any(|&t| ulp_distance(pqr_x87_t(r.z, t), fo).unwrap_or(1) == 0) as u8
        )
        .unwrap();
    }
    println!("  leftover n={n} invert_fail={n_fail}");
    println!("  ulp(t_imp,t_pub) hist 0,1,2,3-7,8-63,64-1k,1k-2^20,big: {hist_dt:?}");
    println!("  t_imp>t_pub {gt}  t_imp<t_pub {lt}");
    println!(
        "  F-exact with raw t_imp: native {exact_t_imp_native}/{n}  x87 {exact_t_imp_x87}/{n}"
    );
    println!(
        "  F-exact with 1-ULP step toward t_imp: native {exact_step_native}/{n}  x87 {exact_step_x87}/{n}"
    );
    println!(
        "  F-exact best of {{t,next_up,next_down}}: native {exact_best3_native}/{n}  x87 {exact_best3_x87}/{n}"
    );
    println!("  among dt=1 n={n_dt1}, toward_up counts / totals per predicate bit:");
    println!("    z_lsb n={sel_zlsb_n:?} up={sel_zlsb_up:?}");
    println!("    t_lsb n={sel_tlsb_n:?} up={sel_tlsb_up:?}");
    println!("    direct n={sel_direct_n:?} up={sel_direct_up:?}");
    println!("    z<1 [ge1,lt1] n={sel_zlt1_n:?} up={sel_zlt1_up:?}");
    println!("  leftover rows where t_graph bits match t_imp / where x87 PQR(t_graph) is exact:");
    for i in 0..12 {
        println!(
            "    {:<22} t_imp-match={}  F-exact={}",
            graph_name[i], match_graph[i], exact_graph_x87[i]
        );
    }

    println!("\n## unspilled x87 t Horner (t never f64) on leftover-aware full mid, cut 1.5");
    let kinds: [(&str, u8); 7] = [
        ("ext t=(x-3.75)/(x+3.75)", 0),
        ("ext t=1-7.5/(x+3.75)", 1),
        ("ext t=(1-3.75/x)/(1+3.75/x)", 2),
        ("ext t=(4x-15)/(4x+15)", 3),
        ("ext t=(2x-7.5)/(2x+7.5)", 4),
        ("ext t=(4*x-15)/(4*x+15) mul4,x", 5),
        ("ext t a=15/4 then bilinear", 6),
    ];
    for (name, k) in kinds {
        report(name, &rows, |z| {
            mid_compose_text(z, t_ext(z, CW_PC64_RN, k), CW_PC64_RN)
        });
        report(&format!("{name} PC53"), &rows, |z| {
            mid_compose_text(z, t_ext(z, CW_PC53_RN, k), CW_PC53_RN)
        });
    }
    report("spill t_pub cut1.5", &rows, |z| mid_compose(z, t_pub(z)));
    report("spill t_divfirst cut1.5", &rows, |z| {
        mid_compose(z, t_divfirst(z))
    });
    report("spill t_4x15 cut1.5", &rows, |z| mid_compose(z, t_4x15(z)));
    report("spill t_oneminus cut1.5", &rows, |z| {
        mid_compose(z, t_oneminus(z))
    });
    report("spill t_scaled cut1.5", &rows, |z| mid_compose(z, t_scaled(z)));
    report("spill t_2x75 cut1.5", &rows, |z| mid_compose(z, t_2x75(z)));
    report("spill t_8x30 cut1.5", &rows, |z| mid_compose(z, t_8x30(z)));
    report("spill t_16x60 cut1.5", &rows, |z| mid_compose(z, t_16x60(z)));
    report("t_pub next_up cut1.5", &rows, |z| {
        mid_compose(z, t_pub(z).next_up())
    });
    report("t_pub next_down cut1.5", &rows, |z| {
        mid_compose(z, t_pub(z).next_down())
    });
    report("t_pub next_toward_0 cut1.5", &rows, |z| {
        let t = t_pub(z);
        mid_compose(z, if t < 0.0 { t.next_up() } else { t.next_down() })
    });
    report("t_pub next_away_0 cut1.5", &rows, |z| {
        let t = t_pub(z);
        mid_compose(z, if t < 0.0 { t.next_down() } else { t.next_up() })
    });
    report("oracle per-row best3 t cut1.5 (ceiling)", &rows, |z| {
        if z >= 1.5 {
            return mid_compose(z, 0.0);
        }
        let Some(fo) = rows
            .iter()
            .find(|(zz, _)| zz.to_bits() == z.to_bits())
            .and_then(|(zz, q)| f::f_or(*zz, *q))
        else {
            return mid_compose(z, t_pub(z));
        };
        let tp = t_pub(z);
        let mut best = (u64::MAX, tp);
        for t in [tp, tp.next_up(), tp.next_down()] {
            let d = ulp_distance(pqr_x87_t(z, t), fo).unwrap_or(u64::MAX);
            if d < best.0 {
                best = (d, t);
            }
        }
        pqr_x87_t(z, best.1)
    });

    println!("\n## cheap 1-bit t selectors on leftover-aware full mid");
    let preds: [(&str, fn(f64) -> bool); 8] = [
        ("z_lsb", |z| lsb(z) == 1),
        ("t_lsb", |z| lsb(t_pub(z)) == 1),
        ("z<1", |z| z < 1.0),
        ("(x-3.75)_lsb", |z| lsb(z - 3.75) == 1),
        ("(x+3.75)_lsb", |z| lsb(z + 3.75) == 1),
        ("(4x-15)_lsb", |z| lsb(4.0 * z - 15.0) == 1),
        ("(4x+15)_lsb", |z| lsb(4.0 * z + 15.0) == 1),
        ("z_bit2", |z| (z.to_bits() >> 1) & 1 == 1),
    ];
    for (name, pred) in preds {
        report(&format!("next_up if {name} else pub"), &rows, |z| {
            let t = t_pub(z);
            mid_compose(z, if pred(z) { t.next_up() } else { t })
        });
        report(&format!("next_down if {name} else pub"), &rows, |z| {
            let t = t_pub(z);
            mid_compose(z, if pred(z) { t.next_down() } else { t })
        });
        report(&format!("step-toward-0 if {name}"), &rows, |z| {
            let t = t_pub(z);
            let t2 = if t < 0.0 { t.next_up() } else { t.next_down() };
            mid_compose(z, if pred(z) { t2 } else { t })
        });
    }

    println!("\n## AABB leftover implied-t [1.5,4)");
    let mut an = 0usize;
    let mut a_fail = 0usize;
    let mut a_hist = [0usize; 8];
    let mut a_gt = 0usize;
    let mut a_lt = 0usize;
    let mut a_exact_step = 0usize;
    let mut a_exact_best3 = 0usize;
    let mut af = fs::File::create(out.join("aabb-implied-t.tsv")).unwrap();
    writeln!(af, "z\tdirect\tulp_pub\tdt\tt_pub\tt_imp").unwrap();
    for r in &tagged {
        if r.z < 1.5 || r.z >= 4.0 {
            continue;
        }
        let Some(fo) = f::f_or(r.z, r.qbits) else {
            continue;
        };
        let fg0 = aabb_x87(r.z);
        let d0 = ulp_distance(fg0, fo).unwrap_or(u64::MAX);
        if d0 == 0 {
            continue;
        }
        an += 1;
        let zz = 1.0 / (2.5 + r.z * r.z);
        let tpub = 13.0 * zz - 1.0;
        let rr = f::horner(&AA, zz) / f::horner(&BB, zz);
        let Some(ti) = invert_aabb_t(fo * r.z, rr, tpub) else {
            a_fail += 1;
            continue;
        };
        let dt = ulp_distance(ti, tpub).unwrap_or(u64::MAX);
        let bucket = match dt {
            0 => 0,
            1 => 1,
            2 => 2,
            3..=7 => 3,
            8..=63 => 4,
            64..=1023 => 5,
            1024..=1048576 => 6,
            _ => 7,
        };
        a_hist[bucket] += 1;
        if ti > tpub {
            a_gt += 1;
        } else if ti < tpub {
            a_lt += 1;
        }
        let step = if ti > tpub {
            tpub.next_up()
        } else {
            tpub.next_down()
        };
        // evaluate AABB Horner at stepped t, published z
        let eval_t = |t: f64| {
            let acc = ((rr * t + E2) * t + E1) * t + E0;
            acc / r.z
        };
        if ulp_distance(eval_t(step), fo).unwrap_or(1) == 0 {
            a_exact_step += 1;
        }
        if [tpub, tpub.next_up(), tpub.next_down()]
            .iter()
            .any(|&t| ulp_distance(eval_t(t), fo).unwrap_or(1) == 0)
        {
            a_exact_best3 += 1;
        }
        writeln!(
            af,
            "{:.16e}\t{}\t{d0}\t{dt}\t{}\t{}",
            r.z,
            r.direct as u8,
            tpub.to_bits(),
            ti.to_bits()
        )
        .unwrap();
    }
    println!("  AABB leftover n={an} invert_fail={a_fail}");
    println!("  ulp(t_imp,t_pub) hist: {a_hist:?}");
    println!("  t_imp>t_pub {a_gt}  t_imp<t_pub {a_lt}");
    println!("  F-exact 1-ULP step toward t_imp (native AABB poly): {a_exact_step}/{an}");
    println!("  F-exact best of 3 t: {a_exact_best3}/{an}");

    println!("\n## baseline cut-1.5 x87 NSWC (bar)");
    report("x87 PQR t_pub + AABB + native CCDD", &rows, |z| {
        mid_compose(z, t_pub(z))
    });
    println!("artifacts in {}", out.display());
}
