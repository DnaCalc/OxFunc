//! Dump Rust-side expm1 intermediates for |tau|<1 po2xn points so we can query
//! live Excel EXP(tau) / LN(u) and compare op-by-op against the pinned em.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use std::collections::BTreeMap;

fn tau_d(r: f64, n: f64) -> f64 {
    -(n * rx::excel_log1p(r))
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
    let mut s = String::from("k,n,tau_bits,u_bits,lnu_bits,em_pinned,em_aprod\n");
    for ((rb, nb), rows) in &byrn {
        let r = f64::from_bits(*rb);
        let n = f64::from_bits(*nb);
        let t = tau_d(r, n);
        if t.abs() >= 1.0 {
            continue;
        }
        let em_ex = match pin_em(rows, r) {
            Some(e) => e,
            None => continue,
        };
        let u = rx::excel_exp(t);
        let lnu = rx::excel_ln(u);
        let aprod = (u - 1.0) * t / lnu;
        s.push_str(&format!(
            "{},{},{:016x},{:016x},{:016x},{:016x},{:016x}\n",
            r.log2() as i64,
            n as i64,
            t.to_bits(),
            u.to_bits(),
            lnu.to_bits(),
            em_ex.to_bits(),
            aprod.to_bits()
        ));
    }
    std::fs::write("../../work/w109/G6-solvers/expm1_intermediates.csv", &s).unwrap();
    println!(
        "wrote expm1_intermediates.csv ({} lines)",
        s.lines().count() - 1
    );
}
