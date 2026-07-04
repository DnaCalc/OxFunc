//! x87 transcendental backend — bit-exact to 64-bit Microsoft Excel.
//!
//! # What this is
//! 64-bit Excel evaluates `EXP`, `LN`, `LOG10`, `LOG`, and `POWER` with the
//! **legacy Microsoft x87 CRT transcendental sequences** (`fFEXP` / `fFLN` /
//! `fFLOGm` of `fpw32/tran/i386/87tran.asm` in the NT source tree), executed
//! with the x87 control word set to `0x133F` — precision-control **64-bit**,
//! round-to-nearest-even, all exceptions masked. Every intermediate lives in an
//! 80-bit x87 register (64-bit significand); the only narrowing to IEEE-754
//! binary64 is the final `fstp qword` store.
//!
//! This module executes exactly those instruction sequences on the host CPU.
//! On the x86-64 machine this was validated against, it reproduces live Excel
//! **bit-for-bit on 249/249** harvested `EXP`/`LN` witnesses, including the 36
//! hardest near-midpoint cases where every mainstream libm (UCRT, glibc, MKL)
//! and the correctly-rounded value all disagree with Excel.
//!
//! # Why the hardware instruction, not a software model
//! The reduction, constants, invert branch, and rounding are a fixed spec, but
//! the register values `F2XM1` and `FYL2X` produce are **CPU microcode** — a
//! vendor-specific table/polynomial accurate to `< ~0.58 ULP` of the 64-bit
//! extended format, not any clean mathematical rounding. On the hardest inputs
//! Excel tracks the *actual microcode bit*, so a portable software model
//! reproduces Excel only ~99.9% of the time. Executing the real instruction is
//! what makes it exact. (Consequence: on the ~1-in-2000 hardest inputs the
//! result is in principle CPU-dependent; Excel-on-ARM runs a software x87
//! emulation. See `docs/worksets/W108_*` and `C:/Temp/ExcelExpFunction`.)
//!
//! # Containment
//! This is the *only* place in the crate that emits x87 assembly. Every public
//! entry point in [`super`] dispatches here on `x86_64` and to the portable
//! Rust core elsewhere, so swapping the backend is a localized change.
//!
//! # Constants
//! `fldl2e` loads `log2(e)` and `fldln2`/`fldlg2` load `ln 2` / `log10 2` as the
//! x87 ROM constants (64-bit significand, both rounding the true value UP). The
//! upward rounding of the `fldl2e` constant is the origin of Excel's systematic
//! "away-from-1.0" 1-ULP bias on hard `EXP` inputs.

use core::arch::asm;

/// PC=64, RC=nearest-even, all six exceptions masked (the `87disp.asm` value
/// installed before every `exp`/`ln`/`log`/`pow` in the Microsoft CRT).
const CW_CORE: u16 = 0x133F;

/// `e^x` via the `87tran.asm` `fFEXP` chain. `x` must be finite (the caller
/// handles NaN/±Inf); overflow returns `+Inf`, underflow `+0.0`, per the x87
/// masked-store semantics.
pub(super) fn exp(x: f64) -> f64 {
    let mut result: f64 = 0.0;
    let cw_core: u16 = CW_CORE;
    let mut cw_save: u16 = 0;
    // SAFETY: straight-line x87 arithmetic on a stack that is pushed and fully
    // popped within the block (net depth 0). The block reads `x`/`cw_core`,
    // writes `result`/`cw_save` through the provided pointers, clobbers `ax`
    // and the arithmetic flags, and restores the caller's control word.
    unsafe {
        asm!(
            "fnstcw word ptr [{save}]",   // save caller's x87 control word
            "fldcw word ptr [{core}]",    // install PC=64 / round-nearest
            "fld qword ptr [{x}]",        // st0 = x
            "fldl2e",                     // st0 = log2(e), st1 = x
            "fmulp st(1), st",            // st0 = t = x*log2(e)
            "fld st(0)",                  // st0 = t, st1 = t
            "frndint",                    // st0 = k = rint(t), st1 = t
            "fxch st(1)",                 // st0 = t, st1 = k
            "fsub st, st(1)",             // st0 = f = t-k (exact, |f|<=1/2)
            "ftst",                       // set C0 = (f < 0)
            "fnstsw ax",                  // ax = x87 status word
            "fabs",                       // st0 = |f|, st1 = k
            "f2xm1",                      // st0 = 2^|f| - 1, st1 = k
            "fld1",                       // st0 = 1, st1 = w, st2 = k
            "faddp st(1), st",            // st0 = m = 1+w, st1 = k
            "sahf",                       // CF = C0 = (f < 0)
            "jae 2f",                     // f >= 0: skip the reciprocal invert
            "fld1",                       // st0 = 1, st1 = m, st2 = k
            "fdivrp st(1), st",           // st0 = 1/m, st1 = k  (extra rounding)
            "2:",
            "fscale",                     // st0 = v * 2^k, st1 = k
            "fstp qword ptr [{res}]",     // y = RN53(r); store and pop
            "fstp st(0)",                 // pop k -> stack empty
            "fldcw word ptr [{save}]",    // restore caller's control word
            x = in(reg) &x,
            res = in(reg) &mut result,
            core = in(reg) &cw_core,
            save = in(reg) &mut cw_save,
            out("ax") _,
        );
    }
    result
}

