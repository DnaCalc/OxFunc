//! W109 G6-01 LOG1P LANE - TAU-LEVEL clean-function test.
//! The collide families reveal em varies by 1-3 CONSECUTIVE ULP within a CR-collapsed
//! tau group => the discriminator is sub-ULP of tau. tau = -n*log1p(r), and the
//! MULTIPLY substrate (SSE2 double vs x87 RN53(RN64) vs x87-extended-ln1p-into-product)
//! shifts the rounding boundary. Race (log1p routine) x (multiply substrate) with the
//! CONFOUND-FREE clean-function test: same tau_double MUST share em_pinned. Winner=0 conflicts.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{Ext80, ext_from_f64 as ef, ext_fyl2xp1, ext_fyl2x, ext_ln2, ext_add, ext_one, ext_mul,
         ext_chs, ext_to_f64, CW_PC64_RN, CW_PC53_RN};
use std::collections::BTreeMap;

fn l1p_cr(r: f64) -> f64 { rx::excel_log1p(r) }
fn l1p_std(r: f64) -> f64 { r.ln_1p() }
fn l1p_fyl_dbl(r: f64) -> f64 {
    if r.abs() < 0.292893218813452 { ext_to_f64(&ext_fyl2xp1(&ext_ln2(), &ef(r), CW_PC64_RN), CW_PC64_RN) }
    else { ext_to_f64(&ext_fyl2x(&ext_ln2(), &ext_add(&ext_one(), &ef(r), CW_PC64_RN), CW_PC64_RN), CW_PC64_RN) }
}
fn ln1p_ext(r: f64) -> Ext80 {
    if r.abs() < 0.292893218813452 { ext_fyl2xp1(&ext_ln2(), &ef(r), CW_PC64_RN) }
    else { ext_fyl2x(&ext_ln2(), &ext_add(&ext_one(), &ef(r), CW_PC64_RN), CW_PC64_RN) }
}

// tau candidates: fn(r,n) -> f64 (the rounded double tau, sign already negated)
type TauFn = fn(f64, f64) -> f64;

// SSE2 double multiply then negate (current model)
fn tau_sse2(r: f64, n: f64, l1p: fn(f64) -> f64) -> f64 { -(n * l1p(r)) }
fn t_sse2_cr(r: f64, n: f64) -> f64 { tau_sse2(r, n, l1p_cr) }
fn t_sse2_ucrt(r: f64, n: f64) -> f64 { tau_sse2(r, n, l1p_std) }
fn t_sse2_fyl(r: f64, n: f64) -> f64 { tau_sse2(r, n, l1p_fyl_dbl) }

// x87 double-rounded multiply RN53(RN64(n*l1p)) then negate exact
fn tau_x87dr(r: f64, n: f64, l1p: fn(f64) -> f64) -> f64 {
    let p = ext_mul(&ef(n), &ef(l1p(r)), CW_PC64_RN); // RN64
    -ext_to_f64(&p, CW_PC64_RN) // RN53 store, then negate
}
fn t_x87dr_cr(r: f64, n: f64) -> f64 { tau_x87dr(r, n, l1p_cr) }
fn t_x87dr_ucrt(r: f64, n: f64) -> f64 { tau_x87dr(r, n, l1p_std) }
fn t_x87dr_fyldbl(r: f64, n: f64) -> f64 { tau_x87dr(r, n, l1p_fyl_dbl) }

// x87 PC53 multiply (single-rounded at 53): RN53(n*l1p) - if the body ran at PC=53
fn tau_x87_pc53(r: f64, n: f64, l1p: fn(f64) -> f64) -> f64 {
    let p = ext_mul(&ef(n), &ef(l1p(r)), CW_PC53_RN);
    -ext_to_f64(&p, CW_PC53_RN)
}
fn t_x87pc53_cr(r: f64, n: f64) -> f64 { tau_x87_pc53(r, n, l1p_cr) }
fn t_x87pc53_ucrt(r: f64, n: f64) -> f64 { tau_x87_pc53(r, n, l1p_std) }

