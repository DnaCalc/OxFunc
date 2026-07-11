//! W109 research surface (`research-x87` feature, `x86_64` only).
//!
//! Public gateway for the calculation-graph search tooling
//! (`smart-fuzzer/tools/calc_graph_racer`): the raw x87 instruction primitives
//! and [`Ext80`] extended-precision temporary from [`super::x87::raw`], plus
//! `f64` wrappers over the production Excel-parity kernels so candidate graphs
//! can cite "the confirmed Excel op" as a single node.
//!
//! Everything here is research plumbing — production function kernels must
//! keep calling the `pub(crate)` entry points in [`super`] directly.

pub use super::x87::raw::{
    CW_PC24_RN, CW_PC53_RN, CW_PC64_RN, Ext80, ext_abs, ext_add, ext_chs, ext_cos, ext_div,
    ext_f2xm1, ext_from_f64, ext_fyl2x, ext_fyl2xp1, ext_l2e, ext_l2t, ext_lg2, ext_ln2, ext_mul,
    ext_one, ext_pi, ext_prem, ext_prem1, ext_rndint, ext_scale, ext_sin, ext_sqrt, ext_sub,
    ext_tan, ext_to_f64,
};

/// Excel `EXP` — the confirmed `87tran.asm` `fFEXP` chain (bit-exact to live
/// 64-bit Excel on the reference host).
pub fn excel_exp(x: f64) -> f64 {
    super::excel_exp(x)
}

/// Excel `LN` — the confirmed `fldln2` + `fyl2x` chain.
pub fn excel_ln(x: f64) -> f64 {
    super::excel_log(x)
}

/// Excel `LOG10` — the confirmed `fldlg2` + `fyl2x` chain.
pub fn excel_log10(x: f64) -> f64 {
    super::excel_log10(x)
}

/// Excel `POWER` positive-power staging: `exp(RN53(RN64(y·ln x)))` with the
/// `y == 0.5 -> sqrt` special case (bit-exact, BUG-FUNC-042 sign-off).
/// `base > 0`, `exp > 0`, both finite.
pub fn excel_pow_positive(base: f64, exp: f64) -> f64 {
    super::excel_pow_positive(base, exp)
}

/// The x87 double-rounded product `RN53(RN64(a*b))` Excel uses inside `POWER`.
pub fn x87_mul(a: f64, b: f64) -> f64 {
    super::x87::mul(a, b)
}

/// The x87 double-rounded reciprocal `RN53(RN64(1/x))` Excel uses to stage
/// negative-exponent `POWER`.
pub fn x87_recip(x: f64) -> f64 {
    super::excel_x87_recip(x)
}

/// Portable faithful `expm1` (deterministic substrate helper, no worksheet
/// counterpart).
pub fn excel_expm1(x: f64) -> f64 {
    super::excel_expm1(x)
}

/// Portable faithful `log1p` (deterministic substrate helper).
pub fn excel_log1p(x: f64) -> f64 {
    super::excel_log1p(x)
}

