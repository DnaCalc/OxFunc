use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{coerce_prepared_to_number, prepare_args_values_only};
use crate::functions::special_math_common::{
    bisect_inverse, ln_gamma, regularized_beta, regularized_gamma_p,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{CalcValue, CoreValue, WorksheetErrorCode};

const BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.BETA_GAMMA_STATS_BASE",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    lift_broadcast_profile: FunctionMeta::DEFAULT_LIFT_BROADCAST_PROFILE,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::None,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
};

pub const BETA_DIST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.BETA.DIST",
    arity: Arity { min: 4, max: 6 },
    ..BASE_META
};
pub const BETA_INV_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.BETA.INV",
    arity: Arity { min: 3, max: 5 },
    ..BASE_META
};
// The legacy BETADIST/BETAINV/GAMMADIST/GAMMAINV compatibility surfaces are scalar-shaped
// by-index and broadcast their first three arguments (`[0,1,2]`) over an array. The modern
// `.`-named BETA.DIST/BETA.INV/GAMMA.DIST/GAMMA.INV surfaces lift natively and carry the
// default. Verified live Excel 16.0 build 20026.
pub const BETADIST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.BETADIST",
    arity: Arity { min: 3, max: 5 },
    lift_broadcast_profile: FunctionMeta::lift_at(&[0, 1, 2]),
    ..BASE_META
};
pub const BETAINV_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.BETAINV",
    arity: Arity { min: 3, max: 5 },
    lift_broadcast_profile: FunctionMeta::lift_at(&[0, 1, 2]),
    ..BASE_META
};
pub const GAMMA_DIST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.GAMMA.DIST",
    arity: Arity::exact(4),
    ..BASE_META
};
pub const GAMMA_INV_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.GAMMA.INV",
    arity: Arity::exact(3),
    ..BASE_META
};
pub const GAMMADIST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.GAMMADIST",
    arity: Arity::exact(4),
    lift_broadcast_profile: FunctionMeta::lift_at(&[0, 1, 2]),
    ..BASE_META
};
pub const GAMMAINV_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.GAMMAINV",
    arity: Arity::exact(3),
    lift_broadcast_profile: FunctionMeta::lift_at(&[0, 1, 2]),
    ..BASE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum BetaGammaStatsError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

fn number_arg(args: &[CalcValue], idx: usize) -> Result<f64, BetaGammaStatsError> {
    match args.get(idx) {
        None => Ok(0.0),
        Some(value) if matches!(value.core(), CoreValue::Missing | CoreValue::Empty) => Ok(0.0),
        Some(value) => coerce_prepared_to_number(value).map_err(BetaGammaStatsError::Coercion),
    }
}

fn bool_flag(value: f64) -> bool {
    value != 0.0
}

fn beta_log_norm(alpha: f64, beta: f64) -> f64 {
    ln_gamma(alpha) + ln_gamma(beta) - ln_gamma(alpha + beta)
}

fn positive_integer(value: f64) -> Option<u32> {
    if value.is_finite() && value >= 1.0 && value <= 200.0 && value.fract() == 0.0 {
        Some(value as u32)
    } else {
        None
    }
}

fn binomial_coefficient(n: u32, k: u32) -> f64 {
    let k = k.min(n - k);
    let mut acc = 1.0;
    for i in 1..=k {
        acc *= (n - k + i) as f64;
        acc /= i as f64;
    }
    acc
}

fn regularized_beta_integer_shape(z: f64, alpha: f64, beta: f64) -> Option<f64> {
    let a = positive_integer(alpha)?;
    let b = positive_integer(beta)?;
    if z == 0.0 {
        return Some(0.0);
    }
    if z == 1.0 {
        return Some(1.0);
    }
    let n = a + b - 1;
    let mut sum = 0.0;
    for j in a..=n {
        sum += binomial_coefficient(n, j) * z.powi(j as i32) * (1.0 - z).powi((n - j) as i32);
    }
    Some(sum)
}

