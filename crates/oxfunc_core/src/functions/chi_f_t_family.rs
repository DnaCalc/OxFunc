use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{coerce_prepared_to_number, run_values_only_prepared};
use crate::functions::special_dist_family::erfc_of_sqrt_half_x;
use crate::functions::special_math_common::{
    bisect_inverse, bratio, gamma, regularized_gamma_q,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

macro_rules! dist_meta {
    // Default arm: surface-native lift (the modern `.`-named distributions evaluate elementwise
    // in their own surface and carry the default lift/broadcast profile).
    ($id:literal, $min:expr, $max:expr) => {
        dist_meta!(
            $id,
            $min,
            $max,
            FunctionMeta::DEFAULT_LIFT_BROADCAST_PROFILE
        )
    };
    // Explicit-lift arm: the legacy compatibility surfaces (CHIDIST/CHIINV/FDIST/FINV/TDIST/TINV)
    // are scalar-shaped by-index and rely on the dispatch layer to broadcast over the named
    // argument positions. Verified live Excel 16.0 build 20026.
    ($id:literal, $min:expr, $max:expr, $lift:expr) => {
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
            arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
            coercion_lift_profile: CoercionLiftProfile::Custom,
            lift_broadcast_profile: $lift,
            kernel_signature_class: KernelSignatureClass::Custom,
            fec_dependency_profile: FecDependencyProfile::None,
            surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
            real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
            error_collapse_profile: FunctionMeta::DEFAULT_ERROR_COLLAPSE_PROFILE,
            precision_rounding_profile: FunctionMeta::DEFAULT_PRECISION_ROUNDING_PROFILE,
        }
    };
}

pub const CHISQ_DIST_META: FunctionMeta = dist_meta!("FUNC.CHISQ.DIST", 3, 3);
pub const CHISQ_DIST_RT_META: FunctionMeta = dist_meta!("FUNC.CHISQ.DIST.RT", 2, 2);
pub const CHISQ_INV_META: FunctionMeta = dist_meta!("FUNC.CHISQ.INV", 2, 2);
pub const CHISQ_INV_RT_META: FunctionMeta = dist_meta!("FUNC.CHISQ.INV.RT", 2, 2);
pub const CHIDIST_META: FunctionMeta =
    dist_meta!("FUNC.CHIDIST", 2, 2, FunctionMeta::lift_at(&[0, 1]));
pub const CHIINV_META: FunctionMeta =
    dist_meta!("FUNC.CHIINV", 2, 2, FunctionMeta::lift_at(&[0, 1]));
pub const F_DIST_META: FunctionMeta = dist_meta!("FUNC.F.DIST", 4, 4);
pub const F_DIST_RT_META: FunctionMeta = dist_meta!("FUNC.F.DIST.RT", 3, 3);
pub const F_INV_META: FunctionMeta = dist_meta!("FUNC.F.INV", 3, 3);
pub const F_INV_RT_META: FunctionMeta = dist_meta!("FUNC.F.INV.RT", 3, 3);
pub const FDIST_META: FunctionMeta =
    dist_meta!("FUNC.FDIST", 3, 3, FunctionMeta::lift_at(&[0, 1, 2]));
pub const FINV_META: FunctionMeta =
    dist_meta!("FUNC.FINV", 3, 3, FunctionMeta::lift_at(&[0, 1, 2]));
pub const T_DIST_META: FunctionMeta = dist_meta!("FUNC.T.DIST", 3, 3);
pub const T_DIST_2T_META: FunctionMeta = dist_meta!("FUNC.T.DIST.2T", 2, 2);
pub const T_DIST_RT_META: FunctionMeta = dist_meta!("FUNC.T.DIST.RT", 2, 2);
pub const T_INV_META: FunctionMeta = dist_meta!("FUNC.T.INV", 2, 2);
pub const T_INV_2T_META: FunctionMeta = dist_meta!("FUNC.T.INV.2T", 2, 2);
pub const TDIST_META: FunctionMeta =
    dist_meta!("FUNC.TDIST", 3, 3, FunctionMeta::lift_at(&[0, 1, 2]));
