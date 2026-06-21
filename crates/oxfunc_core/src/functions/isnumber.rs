use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::run_values_only_prepared;
use crate::resolver::ReferenceSystemProvider;
use crate::value::{CalcArray, CalcValue, CoreValue, WorksheetErrorCode};

pub const ISNUMBER_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ISNUMBER",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::None,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IsnumberEvalError {
    ArityMismatch { expected: usize, actual: usize },
    Preparation(crate::coercion::CoercionError),
}

fn is_number_cell(cell: &CalcValue) -> CalcValue {
    CalcValue::logical(matches!(cell.core(), CoreValue::Number(_)))
}

pub fn eval_isnumber_adapter_prepared(args: &[CalcValue]) -> Result<CalcValue, IsnumberEvalError> {
    if !ISNUMBER_META.arity.accepts(args.len()) {
        return Err(IsnumberEvalError::ArityMismatch {
            expected: ISNUMBER_META.arity.min,
            actual: args.len(),
        });
    }

    match args[0].core() {
        CoreValue::Array(array) => {
            let cells = array.iter_row_major().map(is_number_cell).collect();
            Ok(CalcValue::array(
                CalcArray::new(array.shape(), cells).expect("input array shape is valid"),
            ))
        }
        _ => {
            let is_number = matches!(args[0].core(), CoreValue::Number(_));
            Ok(CalcValue::logical(is_number))
        }
    }
}

pub fn eval_isnumber_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, IsnumberEvalError> {
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
        value: Option<CalcValue>,
    }

    impl ReferenceSystemProvider for MockResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            ReferenceSystemCapabilities::permissive_local()
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<CalcValue, crate::resolver::ReferenceResolutionError> {
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
        let args = [(CalcValue::number(1.0))];
        let got = eval_isnumber_surface(&args, &MockResolver { value: None });
        assert_eq!(got, Ok(CalcValue::logical(true)));
    }

    #[test]
    fn eval_isnumber_on_text_is_false() {
        let args = [(CalcValue::text(ExcelText::from_utf16_code_units(
            "1".encode_utf16().collect(),
        )))];
        let got = eval_isnumber_surface(&args, &MockResolver { value: None });
        assert_eq!(got, Ok(CalcValue::logical(false)));
    }

    #[test]
    fn eval_isnumber_on_error_is_false() {
        let args = [(CalcValue::error(WorksheetErrorCode::NA))];
        let got = eval_isnumber_surface(&args, &MockResolver { value: None });
        assert_eq!(got, Ok(CalcValue::logical(false)));
    }

    #[test]
    fn eval_isnumber_array_lifts_elementwise() {
        let args = [(CalcValue::array(
            CalcArray::from_rows(vec![vec![
                CalcValue::number(1.0),
                CalcValue::text(ExcelText::from_interop_assignment("x")),
                CalcValue::error(WorksheetErrorCode::NA),
                CalcValue::empty(),
            ]])
            .unwrap(),
        ))];
        let got = eval_isnumber_surface(&args, &MockResolver { value: None });
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::logical(true),
                    CalcValue::logical(false),
                    CalcValue::logical(false),
                    CalcValue::logical(false),
                ]])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_isnumber_reference_path_uses_preparation() {
        let args = [CalcValue::reference(ReferenceLike::new(
            ReferenceKind::A1,
            "A1".to_string(),
        ))];
        let got = eval_isnumber_surface(
            &args,
            &MockResolver {
                value: Some(CalcValue::number(3.0)),
            },
        );
        assert_eq!(got, Ok(CalcValue::logical(true)));
    }
}
