//! W109 ERFC body test 3: extra RN53 stores on TOMS-654 branch 190 (small-z).
//!
//! No new constant. `gam1(1/2)` h pinned to `0x3fc06eba8214db6b`. Inner
//! cluster A `0.5+RN53(0.5-j)` vs B `RN53(1-j)`. w = chain / reuse-z /
//! excel_exp(0.5 ln). Frozen discovery only; heldouts unnamed.
//!
//! Usage:
//!   cargo run --release --bin race_erf190_inner -- ../../work/w109/G3-01-dist

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    CW_PC64_RN, Ext80, excel_exp, excel_ln, ext_abs, ext_add, ext_chs, ext_div, ext_f2xm1,
    ext_from_f64, ext_fyl2x, ext_l2e, ext_ln2, ext_mul, ext_one, ext_rndint, ext_scale, ext_sub,
    ext_to_f64,
};
use std::collections::BTreeMap;

const H_BITS: u64 = 0x3fc06eba8214db6b;
const ERF_BANKS: [&str; 7] = [
    "answers-b9train.json",
    "answers-erfp.json",
    "answers-erfm.json",
    "answers-b8erf.json",
    "answers-b7erf.json",
    "answers-b11.json",
    "answers-b10.json",
];
const ERFC_BANKS: [&str; 5] = [
    "answers-erfcp.json",
    "answers-erfcm.json",
    "answers-b7erfc.json",
    "answers-b8erfc.json",
    "answers-b11c.json",
];

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn dbl(x: &Ext80) -> f64 {
    ext_to_f64(x, CW_PC64_RN)
}

fn exp_ext(x: &Ext80) -> Ext80 {
    let t = ext_mul(x, &ext_l2e(), CW_PC64_RN);
    let k = ext_rndint(&t, CW_PC64_RN);
    let f = ext_sub(&t, &k, CW_PC64_RN);
    let neg = dbl(&f) < 0.0;
    let w = ext_f2xm1(&ext_abs(&f, CW_PC64_RN), CW_PC64_RN);
    let mut m = ext_add(&w, &ext_one(), CW_PC64_RN);
    if neg {
        m = ext_div(&ext_one(), &m, CW_PC64_RN);
    }
    ext_scale(&m, &k, CW_PC64_RN)
}

fn ln_ext(x: &Ext80) -> Ext80 {
    ext_fyl2x(&ext_ln2(), x, CW_PC64_RN)
}

fn ext_le(a: &Ext80, b: &Ext80) -> bool {
    dbl(&ext_sub(a, b, CW_PC64_RN)) <= 0.0
}

#[derive(Clone, Copy, Debug)]
enum Inner {
    A,
    B,
}
#[derive(Clone, Copy, Debug)]
enum WMode {
    Chain,
    ReuseZ,
    ExcelExpLn,
}