// FULL x87 chain: ln1p kept EXTENDED (64-bit) into the product, only final store rounds.
fn t_x87_extfyl(r: f64, n: f64) -> f64 {
    let p = ext_mul(&ef(n), &ln1p_ext(r), CW_PC64_RN);
    ext_to_f64(&ext_chs(&p, CW_PC64_RN), CW_PC64_RN)
}
// same but PC53 final store from a PC64 product (double-round)
fn t_x87_extfyl_dr(r: f64, n: f64) -> f64 {
    let p = ext_mul(&ef(n), &ln1p_ext(r), CW_PC64_RN);
    ext_to_f64(&ext_chs(&p, CW_PC64_RN), CW_PC53_RN)
}

fn pin_gen(rows: &[(f64, u64)], r: f64, center: f64) -> Option<f64> {
    let cb = center.to_bits() as i64;
    for d in -48..=48i64 {
        let em = f64::from_bits((cb + d) as u64);
        if em >= 0.0 { continue }
        if rows.iter().all(|(pv, want)| ((pv / em) * r).to_bits() == *want) { return Some(em); }
    }
    None
}
fn load(p: &str) -> BTreeMap<(u64, u64), Vec<(f64, u64)>> {
    let ws: WitnessSet = serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap();
    let mut m = BTreeMap::new();
    for w in &ws.witnesses {
        let a: Vec<f64> = w.args.iter().filter_map(|x| match x { WitnessArg::Scalar(s) => parse_bits_hex(s), _ => None }).collect();
        if a.len() != 5 || a[3] != 0.0 || a[4] != 0.0 { continue }
        let want = parse_bits_hex(&w.expected_bits).unwrap().to_bits();
        m.entry((a[0].to_bits(), a[1].to_bits())).or_insert_with(Vec::new).push((a[2], want));
    }
    m
}

fn main() {
    let cands: [(&str, TauFn); 10] = [
        ("sse2_cr", t_sse2_cr), ("sse2_ucrt", t_sse2_ucrt), ("sse2_fyl", t_sse2_fyl),
        ("x87dr_cr", t_x87dr_cr), ("x87dr_ucrt", t_x87dr_ucrt), ("x87dr_fyldbl", t_x87dr_fyldbl),
        ("x87pc53_cr", t_x87pc53_cr), ("x87pc53_ucrt", t_x87pc53_ucrt),
        ("x87extfyl", t_x87_extfyl), ("x87extfyl_dr", t_x87_extfyl_dr),
    ];
    // pin em on collide
    let data = load("../../work/w109/G6-solvers/answers-pmt-collide.json");
    let mut pins: Vec<(f64, f64, u64)> = Vec::new();
    for ((rb, nb), rows) in &data {
        let r = f64::from_bits(*rb); let n = f64::from_bits(*nb);
        let km = rx::excel_expm1_internal(-(n * rx::excel_log1p(r)));
        if let Some(e) = pin_gen(rows, r, km) { pins.push((r, n, e.to_bits())); }
    }
    println!("collide pinned: {}", pins.len());
    println!("{:<14} {:>7} {:>8} {:>9} {:>9}", "tau-cand", "groups", "conflG", "conflCfg", "emRepro");
    for (nm, tf) in cands {
        let mut groups: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
        let mut repro = 0usize;
        for (r, n, ep) in &pins {
            let tau = tf(*r, *n);
            groups.entry(tau.to_bits()).or_default().push(*ep);
            if rx::excel_expm1_internal(tau).to_bits() == *ep { repro += 1; }
        }
        let mut cg = 0; let mut cc = 0;
        for (_t, ems) in &groups {
            let d: std::collections::BTreeSet<u64> = ems.iter().copied().collect();
            if d.len() > 1 { cg += 1; cc += ems.len(); }
        }
        println!("{:<14} {:>7} {:>8} {:>9} {:>7}/{}", nm, groups.len(), cg, cc, repro, pins.len());
    }
}
