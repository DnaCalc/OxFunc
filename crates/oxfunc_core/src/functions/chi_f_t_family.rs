use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, run_values_only_prepared,
};
use crate::functions::special_math_common::{
    bisect_inverse, gamma, regularized_beta, regularized_gamma_p, regularized_gamma_q,
};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

macro_rules! dist_meta {
    ($id:literal, $min:expr, $max:expr) => {
        FunctionMeta {
            function_id: $id,
            arity: Arity {
                min: $min,
                max: $max,
            },
            determinism: DeterminismClass::Deterministic,
            volatility: VolatilityClass::NonVolatile,
            host_interaction: HostInteractionClass::None,
            thread_safety: ThreadSafetyClass::SafePure,
            arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
            coercion_lift_profile: CoercionLiftProfile::Custom,
            kernel_signature_class: KernelSignatureClass::Custom,
            fec_dependency_profile: FecDependencyProfile::None,
            surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
        }
    };
}

pub const CHISQ_DIST_META: FunctionMeta = dist_meta!("FUNC.CHISQ.DIST", 3, 3);
pub const CHISQ_DIST_RT_META: FunctionMeta = dist_meta!("FUNC.CHISQ.DIST.RT", 2, 2);
pub const CHISQ_INV_META: FunctionMeta = dist_meta!("FUNC.CHISQ.INV", 2, 2);
pub const CHISQ_INV_RT_META: FunctionMeta = dist_meta!("FUNC.CHISQ.INV.RT", 2, 2);
pub const CHIDIST_META: FunctionMeta = dist_meta!("FUNC.CHIDIST", 2, 2);
pub const CHIINV_META: FunctionMeta = dist_meta!("FUNC.CHIINV", 2, 2);
pub const F_DIST_META: FunctionMeta = dist_meta!("FUNC.F.DIST", 4, 4);
pub const F_DIST_RT_META: FunctionMeta = dist_meta!("FUNC.F.DIST.RT", 3, 3);
pub const F_INV_META: FunctionMeta = dist_meta!("FUNC.F.INV", 3, 3);
pub const F_INV_RT_META: FunctionMeta = dist_meta!("FUNC.F.INV.RT", 3, 3);
pub const FDIST_META: FunctionMeta = dist_meta!("FUNC.FDIST", 3, 3);
pub const FINV_META: FunctionMeta = dist_meta!("FUNC.FINV", 3, 3);
pub const T_DIST_META: FunctionMeta = dist_meta!("FUNC.T.DIST", 3, 3);
pub const T_DIST_2T_META: FunctionMeta = dist_meta!("FUNC.T.DIST.2T", 2, 2);
pub const T_DIST_RT_META: FunctionMeta = dist_meta!("FUNC.T.DIST.RT", 2, 2);
pub const T_INV_META: FunctionMeta = dist_meta!("FUNC.T.INV", 2, 2);
pub const T_INV_2T_META: FunctionMeta = dist_meta!("FUNC.T.INV.2T", 2, 2);
pub const TDIST_META: FunctionMeta = dist_meta!("FUNC.TDIST", 3, 3);
pub const TINV_META: FunctionMeta = dist_meta!("FUNC.TINV", 2, 2);

#[derive(Debug, Clone, PartialEq)]
pub enum ChiFTEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

fn arity_error(meta: &FunctionMeta, actual: usize) -> ChiFTEvalError {
    ChiFTEvalError::ArityMismatch {
        expected_min: meta.arity.min,
        expected_max: meta.arity.max,
        actual,
    }
}

fn truncate_positive_integer(value: f64) -> Result<f64, WorksheetErrorCode> {
    let truncated = value.trunc();
    if !truncated.is_finite() || truncated < 1.0 || truncated >= 1e10 {
        Err(WorksheetErrorCode::Num)
    } else {
        Ok(truncated)
    }
}

fn validate_probability_open_unit(probability: f64) -> Result<f64, WorksheetErrorCode> {
    if !probability.is_finite() || probability <= 0.0 || probability >= 1.0 {
        Err(WorksheetErrorCode::Num)
    } else {
        Ok(probability)
    }
}

fn validate_nonnegative_x(x: f64) -> Result<f64, WorksheetErrorCode> {
    if !x.is_finite() || x < 0.0 {
        Err(WorksheetErrorCode::Num)
    } else {
        Ok(x)
    }
}

