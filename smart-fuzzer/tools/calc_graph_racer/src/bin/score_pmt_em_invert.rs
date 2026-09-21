//! Invert Excel PMT (fv=0,type=0) to implied em and score Kahan variants.

use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::financial_time_value_family::{pmt, PaymentTiming};
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Row {
    r: f64,
    nper: f64,
    pv: f64,
    pmt_bits: String,
}

fn bits(s: &str) -> u64 {
    u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap()
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        "../../work/w109/lastbit-12h-20260912/pmt-em/capture.jsonl".into()
    });
    let mut prod = (0usize, 0usize);
    let mut em_match = (0usize, 0usize);
    let mut tau_lt = (0usize, 0usize, 0usize); // prod exact, em exact, n
    let names = [
        "prod PMT",
        "implied em vs kahan (u-1)*t/lnu",
        "implied vs t*(u-1)/lnu",
        "implied vs (u-1)*(t/lnu)",
        "implied vs x87 ((u-1)*t)/lnu",
        "implied vs x87 (u-1)*(t/lnu)",
        "PMT from implied-em reconstruct",
    ];
    let mut exact = [0usize; 7];
    let mut tot = 0usize;
    for line in fs::read_to_string(&path).unwrap().lines() {
        if line.is_empty() {
            continue;
        }
        let row: Row = serde_json::from_str(line).unwrap();
        let want = bits(&row.pmt_bits);
        let Ok(got) = pmt(
            row.r,
            row.nper,
            row.pv,
            0.0,
            PaymentTiming::EndOfPeriod,
        ) else {
            continue;
        };
        tot += 1;
        if got.to_bits() == want {
            exact[0] += 1;
            prod.0 += 1;
        }
        prod.1 += 1;
        let tau = -(row.nper * rx::excel_log1p(row.r));
        let small = tau.abs() < 1.0;
        // implied em from PMT = pv * r / pmt   (fv=0,type=0)
        let excel_pmt = f64::from_bits(want);
        let implied = (row.pv * row.r) / excel_pmt;
        let u = rx::excel_exp(tau);
        let lnu = rx::excel_ln(u);
        let kahan = (u - 1.0) * tau / lnu;
        let k2 = tau * (u - 1.0) / lnu;
        let k3 = (u - 1.0) * (tau / lnu);
        let k4 = rx::x87_mul(u - 1.0, tau) / lnu;
        let k5 = rx::x87_mul(u - 1.0, tau / lnu);
        let cands = [kahan, k2, k3, k4, k5];
        for (i, c) in cands.iter().enumerate() {
            if c.to_bits() == implied.to_bits() {
                exact[i + 1] += 1;
            }
        }
        if kahan.to_bits() == implied.to_bits() {
            em_match.0 += 1;
        }
        em_match.1 += 1;
        // reconstruct PMT from kahan em using production combine
        let recon = ((row.pv / kahan) / 1.0) * row.r;
        if recon.to_bits() == want {
            exact[6] += 1;
        }
        if small {
            tau_lt.2 += 1;
            if got.to_bits() == want {
                tau_lt.0 += 1;
            }
            if kahan.to_bits() == implied.to_bits() {
                tau_lt.1 += 1;
            }
        }
    }
    println!("PMT fv=0 type=0 vs Excel n={tot}");
    for (i, name) in names.iter().enumerate() {
        println!("  {:>5}/{:<5}  {name}", exact[i], tot);
    }
    println!(
        "|tau|<1 prod {}/{}  implied-em==kahan {}/{}",
        tau_lt.0, tau_lt.2, tau_lt.1, tau_lt.2
    );
}