fn regularized_gamma_p_integer_shape(alpha: f64, x: f64) -> Option<f64> {
    let a = positive_integer(alpha)?;
    if x == 0.0 {
        return Some(0.0);
    }
    let mut term = 1.0;
    let mut sum = 1.0;
    for k in 1..a {
        term *= x / k as f64;
        sum += term;
    }
    Some(1.0 - (-x).exp() * sum)
}

fn validate_beta_shape(
    alpha: f64,
    beta: f64,
    lower: f64,
    upper: f64,
) -> Result<(), BetaGammaStatsError> {
    if !alpha.is_finite() || !beta.is_finite() || !lower.is_finite() || !upper.is_finite() {
        return Err(BetaGammaStatsError::Domain(WorksheetErrorCode::Value));
    }
    if alpha <= 0.0 || beta <= 0.0 || upper <= lower {
        return Err(BetaGammaStatsError::Domain(WorksheetErrorCode::Num));
    }
    Ok(())
}

fn beta_dist_kernel(
    x: f64,
    alpha: f64,
    beta: f64,
    cumulative: bool,
    lower: f64,
    upper: f64,
) -> Result<f64, BetaGammaStatsError> {
    validate_beta_shape(alpha, beta, lower, upper)?;
    if !x.is_finite() {
        return Err(BetaGammaStatsError::Domain(WorksheetErrorCode::Value));
    }
    if x < lower || x > upper {
        return Err(BetaGammaStatsError::Domain(WorksheetErrorCode::Num));
    }
    let z = (x - lower) / (upper - lower);
    if cumulative {
        return Ok(regularized_beta_integer_shape(z, alpha, beta)
            .unwrap_or_else(|| regularized_beta(z, alpha, beta)));
    }
    let log_pdf = (alpha - 1.0) * z.ln() + (beta - 1.0) * (1.0 - z).ln()
        - beta_log_norm(alpha, beta)
        - (upper - lower).ln();
    let pdf = log_pdf.exp();
    if pdf.is_finite() {
        Ok(pdf)
    } else {
        Err(BetaGammaStatsError::Domain(WorksheetErrorCode::Num))
    }
}

fn beta_inv_kernel(
    probability: f64,
    alpha: f64,
    beta: f64,
    lower: f64,
    upper: f64,
) -> Result<f64, BetaGammaStatsError> {
    validate_beta_shape(alpha, beta, lower, upper)?;
    if !probability.is_finite() {
        return Err(BetaGammaStatsError::Domain(WorksheetErrorCode::Value));
    }
    if !(0.0..=1.0).contains(&probability) {
        return Err(BetaGammaStatsError::Domain(WorksheetErrorCode::Num));
    }
    let z = bisect_inverse(probability, 0.0, 1.0, |v| regularized_beta(v, alpha, beta));
    Ok(lower + z * (upper - lower))
}

fn validate_gamma_shape(alpha: f64, beta: f64) -> Result<(), BetaGammaStatsError> {
    if !alpha.is_finite() || !beta.is_finite() {
        return Err(BetaGammaStatsError::Domain(WorksheetErrorCode::Value));
    }
    if alpha <= 0.0 || beta <= 0.0 {
        return Err(BetaGammaStatsError::Domain(WorksheetErrorCode::Num));
    }
    Ok(())
}

fn gamma_dist_kernel(
    x: f64,
    alpha: f64,
    beta: f64,
    cumulative: bool,
) -> Result<f64, BetaGammaStatsError> {
    validate_gamma_shape(alpha, beta)?;
    if !x.is_finite() {
        return Err(BetaGammaStatsError::Domain(WorksheetErrorCode::Value));
    }
    if x < 0.0 {
        return Err(BetaGammaStatsError::Domain(WorksheetErrorCode::Num));
    }
    if cumulative {
        let scaled = x / beta;
        return Ok(regularized_gamma_p_integer_shape(alpha, scaled)
            .unwrap_or_else(|| regularized_gamma_p(alpha, scaled)));
    }
    let log_pdf = (alpha - 1.0) * x.ln() - x / beta - ln_gamma(alpha) - alpha * beta.ln();
    let pdf = log_pdf.exp();
    if pdf.is_finite() {
        Ok(pdf)
    } else {
        Err(BetaGammaStatsError::Domain(WorksheetErrorCode::Num))
    }
}

