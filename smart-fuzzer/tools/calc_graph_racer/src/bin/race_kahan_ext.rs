//! W109 G6-01: does PMT deliver tau = -n*log1p(r) EXTENDED (x87 register, never
//! spilled to double) into the exp/Kahan-expm1 chain? Race extended-argument
//! Kahan schedules vs the model-free pinned em oracle. The CSV's tau_bits/u_bits
//! are the DOUBLE-spilled reconstructions; here we recompute tau/u in ext80 from
//! (n, r) so sub-double content is preserved.
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    CW_PC53_RN, CW_PC64_RN, Ext80, ext_abs, ext_add, ext_div, ext_f2xm1, ext_from_f64, ext_fyl2x,
    ext_fyl2xp1, ext_l2e, ext_ln2, ext_mul, ext_one, ext_rndint, ext_scale, ext_sub, ext_to_f64,
};

const RZ: u16 = CW_PC64_RN | 0x0C00; // final store round-toward-zero
const RN: u16 = CW_PC53_RN; // final store RN at double precision

// fFEXP chain on an extended argument, result kept EXTENDED (no final store).
fn exp_chain_ext(arg: &Ext80) -> Ext80 {
    let cw = CW_PC64_RN;
    let t = ext_mul(arg, &ext_l2e(), cw);
    let k = ext_rndint(&t, cw);
    let f = ext_sub(&t, &k, cw);
    let neg = ext_to_f64(&f, cw) < 0.0;
    let w = ext_f2xm1(&ext_abs(&f, cw), cw);
    let mut m = ext_add(&w, &ext_one(), cw);
    if neg {
        m = ext_div(&ext_one(), &m, cw);
    }
    ext_scale(&m, &k, cw)
}

// ln of an extended u via the fyl2x chain (ln2 * log2(u)), kept extended.
fn ln_ext(u: &Ext80) -> Ext80 {
    ext_fyl2x(&ext_ln2(), u, CW_PC64_RN)
}

// tau extended: -n * log1p(r) via fyl2xp1, then multiply by -n, all ext.
fn tau_ext(n: u32, r: f64) -> Ext80 {
    let cw = CW_PC64_RN;
    let l1p = ext_fyl2xp1(&ext_ln2(), &ext_from_f64(r), cw); // ln(1+r) extended
    ext_mul(&ext_from_f64(-(n as f64)), &l1p, cw)
}

