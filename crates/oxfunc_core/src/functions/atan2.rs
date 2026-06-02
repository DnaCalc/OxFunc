use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedValue, coerce_prepared_to_number};
use crate::functions::binary_numeric::{BinaryNumericSurfaceError, eval_binary_numeric_surface};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{FunctionArg, FunctionValue, WorksheetErrorCode};

pub const ATAN2_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ATAN2",
    arity: Arity::exact(2),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::NumsToNum,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
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
    Ok(y_num.atan2(x_num))
}

pub fn eval_atan2_adapter_prepared(
    args: &[PreparedValue],
) -> Result<FunctionValue, Atan2EvalError> {
    if !ATAN2_META.arity.accepts(args.len()) {
        return Err(Atan2EvalError::ArityMismatch {
            expected: ATAN2_META.arity.min,
            actual: args.len(),
        });
    }
    let x_num = coerce_prepared_to_number(&args[0]).map_err(Atan2EvalError::Coercion)?;
    let y_num = coerce_prepared_to_number(&args[1]).map_err(Atan2EvalError::Coercion)?;
    atan2_kernel(x_num, y_num)
        .map(FunctionValue::Number)
        .map_err(|_| Atan2EvalError::ZeroVector)
}

pub fn eval_atan2_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, Atan2EvalError> {
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
    use crate::value::{ExcelText, FunctionArray, FunctionArrayCell};

    struct NoResolver;

    impl ReferenceSystemProvider for NoResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            ReferenceSystemCapabilities::permissive_local()
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<FunctionValue, crate::resolver::ReferenceResolutionError> {
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
    fn eval_atan2_surface_accepts_numeric_text() {
        let got = eval_atan2_surface(
            &[
                FunctionArg::Eval(FunctionValue::Text(ExcelText::from_utf16_code_units(
                    "1".encode_utf16().collect(),
                ))),
                FunctionArg::Eval(FunctionValue::Number(0.0)),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(FunctionValue::Number(0.0)));
    }

    #[test]
    fn eval_atan2_spills_array_and_per_cell_errors() {
        let got = eval_atan2_surface(
            &[
                FunctionArg::Eval(FunctionValue::Array(
                    FunctionArray::from_rows(vec![vec![
                        FunctionArrayCell::Number(0.0),
                        FunctionArrayCell::Number(1.0),
                    ]])
                    .unwrap(),
                )),
                FunctionArg::Eval(FunctionValue::Array(
                    FunctionArray::from_rows(vec![vec![
                        FunctionArrayCell::Number(0.0),
                        FunctionArrayCell::Number(1.0),
                    ]])
                    .unwrap(),
                )),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Error(WorksheetErrorCode::Div0),
                    FunctionArrayCell::Number(std::f64::consts::FRAC_PI_4),
                ]])
                .unwrap()
            ))
        );
    }
}
