//! W109 G6-01: the DECISIVE extended-x87 em race. Workflow convergent hypothesis:
//! Excel's PMT em=(1+r)^-n-1 for |tau|<1 is computed in 80-bit x87 with the argument
//! NEVER spilled to double -> every all-double op-graph caps at 163/234. Test the
//! F2XM1-native (2^y-1, y=-n*log2(1+r) fused) and extended-u-1 forms with FULL range
//! (FSCALE reduction for ln2<|tau|<1), single final round. Model-free po2n em oracle.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    CW_PC64_RN as CW, Ext80, ext_abs, ext_add, ext_chs, ext_div, ext_f2xm1, ext_from_f64 as e,
    ext_fyl2x, ext_fyl2xp1, ext_l2e, ext_ln2, ext_mul, ext_one, ext_rndint, ext_scale, ext_sub,
    ext_to_f64,
};
use std::collections::BTreeMap;

fn tf(x: &Ext80) -> f64 {
    ext_to_f64(x, CW)
}
fn tau_d(r: f64, n: f64) -> f64 {
    -(n * rx::excel_log1p(r))
}

// log2(1+r) in ext: FYL2XP1 for |r|<0.29, else FYL2X(1+r)
fn log2_1p(r: f64) -> Ext80 {
    if r.abs() < 0.292893218813452 {
        ext_fyl2xp1(&ext_one(), &e(r), CW)
    } else {
        ext_fyl2x(&ext_one(), &ext_add(&ext_one(), &e(r), CW), CW)
    }
}
// ln(1+r) in ext = ln2 * log2(1+r)
fn ln_1p(r: f64) -> Ext80 {
    ext_mul(&ext_ln2(), &log2_1p(r), CW)
}

// exp of an extended tau -> ext (fFEXP: 2^(tau*log2e), split, F2XM1)
fn exp_ext(tau: &Ext80) -> Ext80 {
    let t = ext_mul(tau, &ext_l2e(), CW);
    let k = ext_rndint(&t, CW);
    let f = ext_sub(&t, &k, CW);
    let neg = tf(&f) < 0.0;
    let w = ext_f2xm1(&ext_abs(&f, CW), CW);
    let mut m = ext_add(&w, &ext_one(), CW);
    if neg {
        m = ext_div(&ext_one(), &m, CW);
    }
    ext_scale(&m, &k, CW)
}

// ---- candidate em forms (all return em as f64, single final round) ----

// A: F2XM1-native. y = -n*log2(1+r) ext; 2^y - 1 via F2XM1 + FSCALE reduction.
fn a_f2xm1(r: f64, n: f64) -> f64 {
    let y = ext_chs(&ext_mul(&e(n), &log2_1p(r), CW), CW); // -n*log2(1+r) ext
    let k = ext_rndint(&y, CW);
    let f = ext_sub(&y, &k, CW); // |f|<=0.5
    let w = ext_f2xm1(&f, CW); // 2^f - 1 (f can be negative; F2XM1 domain |f|<=1 ok)
    // 2^y - 1 = 2^k*(2^f) - 1 = 2^k*(w+1) - 1 = scale(w+1,k) - 1
    let em = ext_sub(
        &ext_scale(&ext_add(&w, &ext_one(), CW), &k, CW),
        &ext_one(),
        CW,
    );
    ext_to_f64(&em, CW)
}
// A2: same but if |y|<=1 use direct F2XM1(y) (no split), else FSCALE
fn a2_f2xm1_direct(r: f64, n: f64) -> f64 {
    let y = ext_chs(&ext_mul(&e(n), &log2_1p(r), CW), CW);
    if tf(&y).abs() <= 1.0 {
        return ext_to_f64(&ext_f2xm1(&y, CW), CW);
    }
    let k = ext_rndint(&y, CW);
    let f = ext_sub(&y, &k, CW);
    let w = ext_f2xm1(&f, CW);
    ext_to_f64(
        &ext_sub(
            &ext_scale(&ext_add(&w, &ext_one(), CW), &k, CW),
            &ext_one(),
            CW,
        ),
        CW,
    )
}
// B: extended u - 1. tau_ext=-n*ln(1+r); u=exp_ext(tau_ext); em=round(u-1).
fn b_uminus1(r: f64, n: f64) -> f64 {
    let tau = ext_chs(&ext_mul(&e(n), &ln_1p(r), CW), CW);
    let u = exp_ext(&tau);
    ext_to_f64(&ext_sub(&u, &ext_one(), CW), CW)
}
// C: extended Kahan. all ext, tau_ext, u ext, ln(u) ext.
fn c_extkahan(r: f64, n: f64) -> f64 {
    let tau = ext_chs(&ext_mul(&e(n), &ln_1p(r), CW), CW);
    let u = exp_ext(&tau);
    let num = ext_mul(&ext_sub(&u, &ext_one(), CW), &tau, CW);
    let den = ext_fyl2x(&ext_ln2(), &u, CW); // ln(u) ext
    ext_to_f64(&ext_div(&num, &den, CW), CW)
}
// D: F2XM1-native but tau via DOUBLE log1p then extended (tests if the extra log1p bits matter)
fn d_f2xm1_dbltau(r: f64, n: f64) -> f64 {
    let y = ext_mul(&e(tau_d(r, n)), &ext_l2e(), CW); // (double tau)*log2e ext
    if tf(&y).abs() <= 1.0 {
        return ext_to_f64(&ext_f2xm1(&y, CW), CW);
    }
    let k = ext_rndint(&y, CW);
    let f = ext_sub(&y, &k, CW);
    let w = ext_f2xm1(&f, CW);
    ext_to_f64(
        &ext_sub(
            &ext_scale(&ext_add(&w, &ext_one(), CW), &k, CW),
            &ext_one(),
            CW,
        ),
        CW,
    )
}
// E: extended u-1 but tau via DOUBLE log1p (spilled)
fn e_uminus1_dbltau(r: f64, n: f64) -> f64 {
    let u = exp_ext(&e(tau_d(r, n)));
    ext_to_f64(&ext_sub(&u, &ext_one(), CW), CW)
}
// baseline: all-double Kahan
fn k_dbl(r: f64, n: f64) -> f64 {
    let t = tau_d(r, n);
    let u = rx::excel_exp(t);
    (u - 1.0) * t / rx::excel_ln(u)
}

