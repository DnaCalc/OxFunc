//! W109 G6-01 DOUBT-PROBE: is PMT's em = (1+r)^-n - 1 computed by the C-runtime
//! integer-binexp `pow` (same as bond discount factors), NOT by the Kahan
//! internal expm1? Race binary-exponentiation schedules directly against the
//! model-free pinned em oracle (expm1_intermediates.csv: r=2^-k, integer n,
//! em_pinned via pmt=2^-k*RN(pv/em) so ZERO combine confound).
use oxfunc_core::excel_numeric::research as rx;
use rx::{CW_PC53_RN, CW_PC64_RN, Ext80, ext_div, ext_from_f64, ext_mul, ext_one, ext_sub, ext_to_f64};

// ---- exponentiation-by-squaring, SSE2 double, per-op RN53 ----
fn binexp_d(base: f64, mut e: u32) -> f64 {
    let mut result = 1.0f64;
    let mut b = base;
    while e > 0 {
        if e & 1 == 1 { result *= b; }
        e >>= 1;
        if e > 0 { b *= b; }
    }
    result
}
// ---- exponentiation-by-squaring, x87 extended, never spilled (PC64) ----
fn binexp_ext(base: &Ext80, mut e: u32, cw: u16) -> Ext80 {
    let mut result = ext_one();
    let mut b = *base;
    while e > 0 {
        if e & 1 == 1 { result = ext_mul(&result, &b, cw); }
        e >>= 1;
        if e > 0 { b = ext_mul(&b, &b, cw); }
    }
    result
}

// em candidates: em = (1+r)^-n - 1
// A: P=(1+r)^n double binexp; v=1/P double; em=v-1 double
fn a_recip_of_pow(opr: f64, n: u32) -> f64 { let p = binexp_d(opr, n); (1.0 / p) - 1.0 }
// B: b=1/(1+r) double; P=b^n double binexp; em=P-1 double
fn b_pow_of_recip(opr: f64, n: u32) -> f64 { let b = 1.0 / opr; binexp_d(b, n) - 1.0 }
// C: P=(1+r)^n EXT binexp; v=1/P EXT; em=RN53(v-1)
fn c_recip_ext(opr: f64, n: u32) -> f64 {
    let p = binexp_ext(&ext_from_f64(opr), n, CW_PC64_RN);
    let v = ext_div(&ext_one(), &p, CW_PC64_RN);
    ext_to_f64(&ext_sub(&v, &ext_one(), CW_PC64_RN), CW_PC53_RN)
}
// D: b=1/(1+r) EXT; P=b^n EXT binexp; em=RN53(P-1)
fn d_pow_recip_ext(opr: f64, n: u32) -> f64 {
    let b = ext_div(&ext_one(), &ext_from_f64(opr), CW_PC64_RN);
    let p = binexp_ext(&b, n, CW_PC64_RN);
    ext_to_f64(&ext_sub(&p, &ext_one(), CW_PC64_RN), CW_PC53_RN)
}
// E: P=(1+r)^n EXT binexp; SPILL P to double; v=1/P double; em=v-1 double
fn e_recip_ext_spill(opr: f64, n: u32) -> f64 {
    let p = ext_to_f64(&binexp_ext(&ext_from_f64(opr), n, CW_PC64_RN), CW_PC53_RN);
    (1.0 / p) - 1.0
}
// F: P=(1+r)^n double binexp; v=1/P via EXT reciprocal (RN53 store); em=v-1 double
fn f_pow_d_recip_ext(opr: f64, n: u32) -> f64 {
    let p = binexp_d(opr, n);
    let v = ext_to_f64(&ext_div(&ext_one(), &ext_from_f64(p), CW_PC64_RN), CW_PC53_RN);
    v - 1.0
}
// G: v = (1+r)^-n directly via EXT reciprocal-base binexp, then spill v then -1 double
fn g_recip_base_spill(opr: f64, n: u32) -> f64 {
    let b = ext_div(&ext_one(), &ext_from_f64(opr), CW_PC64_RN);
    let v = ext_to_f64(&binexp_ext(&b, n, CW_PC64_RN), CW_PC53_RN);
    v - 1.0
}

fn ulp_err(got: u64, want: u64) -> i64 {
    // both negative-domain doubles; compare as ordered magnitudes
    let g = f64::from_bits(got); let w = f64::from_bits(want);
    if !g.is_finite() { return 1_000_000; }
    // signed difference in ULP of `want`
    let (gi, wi) = (got as i64, want as i64);
    gi - wi
}

fn main() {
    let csv = std::fs::read_to_string("../../work/w109/G6-solvers/expm1_intermediates.csv").unwrap();
    type F = fn(f64, u32) -> f64;
    let cands: [(&str, F); 7] = [
        ("A recip(pow_d)", a_recip_of_pow),
        ("B pow(recip_d)", b_pow_of_recip),
        ("C recip_ext", c_recip_ext),
        ("D pow_recip_ext", d_pow_recip_ext),
        ("E ext_spill_recipd", e_recip_ext_spill),
        ("F powd_recip_ext", f_pow_d_recip_ext),
        ("G recipbase_spill", g_recip_base_spill),
    ];
    let mut score = [0u32; 7];
    let mut tot = 0u32;
    // collect signed-ulp histograms for the best few
    let mut rows: Vec<(i32, u32, u64, [u64; 7])> = Vec::new();
    for line in csv.lines().skip(1) {
        let f: Vec<&str> = line.split(',').collect();
        if f.len() < 6 { continue; }
        let k: i32 = f[0].parse().unwrap();
        let n: u32 = f[1].parse().unwrap();
        let pin = u64::from_str_radix(f[5], 16).unwrap();
        let r = 2f64.powi(k); // r = 2^k (k negative)
        let opr = 1.0 + r; // exact for k in -1..-52
        let mut got = [0u64; 7];
        for (i, (_, fun)) in cands.iter().enumerate() {
            let g = fun(opr, n).to_bits();
            got[i] = g;
            if g == pin { score[i] += 1; }
        }
        tot += 1;
        rows.push((k, n, pin, got));
    }
    println!("=== em = (1+r)^-n - 1 candidate race, N={} (model-free pinned oracle) ===", tot);
    for (i, (name, _)) in cands.iter().enumerate() {
        println!("  {:22} {:3}/{}  ({:.1}%)", name, score[i], tot, 100.0 * score[i] as f64 / tot as f64);
    }
    // signed-ULP error distribution for the top candidate
    let best = (0..7).max_by_key(|&i| score[i]).unwrap();
    println!("\n=== signed ULP error histogram for [{}] ===", cands[best].0);
    let mut hist = std::collections::BTreeMap::new();
    for (_, _, pin, got) in &rows {
        let e = ulp_err(got[best], *pin);
        *hist.entry(e.clamp(-5, 5)).or_insert(0u32) += 1;
    }
    for (e, c) in &hist { println!("  {:+3} ULP : {}", e, c); }
    // show the residual structure: rows where best candidate misses
    println!("\n=== misses for [{}] (k,n, err ULP) ===", cands[best].0);
    let mut nm = 0;
    for (k, n, pin, got) in &rows {
        if got[best] != *pin { nm += 1; if nm <= 30 { println!("  k={:3} n={:3}  err={:+} pin={:016x} got={:016x}", k, n, ulp_err(got[best], *pin), pin, got[best]); } }
    }
}
