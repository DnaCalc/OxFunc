use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, run_values_only_prepared,
};
use crate::functions::normal_dist_common::erf_approx;
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

const SPECIAL_DIST_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SPECIAL_DIST_BASE",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

pub const ERF_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ERF",
    arity: Arity { min: 1, max: 2 },
    ..SPECIAL_DIST_BASE_META
};

pub const ERF_PRECISE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ERF.PRECISE",
    ..SPECIAL_DIST_BASE_META
};

pub const ERFC_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ERFC",
    ..SPECIAL_DIST_BASE_META
};

pub const ERFC_PRECISE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ERFC.PRECISE",
    ..SPECIAL_DIST_BASE_META
};

pub const GAMMA_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.GAMMA",
    ..SPECIAL_DIST_BASE_META
};

pub const GAMMALN_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.GAMMALN",
    ..SPECIAL_DIST_BASE_META
};

pub const GAMMALN_PRECISE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.GAMMALN.PRECISE",
    ..SPECIAL_DIST_BASE_META
};

pub const WEIBULL_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.WEIBULL",
    arity: Arity::exact(4),
    ..SPECIAL_DIST_BASE_META
};

pub const WEIBULL_DIST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.WEIBULL.DIST",
    arity: Arity::exact(4),
    ..SPECIAL_DIST_BASE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum SpecialDistEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

const LANCZOS_G: f64 = 7.0;
const LANCZOS_COEFFS: [f64; 9] = [
    0.999_999_999_999_809_9,
    676.520_368_121_885_1,
    -1_259.139_216_722_402_8,
    771.323_428_777_653_1,
    -176.615_029_162_140_6,
    12.507_343_278_686_905,
    -0.138_571_095_265_720_12,
    9.984_369_578_019_572e-6,
    1.505_632_735_149_311_6e-7,
];

fn arity_error(meta: &FunctionMeta, actual: usize) -> SpecialDistEvalError {
    SpecialDistEvalError::ArityMismatch {
        expected_min: meta.arity.min,
        expected_max: meta.arity.max,
        actual,
    }
}

fn bool_flag_from_number(n: f64) -> bool {
    n != 0.0
}

fn is_integer_like(x: f64) -> bool {
    (x - x.round()).abs() < 1.0e-12
}

fn has_gamma_pole(x: f64) -> bool {
    x <= 0.0 && is_integer_like(x)
}

fn ln_gamma_positive(x: f64) -> Result<f64, WorksheetErrorCode> {
    if !x.is_finite() || x <= 0.0 {
        return Err(WorksheetErrorCode::Num);
    }

    let z = x - 1.0;
    let mut acc = LANCZOS_COEFFS[0];
    for (i, coeff) in LANCZOS_COEFFS.iter().enumerate().skip(1) {
        acc += coeff / (z + i as f64);
    }

    let t = z + LANCZOS_G + 0.5;
    Ok(0.5 * (2.0 * std::f64::consts::PI).ln() + (z + 0.5) * t.ln() - t + acc.ln())
}

fn erf_interval(lower: f64, upper: f64) -> f64 {
    erf_approx(upper) - erf_approx(lower)
}

