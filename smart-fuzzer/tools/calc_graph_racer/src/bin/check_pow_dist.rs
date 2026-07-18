//! W109 lane-1: distribution pow staging race (b24 weibull/binom corpora).
//!
//! For each stdin line "base_bits exp_bits", emit the four candidate pow
//! stagings t = base^exp, each pushed through the site's identified outer:
//!
//!   c1 powf      — platform CRT pow (the b24 agentN reference candidate)
//!   c2 chain-sse — excel_exp(RN53(y * ln53)) plain SSE2 double product
//!   c3 chain-x87 — excel_exp(RN53(RN64(y * ln53))) POWER's double-rounded mul
//!   c4 chain-ext — fully register-continuous: extended fyl2x ln stays on the
//!                  stack, extended product, chain run on the extended arg
//!                  (the erf-path delivery pattern; control)
//!
//! Mode argv[1]:
//!   weibull — outer = -expm1_internal(-t)   (identified cdf wrapper)
//!   binom   — outer = identity              (q^n published directly)
//!
//! Output: "base_bits exp_bits c1 c2 c3 c4"

use oxfunc_core::excel_numeric::research as rx;
use rx::{
    CW_PC64_RN, Ext80, ext_abs, ext_add, ext_div, ext_f2xm1, ext_from_f64, ext_fyl2x, ext_l2e,
    ext_ln2, ext_mul, ext_one, ext_rndint, ext_scale, ext_sub, ext_to_f64,
};
use std::io::BufRead;

const CW: u16 = CW_PC64_RN;

/// fFEXP chain on an already-extended argument (register-continuous entry).
fn exp_chain_from_ext(xe: &Ext80) -> f64 {
    let t = ext_mul(xe, &ext_l2e(), CW);
    let k = ext_rndint(&t, CW);
    let f = ext_sub(&t, &k, CW);
    let neg = ext_to_f64(&f, CW) < 0.0;
    let w = ext_f2xm1(&ext_abs(&f, CW), CW);
    let mut m = ext_add(&w, &ext_one(), CW);
    if neg {
        m = ext_div(&ext_one(), &m, CW);
    }
    let r = ext_scale(&m, &k, CW);
    ext_to_f64(&r, CW)
}

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_default();
    let weibull = match mode.as_str() {
        "weibull" => true,
        "binom" => false,
        _ => {
            eprintln!("usage: check_pow_dist <weibull|binom>  (stdin: base_bits exp_bits)");
            std::process::exit(2);
        }
    };
    let outer = |t: f64| -> f64 {
        if weibull {
            -rx::excel_expm1_internal(-t)
        } else {
            t
        }
    };
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let mut it = line.split_whitespace();
        let (Some(bs), Some(es)) = (it.next(), it.next()) else {
            continue;
        };
        let base = f64::from_bits(u64::from_str_radix(bs.trim_start_matches("0x"), 16).unwrap());
        let y = f64::from_bits(u64::from_str_radix(es.trim_start_matches("0x"), 16).unwrap());

        let c1 = outer(base.powf(y));

        let ln53 = rx::excel_ln(base);
        let c2 = outer(rx::excel_exp(y * ln53));
        let c3 = outer(rx::excel_exp(rx::x87_mul(y, ln53)));

        let ln_ext = ext_fyl2x(&ext_ln2(), &ext_from_f64(base), CW);
        let t_ext = ext_mul(&ext_from_f64(y), &ln_ext, CW);
        let c4 = outer(exp_chain_from_ext(&t_ext));

        println!(
            "{:016x} {:016x} {:016x} {:016x} {:016x} {:016x}",
            base.to_bits(),
            y.to_bits(),
            c1.to_bits(),
            c2.to_bits(),
            c3.to_bits(),
            c4.to_bits()
        );
    }
}
