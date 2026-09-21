//! Score production PMT vs answers-pmt-em.json and dump |tau|<1 misses.

use oxfunc_core::excel_numeric::research::{excel_expm1_internal, excel_log1p};
use oxfunc_core::functions::financial_time_value_family::{pmt, PaymentTiming};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;

#[derive(Deserialize)]
struct Bank {
    witnesses: Vec<W>,
}
#[derive(Deserialize)]
struct W {
    id: String,
    args: Vec<String>,
    expected_bits: String,
}

fn bits(s: &str) -> u64 {
    u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap()
}
fn sulp(got: u64, want: u64) -> i64 {
    fn ord(v: u64) -> u64 {
        if v >> 63 == 0 {
            v | (1u64 << 63)
        } else {
            !v
        }
    }
    (i128::from(ord(got)) - i128::from(ord(want))) as i64
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        "../../work/w109/G6-solvers/answers-pmt-em.json".into()
    });
    let bank: Bank = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let mut hist: BTreeMap<i64, usize> = BTreeMap::new();
    let mut exact = 0;
    let mut n = 0;
    let mut tau_lt1 = 0;
    let mut tau_lt1_exact = 0;
    let mut tau_ge1 = 0;
    let mut tau_ge1_exact = 0;
    let mut miss_lt1 = Vec::new();
    for w in bank.witnesses {
        if w.args.len() < 4 {
            continue;
        }
        let r = f64::from_bits(bits(&w.args[0]));
        let nper = f64::from_bits(bits(&w.args[1]));
        let pv = f64::from_bits(bits(&w.args[2]));
        let fv = f64::from_bits(bits(&w.args[3]));
        let ty = if w.args.len() > 4 {
            f64::from_bits(bits(&w.args[4]))
        } else {
            0.0
        };
        let timing = if ty == 0.0 {
            PaymentTiming::EndOfPeriod
        } else {
            PaymentTiming::BeginningOfPeriod
        };
        let Some(want) = (w.expected_bits.starts_with("0x") && w.expected_bits.len() == 18)
            .then(|| bits(&w.expected_bits))
        else {
            continue;
        };
        let Ok(got) = pmt(r, nper, pv, fv, timing) else {
            continue;
        };
        n += 1;
        let d = sulp(got.to_bits(), want);
        *hist.entry(d.clamp(-8, 8)).or_default() += 1;
        if d == 0 {
            exact += 1;
        }
        let tau = -(nper * excel_log1p(r)).abs();
        let small = tau.abs() < 1.0;
        if small {
            tau_lt1 += 1;
            if d == 0 {
                tau_lt1_exact += 1;
            } else if miss_lt1.len() < 12 {
                miss_lt1.push((w.id, r, nper, pv, fv, ty, d, tau));
            }
        } else {
            tau_ge1 += 1;
            if d == 0 {
                tau_ge1_exact += 1;
            }
        }
    }
    println!("n={n} exact={exact} hist={hist:?}");
    println!("|tau|<1 {tau_lt1_exact}/{tau_lt1}");
    println!("|tau|>=1 {tau_ge1_exact}/{tau_ge1}");
    println!("sample |tau|<1 misses:");
    for m in &miss_lt1 {
        println!("  {m:?}");
    }

    let mut em_exact = 0;
    let mut em_n = 0;
    let mut em_hist: BTreeMap<i64, usize> = BTreeMap::new();
    let bank: Bank = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    for w in bank.witnesses {
        if w.args.len() < 4 {
            continue;
        }
        let r = f64::from_bits(bits(&w.args[0]));
        let nper = f64::from_bits(bits(&w.args[1]));
        let pv = f64::from_bits(bits(&w.args[2]));
        let fv = f64::from_bits(bits(&w.args[3]));
        let ty = if w.args.len() > 4 {
            f64::from_bits(bits(&w.args[4]))
        } else {
            0.0
        };
        if r == 0.0 || !w.expected_bits.starts_with("0x") {
            continue;
        }
        let pmt_x = f64::from_bits(bits(&w.expected_bits));
        let tf = 1.0 + r * ty;
        let den = pmt_x * tf / r - fv;
        if den == 0.0 || !den.is_finite() {
            continue;
        }
        let implied = (pv + fv) / den;
        let tau = -(nper * excel_log1p(r));
        if tau.abs() >= 1.0 {
            continue;
        }
        let model = excel_expm1_internal(tau);
        if !implied.is_finite() || !model.is_finite() {
            continue;
        }
        em_n += 1;
        let d = sulp(model.to_bits(), implied.to_bits());
        *em_hist.entry(d.clamp(-8, 8)).or_default() += 1;
        if d == 0 {
            em_exact += 1;
        }
    }
    println!("implied-em vs excel_expm1_internal |tau|<1 exact {em_exact}/{em_n} hist={em_hist:?}");
}
