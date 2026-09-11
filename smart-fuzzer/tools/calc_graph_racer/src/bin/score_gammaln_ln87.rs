//! Score GAMMALN high-band LN87 vs production native ln against captured banks.

use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::special_dist_family::gammaln_kernel;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;

const LS2PI: f64 = f64::from_bits(0x3FED_67F1_C864_BEB5);
const W: [f64; 6] = [
    f64::from_bits(0xBF5A_B89D_0B9E_43E4),
    f64::from_bits(0x3F4B_67BA_4CDA_D5D1),
    f64::from_bits(0xBF43_80CB_8C0F_E741),
    f64::from_bits(0x3F4A_019F_98CF_38B6),
    f64::from_bits(0xBF66_C16C_16B0_2E5C),
    f64::from_bits(0x3FB5_5555_5555_553B),
];

#[derive(Deserialize)]
struct Bank {
    witnesses: Vec<Witness>,
}
#[derive(Deserialize)]
struct Witness {
    id: String,
    args: Vec<String>,
    expected_bits: String,
}

fn bits(s: &str) -> Option<u64> {
    if !s.starts_with("0x") || s.len() != 18 {
        return None;
    }
    u64::from_str_radix(&s[2..], 16).ok()
}
fn sulp(got: u64, want: u64) -> i64 {
    fn ord(v: u64) -> u64 {
        if v >> 63 == 0 { v | (1u64 << 63) } else { !v }
    }
    (i128::from(ord(got)) - i128::from(ord(want))) as i64
}

fn stirl(lg: f64, x: f64, dr_q: bool, dr_out: bool) -> f64 {
    let q = if dr_q {
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
    } else {
        (x - 0.5) * lg - x + LS2PI
    };
    let z = 1.0 / x;
    let y = z * z;
    let mut w = W[0];
    w = w * y + W[1];
    w = w * y + W[2];
    w = w * y + W[3];
    w = w * y + W[4];
    w = w * y + W[5];
    let corr = z * w;
    if dr_out {
        rx::ext_to_f64(
            &rx::ext_add(
                &rx::ext_from_f64(q),
                &rx::ext_from_f64(corr),
                rx::CW_PC64_RN,
            ),
            rx::CW_PC64_RN,
        )
    } else {
        q + corr
    }
}

fn hist(name: &str, ds: &[i64]) {
    let mut c: BTreeMap<i64, usize> = BTreeMap::new();
    let mut exact = 0;
    let mut max = 0u64;
    for &d in ds {
        if d == 0 {
            exact += 1;
        }
        max = max.max(d.unsigned_abs());
        *c.entry(d.clamp(-8, 8)).or_default() += 1;
    }
    println!("{name:<44} {exact}/{n} max={max} hist={c:?}", n = ds.len());
}

fn load_highband(path: &str) -> Vec<(String, f64, u64)> {
    let bank: Bank = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    let mut rows = Vec::new();
    for w in bank.witnesses {
        let Some(xb) = bits(&w.args[0]) else { continue };
        let Some(wb) = bits(&w.expected_bits) else {
            continue;
        };
        let x = f64::from_bits(xb);
        if x >= 8.0 && x.is_finite() {
            rows.push((format!("{path}:{}", w.id), x, wb));
        }
    }
    rows
}

fn main() {
    let mut paths: Vec<String> = std::env::args().skip(1).collect();
    if paths.is_empty() {
        paths = vec![
            "../../work/w109/G3-02-gamma/answers-gammaln-current-build-discovery-v1.json".into(),
            "../../work/w109/G3-02-gamma/answers-gammaln.json".into(),
            "../../work/w109/G3-02-gamma/answers-dense1.json".into(),
            "../../work/w109/G3-02-gamma/answers-g12dense.json".into(),
            "../../work/w109/G3-02-gamma/answers-L-round3.json".into(),
            "../../work/w109/G3-02-gamma/answers-validate.json".into(),
            "../../work/w109/G3-02-gamma/answers-r0.json".into(),
            "../../work/w109/G3-02-gamma/answers-r2.json".into(),
        ];
    }
    let mut rows = Vec::new();
    for path in &paths {
        if !std::path::Path::new(path).exists() {
            println!("skip missing {path}");
            continue;
        }
        let part = load_highband(path);
        println!("{path} highband {}", part.len());
        rows.extend(part);
    }
    rows.sort_by_key(|r| r.1.to_bits());
    rows.dedup_by_key(|r| r.1.to_bits());
    println!("unique highband rows {}", rows.len());
    let score = |name: &str, eval: fn(f64) -> f64| {
        let ds: Vec<i64> = rows
            .iter()
            .map(|&(_, x, w)| sulp(eval(x).to_bits(), w))
            .collect();
        hist(name, &ds);
        if rows.len() <= 40 {
            for (id, x, w) in &rows {
                let d = sulp(eval(*x).to_bits(), *w);
                if d != 0 {
                    println!("  miss {id} x={x} sulp={d}");
                }
            }
        }
    };
    score("production gammaln_kernel", |x| gammaln_kernel(x).unwrap());
    score("stirl native ln", |x| stirl(x.ln(), x, false, false));
    score("stirl excel_ln native q", |x| {
        stirl(rx::excel_ln(x), x, false, false)
    });
    score("stirl excel_ln DR q native out", |x| {
        stirl(rx::excel_ln(x), x, true, false)
    });
    score("stirl excel_ln DR q DR out", |x| {
        stirl(rx::excel_ln(x), x, true, true)
    });
}