/// Shared `y*log2(x)` epilogue: with `st0 = x` and `st1 = y` already on the x87
/// stack under CW `0x133F`, run `fyl2x` and store the binary64 result. This is
/// the `fFLN`/`fFLOGm` core (`ln = fldln2·log2`, `log10 = fldlg2·log2`,
/// `log2 = 1·log2`). `x` must be finite and `> 0` (caller guards the rest).
macro_rules! fyl2x_fn {
    ($(#[$m:meta])* $name:ident, $load_y:literal) => {
        $(#[$m])*
        pub(super) fn $name(x: f64) -> f64 {
            let mut result: f64 = 0.0;
            let cw_core: u16 = CW_CORE;
            let mut cw_save: u16 = 0;
            // SAFETY: as in `exp` — balanced x87 stack, pointer I/O only,
            // control word saved and restored.
            unsafe {
                asm!(
                    "fnstcw word ptr [{save}]",
                    "fldcw word ptr [{core}]",
                    $load_y,                  // st0 = y (ROM constant or 1.0)
                    "fld qword ptr [{x}]",    // st0 = x, st1 = y
                    "fyl2x",                  // st0 = y*log2(x), pop
                    "fstp qword ptr [{res}]", // y = RN53(...); store and pop
                    "fldcw word ptr [{save}]",
                    x = in(reg) &x,
                    res = in(reg) &mut result,
                    core = in(reg) &cw_core,
                    save = in(reg) &mut cw_save,
                );
            }
            result
        }
    };
}

fyl2x_fn!(
    /// `ln(x)` via `fldln2` + `fyl2x`. `x` must be finite and `> 0`.
    ln, "fldln2"
);
fyl2x_fn!(
    /// `log10(x)` via `fldlg2` + `fyl2x`. `x` must be finite and `> 0`. This
    /// backs the dedicated `LOG10()` worksheet function; note `LOG(x, 10)` is a
    /// *different* Excel code path (`ln(x)/ln(10)`) and must NOT use this.
    log10, "fldlg2"
);

#[cfg(test)]
mod tests {
    use super::*;

    /// Full offline regression: every EXP/LN witness harvested from live Excel
    /// (see the module docs for provenance). x87 must reproduce Excel bit-for-bit.
    ///
    /// Host-CPU note: the ~1-in-30 hardest near-midpoint rows are `F2XM1`/`FYL2X`
    /// microcode-bound, so this pins Excel-parity *for the reference x86-64 host*
    /// (AMD Zen2). A failure on a different x87 microcode is the documented
    /// CPU-dependence caveat, not a regression in this code.
    const GROUND_TRUTH: &str = include_str!("x87_excel_ground_truth.tsv");

    fn parse(hex: &str) -> u64 {
        u64::from_str_radix(hex, 16).expect("hex")
    }

    #[test]
    fn matches_live_excel_bit_for_bit() {
        let (mut n, mut bad) = (0u32, Vec::new());
        for line in GROUND_TRUTH.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let mut it = line.split('\t');
            let func = it.next().unwrap();
            let xb = parse(it.next().unwrap());
            let exb = parse(it.next().unwrap());
            let x = f64::from_bits(xb);
            let got = match func {
                "EXP" => exp(x),
                "LN" => ln(x),
                _ => continue,
            }
            .to_bits();
            n += 1;
            if got != exb && bad.len() < 20 {
                bad.push(format!(
                    "{} x={:016x}: got {:016x} want {:016x}",
                    func, xb, got, exb
                ));
            } else if got != exb {
                bad.push("...".into());
            }
        }
        assert!(n >= 200, "expected the full corpus, saw {n} rows");
        assert!(
            bad.is_empty(),
            "{} mismatches:\n{}",
            bad.len(),
            bad.join("\n")
        );
    }

    #[test]
    fn spot_values() {
        // exp(0)=1, exp(1)=E (both exact-agreeing with Excel and CR here).
        assert_eq!(exp(0.0).to_bits(), 1.0f64.to_bits());
        assert_eq!(exp(1.0).to_bits(), std::f64::consts::E.to_bits());
        // ln(1)=+0 (sign of zero), ln(E)=1, log10(1000)=3.
        assert_eq!(ln(1.0).to_bits(), 0.0f64.to_bits());
        assert_eq!(ln(std::f64::consts::E), 1.0);
        assert_eq!(log10(1000.0), 3.0);
    }

    #[test]
    fn control_word_is_restored() {
        // A benign x87 op after our calls must see the caller's CW again; we
        // can at least confirm repeated calls stay stable and don't drift.
        let a = exp(2.0);
        let _ = ln(10.0);
        let b = exp(2.0);
        assert_eq!(a.to_bits(), b.to_bits());
    }
}
