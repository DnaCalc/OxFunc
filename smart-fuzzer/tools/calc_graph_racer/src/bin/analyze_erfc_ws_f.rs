//! Compare live worksheet F / EXP(-z^2) against identified excel_exp and named F packets.
//! Reads capture.jsonl from the 12h campaign. No Excel. Heldouts unnamed.

use calc_graph_racer::erfc_f_packets as f;
use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::special_dist_family::{erf_precise_kernel, erfc_precise_kernel};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;

#[derive(Deserialize)]
struct Row {
    z: f64,
    #[serde(rename = "erfc_bits")]
    erfc_bits: String,
    #[serde(rename = "erf_bits")]
    erf_bits: String,
    #[serde(rename = "F_div_bits")]
    f_div_bits: String,
    #[serde(rename = "F_mul_bits")]
    f_mul_bits: String,
    #[serde(rename = "exp_neg_z2_bits")]
    exp_neg_z2_bits: String,
    #[serde(rename = "z2_mul_bits")]
    z2_mul_bits: String,
}

fn parse_hex(s: &str) -> u64 {
    u64::from_str_radix(s.trim_start_matches("0x"), 16).expect("hex")
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

fn hist(label: &str, ds: &[i64]) {
    let mut c: BTreeMap<i64, usize> = BTreeMap::new();
    for &d in ds {
        *c.entry(d.clamp(-8, 8)).or_default() += 1;
    }
    let exact = ds.iter().filter(|&&d| d == 0).count();
    println!("{label}: exact {}/{}  hist={:?}", exact, ds.len(), c);
}

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "../../work/w109/lastbit-12h-20260912/erfc-ws-f/capture.jsonl".into());
    let text = fs::read_to_string(&path).expect("capture");
    let rows: Vec<Row> = text
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| serde_json::from_str(l).expect("row"))
        .collect();
    println!("rows {}", rows.len());

    let mut exp_vs_ws = Vec::new();
    let mut x87sq_vs_mul = Vec::new();
    let mut wor_vs_ws_exp = Vec::new();
    let mut fdiv_vs_for = Vec::new();
    let mut fmul_vs_for = Vec::new();

    for r in &rows {
        let z = r.z;
        let z2_ws = f64::from_bits(parse_hex(&r.z2_mul_bits));
        let z2_x87 = rx::x87_mul(z, z);
        x87sq_vs_mul.push(sulp(z2_x87.to_bits(), z2_ws.to_bits()));

        let ws_expn = f64::from_bits(parse_hex(&r.exp_neg_z2_bits));
        let ox_expn = rx::excel_exp(-z2_ws);
        let ox_wor = f::w_rn53(z);
        exp_vs_ws.push(sulp(ox_expn.to_bits(), ws_expn.to_bits()));
        wor_vs_ws_exp.push(sulp(ox_wor.to_bits(), ws_expn.to_bits()));

        let q = f64::from_bits(parse_hex(&r.erfc_bits));
        if let Some(fo) = f::f_or(z, q.to_bits()) {
            let fdiv = f64::from_bits(parse_hex(&r.f_div_bits));
            let fmul = f64::from_bits(parse_hex(&r.f_mul_bits));
            fdiv_vs_for.push(sulp(fdiv.to_bits(), fo.to_bits()));
            fmul_vs_for.push(sulp(fmul.to_bits(), fo.to_bits()));
        }
    }
    hist("x87_mul(z,z) vs worksheet z*z", &x87sq_vs_mul);
    hist("excel_exp(-z2_ws) vs worksheet EXP(-z2)", &exp_vs_ws);
    hist("w_rn53 vs worksheet EXP(-z2)", &wor_vs_ws_exp);
    hist("worksheet F_div vs F_or", &fdiv_vs_for);
    hist("worksheet F_mul vs F_or", &fmul_vs_for);

    let mut prod_erfc = Vec::new();
    let mut prod_erf = Vec::new();
    let mut prod_erf_as_1merfc = Vec::new();
    let mut prod_erfc_as_1merf = Vec::new();
    let mut wrap_erf = Vec::new();
    let mut wrap_erfc = Vec::new();
    for r in &rows {
        let z = r.z;
        let want_q = parse_hex(&r.erfc_bits);
        let want_p = parse_hex(&r.erf_bits);
        let got_q = erfc_precise_kernel(z).unwrap().to_bits();
        let got_p = erf_precise_kernel(z).unwrap().to_bits();
        prod_erfc.push(sulp(got_q, want_q));
        prod_erf.push(sulp(got_p, want_p));
        let one_minus_q = (1.0 - f64::from_bits(got_q)).to_bits();
        let one_minus_p = (1.0 - f64::from_bits(got_p)).to_bits();
        prod_erf_as_1merfc.push(sulp(one_minus_q, want_p));
        prod_erfc_as_1merf.push(sulp(one_minus_p, want_q));
        if z < 0.5 {
            wrap_erf.push(sulp(got_p, want_p));
            wrap_erfc.push(sulp(one_minus_p, want_q));
        } else {
            wrap_erfc.push(sulp(got_q, want_q));
            wrap_erf.push(sulp(one_minus_q, want_p));
        }
    }
    hist("production ERFC vs Excel ERFC", &prod_erfc);
    hist("production ERF vs Excel ERF", &prod_erf);
    hist("1-prod_ERFC vs Excel ERF", &prod_erf_as_1merfc);
    hist("1-prod_ERF vs Excel ERFC", &prod_erfc_as_1merf);
    hist("complement-wrap ERF vs Excel ERF", &wrap_erf);
    hist("complement-wrap ERFC vs Excel ERFC", &wrap_erfc);

    println!("\n## production vs Excel by z band");
    let bands = [
        (0.0, 0.5),
        (0.5, 1.25),
        (1.25, 2.0),
        (2.0, 4.0),
        (4.0, 6.0),
        (6.0, 12.0),
    ];
    for (lo, hi) in bands {
        let mut erf_e = 0;
        let mut erfc_e = 0;
        let mut n = 0;
        for r in &rows {
            if r.z < lo || r.z >= hi {
                continue;
            }
            n += 1;
            let want_q = parse_hex(&r.erfc_bits);
            let want_p = parse_hex(&r.erf_bits);
            if erf_precise_kernel(r.z).unwrap().to_bits() == want_p {
                erf_e += 1;
            }
            if erfc_precise_kernel(r.z).unwrap().to_bits() == want_q {
                erfc_e += 1;
            }
        }
        println!("  [{lo},{hi}) n={n} ERF {erf_e}/{n} ERFC {erfc_e}/{n}");
    }

    fn score_direct(rows: &[(f64, u64)], eval: impl Fn(f64) -> f64) {
        let mut mid = f::Acc::default();
        let mut tail = f::Acc::default();
        for &(z, bits) in rows {
            let g = eval(z);
            if !g.is_finite() {
                continue;
            }
            let d = sulp(g.to_bits(), bits).unsigned_abs();
            if z < 4.0 {
                mid.add(d);
            } else {
                tail.add(d);
            }
        }
        println!("    mid {}  tail {}", f::fmt_acc(&mid), f::fmt_acc(&tail));
    }

    let fdiv_rows: Vec<(f64, u64)> = rows
        .iter()
        .filter(|r| r.z >= 0.5)
        .map(|r| (r.z, parse_hex(&r.f_div_bits)))
        .collect();
    println!(
        "\n## named F vs worksheet F_div bits directly (z>=0.5, n={})",
        fdiv_rows.len()
    );
    println!("  nswc_derfc0");
    score_direct(&fdiv_rows, f::nswc_derfc0);
    println!("  cody_erfcx_f");
    score_direct(&fdiv_rows, f::cody_erfcx_f);
    println!("  as714_x87_n24");
    score_direct(&fdiv_rows, |z| f::cf_as714_x87_n(z, 24));
    println!("  gaut_x87_n24");
    score_direct(&fdiv_rows, |z| f::cf_gautschi_x87_n(z, 24));
    println!("  evenodd_x87_n12");
    score_direct(&fdiv_rows, |z| f::cf_evenodd_as714_x87_n(z, 12));
    println!("  lentz_as714_n24_mask0");
    score_direct(&fdiv_rows, |z| lentz_as714(z, 24, 0));
    println!("  lentz_as714_n24_mask0400000");
    score_direct(&fdiv_rows, |z| lentz_as714(z, 24, 0x0400000));
    println!("  libm_erfcx mul");
    score_direct(&fdiv_rows, |z| libm::erfc(z) * (z * z).exp());
}

fn lentz_as714(x: f64, nterms: u32, mask: u32) -> f64 {
    use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, CW_PC64_RN};
    let one = ext_from_f64(1.0);
    let a_scale = ext_from_f64(0.5 / (x * x));
    let mut facc = one;
    let mut c = one;
    let mut d = ext_from_f64(0.0);
    let mut bit = 0u32;
    for j in 1..=nterms {
        let a = ext_mul(&ext_from_f64(j as f64), &a_scale, CW_PC64_RN);
        let den = ext_add(&one, &ext_mul(&a, &d, CW_PC64_RN), CW_PC64_RN);
        d = ext_div(&one, &den, CW_PC64_RN);
        let cn = ext_add(&one, &ext_div(&a, &c, CW_PC64_RN), CW_PC64_RN);
        c = cn;
        if mask & (1 << bit) != 0 {
            c = ext_from_f64(ext_to_f64(&c, CW_PC64_RN));
        }
        bit += 1;
        facc = ext_mul(&facc, &ext_mul(&c, &d, CW_PC64_RN), CW_PC64_RN);
    }
    f::RPINV / x / ext_to_f64(&facc, CW_PC64_RN)
}