fn pin_em(rows: &[(f64, u64)], r: f64) -> Option<f64> {
    let (pv, pmtb) = rows[rows.len() / 2];
    let pmt = f64::from_bits(pmtb);
    let center = pv / (pmt / r);
    let cb = center.to_bits() as i64;
    for d in -8..=8i64 {
        let em = f64::from_bits((cb + d) as u64);
        if em == -1.0 {
            continue;
        }
        if rows
            .iter()
            .all(|(pv, want)| ((pv / em) * r).to_bits() == *want)
        {
            return Some(em);
        }
    }
    None
}
fn main() {
    let ws: WitnessSet = serde_json::from_str(
        &std::fs::read_to_string("../../work/w109/G6-solvers/answers-pmt-po2n.json").unwrap(),
    )
    .unwrap();
    let mut byrn: BTreeMap<(u64, u64), Vec<(f64, u64)>> = BTreeMap::new();
    for w in &ws.witnesses {
        let a: Vec<f64> = w
            .args
            .iter()
            .filter_map(|x| match x {
                WitnessArg::Scalar(s) => parse_bits_hex(s),
                _ => None,
            })
            .collect();
        if a.len() != 5 || a[3] != 0.0 || a[4] != 0.0 {
            continue;
        }
        let want = parse_bits_hex(&w.expected_bits).unwrap().to_bits();
        byrn.entry((a[0].to_bits(), a[1].to_bits()))
            .or_default()
            .push((a[2], want));
    }
    let cands: [(&str, fn(f64, f64) -> f64); 7] = [
        ("K_dbl(base 163)", k_dbl),
        ("A_f2xm1_split", a_f2xm1),
        ("A2_f2xm1_direct", a2_f2xm1_direct),
        ("B_extu-1", b_uminus1),
        ("C_extkahan", c_extkahan),
        ("D_f2xm1_dbltau", d_f2xm1_dbltau),
        ("E_extu-1_dbltau", e_uminus1_dbltau),
    ];
    let mut sc = [0u32; 7];
    let mut tot = 0u32;
    // split by |tau| vs ln2 to see the F2XM1 domain boundary
    let mut sc_lt = [0u32; 7];
    let mut tot_lt = 0u32; // |tau|<ln2
    for ((rb, nb), rows) in &byrn {
        let r = f64::from_bits(*rb);
        let n = f64::from_bits(*nb);
        if tau_d(r, n).abs() >= 1.0 {
            continue;
        }
        let em_ex = match pin_em(rows, r) {
            Some(e) => e,
            None => continue,
        };
        tot += 1;
        let lt = tau_d(r, n).abs() < 0.6931471805599453;
        if lt {
            tot_lt += 1;
        }
        for (i, (_, f)) in cands.iter().enumerate() {
            if f(r, n).to_bits() == em_ex.to_bits() {
                sc[i] += 1;
                if lt {
                    sc_lt[i] += 1;
                }
            }
        }
    }
    println!(
        "po2n |tau|<1 pinned: {}   (|tau|<ln2 subset: {})",
        tot, tot_lt
    );
    for (i, (nm, _)) in cands.iter().enumerate() {
        println!(
            "  {:<18} {:>3}/{}   (|tau|<ln2: {}/{})",
            nm, sc[i], tot, sc_lt[i], tot_lt
        );
    }
}
