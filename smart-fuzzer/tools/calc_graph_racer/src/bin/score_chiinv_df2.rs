//! Score CHIINV df=2 vs -2*ln forms.

use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::chi_f_t_family::{chisq_inv_kernel, chisq_inv_rt_kernel};
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Row {
    p: f64,
    rt_bits: String,
    inv_bits: String,
    chiinv_bits: String,
}

fn bits(s: &str) -> u64 {
    u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap()
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        "../../work/w109/lastbit-12h-20260912/chiinv-df2/capture.jsonl".into()
    });
    let mut tot = 0usize;
    let mut e = [0usize; 8];
    let names = [
        "prod INV.RT",
        "prod INV",
        "-2*excel_ln(p)",
        "x87_mul(-2, excel_ln p)",
        "-2*native ln p",
        "-2*excel_ln(1-p)",
        "x87_mul(-2, excel_ln(1-p))",
        "prod CHIINV alias",
    ];
    for line in fs::read_to_string(&path).unwrap().lines() {
        if line.is_empty() {
            continue;
        }
        let r: Row = serde_json::from_str(line).unwrap();
        let want_rt = bits(&r.rt_bits);
        let want_inv = bits(&r.inv_bits);
        let ln = rx::excel_ln(r.p);
        let lnq = rx::excel_ln(1.0 - r.p);
        tot += 1;
        let cands = [
            chisq_inv_rt_kernel(r.p, 2.0).unwrap(),
            chisq_inv_kernel(r.p, 2.0).unwrap(),
            -2.0 * ln,
            rx::x87_mul(-2.0, ln),
            -2.0 * r.p.ln(),
            -2.0 * lnq,
            rx::x87_mul(-2.0, lnq),
            // placeholder
            0.0,
        ];
        if cands[0].to_bits() == want_rt {
            e[0] += 1;
        }
        if cands[1].to_bits() == want_inv {
            e[1] += 1;
        }
        if cands[2].to_bits() == want_rt {
            e[2] += 1;
        }
        if cands[3].to_bits() == want_rt {
            e[3] += 1;
        }
        if cands[4].to_bits() == want_rt {
            e[4] += 1;
        }
        if cands[5].to_bits() == want_inv {
            e[5] += 1;
        }
        if cands[6].to_bits() == want_inv {
            e[6] += 1;
        }
        if cands[2].to_bits() == bits(&r.chiinv_bits) {
            e[7] += 1;
        }
        if cands[2].to_bits() != want_rt {
            println!(
                "RT miss p={} excel={:x} -2ln={:x} x87={:x} prod={:x}",
                r.p,
                want_rt,
                cands[2].to_bits(),
                cands[3].to_bits(),
                cands[0].to_bits()
            );
        }
    }
    println!("CHIINV df=2 vs Excel n={tot}");
    for (i, name) in names.iter().enumerate() {
        println!("  {:>5}/{:<5}  {name}", e[i], tot);
    }
}