fn density_flag(value: f64) -> bool {
    value != 0.0
}

pub fn chisq_pdf_kernel(x: f64, deg_freedom: f64) -> Result<f64, WorksheetErrorCode> {
    let x = validate_nonnegative_x(x)?;
    let k = truncate_positive_integer(deg_freedom)?;
    if x == 0.0 {
        return if k == 1.0 {
            Ok(f64::INFINITY)
        } else if k == 2.0 {
            Ok(0.5)
        } else {
            Ok(0.0)
        };
    }
    let half_k = k / 2.0;
    Ok(x.powf(half_k - 1.0) * (-x / 2.0).exp() / (2.0_f64.powf(half_k) * gamma(half_k)))
}

pub fn chisq_dist_kernel(
    x: f64,
    deg_freedom: f64,
    cumulative: bool,
) -> Result<f64, WorksheetErrorCode> {
    let x = validate_nonnegative_x(x)?;
    let k = truncate_positive_integer(deg_freedom)?;
    if cumulative {
        Ok(regularized_gamma_p(k / 2.0, x / 2.0))
    } else {
        chisq_pdf_kernel(x, k)
    }
}

pub fn chisq_dist_rt_kernel(x: f64, deg_freedom: f64) -> Result<f64, WorksheetErrorCode> {
    let x = validate_nonnegative_x(x)?;
    let k = truncate_positive_integer(deg_freedom)?;
    Ok(regularized_gamma_q(k / 2.0, x / 2.0))
}

fn search_upper_bound<F>(target: f64, initial_hi: f64, f: F) -> f64
where
    F: Fn(f64) -> f64,
{
    let mut hi = initial_hi.max(1.0);
    for _ in 0..200 {
        if f(hi) >= target {
            return hi;
        }
        hi *= 2.0;
    }
    hi
}

pub fn chisq_inv_kernel(probability: f64, deg_freedom: f64) -> Result<f64, WorksheetErrorCode> {
    let p = validate_probability_open_unit(probability)?;
    let k = truncate_positive_integer(deg_freedom)?;
    let hi = search_upper_bound(p, k, |x| regularized_gamma_p(k / 2.0, x / 2.0));
    Ok(bisect_inverse(p, 0.0, hi, |x| {
        regularized_gamma_p(k / 2.0, x / 2.0)
    }))
}

pub fn chisq_inv_rt_kernel(probability: f64, deg_freedom: f64) -> Result<f64, WorksheetErrorCode> {
    let p = validate_probability_open_unit(probability)?;
    chisq_inv_kernel(1.0 - p, deg_freedom)
}

pub fn f_pdf_kernel(x: f64, deg1: f64, deg2: f64) -> Result<f64, WorksheetErrorCode> {
    let x = validate_nonnegative_x(x)?;
    let d1 = truncate_positive_integer(deg1)?;
    let d2 = truncate_positive_integer(deg2)?;
    if x == 0.0 {
        return Ok(if d1 < 2.0 {
            f64::INFINITY
        } else if d1 == 2.0 {
            1.0
        } else {
            0.0
        });
    }
    let half_d1 = d1 / 2.0;
    let half_d2 = d2 / 2.0;
    let num = (d1 / d2).powf(half_d1) * x.powf(half_d1 - 1.0);
    let den = gamma(half_d1) * gamma(half_d2) / gamma(half_d1 + half_d2)
        * (1.0 + d1 * x / d2).powf((d1 + d2) / 2.0);
    Ok(num / den)
}

pub fn f_dist_kernel(
    x: f64,
    deg1: f64,
    deg2: f64,
    cumulative: bool,
) -> Result<f64, WorksheetErrorCode> {
    let x = validate_nonnegative_x(x)?;
    let d1 = truncate_positive_integer(deg1)?;
    let d2 = truncate_positive_integer(deg2)?;
    if cumulative {
        let z = d1 * x / (d1 * x + d2);
        Ok(regularized_beta(z, d1 / 2.0, d2 / 2.0))
    } else {
        f_pdf_kernel(x, d1, d2)
    }
}

