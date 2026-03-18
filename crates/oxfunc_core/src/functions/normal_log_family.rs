use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, run_values_only_prepared,
};
use crate::functions::normal_dist_common::{erf_approx, phi_kernel};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

const NORMAL_LOG_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.NORMAL_LOG_BASE",
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

pub const CONFIDENCE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.CONFIDENCE",
    arity: Arity::exact(3),
    ..NORMAL_LOG_BASE_META
};

pub const CONFIDENCE_NORM_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.CONFIDENCE.NORM",
    arity: Arity::exact(3),
    ..NORMAL_LOG_BASE_META
};

pub const LOGNORM_DIST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.LOGNORM.DIST",
    arity: Arity::exact(4),
    ..NORMAL_LOG_BASE_META
};

pub const LOGNORM_INV_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.LOGNORM.INV",
    arity: Arity::exact(3),
    ..NORMAL_LOG_BASE_META
};

pub const LOGNORMDIST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.LOGNORMDIST",
    arity: Arity::exact(3),
    ..NORMAL_LOG_BASE_META
};

pub const NORM_DIST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.NORM.DIST",
    arity: Arity::exact(4),
    ..NORMAL_LOG_BASE_META
};

pub const NORM_INV_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.NORM.INV",
    arity: Arity::exact(3),
    ..NORMAL_LOG_BASE_META
};

pub const NORMSDIST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.NORMSDIST",
    arity: Arity::exact(1),
    ..NORMAL_LOG_BASE_META
};

pub const NORMSINV_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.NORMSINV",
    arity: Arity::exact(1),
    ..NORMAL_LOG_BASE_META
};

pub const NORMDIST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.NORMDIST",
    arity: Arity::exact(4),
    ..NORMAL_LOG_BASE_META
};

pub const NORMINV_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.NORMINV",
    arity: Arity::exact(3),
    ..NORMAL_LOG_BASE_META
};

pub const NORM_S_DIST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.NORM.S.DIST",
    arity: Arity::exact(2),
    ..NORMAL_LOG_BASE_META
};

pub const NORM_S_INV_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.NORM.S.INV",
    arity: Arity::exact(1),
    ..NORMAL_LOG_BASE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum NormalLogEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

fn norm_cdf(x: f64) -> f64 {
    0.5 * (1.0 + erf_approx(x / std::f64::consts::SQRT_2))
}

