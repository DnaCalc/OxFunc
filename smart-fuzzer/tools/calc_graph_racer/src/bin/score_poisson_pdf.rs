use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::discrete_dist_family::poisson_dist_kernel;
use serde::Deserialize;
use std::collections::BTreeMap;
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

fn x87_div(a: f64, b: f64) -> f64 {
    rx::ext_to_f64(
        &rx::ext_div(
            &rx::ext_from_f64(a),
            &rx::ext_from_f64(b),
            rx::CW_PC64_RN,
        ),
        rx::CW_PC64_RN,
    )
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        "smart-fuzzer/work/w109/G3-01-dist/answers-b24-poissonpdf.json".into()
    });
    let bank: Bank = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();

    let names = [
        "prod exp/sqrt(2pix)",
        "k>=2 x87_div(exp,sqrt(2pix))",
        "k>=2 exp/excel_sqrt(2pix)",
        "k>=2 exp/(sqrt2pi*sqrtx)",
        "k>=2 exp/excel_sqrt(2pi)/excel_sqrt(x)",
        "k>=2 x87_div(exp, x87_mul(sqrt2pi,sqrtx))",
        "k>=2 exp * recip(sqrt(2pix))",
        "k>=2 x87_mul(exp, recip(sqrt(2pix)))",
        "k=1 x87_mul(l, exp(-l))",
        "k=1 exp(-l + ln l)",
        "k=1 exp(ln l - l)",
        "k=1 x87_mul(exp(-l), l)",
        "k=1 exp(-l + excel_ln l)",
        "k=1 exp(excel_ln l - l)",
    ];
    let mut exact = vec![0usize; names.len()];
    let mut tot = vec![0usize; names.len()];
    let mut by_k: BTreeMap<i64, (usize, usize)> = BTreeMap::new();
    let mut nprod = 0usize;
    let mut eprod = 0usize;

    for w in bank.witnesses {
        if w.args.len() < 3 || !w.expected_bits.starts_with("0x") {
            continue;
        }
        let x = f64::from_bits(bits(&w.args[0]));
        let mean = f64::from_bits(bits(&w.args[1]));
        let cum = f64::from_bits(bits(&w.args[2]));
        if cum != 0.0 {
            continue;
        }
        let Ok(prod) = poisson_dist_kernel(x, mean, false) else {
            continue;
        };
        let want = bits(&w.expected_bits);
        nprod += 1;
        if prod.to_bits() == want {
            eprod += 1;
        }
        let k = x.trunc() as i64;
        let e = by_k.entry(k).or_insert((0, 0));
        e.1 += 1;
        if prod.to_bits() == want {
            e.0 += 1;
        }

        let mut cands = vec![None; names.len()];
        cands[0] = Some(prod);
        if k >= 2 && mean > 0.0 {
            let xf = k as f64;
            let s0 = (2.0 * std::f64::consts::PI * xf).sqrt();
            let num = prod * s0;
            let s_ex = rx::excel_sqrt(2.0 * std::f64::consts::PI * xf);
            let s_split = (2.0 * std::f64::consts::PI).sqrt() * xf.sqrt();
            let s_ex_split = rx::excel_sqrt(2.0 * std::f64::consts::PI) * rx::excel_sqrt(xf);
            cands[1] = Some(x87_div(num, s0));
            cands[2] = Some(num / s_ex);
            cands[3] = Some(num / s_split);
            cands[4] = Some(num / rx::excel_sqrt(2.0 * std::f64::consts::PI) / rx::excel_sqrt(xf));
            cands[5] = Some(x87_div(
                num,
                rx::x87_mul((2.0 * std::f64::consts::PI).sqrt(), xf.sqrt()),
            ));
            cands[6] = Some(num * (1.0 / s0));
            cands[7] = Some(rx::x87_mul(num, rx::x87_recip(s0)));
            let _ = s_ex_split;
        }
        if k == 1 && mean > 0.0 {
            cands[8] = Some(rx::x87_mul(mean, rx::excel_exp(-mean)));
            cands[9] = Some(rx::excel_exp(-mean + mean.ln()));
            cands[10] = Some(rx::excel_exp(mean.ln() - mean));
            cands[11] = Some(rx::x87_mul(rx::excel_exp(-mean), mean));
            cands[12] = Some(rx::excel_exp(-mean + rx::excel_ln(mean)));
            cands[13] = Some(rx::excel_exp(rx::excel_ln(mean) - mean));
        }
        for (i, c) in cands.iter().enumerate() {
            if let Some(v) = *c {
                tot[i] += 1;
                if v.to_bits() == want {
                    exact[i] += 1;
                }
            }
        }
    }
    println!("production POISSON pdf {eprod}/{nprod}");
    for (k, (e, t)) in by_k.iter().take(8) {
        println!("  k={k} {e}/{t}");
    }
    println!("graphs:");
    for (i, name) in names.iter().enumerate() {
        if tot[i] > 0 {
            println!("  {:>5}/{:<5}  {name}", exact[i], tot[i]);
        }
    }
}