pub fn f_dist_rt_kernel(x: f64, deg1: f64, deg2: f64) -> Result<f64, WorksheetErrorCode> {
    let x = validate_nonnegative_x(x)?;
    let d1 = truncate_positive_integer(deg1)?;
    let d2 = truncate_positive_integer(deg2)?;
    let z = d2 / (d2 + d1 * x);
    Ok(regularized_beta(z, d2 / 2.0, d1 / 2.0))
}

pub fn f_inv_kernel(probability: f64, deg1: f64, deg2: f64) -> Result<f64, WorksheetErrorCode> {
    let p = validate_probability_open_unit(probability)?;
    let d1 = truncate_positive_integer(deg1)?;
    let d2 = truncate_positive_integer(deg2)?;
    let hi = search_upper_bound(p, 1.0, |x| {
        let z = d1 * x / (d1 * x + d2);
        regularized_beta(z, d1 / 2.0, d2 / 2.0)
    });
    Ok(bisect_inverse(p, 0.0, hi, |x| {
        let z = d1 * x / (d1 * x + d2);
        regularized_beta(z, d1 / 2.0, d2 / 2.0)
    }))
}

pub fn f_inv_rt_kernel(probability: f64, deg1: f64, deg2: f64) -> Result<f64, WorksheetErrorCode> {
    let p = validate_probability_open_unit(probability)?;
    f_inv_kernel(1.0 - p, deg1, deg2)
}

fn t_pdf(x: f64, deg_freedom: f64) -> Result<f64, WorksheetErrorCode> {
    let v = truncate_positive_integer(deg_freedom)?;
    let numerator = gamma((v + 1.0) / 2.0);
    let denominator = (v * std::f64::consts::PI).sqrt()
        * gamma(v / 2.0)
        * (1.0 + x * x / v).powf((v + 1.0) / 2.0);
    Ok(numerator / denominator)
}

fn t_cdf(x: f64, deg_freedom: f64) -> Result<f64, WorksheetErrorCode> {
    let v = truncate_positive_integer(deg_freedom)?;
    let xx = v / (v + x * x);
    let ib = regularized_beta(xx, v / 2.0, 0.5);
    Ok(if x >= 0.0 { 1.0 - 0.5 * ib } else { 0.5 * ib })
}

pub fn t_dist_kernel(
    x: f64,
    deg_freedom: f64,
    cumulative: bool,
) -> Result<f64, WorksheetErrorCode> {
    if cumulative {
        t_cdf(x, deg_freedom)
    } else {
        t_pdf(x, deg_freedom)
    }
}

pub fn t_dist_rt_kernel(x: f64, deg_freedom: f64) -> Result<f64, WorksheetErrorCode> {
    let x = validate_nonnegative_x(x)?;
    let v = truncate_positive_integer(deg_freedom)?;
    let xx = v / (v + x * x);
    Ok(0.5 * regularized_beta(xx, v / 2.0, 0.5))
}

pub fn t_dist_2t_kernel(x: f64, deg_freedom: f64) -> Result<f64, WorksheetErrorCode> {
    let x = validate_nonnegative_x(x)?;
    let v = truncate_positive_integer(deg_freedom)?;
    let xx = v / (v + x * x);
    Ok(regularized_beta(xx, v / 2.0, 0.5))
}

pub fn t_inv_kernel(probability: f64, deg_freedom: f64) -> Result<f64, WorksheetErrorCode> {
    let p = validate_probability_open_unit(probability)?;
    let v = truncate_positive_integer(deg_freedom)?;
    if p == 0.5 {
        return Ok(0.0);
    }
    if p < 0.5 {
        return Ok(-t_inv_kernel(1.0 - p, v)?);
    }
    let hi = search_upper_bound(p, 1.0, |x| t_cdf(x, v).unwrap_or(1.0));
    Ok(bisect_inverse(p, 0.0, hi, |x| t_cdf(x, v).unwrap_or(1.0)))
}

pub fn t_inv_2t_kernel(probability: f64, deg_freedom: f64) -> Result<f64, WorksheetErrorCode> {
    let p = validate_probability_open_unit(probability)?;
    t_inv_kernel(1.0 - p / 2.0, deg_freedom)
}

fn map_domain(value: Result<f64, WorksheetErrorCode>) -> EvalValue {
    match value {
        Ok(number) => EvalValue::Number(number),
        Err(code) => EvalValue::Error(code),
    }
}