// ---------- candidate em schedules, all for |tau|<1 (Kahan branch) ----------
// 1: fully extended Kahan, final store RN53
fn k_ext_rn(n: u32, r: f64) -> f64 {
    let cw = CW_PC64_RN;
    let tau = tau_ext(n, r);
    let u = exp_chain_ext(&tau);
    let num = ext_mul(&ext_sub(&u, &ext_one(), cw), &tau, cw);
    let den = ln_ext(&u);
    ext_to_f64(&ext_div(&num, &den, cw), RN)
}
// 2: fully extended Kahan, final store RZ (chop) -> toward zero
fn k_ext_rz(n: u32, r: f64) -> f64 {
    let cw = CW_PC64_RN;
    let tau = tau_ext(n, r);
    let u = exp_chain_ext(&tau);
    let num = ext_mul(&ext_sub(&u, &ext_one(), cw), &tau, cw);
    let den = ln_ext(&u);
    ext_to_f64(&ext_div(&num, &den, cw), RZ)
}
// 3: extended arg -> u, but u SPILLED to double; Kahan denom ln on double u
//    (ext exp, double Kahan). tau for numerator = double-spilled tau.
fn k_extarg_dblkahan(n: u32, r: f64) -> f64 {
    let cw = CW_PC64_RN;
    let tau = tau_ext(n, r);
    let u = ext_to_f64(&exp_chain_ext(&tau), CW_PC53_RN); // spill u
    let td = ext_to_f64(&tau, CW_PC53_RN); // spill tau for numerator
    (u - 1.0) * td / rx::excel_ln(u)
}
// 4: extended arg -> u extended; numerator extended with EXTENDED tau; denom
//    ln on the DOUBLE-spilled u (mix)
fn k_extnum_dblden(n: u32, r: f64) -> f64 {
    let cw = CW_PC64_RN;
    let tau = tau_ext(n, r);
    let uext = exp_chain_ext(&tau);
    let ud = ext_to_f64(&uext, CW_PC53_RN);
    let num = ext_mul(&ext_sub(&uext, &ext_one(), cw), &tau, cw);
    let den = ext_from_f64(rx::excel_ln(ud));
    ext_to_f64(&ext_div(&num, &den, cw), RN)
}
// 5: double tau (spilled) but u,ln kept extended off that double tau
fn k_dbltau_extchain(n: u32, r: f64) -> f64 {
    let cw = CW_PC64_RN;
    let td = ext_to_f64(&tau_ext(n, r), CW_PC53_RN);
    let tau = ext_from_f64(td);
    let u = exp_chain_ext(&tau);
    let num = ext_mul(&ext_sub(&u, &ext_one(), cw), &tau, cw);
    let den = ln_ext(&u);
    ext_to_f64(&ext_div(&num, &den, cw), RN)
}
// 6: extended arg, but the WHOLE em via the naive u-1 path in ext (no Kahan),
//    final RN53 — tests whether small-tau uses plain (u-1) extended.
fn k_ext_um1(n: u32, r: f64) -> f64 {
    let tau = tau_ext(n, r);
    let u = exp_chain_ext(&tau);
    ext_to_f64(&ext_sub(&u, &ext_one(), CW_PC64_RN), RN)
}
// 7: fully extended Kahan but ln via ln1p-style: den = ext ln(u); num uses
//    (u-1) but with u NOT scaled? -> same as 1 but store RN at PC64 then to f64
fn k_ext_rn64(n: u32, r: f64) -> f64 {
    let cw = CW_PC64_RN;
    let tau = tau_ext(n, r);
    let u = exp_chain_ext(&tau);
    let num = ext_mul(&ext_sub(&u, &ext_one(), cw), &tau, cw);
    let den = ln_ext(&u);
    let q = ext_div(&num, &den, cw);
    ext_to_f64(&q, CW_PC64_RN) // default RN53 store == RN
}

fn main() {
    let csv = std::fs::read_to_string("../../work/w109/G6-solvers/expm1_intermediates.csv").unwrap();
    type F = fn(u32, f64) -> f64;
    let cands: [(&str, F); 7] = [
        ("1 ext-all RN", k_ext_rn),
        ("2 ext-all RZ", k_ext_rz),
        ("3 extarg dblKahan", k_extarg_dblkahan),
        ("4 extnum dblden", k_extnum_dblden),
        ("5 dbltau extchain", k_dbltau_extchain),
        ("6 ext (u-1) only", k_ext_um1),
        ("7 ext-all RN64", k_ext_rn64),
    ];
    let mut score = [0u32; 7];
    let mut rows: Vec<(i32, u32, u64, [u64; 7])> = Vec::new();
    let mut tot = 0u32;
    for line in csv.lines().skip(1) {
        let f: Vec<&str> = line.split(',').collect();
        if f.len() < 6 { continue; }
        let k: i32 = f[0].parse().unwrap();
        let n: u32 = f[1].parse().unwrap();
        let pin = u64::from_str_radix(f[5], 16).unwrap();
        let r = 2f64.powi(k);
        let mut got = [0u64; 7];
        for (i, (_, fun)) in cands.iter().enumerate() {
            got[i] = fun(n, r).to_bits();
            if got[i] == pin { score[i] += 1; }
        }
        rows.push((k, n, pin, got));
        tot += 1;
    }
    println!("=== extended-argument Kahan race, N={} ===", tot);
    for (i, (name, _)) in cands.iter().enumerate() {
        println!("  {:22} {:3}/{}  ({:.1}%)", name, score[i], tot, 100.0 * score[i] as f64 / tot as f64);
    }
    let best = (0..7).max_by_key(|&i| score[i]).unwrap();
    println!("\n=== misses for best [{}] (k,n, signed ULP) ===", cands[best].0);
    let mut nm = 0;
    for (k, n, pin, got) in &rows {
        if got[best] != *pin {
            nm += 1;
            if nm <= 25 {
                let d = (got[best] as i64) - (*pin as i64);
                println!("  k={:3} n={:3}  err={:+5} pin={:016x} got={:016x}", k, n, d, pin, got[best]);
            }
        }
    }
    println!("  ... {} misses total", nm);
}
