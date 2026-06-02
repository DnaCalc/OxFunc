use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedValue, run_values_only_prepared};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{
    FunctionArg, FunctionArray, FunctionArrayCell, FunctionValue, WorksheetErrorCode,
};

pub const ISNUMBER_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ISNUMBER",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::None,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IsnumberEvalError {
    ArityMismatch { expected: usize, actual: usize },
    Preparation(crate::coercion::CoercionError),
}

fn is_number_cell(cell: &FunctionArrayCell) -> FunctionArrayCell {
    match cell {
        FunctionArrayCell::Number(_) => FunctionArrayCell::Logical(true),
        FunctionArrayCell::Text(_)
        | FunctionArrayCell::Logical(_)
        | FunctionArrayCell::Error(_)
        | FunctionArrayCell::EmptyCell => FunctionArrayCell::Logical(false),
    }
}

pub fn eval_isnumber_adapter_prepared(
    args: &[PreparedValue],
) -> Result<FunctionValue, IsnumberEvalError> {
    if !ISNUMBER_META.arity.accepts(args.len()) {
        return Err(IsnumberEvalError::ArityMismatch {
            expected: ISNUMBER_META.arity.min,
            actual: args.len(),
        });
    }

    match &args[0] {
        PreparedValue::Eval(FunctionValue::Array(array)) => {
            let cells = array.iter_row_major().map(is_number_cell).collect();
            Ok(FunctionValue::Array(
                FunctionArray::new(array.shape(), cells).expect("input array shape is valid"),
            ))
        }
        _ => {
            let is_number = matches!(args[0], PreparedValue::Eval(FunctionValue::Number(_)));
            Ok(FunctionValue::Logical(is_number))
        }
    }
}

pub fn eval_isnumber_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, IsnumberEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_isnumber_adapter_prepared,
        IsnumberEvalError::Preparation,
    )
}

pub fn map_isnumber_error_to_ws(e: &IsnumberEvalError) -> WorksheetErrorCode {
    match e {
        IsnumberEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        IsnumberEvalError::Preparation(crate::coercion::CoercionError::WorksheetError(code)) => {
            *code
        }
        IsnumberEvalError::Preparation(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::{ExcelText, ReferenceKind, ReferenceLike};

    struct MockResolver {
        value: Option<FunctionValue>,
    }

    impl ReferenceSystemProvider for MockResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            ReferenceSystemCapabilities::permissive_local()
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<FunctionValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            self.value.clone().ok_or(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
        }
    }

    #[test]
    fn eval_isnumber_on_number_is_true() {
        let args = [FunctionArg::Eval(FunctionValue::Number(1.0))];
        let got = eval_isnumber_surface(&args, &MockResolver { value: None });
        assert_eq!(got, Ok(FunctionValue::Logical(true)));
    }

    #[test]
    fn eval_isnumber_on_text_is_false() {
        let args = [FunctionArg::Eval(FunctionValue::Text(
            ExcelText::from_utf16_code_units("1".encode_utf16().collect()),
        ))];
        let got = eval_isnumber_surface(&args, &MockResolver { value: None });
        assert_eq!(got, Ok(FunctionValue::Logical(false)));
    }

    #[test]
    fn eval_isnumber_on_error_is_false() {
        let args = [FunctionArg::Eval(FunctionValue::Error(
            WorksheetErrorCode::NA,
        ))];
        let got = eval_isnumber_surface(&args, &MockResolver { value: None });
        assert_eq!(got, Ok(FunctionValue::Logical(false)));
    }

    #[test]
    fn eval_isnumber_array_lifts_elementwise() {
        let args = [FunctionArg::Eval(FunctionValue::Array(
            FunctionArray::from_rows(vec![vec![
                FunctionArrayCell::Number(1.0),
                FunctionArrayCell::Text(ExcelText::from_interop_assignment("x")),
                FunctionArrayCell::Error(WorksheetErrorCode::NA),
                FunctionArrayCell::EmptyCell,
            ]])
            .unwrap(),
        ))];
        let got = eval_isnumber_surface(&args, &MockResolver { value: None });
        assert_eq!(
            got,
            Ok(FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Logical(true),
                    FunctionArrayCell::Logical(false),
                    FunctionArrayCell::Logical(false),
                    FunctionArrayCell::Logical(false),
                ]])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_isnumber_reference_path_uses_preparation() {
        let args = [FunctionArg::Reference(ReferenceLike::new(
            ReferenceKind::A1,
            "A1".to_string(),
        ))];
        let got = eval_isnumber_surface(
            &args,
            &MockResolver {
                value: Some(FunctionValue::Number(3.0)),
            },
        );
        assert_eq!(got, Ok(FunctionValue::Logical(true)));
    }
}