fn eval_chisq_dist_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, ChiFTEvalError> {
    if !CHISQ_DIST_META.arity.accepts(args.len()) {
        return Err(arity_error(&CHISQ_DIST_META, args.len()));
    }
    let x = coerce_prepared_to_number(&args[0]).map_err(ChiFTEvalError::Coercion)?;
    let df = coerce_prepared_to_number(&args[1]).map_err(ChiFTEvalError::Coercion)?;
    let cumulative = coerce_prepared_to_number(&args[2]).map_err(ChiFTEvalError::Coercion)?;
    Ok(map_domain(chisq_dist_kernel(
        x,
        df,
        density_flag(cumulative),
    )))
}

fn eval_chisq_dist_rt_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, ChiFTEvalError> {
    if !CHISQ_DIST_RT_META.arity.accepts(args.len()) {
        return Err(arity_error(&CHISQ_DIST_RT_META, args.len()));
    }
    let x = coerce_prepared_to_number(&args[0]).map_err(ChiFTEvalError::Coercion)?;
    let df = coerce_prepared_to_number(&args[1]).map_err(ChiFTEvalError::Coercion)?;
    Ok(map_domain(chisq_dist_rt_kernel(x, df)))
}

fn eval_chisq_inv_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, ChiFTEvalError> {
    if !CHISQ_INV_META.arity.accepts(args.len()) {
        return Err(arity_error(&CHISQ_INV_META, args.len()));
    }
    let p = coerce_prepared_to_number(&args[0]).map_err(ChiFTEvalError::Coercion)?;
    let df = coerce_prepared_to_number(&args[1]).map_err(ChiFTEvalError::Coercion)?;
    Ok(map_domain(chisq_inv_kernel(p, df)))
}

fn eval_chisq_inv_rt_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, ChiFTEvalError> {
    if !CHISQ_INV_RT_META.arity.accepts(args.len()) {
        return Err(arity_error(&CHISQ_INV_RT_META, args.len()));
    }
    let p = coerce_prepared_to_number(&args[0]).map_err(ChiFTEvalError::Coercion)?;
    let df = coerce_prepared_to_number(&args[1]).map_err(ChiFTEvalError::Coercion)?;
    Ok(map_domain(chisq_inv_rt_kernel(p, df)))
}

fn eval_f_dist_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, ChiFTEvalError> {
    if !F_DIST_META.arity.accepts(args.len()) {
        return Err(arity_error(&F_DIST_META, args.len()));
    }
    let x = coerce_prepared_to_number(&args[0]).map_err(ChiFTEvalError::Coercion)?;
    let d1 = coerce_prepared_to_number(&args[1]).map_err(ChiFTEvalError::Coercion)?;
    let d2 = coerce_prepared_to_number(&args[2]).map_err(ChiFTEvalError::Coercion)?;
    let cumulative = coerce_prepared_to_number(&args[3]).map_err(ChiFTEvalError::Coercion)?;
    Ok(map_domain(f_dist_kernel(
        x,
        d1,
        d2,
        density_flag(cumulative),
    )))
}

fn eval_f_dist_rt_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, ChiFTEvalError> {
    if !F_DIST_RT_META.arity.accepts(args.len()) {
        return Err(arity_error(&F_DIST_RT_META, args.len()));
    }
    let x = coerce_prepared_to_number(&args[0]).map_err(ChiFTEvalError::Coercion)?;
    let d1 = coerce_prepared_to_number(&args[1]).map_err(ChiFTEvalError::Coercion)?;
    let d2 = coerce_prepared_to_number(&args[2]).map_err(ChiFTEvalError::Coercion)?;
    Ok(map_domain(f_dist_rt_kernel(x, d1, d2)))
}

fn eval_f_inv_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, ChiFTEvalError> {
    if !F_INV_META.arity.accepts(args.len()) {
        return Err(arity_error(&F_INV_META, args.len()));
    }
    let p = coerce_prepared_to_number(&args[0]).map_err(ChiFTEvalError::Coercion)?;
    let d1 = coerce_prepared_to_number(&args[1]).map_err(ChiFTEvalError::Coercion)?;
    let d2 = coerce_prepared_to_number(&args[2]).map_err(ChiFTEvalError::Coercion)?;
    Ok(map_domain(f_inv_kernel(p, d1, d2)))
}

