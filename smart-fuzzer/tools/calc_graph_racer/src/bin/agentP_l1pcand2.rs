//! More log1p candidate routines vs extracted n=1 em: Cephes rational, and
//! x87 fyl2xp1 with PC53 (double-precision) control word.
use oxfunc_core::excel_numeric::research as rx;
use std::collections::BTreeMap;
const CW: u16 = rx::CW_PC64_RN;
const CW53: u16 = rx::CW_PC53_RN;
fn fb(h: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(h.trim_start_matches("0x"), 16).unwrap())
}
fn e(x: f64) -> rx::Ext80 {
    rx::ext_from_f64(x)
}
fn kahan_f64(tau: f64) -> f64 {
    let u = rx::excel_exp(tau);
    let l = rx::excel_ln(u);
    if u == 1.0 {
        tau
    } else if tau.abs() < 1.0 {
        (u - 1.0) * tau / l
    } else {
        u - 1.0
    }
}
fn polevl(x: f64, c: &[f64]) -> f64 {
    let mut a = c[0];
    for &ci in &c[1..] {
        a = a * x + ci;
    }
    a
}
fn p1evl(x: f64, c: &[f64]) -> f64 {
    let mut a = x + c[0];
    for &ci in &c[1..] {
        a = a * x + ci;
    }
    a
}
const LP: [f64; 7] = [
    4.5270000862445199635215E-5,
    4.9854102823193375972212E-1,
    6.5787325942061044846969E0,
    2.9911919328553073277375E1,
    6.0949667980987787057556E1,
    5.7112963590585538103336E1,
    2.0039553499201281259648E1,
];
const LQ: [f64; 6] = [
    1.5062909083469192043167E1,
    8.3047565967967209469434E1,
    2.2176239823732856465394E2,
    3.0909872225312059774938E2,
    2.1642788614495947685003E2,
    6.0118660497603843919306E1,
];
fn cephes_log1p(x: f64) -> f64 {
    let z = 1.0 + x;
    if z < 0.70710678118654752440 || z > 1.41421356237309504880 {
        return rx::excel_ln(z);
    }
    let z2 = x * x;
    let zz = -0.5 * z2 + x * (z2 * polevl(x, &LP) / p1evl(x, &LQ));
    x + zz
}
// fyl2xp1 with PC=53 control word (double-extended reduced to double precision internally)
fn fyl2xp1_pc53(r: f64) -> f64 {
    rx::ext_to_f64(&rx::ext_fyl2xp1(&rx::ext_ln2(), &e(r), CW53), CW53)
}
fn main() {
    let v: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string("../../work/w109/G6-solvers/agentP_log1p_em.json").unwrap(),
    )
    .unwrap();
    let mut rows: Vec<(f64, f64)> = v
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, val)| (fb(k), fb(val.as_str().unwrap())))
        .collect();
    rows.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let n = rows.len();
    let mut res: BTreeMap<&str, u32> = BTreeMap::new();
    for (r, em) in &rows {
        if kahan_f64(-cephes_log1p(*r)).to_bits() == em.to_bits() {
            *res.entry("Cephes rational log1p").or_default() += 1;
        }
        if kahan_f64(-fyl2xp1_pc53(*r)).to_bits() == em.to_bits() {
            *res.entry("fyl2xp1 PC53").or_default() += 1;
        }
        // fyl2x on (1+r) at PC53
        let l = rx::ext_to_f64(&rx::ext_fyl2x(&rx::ext_ln2(), &e(1.0 + *r), CW53), CW53);
        if kahan_f64(-l).to_bits() == em.to_bits() {
            *res.entry("ln(1+r) PC53").or_default() += 1;
        }
    }
    println!("candidates vs extracted em ({} r):", n);
    for (k, c) in &res {
        println!("  {:24} {:3}/{}", k, c, n);
    }
}