fn validate_positive_sigma(sigma: f64) -> Result<(), WorksheetErrorCode> {
    if sigma <= 0.0 || !sigma.is_finite() {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(())
}

fn validate_probability_open_unit(p: f64) -> Result<(), WorksheetErrorCode> {
    if !p.is_finite() || p <= 0.0 || p >= 1.0 {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(())
}

fn cumulative_flag(value: f64) -> bool {
    value != 0.0
}

fn inverse_standard_normal(p: f64) -> Result<f64, WorksheetErrorCode> {
    validate_probability_open_unit(p)?;

    const A: [f64; 6] = [
        -3.969_683_028_665_376e1,
        2.209_460_984_245_205e2,
        -2.759_285_104_469_687e2,
        1.383_577_518_672_69e2,
        -3.066_479_806_614_716e1,
        2.506_628_277_459_239,
    ];
    const B: [f64; 5] = [
        -5.447_609_879_822_406e1,
        1.615_858_368_580_409e2,
        -1.556_989_798_598_866e2,
        6.680_131_188_771_972e1,
        -1.328_068_155_288_572e1,
    ];
    const C: [f64; 6] = [
        -7.784_894_002_430_293e-3,
        -3.223_964_580_411_365e-1,
        -2.400_758_277_161_838,
        -2.549_732_539_343_734,
        4.374_664_141_464_968,
        2.938_163_982_698_783,
    ];
    const D: [f64; 4] = [
        7.784_695_709_041_462e-3,
        3.224_671_290_700_398e-1,
        2.445_134_137_142_996,
        3.754_408_661_907_416,
    ];
    const P_LOW: f64 = 0.02425;
    const P_HIGH: f64 = 1.0 - P_LOW;

    let x = if p < P_LOW {
        let q = (-2.0 * p.ln()).sqrt();
        (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    } else if p <= P_HIGH {
        let q = p - 0.5;
        let r = q * q;
        (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5]) * q
            / (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0)
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    };

    let err = norm_cdf(x) - p;
    Ok(x - err / phi_kernel(x))
}

pub fn norm_s_dist_kernel(z: f64, cumulative: bool) -> Result<f64, WorksheetErrorCode> {
    if cumulative {
        Ok(norm_cdf(z))
    } else {
        Ok(phi_kernel(z))
    }
}

pub fn norm_dist_kernel(
    x: f64,
    mean: f64,
    sigma: f64,
    cumulative: bool,
) -> Result<f64, WorksheetErrorCode> {
    validate_positive_sigma(sigma)?;
    let z = (x - mean) / sigma;
    if cumulative {
        Ok(norm_cdf(z))
    } else {
        Ok(phi_kernel(z) / sigma)
    }
}

pub fn norm_s_inv_kernel(p: f64) -> Result<f64, WorksheetErrorCode> {
    inverse_standard_normal(p)
}

pub fn norm_inv_kernel(p: f64, mean: f64, sigma: f64) -> Result<f64, WorksheetErrorCode> {
    validate_positive_sigma(sigma)?;
    Ok(mean + sigma * inverse_standard_normal(p)?)
}

pub fn lognorm_dist_kernel(
    x: f64,
    mean: f64,
    sigma: f64,
    cumulative: bool,
) -> Result<f64, WorksheetErrorCode> {
    validate_positive_sigma(sigma)?;
    if x <= 0.0 || !x.is_finite() {
        return Err(WorksheetErrorCode::Num);
    }
    let z = (x.ln() - mean) / sigma;
    if cumulative {
        Ok(norm_cdf(z))
    } else {
        Ok(phi_kernel(z) / (x * sigma))
    }
}

pub fn lognorm_inv_kernel(p: f64, mean: f64, sigma: f64) -> Result<f64, WorksheetErrorCode> {
    validate_positive_sigma(sigma)?;
    Ok((mean + sigma * inverse_standard_normal(p)?).exp())
}

pub fn confidence_norm_kernel(
    alpha: f64,
    standard_dev: f64,
    size: f64,
) -> Result<f64, WorksheetErrorCode> {
    validate_probability_open_unit(alpha)?;
    if standard_dev <= 0.0 || size <= 0.0 || !standard_dev.is_finite() || !size.is_finite() {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(inverse_standard_normal(1.0 - alpha / 2.0)? * standard_dev / size.sqrt())
}

fn prepared_len_error(meta: &FunctionMeta, actual: usize) -> NormalLogEvalError {
    NormalLogEvalError::ArityMismatch {
        expected_min: meta.arity.min,
        expected_max: meta.arity.max,
        actual,
    }
}

fn eval_confidence_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, NormalLogEvalError> {
    if !CONFIDENCE_META.arity.accepts(args.len()) {
        return Err(prepared_len_error(&CONFIDENCE_META, args.len()));
    }
    let alpha = coerce_prepared_to_number(&args[0]).map_err(NormalLogEvalError::Coercion)?;
    let stdev = coerce_prepared_to_number(&args[1]).map_err(NormalLogEvalError::Coercion)?;
    let size = coerce_prepared_to_number(&args[2]).map_err(NormalLogEvalError::Coercion)?;
    Ok(match confidence_norm_kernel(alpha, stdev, size) {
        Ok(value) => EvalValue::Number(value),
        Err(code) => EvalValue::Error(code),
    })
}

fn eval_norm_dist_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, NormalLogEvalError> {
    if !NORM_DIST_META.arity.accepts(args.len()) {
        return Err(prepared_len_error(&NORM_DIST_META, args.len()));
    }
    let x = coerce_prepared_to_number(&args[0]).map_err(NormalLogEvalError::Coercion)?;
    let mean = coerce_prepared_to_number(&args[1]).map_err(NormalLogEvalError::Coercion)?;
    let sigma = coerce_prepared_to_number(&args[2]).map_err(NormalLogEvalError::Coercion)?;
    let cumulative = coerce_prepared_to_number(&args[3]).map_err(NormalLogEvalError::Coercion)?;
    Ok(
        match norm_dist_kernel(x, mean, sigma, cumulative_flag(cumulative)) {
            Ok(value) => EvalValue::Number(value),
            Err(code) => EvalValue::Error(code),
        },
    )
}

fn eval_norm_inv_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, NormalLogEvalError> {
    if !NORM_INV_META.arity.accepts(args.len()) {
        return Err(prepared_len_error(&NORM_INV_META, args.len()));
    }
    let p = coerce_prepared_to_number(&args[0]).map_err(NormalLogEvalError::Coercion)?;
    let mean = coerce_prepared_to_number(&args[1]).map_err(NormalLogEvalError::Coercion)?;
    let sigma = coerce_prepared_to_number(&args[2]).map_err(NormalLogEvalError::Coercion)?;
    Ok(match norm_inv_kernel(p, mean, sigma) {
        Ok(value) => EvalValue::Number(value),
        Err(code) => EvalValue::Error(code),
    })
}

fn eval_norm_s_dist_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, NormalLogEvalError> {
    if !NORM_S_DIST_META.arity.accepts(args.len()) {
        return Err(prepared_len_error(&NORM_S_DIST_META, args.len()));
    }
    let z = coerce_prepared_to_number(&args[0]).map_err(NormalLogEvalError::Coercion)?;
    let cumulative = coerce_prepared_to_number(&args[1]).map_err(NormalLogEvalError::Coercion)?;
    Ok(match norm_s_dist_kernel(z, cumulative_flag(cumulative)) {
        Ok(value) => EvalValue::Number(value),
        Err(code) => EvalValue::Error(code),
    })
}

fn eval_norm_s_inv_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, NormalLogEvalError> {
    if !NORM_S_INV_META.arity.accepts(args.len()) {
        return Err(prepared_len_error(&NORM_S_INV_META, args.len()));
    }
    let p = coerce_prepared_to_number(&args[0]).map_err(NormalLogEvalError::Coercion)?;
    Ok(match norm_s_inv_kernel(p) {
        Ok(value) => EvalValue::Number(value),
        Err(code) => EvalValue::Error(code),
    })
}

fn eval_lognorm_dist_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, NormalLogEvalError> {
    if !LOGNORM_DIST_META.arity.accepts(args.len()) {
        return Err(prepared_len_error(&LOGNORM_DIST_META, args.len()));
    }
    let x = coerce_prepared_to_number(&args[0]).map_err(NormalLogEvalError::Coercion)?;
    let mean = coerce_prepared_to_number(&args[1]).map_err(NormalLogEvalError::Coercion)?;
    let sigma = coerce_prepared_to_number(&args[2]).map_err(NormalLogEvalError::Coercion)?;
    let cumulative = coerce_prepared_to_number(&args[3]).map_err(NormalLogEvalError::Coercion)?;
    Ok(
        match lognorm_dist_kernel(x, mean, sigma, cumulative_flag(cumulative)) {
            Ok(value) => EvalValue::Number(value),
            Err(code) => EvalValue::Error(code),
        },
    )
}

fn eval_lognorm_inv_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, NormalLogEvalError> {
    if !LOGNORM_INV_META.arity.accepts(args.len()) {
        return Err(prepared_len_error(&LOGNORM_INV_META, args.len()));
    }
    let p = coerce_prepared_to_number(&args[0]).map_err(NormalLogEvalError::Coercion)?;
    let mean = coerce_prepared_to_number(&args[1]).map_err(NormalLogEvalError::Coercion)?;
    let sigma = coerce_prepared_to_number(&args[2]).map_err(NormalLogEvalError::Coercion)?;
    Ok(match lognorm_inv_kernel(p, mean, sigma) {
        Ok(value) => EvalValue::Number(value),
        Err(code) => EvalValue::Error(code),
    })
}

pub fn eval_confidence_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, NormalLogEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_confidence_prepared,
        NormalLogEvalError::Coercion,
    )
}

pub fn eval_confidence_norm_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, NormalLogEvalError> {
    eval_confidence_surface(args, resolver)
}

pub fn eval_norm_dist_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, NormalLogEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_norm_dist_prepared,
        NormalLogEvalError::Coercion,
    )
}