fn gamma_inv_kernel(probability: f64, alpha: f64, beta: f64) -> Result<f64, BetaGammaStatsError> {
    validate_gamma_shape(alpha, beta)?;
    if !probability.is_finite() {
        return Err(BetaGammaStatsError::Domain(WorksheetErrorCode::Value));
    }
    // Excel: p==0 returns the support lower bound (0); p==1 -> #NUM! (+inf);
    // p<0 or p>1 -> #NUM!. Verified vs live Excel 16.0 build 20026:
    // GAMMA.INV(0,3,2)=0, GAMMA.INV(1,3,2)=#NUM! (BUG-FUNC-039 / oxf-99zz).
    if probability < 0.0 || probability >= 1.0 {
        return Err(BetaGammaStatsError::Domain(WorksheetErrorCode::Num));
    }
    if probability == 0.0 {
        return Ok(0.0);
    }
    let hi = beta * (alpha + 10.0 * alpha.sqrt() + 10.0).max(1.0);
    let x = bisect_inverse(probability, 0.0, hi, |v| {
        regularized_gamma_p(alpha, v / beta)
    });
    Ok(x)
}

fn eval_numeric(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    meta: &FunctionMeta,
    kernel: impl FnOnce(&[CalcValue]) -> Result<f64, BetaGammaStatsError>,
) -> Result<CalcValue, BetaGammaStatsError> {
    if !meta.arity.accepts(args.len()) {
        return Err(BetaGammaStatsError::ArityMismatch {
            expected_min: meta.arity.min,
            expected_max: meta.arity.max,
            actual: args.len(),
        });
    }
    let prepared =
        prepare_args_values_only(args, resolver).map_err(BetaGammaStatsError::Coercion)?;
    kernel(&prepared).map(CalcValue::number)
}

pub fn eval_beta_dist_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, BetaGammaStatsError> {
    eval_numeric(args, resolver, &BETA_DIST_META, |prepared| {
        beta_dist_kernel(
            number_arg(prepared, 0)?,
            number_arg(prepared, 1)?,
            number_arg(prepared, 2)?,
            bool_flag(number_arg(prepared, 3)?),
            prepared
                .get(4)
                .map(|_| number_arg(prepared, 4))
                .transpose()?
                .unwrap_or(0.0),
            prepared
                .get(5)
                .map(|_| number_arg(prepared, 5))
                .transpose()?
                .unwrap_or(1.0),
        )
    })
}

pub fn eval_beta_inv_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, BetaGammaStatsError> {
    eval_numeric(args, resolver, &BETA_INV_META, |prepared| {
        beta_inv_kernel(
            number_arg(prepared, 0)?,
            number_arg(prepared, 1)?,
            number_arg(prepared, 2)?,
            prepared
                .get(3)
                .map(|_| number_arg(prepared, 3))
                .transpose()?
                .unwrap_or(0.0),
            prepared
                .get(4)
                .map(|_| number_arg(prepared, 4))
                .transpose()?
                .unwrap_or(1.0),
        )
    })
}

pub fn eval_betadist_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, BetaGammaStatsError> {
    eval_numeric(args, resolver, &BETADIST_META, |prepared| {
        beta_dist_kernel(
            number_arg(prepared, 0)?,
            number_arg(prepared, 1)?,
            number_arg(prepared, 2)?,
            true,
            prepared
                .get(3)
                .map(|_| number_arg(prepared, 3))
                .transpose()?
                .unwrap_or(0.0),
            prepared
                .get(4)
                .map(|_| number_arg(prepared, 4))
                .transpose()?
                .unwrap_or(1.0),
        )
    })
}

