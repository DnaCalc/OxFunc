//! Race stirl8 associations on the two open high-band GAMMALN rows.

use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::special_dist_family::gammaln_kernel;
use serde::Deserialize;
use std::fs;

const LS2PI: f64 = f64::from_bits(0x3FED_67F1_C864_BEB5);
const W1: f64 = f64::from_bits(0x3FB5_5555_5555_553B);
const W2: f64 = f64::from_bits(0xBF66_C16C_16B0_2E5C);
const W3: f64 = f64::from_bits(0x3F4A_019F_98CF_38B6);
const W4: f64 = f64::from_bits(0xBF43_80CB_8C0F_E741);
const W5: f64 = f64::from_bits(0x3F4B_67BA_4CDA_D5D1);
const W6: f64 = f64::from_bits(0xBF5A_B89D_0B9E_43E4);

fn corr(x: f64, z_recip: bool, mul_dr: bool) -> f64 {
    let z = if z_recip {
        rx::x87_recip(x)
    } else {
        1.0 / x
    };
    let y = z * z;
    let mut w = W6;
    w = w * y + W5;
    w = w * y + W4;
    w = w * y + W3;
    w = w * y + W2;
    w = w * y + W1;
    if mul_dr {
        rx::x87_mul(z, w)
    } else {
        z * w
    }
}

fn q_cur(x: f64, lg: f64) -> f64 {
    let q1 = rx::x87_mul(x - 0.5, lg);
    let q2 = rx::ext_to_f64(
        &rx::ext_sub(&rx::ext_from_f64(q1), &rx::ext_from_f64(x), rx::CW_PC64_RN),
        rx::CW_PC64_RN,
    );
    rx::ext_to_f64(
        &rx::ext_add(
            &rx::ext_from_f64(q2),
            &rx::ext_from_f64(LS2PI),
            rx::CW_PC64_RN,
        ),
        rx::CW_PC64_RN,
    )
}

fn q_split(x: f64, lg: f64) -> f64 {
    let a = rx::x87_mul(x, lg);
    let b = rx::x87_mul(0.5, lg);
    let q1 = rx::ext_to_f64(
        &rx::ext_sub(&rx::ext_from_f64(a), &rx::ext_from_f64(b), rx::CW_PC64_RN),
        rx::CW_PC64_RN,
    );
    let q2 = rx::ext_to_f64(
        &rx::ext_sub(&rx::ext_from_f64(q1), &rx::ext_from_f64(x), rx::CW_PC64_RN),
        rx::CW_PC64_RN,
    );
    rx::ext_to_f64(
        &rx::ext_add(
            &rx::ext_from_f64(q2),
            &rx::ext_from_f64(LS2PI),
            rx::CW_PC64_RN,
        ),
        rx::CW_PC64_RN,
    )
}

fn q_ls2_first(x: f64, lg: f64) -> f64 {
    let q1 = rx::x87_mul(x - 0.5, lg);
    let q2 = rx::ext_to_f64(
        &rx::ext_add(
            &rx::ext_from_f64(q1),
            &rx::ext_from_f64(LS2PI),
            rx::CW_PC64_RN,
        ),
        rx::CW_PC64_RN,
    );
    rx::ext_to_f64(
        &rx::ext_sub(&rx::ext_from_f64(q2), &rx::ext_from_f64(x), rx::CW_PC64_RN),
        rx::CW_PC64_RN,
    )
}

fn out_dr(q: f64, c: f64) -> f64 {
    rx::ext_to_f64(
        &rx::ext_add(
            &rx::ext_from_f64(q),
            &rx::ext_from_f64(c),
            rx::CW_PC64_RN,
        ),
        rx::CW_PC64_RN,
    )
}

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
    let open = [
        (0x40215bf4d43f4d44u64, 0x4023d98694477879u64),
        (0x40234ce3244e3245, 0x40280a8dd6771c9a),
    ];
    let names = [
        "cur",
        "split x*ln-0.5*ln",
        "ls2 first",
        "cur+recip z",
        "cur+x87 corr",
        "cur+recip+x87 corr",
        "split+x87 corr",
        "native out",
        "prod kernel",
        "nextup prod",
    ];
    println!("two open rows:");
    for (name_i, name) in names.iter().enumerate() {
        let mut hit = 0;
        for &(xb, want) in &open {
            let x = f64::from_bits(xb);
            let lg = rx::excel_ln(x);
            let got = match name_i {
                0 => out_dr(q_cur(x, lg), corr(x, false, false)),
                1 => out_dr(q_split(x, lg), corr(x, false, false)),
                2 => out_dr(q_ls2_first(x, lg), corr(x, false, false)),
                3 => out_dr(q_cur(x, lg), corr(x, true, false)),
                4 => out_dr(q_cur(x, lg), corr(x, false, true)),
                5 => out_dr(q_cur(x, lg), corr(x, true, true)),
                6 => out_dr(q_split(x, lg), corr(x, false, true)),
                7 => q_cur(x, lg) + corr(x, false, false),
                8 => gammaln_kernel(x).unwrap(),
                9 => f64::from_bits(gammaln_kernel(x).unwrap().to_bits() + 1),
                _ => 0.0,
            };
            if got.to_bits() == want {
                hit += 1;
            }
        }
        println!("  {hit}/2  {name}");
    }

    let path = std::env::args().nth(1).unwrap_or_else(|| {
        "../../work/w109/G3-02-gamma/answers-gammaln.json".into()
    });
    let bank: Bank = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let mut n = 0usize;
    let mut prod = 0usize;
    let mut nextup = 0usize;
    for w in bank.witnesses {
        if w.args.is_empty() || !w.expected_bits.starts_with("0x") {
            continue;
        }
        let x = f64::from_bits(bits(&w.args[0]));
        if x < 8.0 {
            continue;
        }
        let want = bits(&w.expected_bits);
        n += 1;
        let g = gammaln_kernel(x).unwrap();
        if g.to_bits() == want {
            prod += 1;
        }
        if (g.to_bits() + 1) == want || g.to_bits() == want {
            nextup += 1;
        }
    }
    println!("x>=8 bank production {prod}/{n}  prod-or-nextup {nextup}/{n}");
}