/// Correctly-rounded hardware sqrt (SSE2 `SQRTSD`).
pub fn excel_sqrt(x: f64) -> f64 {
    super::excel_sqrt(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The full live-Excel EXP/LN witness corpus (same provenance as the
    /// `x87::tests` regression). Used here to prove the *raw* primitive layer
    /// is faithful: recomposing the `fFEXP`/`fFLN` chains from `Ext80` ops must
    /// reproduce every witness bit-for-bit, i.e. parking intermediates in
    /// memory via `fld/fstp tbyte` is transparent.
    const GROUND_TRUTH: &str = include_str!("x87_excel_ground_truth.tsv");

    /// `fFEXP` recomposed from raw primitives (memory-parked intermediates).
    fn exp_via_raw(x: f64) -> f64 {
        let cw = CW_PC64_RN;
        let xe = ext_from_f64(x);
        let t = ext_mul(&xe, &ext_l2e(), cw); // t = x * log2(e)
        let k = ext_rndint(&t, cw); // k = rint(t)
        let f = ext_sub(&t, &k, cw); // f = t - k, |f| <= 1/2
        let neg = ext_to_f64(&f, cw) < 0.0;
        let w = ext_f2xm1(&ext_abs(&f, cw), cw); // 2^|f| - 1
        let mut m = ext_add(&w, &ext_one(), cw); // 1 + w
        if neg {
            m = ext_div(&ext_one(), &m, cw); // reciprocal invert branch
        }
        let r = ext_scale(&m, &k, cw); // m * 2^k
        ext_to_f64(&r, cw)
    }

    /// `fFLN` recomposed from raw primitives.
    fn ln_via_raw(x: f64) -> f64 {
        let cw = CW_PC64_RN;
        let r = ext_fyl2x(&ext_ln2(), &ext_from_f64(x), cw);
        ext_to_f64(&r, cw)
    }

    #[test]
    fn raw_recomposition_matches_live_excel_corpus() {
        let (mut n, mut bad) = (0u32, Vec::new());
        for line in GROUND_TRUTH.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let mut it = line.split('\t');
            let func = it.next().unwrap();
            let xb = u64::from_str_radix(it.next().unwrap(), 16).expect("hex");
            let exb = u64::from_str_radix(it.next().unwrap(), 16).expect("hex");
            let x = f64::from_bits(xb);
            let got = match func {
                "EXP" => exp_via_raw(x),
                "LN" => ln_via_raw(x),
                _ => continue,
            }
            .to_bits();
            n += 1;
            if got != exb && bad.len() < 20 {
                bad.push(format!("{func} x={xb:016x}: got {got:016x} want {exb:016x}"));
            }
        }
        assert!(n >= 200, "expected the full corpus, saw {n} rows");
        assert!(bad.is_empty(), "{} mismatches:\n{}", bad.len(), bad.join("\n"));
    }

    #[test]
    fn ext80_is_truly_extended_precision() {
        let cw = CW_PC64_RN;
        // 1 + 2^-60 needs a 61-bit significand: representable at PC=64, not in
        // binary64. The store barrier must round it away; the extended
        // subtraction must recover it exactly.
        let tiny = f64::from_bits(0x3c30000000000000); // 2^-60
        let one = ext_one();
        let t = ext_add(&one, &ext_from_f64(tiny), cw);
        assert_eq!(ext_to_f64(&t, cw), 1.0, "store barrier rounds to binary64");
        let back = ext_sub(&t, &one, cw);
        assert_eq!(ext_to_f64(&back, cw), tiny, "extended kept the low bits");
        // Under PC=53 the same add rounds at double precision: the bits die.
        let t53 = ext_add(&one, &ext_from_f64(tiny), CW_PC53_RN);
        let back53 = ext_sub(&t53, &one, cw);
        assert_eq!(ext_to_f64(&back53, cw), 0.0, "PC=53 rounds like binary64");
    }

    #[test]
    fn prem_and_prem1_disagree_on_half_quotients() {
        let cw = CW_PC64_RN;
        let x = ext_from_f64(7.0);
        let m = ext_from_f64(2.0);
        // 7/2 = 3.5: FPREM truncates the quotient (3) -> 1; FPREM1 rounds it
        // to nearest-even (4) -> -1.
        assert_eq!(ext_to_f64(&ext_prem(&x, &m, cw), cw), 1.0);
        assert_eq!(ext_to_f64(&ext_prem1(&x, &m, cw), cw), -1.0);
        // Large-magnitude reduction requires the C2 completion loop.
        let big = ext_from_f64(1.0e18);
        let r = ext_to_f64(&ext_prem1(&big, &ext_pi(), cw), cw);
        assert!(r.abs() <= core::f64::consts::PI / 2.0 + 1e-9, "r={r}");
    }

    #[test]
    fn raw_instruction_sanity() {
        let cw = CW_PC64_RN;
        // fyl2x: 1 * log2(8) = 3 exactly.
        let r = ext_fyl2x(&ext_one(), &ext_from_f64(8.0), cw);
        assert_eq!(ext_to_f64(&r, cw), 3.0);
        // fyl2xp1: 1 * log2(1 + 0) = 0.
        let r = ext_fyl2xp1(&ext_one(), &ext_from_f64(0.0), cw);
        assert_eq!(ext_to_f64(&r, cw), 0.0);
        // fsin/fcos/fptan agree with libm to ~1 ULP on a small argument.
        let x = 0.5f64;
        let s = ext_to_f64(&ext_sin(&ext_from_f64(x), cw), cw);
        let c = ext_to_f64(&ext_cos(&ext_from_f64(x), cw), cw);
        let t = ext_to_f64(&ext_tan(&ext_from_f64(x), cw), cw);
        assert!((s - x.sin()).abs() <= f64::EPSILON, "sin: {s} vs {}", x.sin());
        assert!((c - x.cos()).abs() <= f64::EPSILON, "cos: {c} vs {}", x.cos());
        assert!((t - x.tan()).abs() <= 2.0 * f64::EPSILON, "tan: {t} vs {}", x.tan());
        // sqrt/abs/chs.
        assert_eq!(ext_to_f64(&ext_sqrt(&ext_from_f64(9.0), cw), cw), 3.0);
        assert_eq!(ext_to_f64(&ext_abs(&ext_from_f64(-2.5), cw), cw), 2.5);
        assert_eq!(ext_to_f64(&ext_chs(&ext_from_f64(2.5), cw), cw), -2.5);
        // fscale: 3 * 2^4 = 48.
        let r = ext_scale(&ext_from_f64(3.0), &ext_from_f64(4.0), cw);
        assert_eq!(ext_to_f64(&r, cw), 48.0);
        // ROM π rounds to the binary64 π on the store barrier.
        assert_eq!(ext_to_f64(&ext_pi(), cw), core::f64::consts::PI);
    }

    #[test]
    fn wrappers_match_production_kernels() {
        // POWER positive staging and the double-rounded helpers must be the
        // production code paths, not lookalikes.
        let p = f64::from_bits(0x3fffffffffffffff);
        assert_eq!(x87_recip(p).to_bits(), 0.5f64.to_bits());
        assert_eq!(excel_pow_positive(2.0, 0.5).to_bits(), 2.0f64.sqrt().to_bits());
        assert_eq!(excel_exp(1.0).to_bits(), std::f64::consts::E.to_bits());
        assert_eq!(excel_ln(std::f64::consts::E), 1.0);
        assert_eq!(excel_log10(1000.0), 3.0);
    }
}
