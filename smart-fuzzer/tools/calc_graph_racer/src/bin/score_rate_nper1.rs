use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::financial_time_value_family::{rate, PaymentTiming};
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Row {
    pmt: f64,
    pv: f64,
    rate0_bits: String,
}

fn bits(s: &str) -> u64 {
    u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap()
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        "../../work/w109/lastbit-12h-20260912/rate-nper1/capture.jsonl".into()
    });
    let mut tot = 0usize;
    let mut e = [0usize; 5];
    let names = [
        "production",
        "-pmt/pv-1",
        "-(pmt/pv)-1",
        "x87_div(-pmt,pv)-1",
        "-(x87_div(pmt,pv)+1)",
    ];
    for line in fs::read_to_string(&path).unwrap().lines() {
        if line.is_empty() {
            continue;
        }
        let r: Row = serde_json::from_str(line).unwrap();
        let want = bits(&r.rate0_bits);
        let cands = [
            rate(
                1.0,
                r.pmt,
                r.pv,
                0.0,
                PaymentTiming::EndOfPeriod,
                None,
            )
            .unwrap_or(f64::NAN),
            -r.pmt / r.pv - 1.0,
            -(r.pmt / r.pv) - 1.0,
            rx::ext_to_f64(
                &rx::ext_div(
                    &rx::ext_from_f64(-r.pmt),
                    &rx::ext_from_f64(r.pv),
                    rx::CW_PC64_RN,
                ),
                rx::CW_PC64_RN,
            ) - 1.0,
            -(rx::ext_to_f64(
                &rx::ext_add(
                    &rx::ext_div(
                        &rx::ext_from_f64(r.pmt),
                        &rx::ext_from_f64(r.pv),
                        rx::CW_PC64_RN,
                    ),
                    &rx::ext_from_f64(1.0),
                    rx::CW_PC64_RN,
                ),
                rx::CW_PC64_RN,
            )),
        ];
        tot += 1;
        for (i, v) in cands.iter().enumerate() {
            if v.is_finite() && v.to_bits() == want {
                e[i] += 1;
            }
        }
        if cands[1].to_bits() != want {
            println!(
                "miss pmt={} pv={} excel={:x} closed={:x} prod={:x}",
                r.pmt,
                r.pv,
                want,
                cands[1].to_bits(),
                cands[0].to_bits()
            );
        }
    }
    println!("RATE nper=1 type=0 vs Excel n={tot}");
    for (i, name) in names.iter().enumerate() {
        println!("  {:>5}/{:<5}  {name}", e[i], tot);
    }
}