pub const TINV_META: FunctionMeta = dist_meta!("FUNC.TINV", 2, 2, FunctionMeta::lift_at(&[0, 1]));

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
    // Excel docs: #NUM! when deg_freedom > 1e10 (strictly greater); 1e10 itself is admitted.
    if !truncated.is_finite() || truncated < 1.0 || truncated > 1e10 {
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
            // CDF limit is finite but density diverges to +inf; Excel returns #NUM!
            Err(WorksheetErrorCode::Num)
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
        // Inverse-problem identity, live Excel 16.0 b20228: CHISQ.DIST(x,1,TRUE)
        // == ERF.PRECISE(SQRT(x/2)) == GAMMA.DIST(x,0.5,2,TRUE) on 154/154
        // distinct x. Rival stagings SQRT(x)/SQRT(2) and SQRT(x)*(1/SQRT(2))
        // fail the same bank.
        // Inverse-problem identity, live Excel 16.0 b20228:
        // CHISQ.DIST(x,df,TRUE) == GAMMA.DIST(x, df/2, 2, TRUE) on 88/88
        // across df in {1,2,3,4,5,6,8,10}. 1-CHIDIST is not the CDF (49/88).
        // Routes df=1/2 through the landed ERF/EXPON gamma specials.
        crate::functions::beta_gamma_stats_family::gamma_dist_kernel(x, k / 2.0, 2.0, true).map_err(
            |e| match e {
                crate::functions::beta_gamma_stats_family::BetaGammaStatsError::Domain(code) => {
                    code
                }
                _ => WorksheetErrorCode::Value,
            },
        )
    } else {
        chisq_pdf_kernel(x, k)
    }
}

pub fn chisq_dist_rt_kernel(x: f64, deg_freedom: f64) -> Result<f64, WorksheetErrorCode> {
    let x = validate_nonnegative_x(x)?;
    let k = truncate_positive_integer(deg_freedom)?;
    // Same capture: CHIDIST(x,1) == CHISQ.DIST.RT(x,1) == ERFC.PRECISE(SQRT(x/2))
    // bit-exactly. Routing through the published ERFC kernel is the identified
    // graph, not a numeric fit.
    if k == 1.0 {
        erfc_of_sqrt_half_x(x)
    } else if k == 2.0 {
        // Inverse-problem identity, live Excel 16.0 b20228: CHIDIST(x,2)
        // == EXP(-x/2) == EXP(-(x/2)) on 68/68 nonnegative rows. The
        // worksheet EXP is the signed-off x87 elementary. The complementary
        // CDF 1-EXP(-x/2) is NOT this graph (9/11 on a follow-up bank).
        Ok(crate::excel_numeric::excel_exp(-(x / 2.0)))
    } else {
        Ok(regularized_gamma_q(k / 2.0, x / 2.0))
    }
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
    let k = truncate_positive_integer(deg_freedom)?;
    // Inverse-problem identity, live Excel 16.0 b20228: CHISQ.INV(p, df)
    // == GAMMA.INV(p, df/2, 2) on 63/63 interior pairs. Endpoints also
    // match: CHISQ.INV(0,2)=0 and CHISQ.INV(1,2)=#NUM!. CHIINV /
    // CHISQ.INV.RT is the right-tail inverse, not GAMMA.INV(1-p,...) (34/63).
    if !probability.is_finite() {
        return Err(WorksheetErrorCode::Num);
    }
    crate::functions::beta_gamma_stats_family::gamma_inv_kernel(probability, k / 2.0, 2.0).map_err(
        |e| match e {
            crate::functions::beta_gamma_stats_family::BetaGammaStatsError::Domain(code) => code,
            _ => WorksheetErrorCode::Value,
        },
    )
}

