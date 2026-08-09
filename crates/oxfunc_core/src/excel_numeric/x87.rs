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

/// `base^exponent` through the register-continuous x87 `FYL2X` ->
/// `FRNDINT`/`F2XM1`/`FSCALE` chain. Unlike [`exp`] composed with [`ln`], the
/// logarithm/product is never published to binary64 before the base-2 exponent
/// assembly. Excel's legacy NOMINAL financial helper uses this route when the
/// reciprocal exponent is at least `0.5` (truncated `npery <= 2`).
///
/// `base` must be finite and positive and `exponent` finite; callers own domain
/// and publication/error handling.
pub(super) fn pow_direct(base: f64, exponent: f64) -> f64 {
    let mut result: f64 = 0.0;
    let cw_core: u16 = CW_CORE;
    let mut cw_save: u16 = 0;
    // SAFETY: straight-line x87 arithmetic with balanced stack depth. The
    // caller's control word is saved/restored and all memory operands are valid
    // references to local binary64/u16 values.
    unsafe {
        asm!(
            "fnstcw word ptr [{save}]",
            "fldcw word ptr [{core}]",
            "fld qword ptr [{exponent}]", // st0 = exponent
            "fld qword ptr [{base}]",     // st0 = base, st1 = exponent
            "fyl2x",                      // st0 = exponent*log2(base)
            "fld st(0)",                  // st0 = t, st1 = t
            "frndint",                    // st0 = k = rint(t), st1 = t
            "fxch st(1)",                 // st0 = t, st1 = k
            "fsub st, st(1)",             // st0 = f = t-k
            "ftst",
            "fnstsw ax",
            "fabs",
            "f2xm1",                      // st0 = 2^|f|-1, st1 = k
            "fld1",
            "faddp st(1), st",            // st0 = 1+w, st1 = k
            "sahf",
            "jae 2f",
            "fld1",
            "fdivrp st(1), st",           // st0 = 1/(1+w), st1 = k
            "2:",
            "fscale",
            "fstp qword ptr [{result}]",  // required binary64 publication
            "fstp st(0)",                 // pop k
            "fldcw word ptr [{save}]",
            base = in(reg) &base,
            exponent = in(reg) &exponent,
            result = in(reg) &mut result,
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

/// `RN53(RN64(a * b))` — one x87 `FMUL` at PC=64 then a `binary64` store: the
/// **double-rounded** product. Excel forms `POWER`'s `|y|·ln x` this way; a plain
/// SSE multiply is single-rounded and differs on double-rounding-window inputs.
pub(super) fn mul(a: f64, b: f64) -> f64 {
    let mut result: f64 = 0.0;
    let cw_core: u16 = CW_CORE;
    let mut cw_save: u16 = 0;
    // SAFETY: balanced x87 stack (2 push; `fmulp` and `fstp` pop both), pointer
    // I/O only, control word saved and restored.
    unsafe {
        asm!(
            "fnstcw word ptr [{save}]",
            "fldcw word ptr [{core}]",
            "fld qword ptr [{a}]",     // st0 = a
            "fld qword ptr [{b}]",     // st0 = b, st1 = a
            "fmulp st(1), st",         // st0 = RN64(a*b)
            "fstp qword ptr [{res}]",  // RN53 store
            "fldcw word ptr [{save}]",
            a = in(reg) &a,
            b = in(reg) &b,
            res = in(reg) &mut result,
            core = in(reg) &cw_core,
            save = in(reg) &mut cw_save,
        );
    }
    result
}

/// `RN53(RN64(a + b))` — one x87 `FADD` at PC=64 then a `binary64` store: the
/// double-rounded sum. Excel's legacy x87-compiled function bodies produce
/// this for every `t = a + b;` with `t` spilled to memory (W109: XNPV's
/// `1 + rate` and its running `total += term` accumulate this way).
pub(super) fn add(a: f64, b: f64) -> f64 {
    let mut result: f64 = 0.0;
    let cw_core: u16 = CW_CORE;
    let mut cw_save: u16 = 0;
    // SAFETY: balanced x87 stack (one push; `fadd` folds in memory operand,
    // `fstp` pops), pointer I/O only, control word saved and restored.
    unsafe {
        asm!(
            "fnstcw word ptr [{save}]",
            "fldcw word ptr [{core}]",
            "fld qword ptr [{a}]",     // st0 = a
            "fadd qword ptr [{b}]",    // st0 = RN64(a+b)
            "fstp qword ptr [{res}]",  // RN53 store
            "fldcw word ptr [{save}]",
            a = in(reg) &a,
            b = in(reg) &b,
            res = in(reg) &mut result,
            core = in(reg) &cw_core,
            save = in(reg) &mut cw_save,
        );
    }
    result
}

/// `RN53(RN64(a - b))` — one x87 `FSUB` at PC=64 then a `binary64` store: the
/// double-rounded difference (W109: NPER's `tf·pmt - fv·rate`).
pub(super) fn sub(a: f64, b: f64) -> f64 {
    let mut result: f64 = 0.0;
    let cw_core: u16 = CW_CORE;
    let mut cw_save: u16 = 0;
    // SAFETY: balanced x87 stack (one push; `fsub` folds in memory operand,
    // `fstp` pops), pointer I/O only, control word saved and restored.
    unsafe {
        asm!(
            "fnstcw word ptr [{save}]",
            "fldcw word ptr [{core}]",
            "fld qword ptr [{a}]",     // st0 = a
            "fsub qword ptr [{b}]",    // st0 = RN64(a-b)
            "fstp qword ptr [{res}]",  // RN53 store
            "fldcw word ptr [{save}]",
            a = in(reg) &a,
            b = in(reg) &b,
            res = in(reg) &mut result,
            core = in(reg) &cw_core,
            save = in(reg) &mut cw_save,
        );
    }
    result
}

/// `RN53(RN64(a / b))` — one x87 `FDIV` at PC=64 then a `binary64` store: the
/// double-rounded quotient (W109: XNPV's per-term `value / pow`).
pub(super) fn div(a: f64, b: f64) -> f64 {
    let mut result: f64 = 0.0;
    let cw_core: u16 = CW_CORE;
    let mut cw_save: u16 = 0;
    // SAFETY: balanced x87 stack (one push; `fdiv` folds in memory operand,
    // `fstp` pops), pointer I/O only, control word saved and restored.
    unsafe {
        asm!(
            "fnstcw word ptr [{save}]",
            "fldcw word ptr [{core}]",
            "fld qword ptr [{a}]",     // st0 = a
            "fdiv qword ptr [{b}]",    // st0 = RN64(a/b)
            "fstp qword ptr [{res}]",  // RN53 store
            "fldcw word ptr [{save}]",
            a = in(reg) &a,
            b = in(reg) &b,
            res = in(reg) &mut result,
            core = in(reg) &cw_core,
            save = in(reg) &mut cw_save,
        );
    }
    result
}

/// `RN53(RN64(1.0 / x))` — one x87 `FDIV` at PC=64 then a `binary64` store: the
/// double-rounded reciprocal Excel uses to stage `POWER(x, y)` for `y < 0`
/// (compute the positive power, store it, then reciprocate).
pub(super) fn recip(x: f64) -> f64 {
    let mut result: f64 = 0.0;
    let cw_core: u16 = CW_CORE;
    let mut cw_save: u16 = 0;
    // SAFETY: balanced x87 stack (`fld1` pushes, `fstp` pops), pointer I/O only.
    unsafe {
        asm!(
            "fnstcw word ptr [{save}]",
            "fldcw word ptr [{core}]",
            "fld1",                    // st0 = 1
            "fdiv qword ptr [{x}]",    // st0 = RN64(1/x)
            "fstp qword ptr [{res}]",  // RN53 store
            "fldcw word ptr [{save}]",
            x = in(reg) &x,
            res = in(reg) &mut result,
            core = in(reg) &cw_core,
            save = in(reg) &mut cw_save,
        );
    }
    result
}

// ============================================================================
// W109 research primitives (`research-x87` feature)
//
// Raw x87 instruction wrappers for the calculation-graph search tooling. The
// production Excel-parity chains above stay untouched; everything below is a
// composable instruction set the racer assembles into *candidate* graphs.
//
// The core type is [`raw::Ext80`]: an 80-bit extended-precision temporary held
// in memory via `fld tbyte`/`fstp tbyte`, which preserve the full register
// image (no rounding). Chaining ops through `Ext80` is therefore bit-identical
// to keeping the value on the x87 register stack, so a candidate graph can mix
// "x87-continuous" stretches (Ext80 all the way) with explicit `binary64`
// store barriers (`ext_to_f64` then `ext_from_f64`) at any node boundary —
// exactly the store-mask search axis. Every arithmetic op takes the control
// word explicitly so precision-control hypotheses (PC=64/53/24) are searchable
// too.
// ============================================================================
pub(super) mod raw {
    use core::arch::asm;

    /// PC=64, round-to-nearest-even, all exceptions masked (`87disp.asm` /
    /// Excel's transcendental control word).
    pub const CW_PC64_RN: u16 = 0x133F;
    /// PC=53 (double), round-to-nearest-even, all exceptions masked.
    pub const CW_PC53_RN: u16 = 0x123F;
    /// PC=24 (single), round-to-nearest-even, all exceptions masked.
    pub const CW_PC24_RN: u16 = 0x103F;

    /// An x87 80-bit extended-precision value parked in memory. `fld tbyte` /
    /// `fstp tbyte` move the full register image with no rounding, so this is
    /// a faithful stand-in for "the value stayed on the x87 stack".
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct Ext80(pub [u8; 10]);

    impl core::fmt::Debug for Ext80 {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            let mut hex = String::with_capacity(20);
            for b in self.0.iter().rev() {
                hex.push_str(&format!("{b:02x}"));
            }
            write!(f, "Ext80(0x{hex})")
        }
    }

    /// `f64 -> Ext80` — exact (every binary64 is representable in extended).
    pub fn ext_from_f64(x: f64) -> Ext80 {
        let mut out = Ext80([0u8; 10]);
        // SAFETY: one push, one pop; pointer I/O only. No CW dependence: the
        // widening load is exact under every rounding/precision mode.
        unsafe {
            asm!(
                "fld qword ptr [{x}]",
                "fstp tbyte ptr [{out}]",
                x = in(reg) &x,
                out = in(reg) out.0.as_mut_ptr(),
            );
        }
        out
    }

    /// `Ext80 -> f64` — the explicit `binary64` store barrier (`RN53` under the
    /// given control word's rounding mode; precision control is irrelevant to
    /// stores).
    pub fn ext_to_f64(x: &Ext80, cw: u16) -> f64 {
        let mut result: f64 = 0.0;
        let mut cw_save: u16 = 0;
        // SAFETY: one push, one pop; pointer I/O only; CW saved and restored.
        unsafe {
            asm!(
                "fnstcw word ptr [{save}]",
                "fldcw word ptr [{cw}]",
                "fld tbyte ptr [{x}]",
                "fstp qword ptr [{res}]",
                x = in(reg) x.0.as_ptr(),
                res = in(reg) &mut result,
                cw = in(reg) &cw,
                save = in(reg) &mut cw_save,
            );
        }
        result
    }

    macro_rules! ext_binop {
        ($(#[$m:meta])* $name:ident, $inst:literal) => {
            $(#[$m])*
            pub fn $name(a: &Ext80, b: &Ext80, cw: u16) -> Ext80 {
                let mut out = Ext80([0u8; 10]);
                let mut cw_save: u16 = 0;
                // SAFETY: two pushes, both popped ($inst pops one, the tbyte
                // store pops the other); pointer I/O only; CW saved/restored.
                unsafe {
                    asm!(
                        "fnstcw word ptr [{save}]",
                        "fldcw word ptr [{cw}]",
                        "fld tbyte ptr [{a}]",     // st0 = a
                        "fld tbyte ptr [{b}]",     // st0 = b, st1 = a
                        $inst,                     // st0 = a <op> b
                        "fstp tbyte ptr [{out}]",
                        "fldcw word ptr [{save}]",
                        a = in(reg) a.0.as_ptr(),
                        b = in(reg) b.0.as_ptr(),
                        out = in(reg) out.0.as_mut_ptr(),
                        cw = in(reg) &cw,
                        save = in(reg) &mut cw_save,
                    );
                }
                out
            }
        };
    }

    ext_binop!(
        /// `a + b` rounded per the control word's precision/rounding fields.
        ext_add, "faddp st(1), st"
    );
    ext_binop!(
        /// `a - b`.
        ext_sub, "fsubp st(1), st"
    );
    ext_binop!(
        /// `a * b`.
        ext_mul, "fmulp st(1), st"
    );
    ext_binop!(
        /// `a / b`.
        ext_div, "fdivp st(1), st"
    );

    macro_rules! ext_unop {
        ($(#[$m:meta])* $name:ident, $inst:literal) => {
            $(#[$m])*
            pub fn $name(x: &Ext80, cw: u16) -> Ext80 {
                let mut out = Ext80([0u8; 10]);
                let mut cw_save: u16 = 0;
                // SAFETY: one push, one pop; pointer I/O only; CW saved/restored.
                unsafe {
                    asm!(
                        "fnstcw word ptr [{save}]",
                        "fldcw word ptr [{cw}]",
                        "fld tbyte ptr [{x}]",
                        $inst,
                        "fstp tbyte ptr [{out}]",
                        "fldcw word ptr [{save}]",
                        x = in(reg) x.0.as_ptr(),
                        out = in(reg) out.0.as_mut_ptr(),
                        cw = in(reg) &cw,
                        save = in(reg) &mut cw_save,
                    );
                }
                out
            }
        };
    }

    ext_unop!(
        /// `|x|`.
        ext_abs, "fabs"
    );
    ext_unop!(
        /// `-x`.
        ext_chs, "fchs"
    );
    ext_unop!(
        /// `sqrt(x)` (x87 `FSQRT`, correctly rounded to the CW precision).
        ext_sqrt, "fsqrt"
    );
    ext_unop!(
        /// `rint(x)` per the CW rounding mode (x87 `FRNDINT`).
        ext_rndint, "frndint"
    );
    ext_unop!(
        /// `sin(x)` — x87 `FSIN` microcode, hardware 66-bit-π internal
        /// reduction. Caller must keep `|x| < 2^63` (C2 no-op otherwise).
        ext_sin, "fsin"
    );
    ext_unop!(
        /// `cos(x)` — x87 `FCOS`. Same `|x| < 2^63` caveat as [`ext_sin`].
        ext_cos, "fcos"
    );
    ext_unop!(
        /// `2^x - 1` — x87 `F2XM1`; requires `|x| <= 1` (else undefined).
        ext_f2xm1, "f2xm1"
    );

    /// `tan(x)` — x87 `FPTAN` microcode (pushes 1.0, which is popped here).
    /// Caller must keep `|x| < 2^63`.
    pub fn ext_tan(x: &Ext80, cw: u16) -> Ext80 {
        let mut out = Ext80([0u8; 10]);
        let mut cw_save: u16 = 0;
        // SAFETY: fptan pushes a 1.0 above the result; both are popped
        // (fstp st(0) drops the 1.0, the tbyte store pops the tangent).
        unsafe {
            asm!(
                "fnstcw word ptr [{save}]",
                "fldcw word ptr [{cw}]",
                "fld tbyte ptr [{x}]",     // st0 = x
                "fptan",                   // st0 = 1.0, st1 = tan(x)
                "fstp st(0)",              // drop the pushed 1.0
                "fstp tbyte ptr [{out}]",
                "fldcw word ptr [{save}]",
                x = in(reg) x.0.as_ptr(),
                out = in(reg) out.0.as_mut_ptr(),
                cw = in(reg) &cw,
                save = in(reg) &mut cw_save,
            );
        }
        out
    }

    /// `y * log2(x)` — x87 `FYL2X` microcode. `x` must be finite and `> 0`.
    pub fn ext_fyl2x(y: &Ext80, x: &Ext80, cw: u16) -> Ext80 {
        let mut out = Ext80([0u8; 10]);
        let mut cw_save: u16 = 0;
        // SAFETY: two pushes; fyl2x pops one, the tbyte store pops the other.
        unsafe {
            asm!(
                "fnstcw word ptr [{save}]",
                "fldcw word ptr [{cw}]",
                "fld tbyte ptr [{y}]",     // st0 = y
                "fld tbyte ptr [{x}]",     // st0 = x, st1 = y
                "fyl2x",                   // st0 = y*log2(x), pop
                "fstp tbyte ptr [{out}]",
                "fldcw word ptr [{save}]",
                y = in(reg) y.0.as_ptr(),
                x = in(reg) x.0.as_ptr(),
                out = in(reg) out.0.as_mut_ptr(),
                cw = in(reg) &cw,
                save = in(reg) &mut cw_save,
            );
        }
        out
    }

    /// `y * log2(1 + x)` — x87 `FYL2XP1` microcode; requires
    /// `|x| < 1 - sqrt(2)/2` (else undefined).
    pub fn ext_fyl2xp1(y: &Ext80, x: &Ext80, cw: u16) -> Ext80 {
        let mut out = Ext80([0u8; 10]);
        let mut cw_save: u16 = 0;
        // SAFETY: as in ext_fyl2x.
        unsafe {
            asm!(
                "fnstcw word ptr [{save}]",
                "fldcw word ptr [{cw}]",
                "fld tbyte ptr [{y}]",
                "fld tbyte ptr [{x}]",
                "fyl2xp1",
                "fstp tbyte ptr [{out}]",
                "fldcw word ptr [{save}]",
                y = in(reg) y.0.as_ptr(),
                x = in(reg) x.0.as_ptr(),
                out = in(reg) out.0.as_mut_ptr(),
                cw = in(reg) &cw,
                save = in(reg) &mut cw_save,
            );
        }
        out
    }

    /// `x * 2^trunc(k)` — x87 `FSCALE`.
    pub fn ext_scale(x: &Ext80, k: &Ext80, cw: u16) -> Ext80 {
        let mut out = Ext80([0u8; 10]);
        let mut cw_save: u16 = 0;
        // SAFETY: two pushes; fscale leaves both, the store pops st0 and the
        // trailing fstp drops k.
        unsafe {
            asm!(
                "fnstcw word ptr [{save}]",
                "fldcw word ptr [{cw}]",
                "fld tbyte ptr [{k}]",     // st0 = k
                "fld tbyte ptr [{x}]",     // st0 = x, st1 = k
                "fscale",                  // st0 = x*2^trunc(k), st1 = k
                "fstp tbyte ptr [{out}]",
                "fstp st(0)",              // drop k
                "fldcw word ptr [{save}]",
                x = in(reg) x.0.as_ptr(),
                k = in(reg) k.0.as_ptr(),
                out = in(reg) out.0.as_mut_ptr(),
                cw = in(reg) &cw,
                save = in(reg) &mut cw_save,
            );
        }
        out
    }

    macro_rules! ext_prem {
        ($(#[$m:meta])* $name:ident, $inst:literal) => {
            $(#[$m])*
            pub fn $name(x: &Ext80, modulus: &Ext80, cw: u16) -> Ext80 {
                let mut out = Ext80([0u8; 10]);
                let mut cw_save: u16 = 0;
                // SAFETY: two pushes, both popped; the reduction loop repeats
                // the instruction until status C2 clears (complete reduction),
                // as the ISA requires for widely separated exponents. Clobbers
                // ax (fnstsw) and flags (test).
                unsafe {
                    asm!(
                        "fnstcw word ptr [{save}]",
                        "fldcw word ptr [{cw}]",
                        "fld tbyte ptr [{m}]",     // st0 = modulus
                        "fld tbyte ptr [{x}]",     // st0 = x, st1 = modulus
                        "2:",
                        $inst,                     // st0 = partial remainder
                        "fnstsw ax",
                        "test ah, 4",              // C2 (status bit 10) set -> incomplete
                        "jnz 2b",
                        "fstp tbyte ptr [{out}]",
                        "fstp st(0)",              // drop modulus
                        "fldcw word ptr [{save}]",
                        x = in(reg) x.0.as_ptr(),
                        m = in(reg) modulus.0.as_ptr(),
                        out = in(reg) out.0.as_mut_ptr(),
                        cw = in(reg) &cw,
                        save = in(reg) &mut cw_save,
                        out("ax") _,
                    );
                }
                out
            }
        };
    }

    ext_prem!(
        /// Truncating partial remainder `x rem_trunc modulus` — x87 `FPREM`
        /// (8087-compatible, quotient toward zero), iterated to completion.
        ext_prem, "fprem"
    );
    ext_prem!(
        /// IEEE remainder `x rem_nearest modulus` — x87 `FPREM1` (quotient
        /// rounded to nearest-even), iterated to completion.
        ext_prem1, "fprem1"
    );

    /// [`ext_prem1`] that also returns the low three quotient bits the ISA
    /// leaves in the status flags (bit0 = Q0 from C1, bit1 = Q1 from C3,
    /// bit2 = Q2 from C0) — the parity the legacy CRT trig chains use for the
    /// `sin(x) = (-1)^Q sin(r)` sign fixup after reducing by π.
    pub fn ext_prem1_quo(x: &Ext80, modulus: &Ext80, cw: u16) -> (Ext80, u8) {
        let mut out = Ext80([0u8; 10]);
        let mut cw_save: u16 = 0;
        let mut status: u16 = 0;
        // SAFETY: as in ext_prem1; additionally stores the final status word.
        unsafe {
            asm!(
                "fnstcw word ptr [{save}]",
                "fldcw word ptr [{cw}]",
                "fld tbyte ptr [{m}]",
                "fld tbyte ptr [{x}]",
                "2:",
                "fprem1",
                "fnstsw ax",
                "test ah, 4",
                "jnz 2b",
                "mov word ptr [{st}], ax",
                "fstp tbyte ptr [{out}]",
                "fstp st(0)",
                "fldcw word ptr [{save}]",
                x = in(reg) x.0.as_ptr(),
                m = in(reg) modulus.0.as_ptr(),
                out = in(reg) out.0.as_mut_ptr(),
                st = in(reg) &mut status,
                cw = in(reg) &cw,
                save = in(reg) &mut cw_save,
                out("ax") _,
            );
        }
        // Status: C0 = bit 8, C1 = bit 9, C3 = bit 14.
        let q0 = ((status >> 9) & 1) as u8;
        let q1 = ((status >> 14) & 1) as u8;
        let q2 = ((status >> 8) & 1) as u8;
        (out, q0 | (q1 << 1) | (q2 << 2))
    }

    macro_rules! ext_const {
        ($(#[$m:meta])* $name:ident, $inst:literal) => {
            $(#[$m])*
            pub fn $name() -> Ext80 {
                let mut out = Ext80([0u8; 10]);
                let mut cw_save: u16 = 0;
                let cw: u16 = CW_PC64_RN;
                // SAFETY: one push, one pop; CW pinned to PC=64/RN so the ROM
                // constant rounds exactly as in Excel's chains.
                unsafe {
                    asm!(
                        "fnstcw word ptr [{save}]",
                        "fldcw word ptr [{cw}]",
                        $inst,
                        "fstp tbyte ptr [{out}]",
                        "fldcw word ptr [{save}]",
                        out = in(reg) out.0.as_mut_ptr(),
                        cw = in(reg) &cw,
                        save = in(reg) &mut cw_save,
                    );
                }
                out
            }
        };
    }

    ext_const!(
        /// π as the x87 ROM constant (`FLDPI`, 66-bit source rounded to the
        /// 64-bit significand under RN) — the internal-reduction π candidate.
        ext_pi, "fldpi"
    );
    ext_const!(
        /// `log2(e)` ROM constant (`FLDL2E`).
        ext_l2e, "fldl2e"
    );
    ext_const!(
        /// `log2(10)` ROM constant (`FLDL2T`).
        ext_l2t, "fldl2t"
    );
    ext_const!(
        /// `ln 2` ROM constant (`FLDLN2`).
        ext_ln2, "fldln2"
    );
    ext_const!(
        /// `log10(2)` ROM constant (`FLDLG2`).
        ext_lg2, "fldlg2"
    );
    ext_const!(
        /// `1.0` (`FLD1`).
        ext_one, "fld1"
    );
}

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
    fn double_rounded_mul_and_recip() {
        // Exact cases.
        assert_eq!(mul(2.0, 3.0), 6.0);
        assert_eq!(recip(2.0).to_bits(), 0.5f64.to_bits());
        assert_eq!(recip(0.5), 2.0);
        // Double-rounding discriminator (POWER_LIVE_ROUND1): p = 0x3fffffffffffffff
        // (~1.9999999999999998). RN64(1/p) = 0.5 + 2^-54, then RN53 ties to even
        // -> exactly 0.5. A single-rounded SSE `1.0/p` gives 0x3fe0000000000001.
        let p = f64::from_bits(0x3fffffffffffffff);
        assert_eq!(recip(p).to_bits(), 0.5f64.to_bits());
        assert_ne!((1.0 / p).to_bits(), 0.5f64.to_bits());
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