pub fn eval_betainv_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, BetaGammaStatsError> {
    eval_numeric(args, resolver, &BETAINV_META, |prepared| {
        beta_inv_kernel(
            number_arg(prepared, 0)?,
            number_arg(prepared, 1)?,
            number_arg(prepared, 2)?,
            prepared
                .get(3)
                .map(|_| number_arg(prepared, 3))
                .transpose()?
                .unwrap_or(0.0),
            prepared
                .get(4)
                .map(|_| number_arg(prepared, 4))
                .transpose()?
                .unwrap_or(1.0),
        )
    })
}

pub fn eval_gamma_dist_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, BetaGammaStatsError> {
    eval_numeric(args, resolver, &GAMMA_DIST_META, |prepared| {
        gamma_dist_kernel(
            number_arg(prepared, 0)?,
            number_arg(prepared, 1)?,
            number_arg(prepared, 2)?,
            bool_flag(number_arg(prepared, 3)?),
        )
    })
}

pub fn eval_gamma_inv_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, BetaGammaStatsError> {
    eval_numeric(args, resolver, &GAMMA_INV_META, |prepared| {
        gamma_inv_kernel(
            number_arg(prepared, 0)?,
            number_arg(prepared, 1)?,
            number_arg(prepared, 2)?,
        )
    })
}

pub fn eval_gammadist_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, BetaGammaStatsError> {
    eval_numeric(args, resolver, &GAMMADIST_META, |prepared| {
        gamma_dist_kernel(
            number_arg(prepared, 0)?,
            number_arg(prepared, 1)?,
            number_arg(prepared, 2)?,
            bool_flag(number_arg(prepared, 3)?),
        )
    })
}

pub fn eval_gammainv_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, BetaGammaStatsError> {
    eval_gamma_inv_surface(args, resolver)
}