pub fn erf_kernel(lower: f64, upper: Option<f64>) -> Result<f64, WorksheetErrorCode> {
    if !lower.is_finite() || upper.is_some_and(|v| !v.is_finite()) {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(match upper {
        Some(upper) => erf_interval(lower, upper),
        None => erf_approx(lower),
    })
}

pub fn erf_precise_kernel(x: f64) -> Result<f64, WorksheetErrorCode> {
    if !x.is_finite() {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(erf_approx(x))
}

// Excel-emulation for the positive-tail ERFC regime.
//
// Policy: DnaCalc emulates Excel's observed output bits; mathematical
// correct-rounding is diagnostic only. See docs/function-lane/
// ERFC_EXCEL_EMULATION.md for the regime-map evidence behind the rule
// below.
//
// Rule (evidence-scored against the full widened witness set post 8e435fb):
// - x < 3.0 or x non-positive: libm::erfc. Matches Excel at 0, 0.5, 1,
//   1.25, 2.75, 2.8, and every tested negative point down to -10.
// - x >= 3.0 on Windows-MSVC: UCRT erfc. Matches Excel bit-exactly at the
//   tested large-x points (x=3, x=4, x=8). On other platforms the same
//   range stays on libm; non-Windows hosts have an accepted residual
//   divergence in the x>=3 tail.
//
// The positive subrange [1.5, 3.0) and several isolated large-x points
// (3.5, 5, 6, 10) remain blocked under every owner-local rule family we
// scored (libm, UCRT, min/max of both, bounded ULP adjustments on
// subranges) — Excel uses a polynomial that neither libm nor UCRT
// reproduces there. Those inputs are captured as #[ignore]d exact-bit
// witnesses below so the bits are preserved for future kernel work.
#[cfg(all(target_os = "windows", target_env = "msvc"))]
#[link(name = "ucrt")]
unsafe extern "C" {
    #[link_name = "erfc"]
    fn ucrt_erfc(x: f64) -> f64;
}

fn excel_erfc(x: f64) -> f64 {
    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    {
        if x >= 3.0 {
            return unsafe { ucrt_erfc(x) };
        }
    }
    libm::erfc(x)
}

pub fn erfc_kernel(x: f64) -> Result<f64, WorksheetErrorCode> {
    if !x.is_finite() {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(excel_erfc(x))
}

pub fn erfc_precise_kernel(x: f64) -> Result<f64, WorksheetErrorCode> {
    erfc_kernel(x)
}

pub fn gamma_kernel(x: f64) -> Result<f64, WorksheetErrorCode> {
    if !x.is_finite() || has_gamma_pole(x) {
        return Err(WorksheetErrorCode::Num);
    }

    let ln_gamma = if x < 0.5 {
        let reflected = 1.0 - x;
        let denom = (std::f64::consts::PI * x).sin();
        if denom == 0.0 || !denom.is_finite() {
            return Err(WorksheetErrorCode::Num);
        }
        std::f64::consts::PI.ln() - denom.abs().ln() - ln_gamma_positive(reflected)?
    } else {
        ln_gamma_positive(x)?
    };

    if ln_gamma > f64::MAX.ln() {
        return Err(WorksheetErrorCode::Num);
    }

    let magnitude = ln_gamma.exp();
    let value = if x < 0.5 && (std::f64::consts::PI * x).sin().is_sign_negative() {
        -magnitude
    } else {
        magnitude
    };

    if !value.is_finite() {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(value)
}

pub fn gammaln_kernel(x: f64) -> Result<f64, WorksheetErrorCode> {
    ln_gamma_positive(x)
}

pub fn gammaln_precise_kernel(x: f64) -> Result<f64, WorksheetErrorCode> {
    gammaln_kernel(x)
}

pub fn weibull_dist_kernel(
    x: f64,
    alpha: f64,
    beta: f64,
    cumulative: bool,
) -> Result<f64, WorksheetErrorCode> {
    if !x.is_finite()
        || !alpha.is_finite()
        || !beta.is_finite()
        || x < 0.0
        || alpha <= 0.0
        || beta <= 0.0
    {
        return Err(WorksheetErrorCode::Num);
    }

    if x == 0.0 {
        if cumulative {
            return Ok(0.0);
        }
        return Ok(0.0);
    }

    let ratio = x / beta;
    let power = ratio.powf(alpha);
    let exp_term = (-power).exp();
    let value = if cumulative {
        1.0 - exp_term
    } else {
        (alpha / beta) * ratio.powf(alpha - 1.0) * exp_term
    };

    if !value.is_finite() {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(value)
}

pub fn weibull_kernel(
    x: f64,
    alpha: f64,
    beta: f64,
    cumulative: bool,
) -> Result<f64, WorksheetErrorCode> {
    weibull_dist_kernel(x, alpha, beta, cumulative)
}

fn eval_erf_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, SpecialDistEvalError> {
    if !ERF_META.arity.accepts(args.len()) {
        return Err(arity_error(&ERF_META, args.len()));
    }
    let lower = coerce_prepared_to_number(&args[0]).map_err(SpecialDistEvalError::Coercion)?;
    let upper = if args.len() > 1 {
        Some(coerce_prepared_to_number(&args[1]).map_err(SpecialDistEvalError::Coercion)?)
    } else {
        None
    };
    Ok(match erf_kernel(lower, upper) {
        Ok(value) => EvalValue::Number(value),
        Err(code) => EvalValue::Error(code),
    })
}

fn eval_unary_prepared(
    args: &[PreparedArgValue],
    meta: &FunctionMeta,
    kernel: fn(f64) -> Result<f64, WorksheetErrorCode>,
) -> Result<EvalValue, SpecialDistEvalError> {
    if !meta.arity.accepts(args.len()) {
        return Err(arity_error(meta, args.len()));
    }
    let x = coerce_prepared_to_number(&args[0]).map_err(SpecialDistEvalError::Coercion)?;
    Ok(match kernel(x) {
        Ok(value) => EvalValue::Number(value),
        Err(code) => EvalValue::Error(code),
    })
}

fn eval_weibull_prepared(
    args: &[PreparedArgValue],
    meta: &FunctionMeta,
    kernel: fn(f64, f64, f64, bool) -> Result<f64, WorksheetErrorCode>,
) -> Result<EvalValue, SpecialDistEvalError> {
    if !meta.arity.accepts(args.len()) {
        return Err(arity_error(meta, args.len()));
    }
    let x = coerce_prepared_to_number(&args[0]).map_err(SpecialDistEvalError::Coercion)?;
    let alpha = coerce_prepared_to_number(&args[1]).map_err(SpecialDistEvalError::Coercion)?;
    let beta = coerce_prepared_to_number(&args[2]).map_err(SpecialDistEvalError::Coercion)?;
    let cumulative = coerce_prepared_to_number(&args[3]).map_err(SpecialDistEvalError::Coercion)?;
    Ok(
        match kernel(x, alpha, beta, bool_flag_from_number(cumulative)) {
            Ok(value) => EvalValue::Number(value),
            Err(code) => EvalValue::Error(code),
        },
    )
}

pub fn eval_erf_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, SpecialDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_erf_prepared,
        SpecialDistEvalError::Coercion,
    )
}

pub fn eval_erf_precise_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, SpecialDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_unary_prepared(prepared, &ERF_PRECISE_META, erf_precise_kernel),
        SpecialDistEvalError::Coercion,
    )
}

pub fn eval_erfc_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, SpecialDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_unary_prepared(prepared, &ERFC_META, erfc_kernel),
        SpecialDistEvalError::Coercion,
    )
}