fn eval_f_inv_rt_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, ChiFTEvalError> {
    if !F_INV_RT_META.arity.accepts(args.len()) {
        return Err(arity_error(&F_INV_RT_META, args.len()));
    }
    let p = coerce_prepared_to_number(&args[0]).map_err(ChiFTEvalError::Coercion)?;
    let d1 = coerce_prepared_to_number(&args[1]).map_err(ChiFTEvalError::Coercion)?;
    let d2 = coerce_prepared_to_number(&args[2]).map_err(ChiFTEvalError::Coercion)?;
    Ok(map_domain(f_inv_rt_kernel(p, d1, d2)))
}

fn eval_t_dist_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, ChiFTEvalError> {
    if !T_DIST_META.arity.accepts(args.len()) {
        return Err(arity_error(&T_DIST_META, args.len()));
    }
    let x = coerce_prepared_to_number(&args[0]).map_err(ChiFTEvalError::Coercion)?;
    let df = coerce_prepared_to_number(&args[1]).map_err(ChiFTEvalError::Coercion)?;
    let cumulative = coerce_prepared_to_number(&args[2]).map_err(ChiFTEvalError::Coercion)?;
    Ok(map_domain(t_dist_kernel(x, df, density_flag(cumulative))))
}

fn eval_t_dist_2t_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, ChiFTEvalError> {
    if !T_DIST_2T_META.arity.accepts(args.len()) {
        return Err(arity_error(&T_DIST_2T_META, args.len()));
    }
    let x = coerce_prepared_to_number(&args[0]).map_err(ChiFTEvalError::Coercion)?;
    let df = coerce_prepared_to_number(&args[1]).map_err(ChiFTEvalError::Coercion)?;
    Ok(map_domain(t_dist_2t_kernel(x, df)))
}

fn eval_t_dist_rt_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, ChiFTEvalError> {
    if !T_DIST_RT_META.arity.accepts(args.len()) {
        return Err(arity_error(&T_DIST_RT_META, args.len()));
    }
    let x = coerce_prepared_to_number(&args[0]).map_err(ChiFTEvalError::Coercion)?;
    let df = coerce_prepared_to_number(&args[1]).map_err(ChiFTEvalError::Coercion)?;
    Ok(map_domain(t_dist_rt_kernel(x, df)))
}

fn eval_t_inv_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, ChiFTEvalError> {
    if !T_INV_META.arity.accepts(args.len()) {
        return Err(arity_error(&T_INV_META, args.len()));
    }
    let p = coerce_prepared_to_number(&args[0]).map_err(ChiFTEvalError::Coercion)?;
    let df = coerce_prepared_to_number(&args[1]).map_err(ChiFTEvalError::Coercion)?;
    Ok(map_domain(t_inv_kernel(p, df)))
}

fn eval_t_inv_2t_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, ChiFTEvalError> {
    if !T_INV_2T_META.arity.accepts(args.len()) {
        return Err(arity_error(&T_INV_2T_META, args.len()));
    }
    let p = coerce_prepared_to_number(&args[0]).map_err(ChiFTEvalError::Coercion)?;
    let df = coerce_prepared_to_number(&args[1]).map_err(ChiFTEvalError::Coercion)?;
    Ok(map_domain(t_inv_2t_kernel(p, df)))
}

fn eval_tdist_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, ChiFTEvalError> {
    if !TDIST_META.arity.accepts(args.len()) {
        return Err(arity_error(&TDIST_META, args.len()));
    }
    let x = coerce_prepared_to_number(&args[0]).map_err(ChiFTEvalError::Coercion)?;
    let df = coerce_prepared_to_number(&args[1]).map_err(ChiFTEvalError::Coercion)?;
    let tails = coerce_prepared_to_number(&args[2]).map_err(ChiFTEvalError::Coercion)?;
    let tails = tails.trunc();
    let result = match tails as i32 {
        1 => t_dist_rt_kernel(x, df),
        2 => t_dist_2t_kernel(x, df),
        _ => Err(WorksheetErrorCode::Num),
    };
    Ok(map_domain(result))
}

