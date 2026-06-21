use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{coerce_prepared_to_number, prepare_arg_values_only};
use crate::functions::chi_f_t_family::t_inv_2t_kernel;
use crate::functions::normal_dist_common::erf_approx;
use crate::functions::variance_common::{VarianceDivisor, stdev_from_values};
use crate::resolver::{ReferenceSystemProvider, resolve_eval_value};
use crate::value::WorksheetErrorCode;
use crate::value::{CalcValue, CoreValue};

pub const CONFIDENCE_T_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.CONFIDENCE.T",
    arity: Arity::exact(3),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    // CONFIDENCE.T is scalar-shaped by-index and broadcasts its first three arguments over an
    // array (`[0,1,2]`). Verified live Excel 16.0 build 20026.
    lift_broadcast_profile: FunctionMeta::lift_at(&[0, 1, 2]),
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::None,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
    error_collapse_profile: FunctionMeta::DEFAULT_ERROR_COLLAPSE_PROFILE,
};

pub const Z_TEST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.Z.TEST",
    arity: Arity { min: 2, max: 3 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    // Z.TEST keeps its data array fixed and broadcasts the scalar x/sigma arguments at positions
    // `1` and `2` over an array (`[1,2]`). Verified live Excel 16.0 build 20026.
    lift_broadcast_profile: FunctionMeta::lift_at(&[1, 2]),
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::RefOnly,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
    error_collapse_profile: FunctionMeta::DEFAULT_ERROR_COLLAPSE_PROFILE,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ConfidenceTestEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

fn scalar_number(
    arg: &CalcValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<f64, ConfidenceTestEvalError> {
    let prepared =
        prepare_arg_values_only(arg, resolver).map_err(ConfidenceTestEvalError::Coercion)?;
    match prepared.core() {
        CoreValue::Missing | CoreValue::Empty => Ok(0.0),
        _ => coerce_prepared_to_number(&prepared).map_err(ConfidenceTestEvalError::Coercion),
    }
}

fn standard_normal_cdf(x: f64) -> f64 {
    0.5 * (1.0 + erf_approx(x / std::f64::consts::SQRT_2))
}

fn collect_numeric_values_from_eval(
    value: CalcValue,
    out: &mut Vec<f64>,
) -> Result<(), ConfidenceTestEvalError> {
    match value.core() {
        CoreValue::Array(array) => {
            for cell in array.iter_row_major() {
                match cell.core() {
                    CoreValue::Number(n) => out.push(*n),
                    CoreValue::Error(code) => {
                        return Err(ConfidenceTestEvalError::Domain(*code));
                    }
                    CoreValue::Text(_)
                    | CoreValue::Logical(_)
                    | CoreValue::Empty
                    | CoreValue::Missing => {}
                    CoreValue::Array(_) | CoreValue::Reference(_) => {}
                }
            }
            Ok(())
        }
        CoreValue::Number(n) => {
            out.push(*n);
            Ok(())
        }
        CoreValue::Text(_) | CoreValue::Logical(_) => Ok(()),
        CoreValue::Error(code) => Err(ConfidenceTestEvalError::Domain(*code)),
        CoreValue::Reference(_) => Err(ConfidenceTestEvalError::Domain(WorksheetErrorCode::Value)),
        _ => Err(ConfidenceTestEvalError::Domain(WorksheetErrorCode::Value)),
    }
}

fn collect_numeric_values(
    arg: &CalcValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<Vec<f64>, ConfidenceTestEvalError> {
    let eval = match arg.core() {
        CoreValue::Reference(reference) => resolve_eval_value(resolver, reference)
            .map_err(|e| ConfidenceTestEvalError::Coercion(CoercionError::RefResolution(e)))?,
        CoreValue::Missing | CoreValue::Empty => CalcValue::number(0.0),
        _ => arg.clone(),
    };
    let mut out = Vec::new();
    collect_numeric_values_from_eval(eval, &mut out)?;
    Ok(out)
}

pub fn confidence_t_kernel(
    alpha: f64,
    standard_dev: f64,
    size: f64,
) -> Result<f64, WorksheetErrorCode> {
    if !alpha.is_finite() || !standard_dev.is_finite() || !size.is_finite() {
        return Err(WorksheetErrorCode::Value);
    }
    let n = size.trunc();
    if !(0.0 < alpha && alpha < 1.0) || standard_dev <= 0.0 || n < 2.0 {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(t_inv_2t_kernel(alpha, n - 1.0)? * standard_dev / n.sqrt())
}

pub fn z_test_kernel(
    values: &[f64],
    x: f64,
    sigma: Option<f64>,
) -> Result<f64, WorksheetErrorCode> {
    if values.is_empty() || !x.is_finite() {
        return Err(WorksheetErrorCode::Value);
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let sigma = match sigma {
        Some(value) => value,
        None => stdev_from_values(values, VarianceDivisor::Sample)?,
    };
    if sigma <= 0.0 || !sigma.is_finite() {
        return Err(WorksheetErrorCode::Num);
    }
    let z = (mean - x) / (sigma / (values.len() as f64).sqrt());
    Ok(1.0 - standard_normal_cdf(z))
}

pub fn eval_confidence_t_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, ConfidenceTestEvalError> {
    if !CONFIDENCE_T_META.arity.accepts(args.len()) {
        return Err(ConfidenceTestEvalError::ArityMismatch {
            expected_min: CONFIDENCE_T_META.arity.min,
            expected_max: CONFIDENCE_T_META.arity.max,
            actual: args.len(),
        });
    }
    confidence_t_kernel(
        scalar_number(&args[0], resolver)?,
        scalar_number(&args[1], resolver)?,
        scalar_number(&args[2], resolver)?,
    )
    .map(CalcValue::number)
    .map_err(ConfidenceTestEvalError::Domain)
}

pub fn eval_z_test_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, ConfidenceTestEvalError> {
    if !Z_TEST_META.arity.accepts(args.len()) {
        return Err(ConfidenceTestEvalError::ArityMismatch {
            expected_min: Z_TEST_META.arity.min,
            expected_max: Z_TEST_META.arity.max,
            actual: args.len(),
        });
    }
    let values = collect_numeric_values(&args[0], resolver)?;
    z_test_kernel(
        &values,
        scalar_number(&args[1], resolver)?,
        args.get(2)
            .map(|arg| scalar_number(arg, resolver))
            .transpose()?,
    )
    .map(CalcValue::number)
    .map_err(ConfidenceTestEvalError::Domain)
}

pub fn map_confidence_test_error_to_ws(error: &ConfidenceTestEvalError) -> WorksheetErrorCode {
    match error {
        ConfidenceTestEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        ConfidenceTestEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        ConfidenceTestEvalError::Coercion(_) => WorksheetErrorCode::Value,
        ConfidenceTestEvalError::Domain(code) => *code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::{CalcArray, ExcelText, ReferenceLike};

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
            assert_eq!(reference.target(), "A1:A5");
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(3.0)],
                    vec![CalcValue::number(6.0)],
                    vec![CalcValue::number(7.0)],
                    vec![CalcValue::number(8.0)],
                    vec![CalcValue::number(6.0)],
                ])
                .unwrap(),
            ))
        }
    }

    fn num(n: f64) -> CalcValue {
        CalcValue::number(n)
    }

    #[test]
    fn confidence_t_matches_known_lane() {
        let got = confidence_t_kernel(0.05, 2.5, 50.0).unwrap();
        assert!((got - 0.7104921387393248).abs() < 1e-9);
    }

    #[test]
    fn confidence_t_rejects_invalid_domain() {
        assert_eq!(
            confidence_t_kernel(1.0, 2.0, 10.0),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            confidence_t_kernel(0.05, 0.0, 10.0),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            confidence_t_kernel(0.05, 2.0, 1.0),
            Err(WorksheetErrorCode::Num)
        );
    }

    #[test]
    fn z_test_matches_seed_lane_with_supplied_sigma() {
        let values = [3.0, 6.0, 7.0, 8.0, 6.0];
        let got = z_test_kernel(&values, 4.0, Some(1.5)).unwrap();
        assert!((got - 0.0014345563960383074).abs() < 1e-12);
    }

    #[test]
    fn z_test_surface_collects_numeric_reference_values() {
        let got = eval_z_test_surface(
            &[
                CalcValue::reference(ReferenceLike::new(
                    crate::value::ReferenceKind::Area,
                    "A1:A5".to_string(),
                )),
                num(4.0),
                num(1.5),
            ],
            &NoResolver,
        )
        .unwrap();
        assert!(
            matches!(got.core(), CoreValue::Number(n) if (*n - 0.0014345563960383074).abs() < 1e-12)
        );
    }

    #[test]
    fn confidence_surface_accepts_numeric_text() {
        let got = eval_confidence_t_surface(
            &[
                (CalcValue::text(ExcelText::from_utf16_code_units(
                    "0.05".encode_utf16().collect(),
                ))),
                num(2.5),
                num(50.0),
            ],
            &NoResolver,
        )
        .unwrap();
        assert!(
            matches!(got.core(), CoreValue::Number(n) if (*n - 0.7104921387393248).abs() < 1e-9)
        );
    }
}