pub fn eval_norm_inv_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, NormalLogEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_norm_inv_prepared,
        NormalLogEvalError::Coercion,
    )
}

pub fn eval_norm_s_dist_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, NormalLogEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_norm_s_dist_prepared,
        NormalLogEvalError::Coercion,
    )
}

pub fn eval_norm_s_inv_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, NormalLogEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_norm_s_inv_prepared,
        NormalLogEvalError::Coercion,
    )
}

pub fn eval_normdist_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, NormalLogEvalError> {
    eval_norm_dist_surface(args, resolver)
}

pub fn eval_norminv_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, NormalLogEvalError> {
    eval_norm_inv_surface(args, resolver)
}

pub fn eval_normsdist_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, NormalLogEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            if !NORMSDIST_META.arity.accepts(prepared.len()) {
                return Err(prepared_len_error(&NORMSDIST_META, prepared.len()));
            }
            let z =
                coerce_prepared_to_number(&prepared[0]).map_err(NormalLogEvalError::Coercion)?;
            Ok(EvalValue::Number(norm_cdf(z)))
        },
        NormalLogEvalError::Coercion,
    )
}

pub fn eval_normsinv_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, NormalLogEvalError> {
    eval_norm_s_inv_surface(args, resolver)
}

pub fn eval_lognorm_dist_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, NormalLogEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_lognorm_dist_prepared,
        NormalLogEvalError::Coercion,
    )
}

