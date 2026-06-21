use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::coerce_prepared_to_number;
use crate::functions::binary_numeric::{BinaryNumericSurfaceError, eval_binary_numeric_surface};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const ATAN2_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ATAN2",
    arity: Arity::exact(2),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    lift_broadcast_profile: FunctionMeta::DEFAULT_LIFT_BROADCAST_PROFILE,
    kernel_signature_class: KernelSignatureClass::NumsToNum,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
    error_collapse_profile: FunctionMeta::DEFAULT_ERROR_COLLAPSE_PROFILE,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Atan2EvalError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
    ZeroVector,
    Domain(WorksheetErrorCode),
}

impl From<BinaryNumericSurfaceError> for Atan2EvalError {
    fn from(value: BinaryNumericSurfaceError) -> Self {
        match value {
            BinaryNumericSurfaceError::ArityMismatch { expected, actual } => {
                Self::ArityMismatch { expected, actual }
            }
            BinaryNumericSurfaceError::Coercion(error) => Self::Coercion(error),
            BinaryNumericSurfaceError::Domain(WorksheetErrorCode::Div0) => Self::ZeroVector,
            BinaryNumericSurfaceError::Domain(code) => Self::Domain(code),
        }
    }
}

pub fn atan2_kernel(x_num: f64, y_num: f64) -> Result<f64, WorksheetErrorCode> {
    if x_num == 0.0 && y_num == 0.0 {
        return Err(WorksheetErrorCode::Div0);
    }
    // Excel returns #NUM! when the implicit y/x ratio overflows to infinity
    // (x != 0 and |y| >> |x|), e.g. ATAN2(1e-200, 1e109). A finite ratio stays a
    // number, and the axis case x == 0 (ratio undefined, not overflowing) keeps
    // its ±pi/2 value. Pinned against live Excel 16.0 b20026 (BUG-FUNC-027 B3).
    if x_num != 0.0 && (y_num / x_num).is_infinite() {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(y_num.atan2(x_num))
}

pub fn eval_atan2_adapter_prepared(args: &[CalcValue]) -> Result<CalcValue, Atan2EvalError> {
    if !ATAN2_META.arity.accepts(args.len()) {
        return Err(Atan2EvalError::ArityMismatch {
            expected: ATAN2_META.arity.min,
            actual: args.len(),
        });
    }
    let x_num = coerce_prepared_to_number(&args[0]).map_err(Atan2EvalError::Coercion)?;
    let y_num = coerce_prepared_to_number(&args[1]).map_err(Atan2EvalError::Coercion)?;
    atan2_kernel(x_num, y_num)
        .map(CalcValue::number)
        .map_err(|_| Atan2EvalError::ZeroVector)
}

pub fn eval_atan2_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, Atan2EvalError> {
    eval_binary_numeric_surface(args, resolver, atan2_kernel).map_err(Atan2EvalError::from)
}

pub fn map_atan2_error_to_ws(e: &Atan2EvalError) -> WorksheetErrorCode {
    match e {
        Atan2EvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        Atan2EvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        Atan2EvalError::Coercion(_) => WorksheetErrorCode::Value,
        Atan2EvalError::ZeroVector => WorksheetErrorCode::Div0,
        Atan2EvalError::Domain(code) => *code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::{CalcArray, ExcelText};

    struct NoResolver;

    impl ReferenceSystemProvider for NoResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            ReferenceSystemCapabilities::permissive_local()
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

    #[test]
    fn atan2_kernel_matches_excel_axis_order() {
        assert_eq!(atan2_kernel(1.0, 0.0), Ok(0.0));
        assert_eq!(atan2_kernel(0.0, 1.0), Ok(std::f64::consts::FRAC_PI_2));
    }

    #[test]
    fn atan2_kernel_zero_vector_is_div0() {
        assert_eq!(atan2_kernel(0.0, 0.0), Err(WorksheetErrorCode::Div0));
    }

    #[test]
    fn atan2_overflowing_ratio_is_num_axis_stays_finite() {
        // BUG-FUNC-027 B3: Excel returns #NUM! when y/x overflows to infinity
        // (verified vs live Excel 16.0 b20026), while the axis case x==0 stays valid.
        assert_eq!(atan2_kernel(1e-200, 1e109), Err(WorksheetErrorCode::Num)); // y/x = inf
        assert_eq!(atan2_kernel(-1e-200, -6e199), Err(WorksheetErrorCode::Num)); // witness
        assert!(atan2_kernel(1e-200, 1e108).is_ok()); // y/x = 1e308, finite
        assert!(atan2_kernel(0.0, 5.0).is_ok()); // axis x==0 -> +pi/2, not #NUM!
        assert!(atan2_kernel(5.0, 0.0).is_ok()); // axis y==0 -> 0
    }

    #[test]
    fn eval_atan2_surface_accepts_numeric_text() {
        let got = eval_atan2_surface(
            &[
                (CalcValue::text(ExcelText::from_utf16_code_units(
                    "1".encode_utf16().collect(),
                ))),
                (CalcValue::number(0.0)),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(CalcValue::number(0.0)));
    }

    #[test]
    fn eval_atan2_spills_array_and_per_cell_errors() {
        let got = eval_atan2_surface(
            &[
                (CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::number(0.0),
                        CalcValue::number(1.0),
                    ]])
                    .unwrap(),
                )),
                (CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::number(0.0),
                        CalcValue::number(1.0),
                    ]])
                    .unwrap(),
                )),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::error(WorksheetErrorCode::Div0),
                    CalcValue::number(std::f64::consts::FRAC_PI_4),
                ]])
                .unwrap()
            ))
        );
    }
}
