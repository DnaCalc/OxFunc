//! Score POISSON k=2 elementary forms vs b24 bank.

use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::discrete_dist_family::poisson_dist_kernel;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Bank {
    witnesses: Vec<W>,
}
#[derive(Deserialize)]
struct W {
    args: Vec<String>,
    expected_bits: String,
}

fn bits(s: &str) -> u64 {
    u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap()
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        "../../work/w109/G3-01-dist/answers-b24-poissonpdf.json".into()
    });
    let bank: Bank = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let mut tot = 0usize;
    let mut e = [0usize; 6];
    let names = [
        "production",
        "l*l/2*exp",
        "(l*l)*exp/2",
        "x87_mul(l*l/2, exp)",
        "exp(-l+2lnl-ln2)",
        "exp(-l)*l*l/2",
    ];
    for w in bank.witnesses {
        if w.args.len() < 3 || !w.expected_bits.starts_with("0x") {
            continue;
        }
        let x = f64::from_bits(bits(&w.args[0]));
        let mean = f64::from_bits(bits(&w.args[1]));
        let cum = f64::from_bits(bits(&w.args[2]));
        if cum != 0.0 || x.trunc() != 2.0 || mean <= 0.0 {
            continue;
        }
        let want = bits(&w.expected_bits);
        let eexp = rx::excel_exp(-mean);
        let cands = [
            poisson_dist_kernel(2.0, mean, false).unwrap(),
            mean * mean / 2.0 * eexp,
            (mean * mean) * eexp / 2.0,
            rx::x87_mul(mean * mean / 2.0, eexp),
            rx::excel_exp(-mean + 2.0 * mean.ln() - std::f64::consts::LN_2),
            eexp * mean * mean / 2.0,
        ];
        tot += 1;
        for (i, v) in cands.iter().enumerate() {
            if v.to_bits() == want {
                e[i] += 1;
            }
        }
    }
    println!("POISSON k=2 b24 n={tot}");
    for (i, name) in names.iter().enumerate() {
        println!("  {:>5}/{:<5}  {name}", e[i], tot);
    }
}
