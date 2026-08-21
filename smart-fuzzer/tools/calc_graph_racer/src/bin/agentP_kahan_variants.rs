//! agent-P: decode the Kahan expm1 chain for n=1 (t=-log1p(r), single transcendental).
//! em = f(u,t,lnu) where u=exp(t), lnu=ln(u). Test spill levels + operand orders.
//! Report exact-match vs the PINNED em oracle + signed-ulp residual hist.
use oxfunc_core::excel_numeric::research as rx;
use std::collections::BTreeMap;

const CW: u16 = rx::CW_PC64_RN;

fn fb(h: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(h.trim_start_matches("0x"), 16).unwrap())
}
fn ulp(a: f64, b: f64) -> i64 {
    a.to_bits() as i64 - b.to_bits() as i64
}
fn e(x: f64) -> rx::Ext80 {
    rx::ext_from_f64(x)
}
fn tf(x: &rx::Ext80) -> f64 {
    rx::ext_to_f64(x, CW)
}

// exp(arg) as extended Ext80 (fFEXP chain, no final f64 store), arg extended.
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
// ln(x) extended via fyl2x(ln2,x), x extended.
fn ln_ext(x: &rx::Ext80) -> rx::Ext80 {
    rx::ext_fyl2x(&rx::ext_ln2(), x, CW)
}

fn load_pinned(path: &str) -> Vec<(f64, i64, f64)> {
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
    let exact = *m.get(&0).unwrap_or(&0);
    print!(
        "  {:40} {:4}/{:4} ({:5.1}%)  ",
        name,
        exact,
        n,
        100.0 * exact as f64 / n as f64
    );
    // only show small-ulp buckets compactly
    let mut big = 0u32;
    for (k, c) in &m {
        if k.abs() <= 3 {
            print!("{}:{} ", k, c);
        } else {
            big += c;
        }
    }
    if big > 0 {
        print!(" |>3|:{}", big);
    }
    println!();
}

// t is the (negated log1p) argument as f64. Return em per each variant.
fn variants(t: f64) -> Vec<(&'static str, f64)> {
    // f64 u and lnu (stored)
    let u = rx::excel_exp(t);
    let um1 = u - 1.0; // exact (Sterbenz) when u in [0.5,2)
    let lnu = rx::excel_ln(u);
    let mut out: Vec<(&'static str, f64)> = Vec::new();
    if u == 1.0 {
        // all variants degenerate to t
        return vec![("u==1", t)];
    }
    // --- pure f64 orderings ---
    out.push(("f64 (u-1)*t/lnu", (um1 * t) / lnu));
    out.push(("f64 (u-1)/lnu*t", (um1 / lnu) * t));
    out.push(("f64 t/lnu*(u-1)", (t / lnu) * um1));
    out.push(("f64 t*(u-1)/lnu", (t * um1) / lnu));
    out.push(("f64 (u-1)*(t/lnu)", um1 * (t / lnu)));
    // --- extended spill, f64 u & lnu inputs, single final store ---
    let (ue, um1e, lnue) = (e(u), e(um1), e(lnu));
    let te = e(t);
    // ext (u-1)*t / lnu
    out.push((
        "ext[(u-1)*t/lnu] (f64 u,lnu)",
        tf(&rx::ext_div(&rx::ext_mul(&um1e, &te, CW), &lnue, CW)),
    ));
    out.push((
        "ext[(u-1)/lnu*t]",
        tf(&rx::ext_mul(&rx::ext_div(&um1e, &lnue, CW), &te, CW)),
    ));
    out.push((
        "ext[t/lnu*(u-1)]",
        tf(&rx::ext_mul(&rx::ext_div(&te, &lnue, CW), &um1e, CW)),
    ));
    // --- extended u, extended lnu (u kept extended from exp) ---
    // recompute t extended = t (already f64); exp on ext(t)
    let ue2 = exp_ext(&te);
    let um1e2 = rx::ext_sub(&ue2, &rx::ext_one(), CW);
    let lnue2 = ln_ext(&ue2);
    out.push((
        "ext all[(u-1)*t/lnu] (ext u,lnu)",
        tf(&rx::ext_div(&rx::ext_mul(&um1e2, &te, CW), &lnue2, CW)),
    ));
    // ext u stored to f64 for u-1 and lnu (u stored, but arithmetic extended)
    let _ = ue;
    out
}

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "../../work/w109/G6-solvers/pmt_em_pinned.json".into());
    let rows = load_pinned(&path);
    let n1: Vec<_> = rows.iter().filter(|(_, n, _)| *n == 1).cloned().collect();
    println!("n=1 pinned: {}", n1.len());

    // collect residuals per variant
    let names: Vec<&str> = variants(-rx::excel_log1p(n1[0].0))
        .iter()
        .map(|x| x.0)
        .collect();
    let mut res: Vec<Vec<i64>> = vec![Vec::new(); names.len()];
    for (r, _, em_x) in &n1 {
        let t = -rx::excel_log1p(*r);
        for (i, (_, em)) in variants(t).into_iter().enumerate() {
            res[i].push(ulp(em, *em_x));
        }
    }
    println!("\n=== n=1 Kahan-chain variants (candidate - excel ulp) ===");
    for (i, nm) in names.iter().enumerate() {
        hist(nm, &res[i]);
    }
}