pub fn eval_chisq_dist_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_chisq_dist_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_chisq_dist_rt_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_chisq_dist_rt_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_chisq_inv_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_chisq_inv_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_chisq_inv_rt_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_chisq_inv_rt_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_chidist_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_chisq_dist_rt_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_chiinv_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_chisq_inv_rt_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_f_dist_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_f_dist_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_f_dist_rt_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_f_dist_rt_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_f_inv_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_f_inv_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_f_inv_rt_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_f_inv_rt_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_fdist_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_f_dist_rt_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_finv_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_f_inv_rt_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_t_dist_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_t_dist_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_t_dist_2t_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_t_dist_2t_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_t_dist_rt_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_t_dist_rt_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_t_inv_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_t_inv_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_t_inv_2t_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_t_inv_2t_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_tdist_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_tdist_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_tinv_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_t_inv_2t_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn map_chi_f_t_error_to_ws(error: &ChiFTEvalError) -> WorksheetErrorCode {
    match error {
        ChiFTEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        ChiFTEvalError::Coercion(coercion) => coercion_to_ws(coercion),
    }
}

fn coercion_to_ws(error: &CoercionError) -> WorksheetErrorCode {
    match error {
        CoercionError::WorksheetError(code) => *code,
        CoercionError::RefResolution(_) => WorksheetErrorCode::Ref,
        CoercionError::MissingArg
        | CoercionError::EmptyCell
        | CoercionError::NonNumericText(_)
        | CoercionError::UnsupportedValueKind(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::WorksheetErrorCode;

    #[test]
    fn chisq_family_matches_known_rows() {
        assert!((chisq_dist_kernel(0.5, 1.0, true).unwrap() - 0.520_499_88).abs() < 1e-8);
        assert!(
            (chisq_dist_kernel(2.0, 3.0, false).unwrap() - 0.207_553_748_710_297).abs() < 1e-10
        );
        assert!((chisq_dist_rt_kernel(18.307, 10.0).unwrap() - 0.050_000_589_130_671).abs() < 1e-8);
        assert!((chisq_inv_rt_kernel(0.0500006, 10.0).unwrap() - 18.307).abs() < 1e-4);
    }

    #[test]
    fn f_family_matches_known_rows() {
        assert!((f_dist_kernel(15.2069, 6.0, 4.0, true).unwrap() - 0.99).abs() < 5e-5);
        assert!((f_dist_kernel(15.2069, 6.0, 4.0, false).unwrap() - 0.001_223_8).abs() < 5e-7);
        assert!((f_inv_rt_kernel(0.01, 6.0, 4.0).unwrap() - 15.20686).abs() < 1e-4);
        assert!((f_inv_kernel(0.99, 6.0, 4.0).unwrap() - 15.20686).abs() < 1e-4);
    }

    #[test]
    fn t_family_matches_known_rows() {
        assert!((t_dist_kernel(60.0, 1.0, true).unwrap() - 0.994_695_33).abs() < 1e-8);
        assert!((t_dist_kernel(8.0, 3.0, false).unwrap() - 0.000_736_91).abs() < 1e-8);
        assert!((t_dist_2t_kernel(1.959_999_998, 60.0).unwrap() - 0.054_644_93).abs() < 1e-7);
        assert!((t_dist_rt_kernel(1.959_999_998, 60.0).unwrap() - 0.027_322_465).abs() < 1e-8);
        assert!((t_inv_2t_kernel(0.054_644_93, 60.0).unwrap() - 1.959_999_998).abs() < 1e-6);
    }

    #[test]
    fn compatibility_aliases_follow_modern_functions() {
        assert!((chisq_dist_rt_kernel(18.307, 10.0).unwrap() - 0.050_000_589_130_671).abs() < 1e-8);
        assert!((f_dist_rt_kernel(15.2069, 6.0, 4.0).unwrap() - 0.01).abs() < 5e-5);
        assert!((t_dist_2t_kernel(1.959_999_998, 60.0).unwrap() - 0.054_644_93).abs() < 1e-7);
    }

    #[test]
    fn invalid_domains_return_num_errors() {
        assert_eq!(
            chisq_dist_kernel(-1.0, 1.0, true),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            f_dist_kernel(-1.0, 1.0, 1.0, true),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(t_dist_2t_kernel(-1.0, 10.0), Err(WorksheetErrorCode::Num));
        assert_eq!(t_dist_rt_kernel(-1.0, 10.0), Err(WorksheetErrorCode::Num));
    }
}