pub fn map_beta_gamma_stats_error_to_ws(error: &BetaGammaStatsError) -> WorksheetErrorCode {
    match error {
        BetaGammaStatsError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        BetaGammaStatsError::Coercion(CoercionError::WorksheetError(code)) => *code,
        BetaGammaStatsError::Coercion(_) => WorksheetErrorCode::Value,
        BetaGammaStatsError::Domain(code) => *code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::ExcelText;

    struct NoResolver;
    impl ReferenceSystemProvider for NoResolver {
        fn capabilities(&self) -> crate::resolver::ReferenceSystemCapabilities {
            crate::resolver::ReferenceSystemCapabilities::permissive_local()
        }
        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<CalcValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            Err(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
        }
    }

    fn num(n: f64) -> CalcValue {
        CalcValue::number(n)
    }

    fn txt(s: &str) -> CalcValue {
        CalcValue::text(ExcelText::from_utf16_code_units(s.encode_utf16().collect()))
    }

    fn assert_close(value: CalcValue, expected: f64) {
        match value.core() {
            CoreValue::Number(n) => assert!((*n - expected).abs() < 1e-9, "{n} vs {expected}"),
            other => panic!("expected number, got {other:?}"),
        }
    }

    fn assert_number_bits(value: &CalcValue, expected_bits: u64) {
        match value.core() {
            CoreValue::Number(n) => assert_eq!(
                n.to_bits(),
                expected_bits,
                "{n} vs {}",
                f64::from_bits(expected_bits)
            ),
            other => panic!("expected number, got {other:?}"),
        }
    }

    #[test]
    fn beta_and_legacy_aliases_match() {
        let modern =
            eval_beta_dist_surface(&[num(0.5), num(2.0), num(3.0), num(1.0)], &NoResolver).unwrap();
        let legacy = eval_betadist_surface(&[num(0.5), num(2.0), num(3.0)], &NoResolver).unwrap();
        assert_close(modern.clone(), 0.6875);
        assert_number_bits(&modern, 0x3fe6_0000_0000_0000);
        assert_eq!(modern, legacy);
    }

    #[test]
    fn beta_inverse_aliases_match() {
        let modern = eval_beta_inv_surface(&[num(0.6), num(2.0), num(3.0)], &NoResolver).unwrap();
        let legacy = eval_betainv_surface(&[num(0.6), num(2.0), num(3.0)], &NoResolver).unwrap();
        match (modern.core(), legacy.core()) {
            (CoreValue::Number(a), CoreValue::Number(b)) => assert!((*a - *b).abs() < 1e-9),
            other => panic!("unexpected values {other:?}"),
        }
    }

    #[test]
    fn gamma_and_legacy_aliases_match() {
        let modern =
            eval_gamma_dist_surface(&[num(2.0), num(3.0), num(2.0), num(1.0)], &NoResolver)
                .unwrap();
        let legacy =
            eval_gammadist_surface(&[num(2.0), num(3.0), num(2.0), num(1.0)], &NoResolver).unwrap();
        match (modern.core(), legacy.core()) {
            (CoreValue::Number(a), CoreValue::Number(b)) => {
                assert!((*a - *b).abs() < 1e-12);
                assert!((*a - 0.08030139707139416).abs() < 1e-9);
            }
            other => panic!("unexpected values {other:?}"),
        }
    }

    #[test]
    fn gamma_inverse_aliases_match() {
        let modern = eval_gamma_inv_surface(&[num(0.5), num(3.0), num(2.0)], &NoResolver).unwrap();
        let legacy = eval_gammainv_surface(&[num(0.5), num(3.0), num(2.0)], &NoResolver).unwrap();
        match (modern.core(), legacy.core()) {
            (CoreValue::Number(a), CoreValue::Number(b)) => assert!((*a - *b).abs() < 1e-9),
            other => panic!("unexpected values {other:?}"),
        }
    }

    #[test]
    fn numeric_text_and_domain_errors_are_exercised() {
        let got = eval_beta_dist_surface(&[txt("0.5"), num(2.0), num(3.0), num(1.0)], &NoResolver)
            .unwrap();
        assert_close(got, 0.6875);
        assert_eq!(
            eval_beta_dist_surface(&[num(1.5), num(2.0), num(3.0), num(1.0)], &NoResolver),
            Err(BetaGammaStatsError::Domain(WorksheetErrorCode::Num))
        );
        assert_eq!(
            eval_gamma_dist_surface(&[num(-1.0), num(2.0), num(3.0), num(1.0)], &NoResolver),
            Err(BetaGammaStatsError::Domain(WorksheetErrorCode::Num))
        );
    }

    // BUG-FUNC-039 item 3: GAMMA.INV p=0 and p=1 boundary cases.
    #[test]
    fn gamma_inv_boundary_cases() {
        // p=0 -> 0 (Excel: GAMMA.INV(0,..) returns the support lower bound; live
        // Excel 16.0 build 20026 GAMMA.INV(0,3,2)=0).
        assert_eq!(gamma_inv_kernel(0.0, 1.0, 1.0), Ok(0.0));
        // p<0 -> #NUM!.
        assert_eq!(
            gamma_inv_kernel(-0.1, 1.0, 1.0),
            Err(BetaGammaStatsError::Domain(WorksheetErrorCode::Num))
        );
        // p=1 -> #NUM! (Excel: p must be < 1).
        assert_eq!(
            gamma_inv_kernel(1.0, 1.0, 1.0),
            Err(BetaGammaStatsError::Domain(WorksheetErrorCode::Num))
        );
        // Normal interior case still works.
        let v = gamma_inv_kernel(0.5, 3.0, 2.0).unwrap();
        assert!(
            (v - 5.348_).abs() < 0.01,
            "GAMMA.INV(0.5,3,2) ~ 5.348, got {v}"
        );
    }
}
