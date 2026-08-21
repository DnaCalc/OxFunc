//! W109 G6-01: the Kahan expm1 correction denominator. exp & LN(u) are PROVEN
//! bit-exact to Excel, yet (u-1)*t/LN(u) only makes 163/234. Hypothesis: the
//! correction uses ln(u)=log1p(u-1) (u-1 EXACT) via FYL2XP1, not fyl2x(u), which
//! differs when u~=1. Race denominator variants; also numerator associations.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{CW_PC64_RN as CW, ext_from_f64 as e, ext_fyl2xp1, ext_ln2, ext_to_f64};
use std::collections::BTreeMap;

fn tau_d(r: f64, n: f64) -> f64 {
    -(n * rx::excel_log1p(r))
}
// ln(u) deliveries:
fn lnu_fyl2x(u: f64) -> f64 {
    rx::excel_ln(u)
} // fyl2x(u) [current]
fn lnu_fyl2xp1(u: f64) -> f64 {
    ext_to_f64(&ext_fyl2xp1(&ext_ln2(), &e(u - 1.0), CW), CW)
} // log1p(u-1) x87
fn lnu_log1pcr(u: f64) -> f64 {
    rx::excel_log1p(u - 1.0)
} // log1p(u-1) portable CR

// em forms, parameterized by which ln(u) and which association
fn em(r: f64, n: f64, lnu: fn(f64) -> f64, assoc: u8) -> f64 {
    let t = tau_d(r, n);
    let u = rx::excel_exp(t);
    if u == 1.0 {
        return t;
    }
    if t.abs() >= 1.0 {
        return u - 1.0;
    }
    let d = lnu(u);
    match assoc {
        0 => (u - 1.0) * t / d,   // prod-first
        1 => (u - 1.0) * (t / d), // kahan canonical
        2 => (t / d) * (u - 1.0), // divide-first
        _ => (u - 1.0) * t / d,
    }
}

fn sordi(u: u64) -> i64 {
    if u < 1 << 63 {
        u as i64
    } else {
        -((u ^ (1u64 << 63)) as i64)
    }
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
    let denoms: [(&str, fn(f64) -> f64); 3] = [
        ("fyl2x(u)", lnu_fyl2x),
        ("fyl2xp1(u-1)", lnu_fyl2xp1),
        ("log1pCR(u-1)", lnu_log1pcr),
    ];
    let mut tot = 0u32;
    let mut score = [[0u32; 3]; 3];
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
        for (di, (_, ld)) in denoms.iter().enumerate() {
            for asc in 0..3u8 {
                if em(r, n, *ld, asc).to_bits() == em_ex.to_bits() {
                    score[di][asc as usize] += 1;
                }
            }
        }
    }
    println!("|tau|<1 pinned: {}", tot);
    println!(
        "{:<16} {:>10} {:>10} {:>10}",
        "denom\\assoc", "prod", "kahan", "divfirst"
    );
    for (di, (nm, _)) in denoms.iter().enumerate() {
        println!(
            "{:<16} {:>10} {:>10} {:>10}",
            nm, score[di][0], score[di][1], score[di][2]
        );
    }
}
