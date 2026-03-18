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

pub fn erfc_kernel(x: f64) -> Result<f64, WorksheetErrorCode> {
    if !x.is_finite() {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(1.0 - erf_approx(x))
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

    #[test]
    fn erf_family_matches_seed_rows() {
        assert_close(erf_kernel(1.0, None).unwrap(), 0.8427007929497149, 1e-6);
        assert_close(
            erf_kernel(0.0, Some(1.0)).unwrap(),
            0.8427007929497149,
            1e-6,
        );
        assert_close(erf_precise_kernel(1.0).unwrap(), 0.8427007929497149, 1e-6);
        assert_close(erfc_kernel(1.0).unwrap(), 0.15729920705028513, 1e-6);
        assert_close(erfc_precise_kernel(-1.0).unwrap(), 1.8427007929497148, 1e-6);
        assert_close(
            erf_kernel(1.0, Some(2.0)).unwrap(),
            0.15262147206923782,
            1e-6,
        );
        assert_close(
            erf_kernel(2.0, Some(1.0)).unwrap(),
            -0.15262147206923782,
            1e-6,
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