pub fn eval_lognorm_inv_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, NormalLogEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_lognorm_inv_prepared,
        NormalLogEvalError::Coercion,
    )
}

pub fn eval_lognormdist_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, NormalLogEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            if !LOGNORMDIST_META.arity.accepts(prepared.len()) {
                return Err(prepared_len_error(&LOGNORMDIST_META, prepared.len()));
            }
            let x =
                coerce_prepared_to_number(&prepared[0]).map_err(NormalLogEvalError::Coercion)?;
            let mean =
                coerce_prepared_to_number(&prepared[1]).map_err(NormalLogEvalError::Coercion)?;
            let sigma =
                coerce_prepared_to_number(&prepared[2]).map_err(NormalLogEvalError::Coercion)?;
            Ok(match lognorm_dist_kernel(x, mean, sigma, true) {
                Ok(value) => EvalValue::Number(value),
                Err(code) => EvalValue::Error(code),
            })
        },
        NormalLogEvalError::Coercion,
    )
}

pub fn map_normal_log_error_to_ws(e: &NormalLogEvalError) -> WorksheetErrorCode {
    match e {
        NormalLogEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        NormalLogEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        NormalLogEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn norm_family_matches_excel_probe_lanes() {
        assert!(
            (norm_dist_kernel(42.0, 40.0, 1.5, false).unwrap() - 0.109340049783996).abs() < 1e-12
        );
        assert!(
            (norm_dist_kernel(42.0, 40.0, 1.5, true).unwrap() - 0.908788780274132).abs() < 1e-7
        );
        assert!((norm_s_dist_kernel(1.0, false).unwrap() - 0.241970724519143).abs() < 1e-12);
        assert!((norm_s_dist_kernel(1.0, true).unwrap() - 0.841344746068543).abs() < 1e-7);
        assert!((norm_inv_kernel(0.9, 40.0, 1.5).unwrap() - 41.9223273483169).abs() < 1e-7);
        assert!((norm_s_inv_kernel(0.9).unwrap() - 1.2815515655446).abs() < 1e-7);
        assert!(
            (confidence_norm_kernel(0.05, 2.5, 100.0).unwrap() - 0.489990996135013).abs() < 1e-7
        );
    }

    #[test]
    fn lognorm_family_matches_excel_probe_lanes() {
        let mean = 4.0_f64.ln();
        assert!(
            (lognorm_dist_kernel(4.0, mean, 0.2, false).unwrap() - 0.498677850501791).abs() < 1e-12
        );
        assert!((lognorm_dist_kernel(4.0, mean, 0.2, true).unwrap() - 0.5).abs() < 1e-7);
        assert!((lognorm_inv_kernel(0.9, mean, 0.2).unwrap() - 5.16861455178106).abs() < 1e-6);
    }

    #[test]
    fn family_domain_errors_match_excel_probe_lanes() {
        assert_eq!(
            norm_dist_kernel(1.0, 0.0, 0.0, true),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(norm_inv_kernel(0.0, 0.0, 1.0), Err(WorksheetErrorCode::Num));
        assert_eq!(norm_s_inv_kernel(1.0), Err(WorksheetErrorCode::Num));
        assert_eq!(
            lognorm_dist_kernel(0.0, 0.0, 1.0, true),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            lognorm_inv_kernel(0.5, 0.0, 0.0),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            confidence_norm_kernel(0.0, 1.0, 10.0),
            Err(WorksheetErrorCode::Num)
        );
    }
}