pub fn chisq_inv_rt_kernel(probability: f64, deg_freedom: f64) -> Result<f64, WorksheetErrorCode> {
    let p = validate_probability_open_unit(probability)?;
    let k = truncate_positive_integer(deg_freedom)?;
    // Excel inverts its published right-tail surface (CHIDIST = Q) directly,
    // not P at 1-p: the complement staging carries a systematic -5..-33-ULP
    // bias on the b14 CHIINV corpus (W109). Q is decreasing in x, so invert
    // the negated forward at -p to keep the increasing-forward convention.
    let hi = search_upper_bound(-p, k, |x| -regularized_gamma_q(k / 2.0, x / 2.0));
    Ok(bisect_inverse(-p, 0.0, hi, |x| {
        -regularized_gamma_q(k / 2.0, x / 2.0)
    }))
}

pub fn f_pdf_kernel(x: f64, deg1: f64, deg2: f64) -> Result<f64, WorksheetErrorCode> {
    let x = validate_nonnegative_x(x)?;
    let d1 = truncate_positive_integer(deg1)?;
    let d2 = truncate_positive_integer(deg2)?;
    if x == 0.0 {
        return if d1 < 2.0 {
            // Density diverges to +inf; Excel returns #NUM!
            Err(WorksheetErrorCode::Num)
        } else if d1 == 2.0 {
            Ok(1.0)
        } else {
            Ok(0.0)
        };
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
        // W109: Excel's beta wrappers pass the ACCURATE complement pair, not
        // 1-z (identified stagings; b22 gate).
        let num = d1 * x;
        let den = num + d2;
        Ok(bratio(d1 / 2.0, d2 / 2.0, num / den, d2 / den).0)
    } else {
        f_pdf_kernel(x, d1, d2)
    }
}

pub fn f_dist_rt_kernel(x: f64, deg1: f64, deg2: f64) -> Result<f64, WorksheetErrorCode> {
    let x = validate_nonnegative_x(x)?;
    let d1 = truncate_positive_integer(deg1)?;
    let d2 = truncate_positive_integer(deg2)?;
    // W109 identified staging: x = d2/den, y = d1*F/den (accurate complement).
    let num = d1 * x;
    let den = d2 + num;
    Ok(bratio(d2 / 2.0, d1 / 2.0, d2 / den, num / den).0)
}

pub fn f_inv_kernel(probability: f64, deg1: f64, deg2: f64) -> Result<f64, WorksheetErrorCode> {
    let p = validate_probability_open_unit(probability)?;
    let d1 = truncate_positive_integer(deg1)?;
    let d2 = truncate_positive_integer(deg2)?;
    let fwd = move |x: f64| {
        let num = d1 * x;
        let den = num + d2;
        bratio(d1 / 2.0, d2 / 2.0, num / den, d2 / den).0
    };
    let hi = search_upper_bound(p, 1.0, fwd);
    Ok(bisect_inverse(p, 0.0, hi, fwd))
}

pub fn f_inv_rt_kernel(probability: f64, deg1: f64, deg2: f64) -> Result<f64, WorksheetErrorCode> {
    let p = validate_probability_open_unit(probability)?;
    let d1 = truncate_positive_integer(deg1)?;
    let d2 = truncate_positive_integer(deg2)?;
    // Invert the published right-tail surface (f_dist_rt's accurate complement
    // form) directly at p, not the CDF at 1-p — same principle as CHIINV
    // (W109 b14+b19: the complement staging carries a systematic small-p bias).
    // The surface is decreasing in x, so invert the negated forward at -p.
    let f = move |x: f64| {
        let num = d1 * x;
        let den = d2 + num;
        -bratio(d2 / 2.0, d1 / 2.0, d2 / den, num / den).0
    };
    let hi = search_upper_bound(-p, 1.0, f);
    Ok(bisect_inverse(-p, 0.0, hi, f))
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
    // W109 identified staging: x = df/den, y = t^2/den (accurate complement).
    let t2 = x * x;
    let den = v + t2;
    let ib = bratio(v / 2.0, 0.5, v / den, t2 / den).0;
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
    let t2 = x * x;
    let den = v + t2;
    Ok(0.5 * bratio(v / 2.0, 0.5, v / den, t2 / den).0)
}