fn branch190_p(z: f64, inner: Inner, wmode: WMode, inner_dbl: bool) -> f64 {
    let a = ef(0.5);
    let mut x = ext_mul(&ef(z), &ef(z), CW_PC64_RN);
    x = ef(dbl(&x)); // zz_dbl = true, best check_erf190 mask
    if dbl(&x) == 0.0 {
        return 0.0;
    }
    let mut an = ef(3.0);
    let mut c = x;
    let mut sum = ext_div(&x, &ext_add(&a, &ef(3.0), CW_PC64_RN), CW_PC64_RN);
    let tol = ext_div(
        &ext_mul(&ef(3.0), &ef(5e-15), CW_PC64_RN),
        &ext_add(&a, &ext_one(), CW_PC64_RN),
        CW_PC64_RN,
    );
    for _ in 0..200 {
        an = ext_add(&an, &ext_one(), CW_PC64_RN);
        c = ext_chs(&ext_mul(&c, &ext_div(&x, &an, CW_PC64_RN), CW_PC64_RN), CW_PC64_RN);
        let t = ext_div(&c, &ext_add(&a, &an, CW_PC64_RN), CW_PC64_RN);
        sum = ext_add(&sum, &t, CW_PC64_RN);
        if ext_le(&ext_abs(&t, CW_PC64_RN), &tol) {
            break;
        }
    }
    let inner_poly = ext_add(
        &ext_mul(
            &ext_sub(
                &ext_div(&sum, &ef(6.0), CW_PC64_RN),
                &ext_div(&ef(0.5), &ext_add(&a, &ef(2.0), CW_PC64_RN), CW_PC64_RN),
                CW_PC64_RN,
            ),
            &x,
            CW_PC64_RN,
        ),
        &ext_div(&ext_one(), &ext_add(&a, &ext_one(), CW_PC64_RN), CW_PC64_RN),
        CW_PC64_RN,
    );
    let j = ext_mul(&ext_mul(&a, &x, CW_PC64_RN), &inner_poly, CW_PC64_RN);
    let mut inner_e = match inner {
        Inner::A => ext_add(
            &ef(0.5),
            &ext_sub(&ef(0.5), &j, CW_PC64_RN),
            CW_PC64_RN,
        ),
        Inner::B => ext_sub(&ef(1.0), &j, CW_PC64_RN),
    };
    if inner_dbl {
        inner_e = ef(dbl(&inner_e));
    }
    let g = ef(1.0 + f64::from_bits(H_BITS));
    let w = match wmode {
        WMode::ReuseZ => ef(z),
        WMode::ExcelExpLn => {
            let zz = z * z;
            ef(excel_exp(0.5 * excel_ln(zz)))
        }
        WMode::Chain => {
            let zl = ext_mul(&a, &ln_ext(&x), CW_PC64_RN);
            exp_ext(&zl)
        }
    };
    // wg_first: (w*g)*inner
    dbl(&ext_mul(&ext_mul(&w, &g, CW_PC64_RN), &inner_e, CW_PC64_RN))
}

fn load_pos(dir: &str, banks: &[&str], lo: f64, hi: f64) -> Vec<(f64, u64)> {
    let mut rows = BTreeMap::new();
    for name in banks {
        let path = format!("{dir}/{name}");
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let bank: WitnessSet = serde_json::from_str(&text).expect(name);
        for w in &bank.witnesses {
            let z = match &w.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).expect("z"),
                _ => continue,
            };
            let Some(exp) = parse_bits_hex(&w.expected_bits) else {
                continue;
            };
            if z > lo && z < hi {
                rows.insert(z.to_bits(), exp.to_bits());
            }
        }
    }
    rows.into_iter()
        .map(|(z, e)| (f64::from_bits(z), e))
        .collect()
}

fn main() {
    let dir = std::env::args().nth(1).expect("G3-01-dist");
    assert!(!dir.contains("heldout"));
    let erf = load_pos(&dir, &ERF_BANKS, 0.0, 0.5);
    let erfc = load_pos(&dir, &ERFC_BANKS, 0.0, 0.5);
    println!(
        "ERF z<0.5 rows={} ERFC z<0.5 rows={}; heldout absent",
        erf.len(),
        erfc.len()
    );
    println!(
        "check_erf190 1024-mask best was 850/1508 (zz stored, series ext, inner ext, wg_first, gam1 ret dbl)"
    );

    println!(
        "{:<28} {:>22} {:>22}",
        "cfg", "ERF P exact", "ERFC Q=1-P exact"
    );
    for inner in [Inner::A, Inner::B] {
        for wmode in [WMode::Chain, WMode::ReuseZ, WMode::ExcelExpLn] {
            for inner_dbl in [false, true] {
                let mut pe = 0usize;
                let mut qe = 0usize;
                let mut pmax = 0u64;
                let mut qmax = 0u64;
                for &(z, e) in &erf {
                    let p = branch190_p(z, inner, wmode, inner_dbl);
                    let d = ulp_distance(p, f64::from_bits(e)).unwrap_or(u64::MAX);
                    if d == 0 {
                        pe += 1;
                    }
                    pmax = pmax.max(d);
                }
                for &(z, e) in &erfc {
                    let p = branch190_p(z, inner, wmode, inner_dbl);
                    let q = 1.0 - p;
                    let d = ulp_distance(q, f64::from_bits(e)).unwrap_or(u64::MAX);
                    if d == 0 {
                        qe += 1;
                    }
                    qmax = qmax.max(d);
                }
                println!(
                    "{inner:?}/{wmode:?}/idbl={inner_dbl:<5} {pe:>5}/{:<5} max={pmax:<4} {qe:>5}/{:<5} max={qmax}",
                    erf.len(),
                    erfc.len()
                );
            }
        }
    }
}
