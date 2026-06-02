use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedValue, prepare_arg_values_only};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{FunctionArg, FunctionValue, WorksheetErrorCode};

pub const SWITCH_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SWITCH",
    arity: Arity {
        min: 3,
        max: usize::MAX,
    },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

#[derive(Debug, Clone, PartialEq)]
pub enum SwitchEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    ExprPreparation(CoercionError),
    CandidatePreparation(CoercionError),
    ResultPreparation(CoercionError),
    NotAvailable,
}

fn prepared_equal(
    left: &PreparedValue,
    right: &PreparedValue,
    _resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<bool, CoercionError> {
    match (left, right) {
        (
            PreparedValue::Eval(FunctionValue::Text(a)),
            PreparedValue::Eval(FunctionValue::Text(b)),
        ) => Ok(a
            .to_string_lossy()
            .eq_ignore_ascii_case(&b.to_string_lossy())),
        (
            PreparedValue::Eval(FunctionValue::Number(a)),
            PreparedValue::Eval(FunctionValue::Number(b)),
        ) => Ok(a == b),
        (
            PreparedValue::Eval(FunctionValue::Logical(a)),
            PreparedValue::Eval(FunctionValue::Logical(b)),
        ) => Ok(a == b),
        (
            PreparedValue::Eval(FunctionValue::Error(a)),
            PreparedValue::Eval(FunctionValue::Error(b)),
        ) => Ok(a == b),
        (PreparedValue::MissingArg, PreparedValue::MissingArg)
        | (PreparedValue::EmptyCell, PreparedValue::EmptyCell) => Ok(true),
        // Excel matches numeric text only as text, not against numbers.
        (PreparedValue::Eval(FunctionValue::Reference(_)), _)
        | (_, PreparedValue::Eval(FunctionValue::Reference(_))) => {
            Err(CoercionError::UnsupportedValueKind("reference_like"))
        }
        (PreparedValue::Eval(FunctionValue::Array(_)), _)
        | (_, PreparedValue::Eval(FunctionValue::Array(_))) => {
            Err(CoercionError::UnsupportedValueKind("array"))
        }
        _ => Ok(false),
    }
}

pub fn eval_switch_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, SwitchEvalError> {
    if !SWITCH_META.arity.accepts(args.len()) {
        return Err(SwitchEvalError::ArityMismatch {
            expected_min: SWITCH_META.arity.min,
            expected_max: SWITCH_META.arity.max,
            actual: args.len(),
        });
    }
    let expr =
        prepare_arg_values_only(&args[0], resolver).map_err(SwitchEvalError::ExprPreparation)?;
    let has_default = args.len() % 2 == 0;
    let pair_len = if has_default {
        args.len() - 1
    } else {
        args.len()
    };
    let mut idx = 1usize;
    while idx + 1 < pair_len {
        let candidate = prepare_arg_values_only(&args[idx], resolver)
            .map_err(SwitchEvalError::CandidatePreparation)?;
        if prepared_equal(&expr, &candidate, resolver)
            .map_err(SwitchEvalError::CandidatePreparation)?
        {
            let prepared = prepare_arg_values_only(&args[idx + 1], resolver)
                .map_err(SwitchEvalError::ResultPreparation)?;
            return Ok(match prepared {
                PreparedValue::Eval(v) => v,
                PreparedValue::MissingArg => FunctionValue::Error(WorksheetErrorCode::NA),
                PreparedValue::EmptyCell => FunctionValue::Number(0.0),
            });
        }
        idx += 2;
    }
    if has_default {
        let prepared = prepare_arg_values_only(args.last().expect("default exists"), resolver)
            .map_err(SwitchEvalError::ResultPreparation)?;
        return Ok(match prepared {
            PreparedValue::Eval(v) => v,
            PreparedValue::MissingArg => FunctionValue::Error(WorksheetErrorCode::NA),
            PreparedValue::EmptyCell => FunctionValue::Number(0.0),
        });
    }
    Err(SwitchEvalError::NotAvailable)
}

pub fn map_switch_error_to_ws(err: &SwitchEvalError) -> WorksheetErrorCode {
    match err {
        SwitchEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        SwitchEvalError::ExprPreparation(CoercionError::WorksheetError(code))
        | SwitchEvalError::CandidatePreparation(CoercionError::WorksheetError(code))
        | SwitchEvalError::ResultPreparation(CoercionError::WorksheetError(code)) => *code,
        SwitchEvalError::NotAvailable => WorksheetErrorCode::NA,
        _ => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::ExcelText;

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
    fn switch_matches_case_insensitive_text_and_exact_types() {
        let got = eval_switch_surface(
            &[
                FunctionArg::Eval(FunctionValue::Text(ExcelText::from_interop_assignment("a"))),
                FunctionArg::Eval(FunctionValue::Text(ExcelText::from_interop_assignment("A"))),
                FunctionArg::Eval(FunctionValue::Number(1.0)),
                FunctionArg::Eval(FunctionValue::Text(ExcelText::from_interop_assignment("a"))),
                FunctionArg::Eval(FunctionValue::Number(2.0)),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(FunctionValue::Number(1.0)));

        let got = eval_switch_surface(
            &[
                FunctionArg::Eval(FunctionValue::Text(ExcelText::from_interop_assignment("2"))),
                FunctionArg::Eval(FunctionValue::Number(2.0)),
                FunctionArg::Eval(FunctionValue::Number(1.0)),
                FunctionArg::Eval(FunctionValue::Text(ExcelText::from_interop_assignment("2"))),
                FunctionArg::Eval(FunctionValue::Number(2.0)),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(FunctionValue::Number(2.0)));
    }

    #[test]
    fn switch_uses_default_or_na() {
        let got = eval_switch_surface(
            &[
                FunctionArg::Eval(FunctionValue::Number(2.0)),
                FunctionArg::Eval(FunctionValue::Number(1.0)),
                FunctionArg::Eval(FunctionValue::Text(ExcelText::from_interop_assignment("a"))),
                FunctionArg::Eval(FunctionValue::Number(3.0)),
                FunctionArg::Eval(FunctionValue::Text(ExcelText::from_interop_assignment("c"))),
                FunctionArg::Eval(FunctionValue::Text(ExcelText::from_interop_assignment("d"))),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(FunctionValue::Text(ExcelText::from_interop_assignment("d")))
        );

        let got = eval_switch_surface(
            &[
                FunctionArg::Eval(FunctionValue::Number(2.0)),
                FunctionArg::Eval(FunctionValue::Number(1.0)),
                FunctionArg::Eval(FunctionValue::Text(ExcelText::from_interop_assignment("a"))),
            ],
            &NoResolver,
        );
        assert_eq!(got, Err(SwitchEvalError::NotAvailable));
    }

    #[test]
    fn switch_is_lazy_over_unmatched_pairs_and_results() {
        let got = eval_switch_surface(
            &[
                FunctionArg::Eval(FunctionValue::Number(2.0)),
                FunctionArg::Eval(FunctionValue::Number(2.0)),
                FunctionArg::Eval(FunctionValue::Text(ExcelText::from_interop_assignment("a"))),
                FunctionArg::Eval(FunctionValue::Error(WorksheetErrorCode::Div0)),
                FunctionArg::Eval(FunctionValue::Text(ExcelText::from_interop_assignment("b"))),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(FunctionValue::Text(ExcelText::from_interop_assignment("a")))
        );
    }
}