pub fn t_dist_2t_kernel(x: f64, deg_freedom: f64) -> Result<f64, WorksheetErrorCode> {
    let x = validate_nonnegative_x(x)?;
    let v = truncate_positive_integer(deg_freedom)?;
    let t2 = x * x;
    let den = v + t2;
    Ok(bratio(v / 2.0, 0.5, v / den, t2 / den).0)
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
    let v = truncate_positive_integer(deg_freedom)?;
    // Invert the published two-tail surface (t_dist_2t's staging) directly at
    // p, not the one-tail CDF at 1-p/2 — same principle as CHIINV/FINV
    // (W109 b19: residuals collapse from -4..-238 to mostly +-1..7).
    // The surface is decreasing in x, so invert the negated forward at -p.
    let f = move |x: f64| {
        let t2 = x * x;
        let den = v + t2;
        -bratio(v / 2.0, 0.5, v / den, t2 / den).0
    };
    let hi = search_upper_bound(-p, 1.0, f);
    Ok(bisect_inverse(-p, 0.0, hi, f))
}

fn map_domain(value: Result<f64, WorksheetErrorCode>) -> CalcValue {
    match value {
        Ok(number) => CalcValue::number(number),
        Err(code) => CalcValue::error(code),
    }
}

fn eval_chisq_dist_prepared(args: &[CalcValue]) -> Result<CalcValue, ChiFTEvalError> {
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

fn eval_chisq_dist_rt_prepared(args: &[CalcValue]) -> Result<CalcValue, ChiFTEvalError> {
    if !CHISQ_DIST_RT_META.arity.accepts(args.len()) {
        return Err(arity_error(&CHISQ_DIST_RT_META, args.len()));
    }
    let x = coerce_prepared_to_number(&args[0]).map_err(ChiFTEvalError::Coercion)?;
    let df = coerce_prepared_to_number(&args[1]).map_err(ChiFTEvalError::Coercion)?;
    Ok(map_domain(chisq_dist_rt_kernel(x, df)))
}

fn eval_chisq_inv_prepared(args: &[CalcValue]) -> Result<CalcValue, ChiFTEvalError> {
    if !CHISQ_INV_META.arity.accepts(args.len()) {
        return Err(arity_error(&CHISQ_INV_META, args.len()));
    }
    let p = coerce_prepared_to_number(&args[0]).map_err(ChiFTEvalError::Coercion)?;
    let df = coerce_prepared_to_number(&args[1]).map_err(ChiFTEvalError::Coercion)?;
    Ok(map_domain(chisq_inv_kernel(p, df)))
}

fn eval_chisq_inv_rt_prepared(args: &[CalcValue]) -> Result<CalcValue, ChiFTEvalError> {
    if !CHISQ_INV_RT_META.arity.accepts(args.len()) {
        return Err(arity_error(&CHISQ_INV_RT_META, args.len()));
    }
    let p = coerce_prepared_to_number(&args[0]).map_err(ChiFTEvalError::Coercion)?;
    let df = coerce_prepared_to_number(&args[1]).map_err(ChiFTEvalError::Coercion)?;
    Ok(map_domain(chisq_inv_rt_kernel(p, df)))
}

fn eval_f_dist_prepared(args: &[CalcValue]) -> Result<CalcValue, ChiFTEvalError> {
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

fn eval_f_dist_rt_prepared(args: &[CalcValue]) -> Result<CalcValue, ChiFTEvalError> {
    if !F_DIST_RT_META.arity.accepts(args.len()) {
        return Err(arity_error(&F_DIST_RT_META, args.len()));
    }
    let x = coerce_prepared_to_number(&args[0]).map_err(ChiFTEvalError::Coercion)?;
    let d1 = coerce_prepared_to_number(&args[1]).map_err(ChiFTEvalError::Coercion)?;
    let d2 = coerce_prepared_to_number(&args[2]).map_err(ChiFTEvalError::Coercion)?;
    Ok(map_domain(f_dist_rt_kernel(x, d1, d2)))
}

fn eval_f_inv_prepared(args: &[CalcValue]) -> Result<CalcValue, ChiFTEvalError> {
    if !F_INV_META.arity.accepts(args.len()) {
        return Err(arity_error(&F_INV_META, args.len()));
    }
    let p = coerce_prepared_to_number(&args[0]).map_err(ChiFTEvalError::Coercion)?;
    let d1 = coerce_prepared_to_number(&args[1]).map_err(ChiFTEvalError::Coercion)?;
    let d2 = coerce_prepared_to_number(&args[2]).map_err(ChiFTEvalError::Coercion)?;
    Ok(map_domain(f_inv_kernel(p, d1, d2)))
}

fn eval_f_inv_rt_prepared(args: &[CalcValue]) -> Result<CalcValue, ChiFTEvalError> {
    if !F_INV_RT_META.arity.accepts(args.len()) {
        return Err(arity_error(&F_INV_RT_META, args.len()));
    }
    let p = coerce_prepared_to_number(&args[0]).map_err(ChiFTEvalError::Coercion)?;
    let d1 = coerce_prepared_to_number(&args[1]).map_err(ChiFTEvalError::Coercion)?;
    let d2 = coerce_prepared_to_number(&args[2]).map_err(ChiFTEvalError::Coercion)?;
    Ok(map_domain(f_inv_rt_kernel(p, d1, d2)))
}

fn eval_t_dist_prepared(args: &[CalcValue]) -> Result<CalcValue, ChiFTEvalError> {
    if !T_DIST_META.arity.accepts(args.len()) {
        return Err(arity_error(&T_DIST_META, args.len()));
    }
    let x = coerce_prepared_to_number(&args[0]).map_err(ChiFTEvalError::Coercion)?;
    let df = coerce_prepared_to_number(&args[1]).map_err(ChiFTEvalError::Coercion)?;
    let cumulative = coerce_prepared_to_number(&args[2]).map_err(ChiFTEvalError::Coercion)?;
    Ok(map_domain(t_dist_kernel(x, df, density_flag(cumulative))))
}

fn eval_t_dist_2t_prepared(args: &[CalcValue]) -> Result<CalcValue, ChiFTEvalError> {
    if !T_DIST_2T_META.arity.accepts(args.len()) {
        return Err(arity_error(&T_DIST_2T_META, args.len()));
    }
    let x = coerce_prepared_to_number(&args[0]).map_err(ChiFTEvalError::Coercion)?;
    let df = coerce_prepared_to_number(&args[1]).map_err(ChiFTEvalError::Coercion)?;
    Ok(map_domain(t_dist_2t_kernel(x, df)))
}

fn eval_t_dist_rt_prepared(args: &[CalcValue]) -> Result<CalcValue, ChiFTEvalError> {
    if !T_DIST_RT_META.arity.accepts(args.len()) {
        return Err(arity_error(&T_DIST_RT_META, args.len()));
    }
    let x = coerce_prepared_to_number(&args[0]).map_err(ChiFTEvalError::Coercion)?;
    let df = coerce_prepared_to_number(&args[1]).map_err(ChiFTEvalError::Coercion)?;
    Ok(map_domain(t_dist_rt_kernel(x, df)))
}

fn eval_t_inv_prepared(args: &[CalcValue]) -> Result<CalcValue, ChiFTEvalError> {
    if !T_INV_META.arity.accepts(args.len()) {
        return Err(arity_error(&T_INV_META, args.len()));
    }
    let p = coerce_prepared_to_number(&args[0]).map_err(ChiFTEvalError::Coercion)?;
    let df = coerce_prepared_to_number(&args[1]).map_err(ChiFTEvalError::Coercion)?;
    Ok(map_domain(t_inv_kernel(p, df)))
}

fn eval_t_inv_2t_prepared(args: &[CalcValue]) -> Result<CalcValue, ChiFTEvalError> {
    if !T_INV_2T_META.arity.accepts(args.len()) {
        return Err(arity_error(&T_INV_2T_META, args.len()));
    }
    let p = coerce_prepared_to_number(&args[0]).map_err(ChiFTEvalError::Coercion)?;
    let df = coerce_prepared_to_number(&args[1]).map_err(ChiFTEvalError::Coercion)?;
    Ok(map_domain(t_inv_2t_kernel(p, df)))
}

fn eval_tdist_prepared(args: &[CalcValue]) -> Result<CalcValue, ChiFTEvalError> {
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
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_chisq_dist_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_chisq_dist_rt_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_chisq_dist_rt_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_chisq_inv_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_chisq_inv_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_chisq_inv_rt_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_chisq_inv_rt_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_chidist_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_chisq_dist_rt_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_chiinv_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_chisq_inv_rt_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_f_dist_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_f_dist_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_f_dist_rt_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_f_dist_rt_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_f_inv_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_f_inv_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_f_inv_rt_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_f_inv_rt_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_fdist_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_f_dist_rt_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_finv_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_f_inv_rt_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_t_dist_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_t_dist_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_t_dist_2t_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_t_dist_2t_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_t_dist_rt_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_t_dist_rt_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_t_inv_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_t_inv_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_t_inv_2t_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_t_inv_2t_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_tdist_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, ChiFTEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_tdist_prepared,
        ChiFTEvalError::Coercion,
    )
}

pub fn eval_tinv_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, ChiFTEvalError> {
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
    fn chisq_df1_matches_published_erf_erfc_sqrt_half_x() {
        use crate::functions::special_dist_family::{erf_of_sqrt_half_x, erfc_of_sqrt_half_x};
        for x in [0.0, 0.25, 0.5, 1.0, 2.0, 3.841458820694124, 10.0, 20.0] {
            let rt = chisq_dist_rt_kernel(x, 1.0).unwrap();
            let cdf = chisq_dist_kernel(x, 1.0, true).unwrap();
            assert_eq!(rt.to_bits(), erfc_of_sqrt_half_x(x).unwrap().to_bits(), "rt x={x}");
            assert_eq!(cdf.to_bits(), erf_of_sqrt_half_x(x).unwrap().to_bits(), "cdf x={x}");
        }
        // Exact publication at zero does not depend on the ERFC body.
        assert_eq!(
            chisq_dist_rt_kernel(0.0, 1.0).unwrap().to_bits(),
            0x3ff0000000000000
        );
        // Live Excel 16.0 b20228 CHIDIST(1,1) is 0x3fd44ed0bb7cb209.
        // The current ERFC.PRECISE body is still one ULP off that witness;
        // the df=1 dispatch only claims the composition, not the ERFC body.
        for x in [0.0, 0.125, 0.5, 1.0, 2.0, 8.0, 16.0, 40.0] {
            let rt = chisq_dist_rt_kernel(x, 2.0).unwrap();
            let exp = crate::excel_numeric::excel_exp(-(x / 2.0));
            assert_eq!(rt.to_bits(), exp.to_bits(), "df=2 rt x={x}");
            let cdf = chisq_dist_kernel(x, 2.0, true).unwrap();
            let expon = crate::functions::discrete_dist_family::expon_dist_kernel(
                x / 2.0,
                1.0,
                true,
            )
            .unwrap();
            assert_eq!(cdf.to_bits(), expon.to_bits(), "df=2 cdf x={x}");
        }
    }

    #[test]
    fn even_df_chidist_matches_poisson_cdf_identity() {
        // Live Excel 16.0 b20228: CHIDIST(x, 2(k+1)) == POISSON.DIST(k, x/2, TRUE).
        // After the Poisson CDF dispatch, OxFunc reproduces that identity.
        for i in 1..40 {
            let x = 0.5 * f64::from(i);
            let chi4 = chisq_dist_rt_kernel(x, 4.0).unwrap();
            let pois1 = crate::functions::discrete_dist_family::poisson_dist_kernel(
                1.0, x / 2.0, true,
            )
            .unwrap();
            assert_eq!(chi4.to_bits(), pois1.to_bits(), "df=4 x={x}");
            let chi6 = chisq_dist_rt_kernel(x, 6.0).unwrap();
            let pois2 = crate::functions::discrete_dist_family::poisson_dist_kernel(
                2.0, x / 2.0, true,
            )
            .unwrap();
            assert_eq!(chi6.to_bits(), pois2.to_bits(), "df=6 x={x}");
        }
    }

    #[test]
    fn chisq_cdf_matches_gamma_scale_two_identity() {
        for df in [1.0, 2.0, 3.0, 4.0, 5.0, 10.0] {
            for x in [0.0, 0.5, 1.0, 2.0, 8.0, 20.0] {
                let chi = chisq_dist_kernel(x, df, true).unwrap();
                let gam = crate::functions::beta_gamma_stats_family::gamma_dist_kernel(
                    x, df / 2.0, 2.0, true,
                )
                .unwrap();
                assert_eq!(chi.to_bits(), gam.to_bits(), "df={df} x={x}");
            }
        }
    }

    #[test]
    fn chisq_inv_matches_gamma_inv_scale_two_identity() {
        for df in [1.0, 2.0, 3.0, 4.0, 5.0, 10.0] {
            for p in [0.01, 0.05, 0.25, 0.5, 0.9, 0.99] {
                let chi = chisq_inv_kernel(p, df).unwrap();
                let gam = crate::functions::beta_gamma_stats_family::gamma_inv_kernel(
                    p,
                    df / 2.0,
                    2.0,
                )
                .unwrap();
                assert_eq!(chi.to_bits(), gam.to_bits(), "df={df} p={p}");
            }
        }
        assert_eq!(chisq_inv_kernel(0.0, 2.0), Ok(0.0));
        assert_eq!(chisq_inv_kernel(1.0, 2.0), Err(WorksheetErrorCode::Num));
    }

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

    // BUG-FUNC-039 item 1: x=0 divergence published as #NUM!, not +inf.
    #[test]
    fn chisq_pdf_x0_df1_returns_num_not_inf() {
        assert_eq!(chisq_pdf_kernel(0.0, 1.0), Err(WorksheetErrorCode::Num));
        // df=2 at x=0 is finite (0.5); must not be broken by the fix.
        assert!((chisq_pdf_kernel(0.0, 2.0).unwrap() - 0.5).abs() < 1e-15);
        // df=3 at x=0 is 0.0.
        assert_eq!(chisq_pdf_kernel(0.0, 3.0), Ok(0.0));
    }

    // BUG-FUNC-039 item 1: F.DIST(0, d1<2, d2, FALSE) -> #NUM!.
    #[test]
    fn f_pdf_x0_d1lt2_returns_num_not_inf() {
        assert_eq!(f_pdf_kernel(0.0, 1.0, 5.0), Err(WorksheetErrorCode::Num));
        // d1=2 at x=0 is finite (1.0); must not be broken.
        assert!((f_pdf_kernel(0.0, 2.0, 5.0).unwrap() - 1.0).abs() < 1e-15);
        // d1=3 at x=0 is 0.0.
        assert_eq!(f_pdf_kernel(0.0, 3.0, 5.0), Ok(0.0));
    }

    // BUG-FUNC-039 item 2: df exactly 1e10 must be admitted; df > 1e10 rejected.
    #[test]
    fn df_boundary_at_1e10_is_admitted() {
        // df = 1e10 exactly: should succeed (just needs to parse without error).
        assert!(chisq_dist_kernel(1.0, 1e10, false).is_ok());
        // df = 1e10 + 1 (next representable float above 1e10): must be rejected.
        let above = 1e10 + 1.0;
        assert_eq!(
            chisq_dist_kernel(1.0, above, false),
            Err(WorksheetErrorCode::Num)
        );
        // F family too.
        assert!(f_dist_kernel(1.0, 1e10, 1e10, false).is_ok());
        assert_eq!(
            f_dist_kernel(1.0, above, 1e10, false),
            Err(WorksheetErrorCode::Num)
        );
    }
}
