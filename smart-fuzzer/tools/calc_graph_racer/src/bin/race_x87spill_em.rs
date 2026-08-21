//! W109 G6-01 SLP search: the x87-SPILL Kahan expm1. Precedent: XNPV and legacy
//! financial bodies are per-op DOUBLE-ROUNDED x87 (compute at PC=64 -> RN64, store
//! to binary64 -> RN53). Test the Kahan (u-1)*tau/ln(u) with each op x87-double-rounded
//! vs SSE2 single-rounded, all combinations, incl. tau=-n*log1p double-rounded.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{CW_PC64_RN as CW, ext_div, ext_from_f64 as e, ext_mul, ext_to_f64};
use std::collections::BTreeMap;

// x87 double-rounded ops: compute at PC=64 (RN64) then store to binary64 (RN53).
fn dmul(a: f64, b: f64) -> f64 {
    ext_to_f64(&ext_mul(&e(a), &e(b), CW), CW)
}
fn ddiv(a: f64, b: f64) -> f64 {
    ext_to_f64(&ext_div(&e(a), &e(b), CW), CW)
}

fn tau_sse(r: f64, n: f64) -> f64 {
    -(n * rx::excel_log1p(r))
}
fn tau_x87(r: f64, n: f64) -> f64 {
    -dmul(n, rx::excel_log1p(r))
} // n*log1p double-rounded, negate

// Kahan variants: bits = (num_dr, div_dr, tau_dr)
fn em(r: f64, n: f64, num_dr: bool, div_dr: bool, tau_dr: bool) -> f64 {
    let t = if tau_dr { tau_x87(r, n) } else { tau_sse(r, n) };
    let u = rx::excel_exp(t);
    if u == 1.0 {
        return t;
    }
    if t.abs() >= 1.0 {
        return u - 1.0;
    }
    let b = u - 1.0; // exact
    let lnu = rx::excel_ln(u);
    let num = if num_dr { dmul(b, t) } else { b * t };
    if div_dr { ddiv(num, lnu) } else { num / lnu }
}

fn sordi(u: u64) -> i64 {
    if u < 1 << 63 {
        u as i64
    } else {
        -((u ^ (1u64 << 63)) as i64)
    }
}
fn pin_po2(rows: &[(f64, u64)], r: f64) -> Option<f64> {
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
fn pin_gen(rows: &[(f64, u64)], r: f64, center: f64) -> Option<f64> {
    let cb = center.to_bits() as i64;
    for d in -12..=12i64 {
        let em = f64::from_bits((cb + d) as u64);
        if em >= 0.0 {
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
fn load(p: &str) -> BTreeMap<(u64, u64), Vec<(f64, u64)>> {
    let ws: WitnessSet = serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap();
    let mut m = BTreeMap::new();
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
        m.entry((a[0].to_bits(), a[1].to_bits()))
            .or_insert_with(Vec::new)
            .push((a[2], want));
    }
    m
}
fn main() {
    let variants = [
        ("sse(base)", false, false, false),
        ("num_dr", true, false, false),
        ("div_dr", false, true, false),
        ("num+div_dr", true, true, false),
        ("tau_dr", false, false, true),
        ("num+tau_dr", true, false, true),
        ("num+div+tau_dr", true, true, true),
        ("div+tau_dr", false, true, true),
    ];
    for (src, path, po2) in [
        (
            "po2n",
            "../../work/w109/G6-solvers/answers-pmt-po2n.json",
            true,
        ),
        (
            "gen",
            "../../work/w109/G6-solvers/answers-pmt-genrate.json",
            false,
        ),
    ] {
        let data = load(path);
        let mut pins: Vec<(f64, f64, f64)> = Vec::new(); // (r,n,em_pinned)
        for ((rb, nb), rows) in &data {
            let r = f64::from_bits(*rb);
            let n = f64::from_bits(*nb);
            if tau_sse(r, n).abs() >= 1.0 {
                continue;
            }
            let km = em(r, n, false, false, false);
            let p = if po2 {
                pin_po2(rows, r)
            } else {
                pin_gen(rows, r, km)
            };
            if let Some(e) = p {
                pins.push((r, n, e));
            }
        }
        println!("=== {} ({} |tau|<1 pinned) ===", src, pins.len());
        for (nm, a, b, c) in variants {
            let ok = pins
                .iter()
                .filter(|(r, n, ep)| em(*r, *n, a, b, c).to_bits() == ep.to_bits())
                .count();
            // signed residual dist for this variant
            let mut plus = 0;
            let mut minus = 0;
            for (r, n, ep) in &pins {
                let d = sordi(em(*r, *n, a, b, c).to_bits()) - sordi(ep.to_bits());
                if d > 0 {
                    plus += 1
                } else if d < 0 {
                    minus += 1
                }
            }
            println!(
                "  {:<16} {:>3}/{}   (miss +{} / -{})",
                nm,
                ok,
                pins.len(),
                plus,
                minus
            );
        }
    }
}