pub fn eval_erfc_precise_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, SpecialDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_unary_prepared(prepared, &ERFC_PRECISE_META, erfc_precise_kernel),
        SpecialDistEvalError::Coercion,
    )
}

pub fn eval_gamma_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, SpecialDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_unary_prepared(prepared, &GAMMA_META, gamma_kernel),
        SpecialDistEvalError::Coercion,
    )
}

pub fn eval_gammaln_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, SpecialDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_unary_prepared(prepared, &GAMMALN_META, gammaln_kernel),
        SpecialDistEvalError::Coercion,
    )
}

pub fn eval_gammaln_precise_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, SpecialDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_unary_prepared(prepared, &GAMMALN_PRECISE_META, gammaln_precise_kernel),
        SpecialDistEvalError::Coercion,
    )
}

pub fn eval_weibull_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, SpecialDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_weibull_prepared(prepared, &WEIBULL_META, weibull_kernel),
        SpecialDistEvalError::Coercion,
    )
}

pub fn eval_weibull_dist_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, SpecialDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_weibull_prepared(prepared, &WEIBULL_DIST_META, weibull_dist_kernel),
        SpecialDistEvalError::Coercion,
    )
}

pub fn map_special_dist_error_to_ws(error: &SpecialDistEvalError) -> WorksheetErrorCode {
    match error {
        SpecialDistEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        SpecialDistEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        SpecialDistEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::ReferenceLike;

    struct NoResolver;

    impl ReferenceResolver for NoResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            ResolverCapabilities::permissive_local()
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            Err(RefResolutionError::UnresolvedReference {
                target: reference.target.clone(),
            })
        }
    }

    fn assert_close(actual: f64, expected: f64, tol: f64) {
        let delta = (actual - expected).abs();
        assert!(
            delta <= tol,
            "expected {expected}, got {actual}, delta {delta}"
        );
    }

    fn assert_bits_eq(label: &str, actual: f64, expected: f64) {
        assert_eq!(
            actual.to_bits(),
            expected.to_bits(),
            "{label}: actual {actual:e} ({:#018x}) vs expected {expected:e} ({:#018x})",
            actual.to_bits(),
            expected.to_bits()
        );
    }

    #[test]
    fn erfc_one_matches_excel_exact_bits() {
        assert_bits_eq("erfc(1)", erfc_kernel(1.0).unwrap(), 0.15729920705028513);
    }

    #[test]
    fn erfc_family_direct_call_witnesses() {
        // Excel-anchored exact bits. These are all in the libm-matching
        // regime (x <= 1.25, or all negatives).
        assert_bits_eq("erfc(0)", erfc_kernel(0.0).unwrap(), 1.0);
        assert_bits_eq("erfc(0.5)", erfc_kernel(0.5).unwrap(), 0.4795001221869535);
        assert_bits_eq("erfc(1)", erfc_kernel(1.0).unwrap(), 0.15729920705028513);
        assert_bits_eq("erfc(1.25)", erfc_kernel(1.25).unwrap(), 0.07709987174354177);
        assert_bits_eq("erfc(-1)", erfc_kernel(-1.0).unwrap(), 1.8427007929497148);
        assert_bits_eq("erfc(-2)", erfc_kernel(-2.0).unwrap(), 1.9953222650189528);

        // Libm-matching pockets within the positive regime (small island).
        assert_bits_eq(
            "erfc(2.75)",
            erfc_kernel(2.75).unwrap(),
            0.00010062192211963684,
        );
        assert_bits_eq(
            "erfc(2.8)",
            erfc_kernel(2.8).unwrap(),
            7.501319466545911e-5,
        );

        // ERFC.PRECISE delegates to the same kernel; spot-check parity at
        // one representative in-range anchor and one blocked-regime pocket.
        assert_bits_eq(
            "erfc.precise(0)",
            erfc_precise_kernel(0.0).unwrap(),
            1.0,
        );
        assert_bits_eq(
            "erfc.precise(1)",
            erfc_precise_kernel(1.0).unwrap(),
            0.15729920705028513,
        );
        assert_bits_eq(
            "erfc.precise(-1)",
            erfc_precise_kernel(-1.0).unwrap(),
            1.8427007929497148,
        );

        // Stable family controls across the full widened positive range.
        let xs: &[f64] = &[
            0.0, 0.5, 1.0, 1.25, 1.5, 1.75, 1.9, 2.0, 2.1, 2.25, 2.5, 2.6, 2.7, 2.75, 2.8, 2.9,
            3.0, 3.5, 4.0, 5.0, 6.0, 8.0, 10.0,
        ];
        let mut prev: Option<f64> = None;
        for &x in xs {
            let v = erfc_kernel(x).unwrap();
            // Range: erfc(x) in (0, 1] for x >= 0 (equals 1 exactly at 0).
            assert!(v > 0.0 && v <= 1.0, "erfc({x}) = {v} out of range");
            // Strict monotone-decreasing on positives.
            if let Some(p) = prev {
                assert!(p > v, "monotone: erfc(prev) > erfc({x}), got {p} !> {v}");
            }
            prev = Some(v);
        }

        // Reflection: erfc(-x) + erfc(x) ≈ 2 (tight ULP-scale bound; exact equality
        // is not guaranteed because both summands are rounded f64 values).
        let e_one = erfc_kernel(1.0).unwrap();
        let e_neg_one = erfc_kernel(-1.0).unwrap();
        let e_two = erfc_kernel(2.0).unwrap();
        let e_neg_two = erfc_kernel(-2.0).unwrap();
        assert!(
            (e_neg_one + e_one - 2.0).abs() < 1e-15,
            "reflection erfc(-1)+erfc(1) = {}",
            e_neg_one + e_one
        );
        assert!(
            (e_neg_two + e_two - 2.0).abs() < 1e-15,
            "reflection erfc(-2)+erfc(2) = {}",
            e_neg_two + e_two
        );
    }

    // Excel-matching exact-bit witnesses for the large-x regime closed by
    // the UCRT branch on Windows-MSVC. On other platforms UCRT is not
    // available and libm's residual divergence in this range is an
    // accepted platform gap.
    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    #[test]
    fn erfc_large_positive_tail_matches_excel_on_windows() {
        assert_bits_eq(
            "erfc(3)",
            erfc_kernel(3.0).unwrap(),
            2.209049699858544e-5,
        );
        assert_bits_eq(
            "erfc(4)",
            erfc_kernel(4.0).unwrap(),
            1.5417257900280017e-8,
        );
        assert_bits_eq(
            "erfc(8)",
            erfc_kernel(8.0).unwrap(),
            1.1224297172982929e-29,
        );
        assert_bits_eq(
            "erfc.precise(3)",
            erfc_precise_kernel(3.0).unwrap(),
            2.209049699858544e-5,
        );
        assert_bits_eq(
            "erfc.precise(4)",
            erfc_precise_kernel(4.0).unwrap(),
            1.5417257900280017e-8,
        );
        assert_bits_eq(
            "erfc.precise(8)",
            erfc_precise_kernel(8.0).unwrap(),
            1.1224297172982929e-29,
        );
    }

    // Exact-bit Excel witnesses captured for the still-blocked positive
    // regime. No rule family we scored (libm, UCRT, min/max, bounded ULP
    // adjustments on subranges) reproduces these under owner-local code.
    // Kept as #[ignore]d diagnostic so the bits are preserved against the
    // artifact drifting; enable with `cargo test -- --ignored` when a
    // kernel candidate is in flight.
    #[test]
    #[ignore = "Excel polynomial not yet reproduced in OxFunc; see docs/function-lane/ERFC_EXCEL_EMULATION.md"]
    fn erfc_known_blocked_excel_witnesses() {
        let cases: &[(f64, f64)] = &[
            (1.5, 0.03389485352468927),
            (1.75, 0.013328328780817557),
            (1.9, 0.007209570764742528),
            (2.0, 0.0046777349810472645),
            (2.1, 0.002979466656332985),
            (2.25, 0.0014627165866811515),
            (2.5, 0.00040695201744495886),
            (2.6, 0.000236034416529349),
            (2.7, 0.0001343327399405242),
            (2.9, 4.1097878099458844e-5),
            (3.5, 7.430983723414129e-7),
            (5.0, 1.537459794428034e-12),
            (6.0, 2.151973671249892e-17),
            (10.0, 2.0884875837625446e-45),
        ];
        for (x, excel) in cases.iter().copied() {
            let got = erfc_kernel(x).unwrap();
            assert_eq!(
                got.to_bits(),
                excel.to_bits(),
                "erfc({x}): got {got:e} ({:#018x}), excel {excel:e} ({:#018x})",
                got.to_bits(),
                excel.to_bits()
            );
        }
    }

    #[test]
    fn erf_family_matches_seed_rows() {
        assert_close(erf_kernel(1.0, None).unwrap(), 0.8427007929497149, 1e-15);
        assert_close(
            erf_kernel(0.0, Some(1.0)).unwrap(),
            0.8427007929497149,
            1e-15,
        );
        assert_close(erf_precise_kernel(1.0).unwrap(), 0.8427007929497149, 1e-15);
        assert_close(erfc_kernel(1.0).unwrap(), 0.15729920705028513, 1e-15);
        assert_close(
            erfc_precise_kernel(-1.0).unwrap(),
            1.8427007929497148,
            1e-15,
        );
        assert_close(
            erf_kernel(1.0, Some(2.0)).unwrap(),
            0.15262147206923782,
            1e-15,
        );
        assert_close(
            erf_kernel(2.0, Some(1.0)).unwrap(),
            -0.15262147206923782,
            1e-15,
        );
    }

    #[test]
    fn gamma_family_matches_seed_rows() {
        assert_close(gamma_kernel(5.0).unwrap(), 24.0, 1e-12);
        assert_close(gamma_kernel(0.5).unwrap(), 1.772453850905516, 1e-12);
        assert_close(gamma_kernel(-0.5).unwrap(), -3.5449077018110318, 1e-10);
        assert_close(gammaln_kernel(5.0).unwrap(), 3.1780538303479458, 1e-12);
        assert_close(
            gammaln_precise_kernel(0.5).unwrap(),
            0.5723649429247001,
            1e-12,
        );
        assert_eq!(gamma_kernel(-1.0), Err(WorksheetErrorCode::Num));
        assert_eq!(gammaln_kernel(0.0), Err(WorksheetErrorCode::Num));
        assert_eq!(gamma_kernel(172.0), Err(WorksheetErrorCode::Num));
    }

    #[test]
    fn weibull_family_matches_seed_rows() {
        assert_close(
            weibull_kernel(2.0, 3.0, 4.0, true).unwrap(),
            0.11750309741540463,
            1e-12,
        );
        assert_close(
            weibull_dist_kernel(2.0, 3.0, 4.0, false).unwrap(),
            0.1654681692346117,
            1e-12,
        );
        assert_eq!(weibull_dist_kernel(0.0, 3.0, 4.0, true), Ok(0.0));
        assert_eq!(weibull_dist_kernel(0.0, 3.0, 4.0, false), Ok(0.0));
        assert_eq!(weibull_dist_kernel(0.0, 0.5, 4.0, false), Ok(0.0));
        assert_eq!(weibull_dist_kernel(0.0, 1.0, 4.0, false), Ok(0.0));
        assert_eq!(
            weibull_dist_kernel(-1.0, 3.0, 4.0, true),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            weibull_dist_kernel(2.0, 0.0, 4.0, true),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            weibull_dist_kernel(2.0, 3.0, 0.0, true),
            Err(WorksheetErrorCode::Num)
        );
    }

    #[test]
    fn surface_evaluators_follow_flag_and_error_contracts() {
        let resolver = NoResolver;
        let weibull_cdf = eval_weibull_dist_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(2.0)),
                CallArgValue::Eval(EvalValue::Number(3.0)),
                CallArgValue::Eval(EvalValue::Number(4.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
            ],
            &resolver,
        );
        match weibull_cdf {
            Ok(EvalValue::Number(value)) => assert_close(value, 0.11750309741540463, 1e-12),
            other => panic!("unexpected weibull cdf result: {other:?}"),
        }

        let weibull_pdf = eval_weibull_dist_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(2.0)),
                CallArgValue::Eval(EvalValue::Number(3.0)),
                CallArgValue::Eval(EvalValue::Number(4.0)),
                CallArgValue::Eval(EvalValue::Number(0.0)),
            ],
            &resolver,
        );
        match weibull_pdf {
            Ok(EvalValue::Number(value)) => assert_close(value, 0.1654681692346117, 1e-12),
            other => panic!("unexpected weibull pdf result: {other:?}"),
        }

        assert_eq!(
            eval_gamma_surface(&[CallArgValue::Eval(EvalValue::Number(-1.0))], &resolver),
            Ok(EvalValue::Error(WorksheetErrorCode::Num))
        );
        assert_eq!(
            eval_erf_surface(&[], &resolver),
            Err(SpecialDistEvalError::ArityMismatch {
                expected_min: 1,
                expected_max: 2,
                actual: 0,
            })
        );
    }

    #[test]
    fn metadata_profiles_match_batch_shape() {
        assert_eq!(
            ERF_META.arg_preparation_profile,
            ArgPreparationProfile::ValuesOnlyPreAdapter
        );
        assert_eq!(
            WEIBULL_DIST_META.surface_fec_dependency_profile,
            FecDependencyProfile::RefOnly
        );
    }
}
