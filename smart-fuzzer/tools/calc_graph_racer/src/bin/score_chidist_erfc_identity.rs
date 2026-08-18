//! Score OxFunc CHIDIST(x,1) and ERFC.PRECISE(sqrt(x/2)) against the
//! live-Excel identity capture. Discovery-only; no production change.

use oxfunc_core::functions::chi_f_t_family::{
    chisq_dist_kernel, chisq_dist_rt_kernel,
};
use oxfunc_core::functions::special_dist_family::{
    erf_precise_kernel, erfc_precise_kernel,
};
use serde::Deserialize;
use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct Row {
    x: f64,
    chi: Option<f64>,
    eq_sqrt_x_over_2: String,
}

fn parse_csv(path: &std::path::Path) -> Result<Vec<Row>, Box<dyn Error>> {
    let text = fs::read_to_string(path)?;
    let mut out = Vec::new();
    let mut lines = text.lines();
    let header = lines.next().ok_or("empty csv")?;
    let cols: Vec<&str> = header.split(',').map(|s| s.trim_matches('"')).collect();
    let ix = |name: &str| cols.iter().position(|c| *c == name);
    let i_x = ix("x").ok_or("no x")?;
    let i_chi = ix("chi").ok_or("no chi")?;
    let i_eq = ix("eq_sqrt_x_over_2").ok_or("no eq")?;
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split(',').map(|s| s.trim_matches('"')).collect();
        let x: f64 = parts[i_x].parse()?;
        let chi = if parts[i_chi].is_empty() {
            None
        } else {
            Some(parts[i_chi].parse()?)
        };
        out.push(Row {
            x,
            chi,
            eq_sqrt_x_over_2: parts[i_eq].to_string(),
        });
    }
    Ok(out)
}

fn ulp(a: f64, b: f64) -> u64 {
    if !a.is_finite() || !b.is_finite() {
        return u64::MAX;
    }
    a.to_bits().abs_diff(b.to_bits())
}

fn main() -> Result<(), Box<dyn Error>> {
    let csv = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(
                "smart-fuzzer/work/w109/inverse-decomp/chidist-erfc-latest/pairs.csv",
            )
        });
    let rows = parse_csv(&csv)?;
    let mut n = 0u32;
    let mut excel_eq = 0u32;
    let mut chidist_exact = 0u32;
    let mut erfc_exact = 0u32;
    let mut local_eq = 0u32;
    let mut erf_cdf_exact = 0u32;
    let mut chidist_ulp = 0u64;
    let mut erfc_ulp = 0u64;
    let mut chidist_max = 0u64;
    let mut erfc_max = 0u64;
    for row in &rows {
        let Some(excel) = row.chi else { continue };
        n += 1;
        if row.eq_sqrt_x_over_2.eq_ignore_ascii_case("true") {
            excel_eq += 1;
        }
        let z = (row.x / 2.0).sqrt();
        let local_chi = chisq_dist_rt_kernel(row.x, 1.0).expect("chidist");
        let local_erfc = erfc_precise_kernel(z).expect("erfc");
        let local_cdf = chisq_dist_kernel(row.x, 1.0, true).expect("cdf");
        let local_erf = erf_precise_kernel(z).expect("erf");
        if local_chi.to_bits() == excel.to_bits() {
            chidist_exact += 1;
        }
        if local_erfc.to_bits() == excel.to_bits() {
            erfc_exact += 1;
        }
        if local_chi.to_bits() == local_erfc.to_bits() {
            local_eq += 1;
        }
        if local_cdf.to_bits() == local_erf.to_bits() {
            erf_cdf_exact += 1;
        }
        let u1 = ulp(local_chi, excel);
        let u2 = ulp(local_erfc, excel);
        chidist_ulp += u1.min(1_000_000);
        erfc_ulp += u2.min(1_000_000);
        chidist_max = chidist_max.max(u1);
        erfc_max = erfc_max.max(u2);
    }
    println!("rows                 {n}");
    println!("excel identity exact {excel_eq}/{n}");
    println!("oxfunc CHIDIST==xls  {chidist_exact}/{n}  maxULP={chidist_max} clipped_sum={chidist_ulp}");
    println!("oxfunc ERFC(z)==xls  {erfc_exact}/{n}  maxULP={erfc_max} clipped_sum={erfc_ulp}");
    println!("oxfunc CHIDIST==ERFC {local_eq}/{n}");
    println!("oxfunc CDF==ERF(z)   {erf_cdf_exact}/{n}");
    Ok(())
}
