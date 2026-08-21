//! W109 G6-01: test whether the expm1 |tau|<1 Kahan-miss residual is REAL or a
//! po2 (r=2^-k) sampling artifact. Generic rates r=m*2^-k (m odd, 1+r still exact so
//! log1p=CR). Pin PMT's own em via 128-consecutive-pv intersection (r NOT exact ->
//! account for the *r rounding), compare to the all-double Kahan model, record the
//! signed residual structure.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use std::collections::BTreeMap;

fn kahan(r: f64, n: f64) -> f64 {
    rx::excel_expm1_internal(-(n * rx::excel_log1p(r)))
}
fn sordi(u: u64) -> i64 {
    if u < 1 << 63 {
        u as i64
    } else {
        -((u ^ (1u64 << 63)) as i64)
    }
}

// pin em at generic r: find em s.t. RN(RN(pv/em)*r)==pmt for ALL rows. Search around model.
fn pin_em(rows: &[(f64, u64)], r: f64, center: f64) -> Option<f64> {
    let cb = center.to_bits() as i64;
    let mut best: Option<(f64, u32)> = None;
    for d in -12..=12i64 {
        let em = f64::from_bits((cb + d) as u64);
        if em >= 0.0 {
            continue;
        }
        let ok = rows
            .iter()
            .filter(|(pv, want)| ((pv / em) * r).to_bits() == *want)
            .count() as u32;
        if best.map_or(true, |b| ok > b.1) {
            best = Some((em, ok));
        }
    }
    best.filter(|b| b.1 as usize == rows.len()).map(|b| b.0)
}

fn main() {
    let ws: WitnessSet = serde_json::from_str(
        &std::fs::read_to_string("../../work/w109/G6-solvers/answers-pmt-genrate.json").unwrap(),
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
        if a.len() != 5 {
            continue;
        }
        let want = parse_bits_hex(&w.expected_bits).unwrap().to_bits();
        byrn.entry((a[0].to_bits(), a[1].to_bits()))
            .or_default()
            .push((a[2], want));
    }
    let mut pinned = 0u32;
    let mut kmatch = 0u32;
    let mut unpinnable = 0u32;
    let mut resid: BTreeMap<i64, u32> = BTreeMap::new();
    let mut small_resid: BTreeMap<i64, u32> = BTreeMap::new(); // |tau|<0.1
    for ((rb, nb), rows) in &byrn {
        let r = f64::from_bits(*rb);
        let n = f64::from_bits(*nb);
        let tau = -(n * rx::excel_log1p(r));
        if tau.abs() >= 1.0 {
            continue;
        }
        let km = kahan(r, n);
        let em = match pin_em(rows, r, km) {
            Some(e) => e,
            None => {
                unpinnable += 1;
                continue;
            }
        };
        pinned += 1;
        let d = (sordi(em.to_bits()) - sordi(km.to_bits())).clamp(-3, 3);
        *resid.entry(d).or_default() += 1;
        if tau.abs() < 0.1 {
            *small_resid.entry(d).or_default() += 1;
        }
        if d == 0 {
            kmatch += 1;
        }
    }
    println!(
        "generic-rate (r=m*2^-k) |tau|<1: pinned {}, unpinnable {}",
        pinned, unpinnable
    );
    println!(
        "Kahan-model matches: {}/{} ({:.0}%)",
        kmatch,
        pinned,
        100.0 * kmatch as f64 / pinned as f64
    );
    println!(
        "signed residual (em_pinned - Kahan, real-value ulps): {:?}",
        resid.iter().map(|(k, v)| (*k, *v)).collect::<Vec<_>>()
    );
    println!(
        "  small |tau|<0.1 subset: {:?}",
        small_resid
            .iter()
            .map(|(k, v)| (*k, *v))
            .collect::<Vec<_>>()
    );
    println!("(if ~30% miss with toward-zero bias persists -> REAL; if ~0% miss -> po2 artifact)");
}
