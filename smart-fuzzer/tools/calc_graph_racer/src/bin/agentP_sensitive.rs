//! agent-P: decode tau = -n*log1p(r) on the SENSITIVE lane (tau>2, r<0, em large).
//! Here em = exp(tau)-1 ~ exp(tau), hypersensitive to tau's exact bits, so the
//! tau-delivery op-graph (log1p + the *n product + store rounding) is pinned by
//! exact-match. em store insensitive to the combine op (em large). GOLD pv=1.
use oxfunc_core::excel_numeric::research as rx;
use std::collections::BTreeMap;

const CW: u16 = rx::CW_PC64_RN;

fn fb(h: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(h.trim_start_matches("0x"), 16).unwrap())
}
fn e(x: f64) -> rx::Ext80 {
    rx::ext_from_f64(x)
}
fn tf(x: &rx::Ext80) -> f64 {
    rx::ext_to_f64(x, CW)
}
fn load(path: &str) -> Vec<(f64, i64, f64)> {
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("read")).expect("json");
    let mut out = Vec::new();
    for (k, val) in v.as_object().unwrap() {
        if val.is_null() {
            continue;
        }
        let (rh, nh) = k.split_once('|').unwrap();
        out.push((fb(rh), nh.parse().unwrap(), fb(val.as_str().unwrap())));
    }
    out
}
fn hist(name: &str, v: &[i64]) {
    let mut m: BTreeMap<i64, u32> = BTreeMap::new();
    for &x in v {
        *m.entry(x).or_default() += 1;
    }
    let n = v.len();
    if n == 0 {
        println!("  {:40} (0)", name);
        return;
    }
    let exact = *m.get(&0).unwrap_or(&0);
    print!(
        "  {:40} {:4}/{:4} ({:5.1}%)  ",
        name,
        exact,
        n,
        100.0 * exact as f64 / n as f64
    );
    let mut big = 0;
    for (k, c) in &m {
        if k.abs() <= 2 {
            print!("{}:{} ", k, c);
        } else {
            big += c;
        }
    }
    if big > 0 {
        print!("|>2|:{}", big);
    }
    println!();
}
fn log1p_ext(r: f64) -> rx::Ext80 {
    rx::ext_fyl2xp1(&rx::ext_ln2(), &e(r), CW)
}
fn exp_ext(arg: &rx::Ext80) -> rx::Ext80 {
    let z = rx::ext_mul(arg, &rx::ext_l2e(), CW);
    let k = rx::ext_rndint(&z, CW);
    let f = rx::ext_sub(&z, &k, CW);
    let neg = tf(&f) < 0.0;
    let w = rx::ext_f2xm1(&rx::ext_abs(&f, CW), CW);
    let mut m = rx::ext_add(&w, &rx::ext_one(), CW);
    if neg {
        m = rx::ext_div(&rx::ext_one(), &m, CW);
    }
    rx::ext_scale(&m, &k, CW)
}

fn main() {
    let rows = load("../../work/w109/G6-solvers/pmt_em_gold_pv1.json");
    let sens: Vec<_> = rows
        .iter()
        .filter(|(r, n, _)| -(*n as f64) * rx::excel_log1p(*r) > 2.0)
        .cloned()
        .collect();
    println!(
        "sensitive lane (tau=-n*log1p(r) > 2, em large): {} rows",
        sens.len()
    );

    // Each candidate produces em; compare to gold. Enumerate tau-delivery x em-store.
    // tau deliveries as either f64 or Ext80; em = exp(tau)-1.
    println!("\n=== em = exp(tau)-1, tau-delivery matrix vs gold ===");

    // helper closures for tau (f64 stored)
    let deliveries: Vec<(&str, Box<dyn Fn(f64, f64) -> f64>)> = vec![
        // log1p portable, product f64 (SSE)
        (
            "Lport, P=f64(-n*L)",
            Box::new(|r, n| -n * rx::excel_log1p(r)),
        ),
        // log1p portable, product x87 double-rounded
        (
            "Lport, P=x87dr(-n*L)",
            Box::new(|r, n| tf(&rx::ext_mul(&e(-n), &e(rx::excel_log1p(r)), CW))),
        ),
        // log1p fyl2xp1 stored, product f64
        (
            "Lf2xp1st, P=f64(-n*L)",
            Box::new(|r, n| -n * tf(&log1p_ext(r))),
        ),
        // log1p fyl2xp1 stored, product x87 dr
        (
            "Lf2xp1st, P=x87dr",
            Box::new(|r, n| tf(&rx::ext_mul(&e(-n), &log1p_ext(r), CW))),
        ),
        // log1p fyl2xp1 EXTENDED, product extended then stored to f64
        (
            "Lf2xp1ext, P=ext->f64",
            Box::new(|r, n| tf(&rx::ext_mul(&e(-n), &log1p_ext(r), CW))),
        ),
    ];
    for (dn, df) in &deliveries {
        // em = exp(tau_f64) - 1  (f64)
        let mut r_f = Vec::new();
        for (r, n, em_x) in &sens {
            let tau = df(*r, *n as f64);
            let em = rx::excel_exp(tau) - 1.0;
            r_f.push(em.to_bits() as i64 - em_x.to_bits() as i64);
        }
        hist(&format!("{}  [exp(tau_f64)-1]", dn), &r_f);
    }

    // Fully-extended tau kept into exp (log1p ext, product ext, NO store of tau)
    println!("\n=== tau kept EXTENDED into exp (no tau store) ===");
    let mut r_ee = Vec::new(); // exp(tau_ext) -> f64 store, then -1 f64
    let mut r_eem = Vec::new(); // exp(tau_ext), RN53(u_ext - 1)
    for (r, n, em_x) in &sens {
        let tau_ext = rx::ext_mul(&e(-(*n as f64)), &log1p_ext(*r), CW);
        let uext = exp_ext(&tau_ext);
        let u = tf(&uext);
        r_ee.push((u - 1.0).to_bits() as i64 - em_x.to_bits() as i64);
        r_eem.push(
            tf(&rx::ext_sub(&uext, &rx::ext_one(), CW)).to_bits() as i64 - em_x.to_bits() as i64,
        );
    }
    hist("extTau->exp; u(f64)-1", &r_ee);
    hist("extTau->exp; RN53(u_ext-1)", &r_eem);

    // Also: n as exact integer times L (n is integer!). product could be n*L with n exact.
    // Test using ext with -n exact vs f64 n (same, n<2^53). Also test double-rounded exp.
    println!("\n=== exp store variants on best f64 tau (Lport,P=f64) ===");
    let mut r_expdr = Vec::new(); // exp x87 double-rounded? (already RN53). test exp_rz
    for (r, n, em_x) in &sens {
        let tau = -(*n as f64) * rx::excel_log1p(*r);
        // em = RN53(exp_ext(tau) - 1) fully extended tail
        let uext = exp_ext(&e(tau));
        let em = tf(&rx::ext_sub(&uext, &rx::ext_one(), CW));
        r_expdr.push(em.to_bits() as i64 - em_x.to_bits() as i64);
    }
    hist("exp_ext(tau_f64); RN53(u_ext-1)", &r_expdr);
}
