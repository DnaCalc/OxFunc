use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedArgValue, prepare_arg_values_only};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

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
    left: &PreparedArgValue,
    right: &PreparedArgValue,
    _resolver: &impl ReferenceResolver,
) -> Result<bool, CoercionError> {
    match (left, right) {
        (
            PreparedArgValue::Eval(EvalValue::Text(a)),
            PreparedArgValue::Eval(EvalValue::Text(b)),
        ) => Ok(a
            .to_string_lossy()
            .eq_ignore_ascii_case(&b.to_string_lossy())),
        (
            PreparedArgValue::Eval(EvalValue::Number(a)),
            PreparedArgValue::Eval(EvalValue::Number(b)),
        ) => Ok(a == b),
        (
            PreparedArgValue::Eval(EvalValue::Logical(a)),
            PreparedArgValue::Eval(EvalValue::Logical(b)),
        ) => Ok(a == b),
        (
            PreparedArgValue::Eval(EvalValue::Error(a)),
            PreparedArgValue::Eval(EvalValue::Error(b)),
        ) => Ok(a == b),
        (PreparedArgValue::MissingArg, PreparedArgValue::MissingArg)
        | (PreparedArgValue::EmptyCell, PreparedArgValue::EmptyCell) => Ok(true),
        // Excel matches numeric text only as text, not against numbers.
        (PreparedArgValue::Eval(EvalValue::Reference(_)), _)
        | (_, PreparedArgValue::Eval(EvalValue::Reference(_))) => {
            Err(CoercionError::UnsupportedValueKind("reference_like"))
        }
        (PreparedArgValue::Eval(EvalValue::Array(_)), _)
        | (_, PreparedArgValue::Eval(EvalValue::Array(_))) => {
            Err(CoercionError::UnsupportedValueKind("array"))
        }
        (PreparedArgValue::Eval(EvalValue::Lambda(_)), _)
        | (_, PreparedArgValue::Eval(EvalValue::Lambda(_))) => {
            Err(CoercionError::UnsupportedValueKind("lambda_value"))
        }
        _ => Ok(false),
    }
}

pub fn eval_switch_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, SwitchEvalError> {
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
                PreparedArgValue::Eval(v) => v,
                PreparedArgValue::MissingArg => EvalValue::Error(WorksheetErrorCode::NA),
                PreparedArgValue::EmptyCell => EvalValue::Number(0.0),
            });
        }
        idx += 2;
    }
    if has_default {
        let prepared = prepare_arg_values_only(args.last().expect("default exists"), resolver)
            .map_err(SwitchEvalError::ResultPreparation)?;
        return Ok(match prepared {
            PreparedArgValue::Eval(v) => v,
            PreparedArgValue::MissingArg => EvalValue::Error(WorksheetErrorCode::NA),
            PreparedArgValue::EmptyCell => EvalValue::Number(0.0),
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
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ExcelText, ReferenceLike};

    struct NoResolver;
    impl ReferenceResolver for NoResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            ResolverCapabilities::permissive_local()
        }
        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            Err(RefResolutionError::UnresolvedReference {
                target: reference.target.clone(),
            })
        }
    }

    #[test]
    fn switch_matches_case_insensitive_text_and_exact_types() {
        let got = eval_switch_surface(
            &[
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("a"))),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("A"))),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("a"))),
                CallArgValue::Eval(EvalValue::Number(2.0)),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(1.0)));

        let got = eval_switch_surface(
            &[
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("2"))),
                CallArgValue::Eval(EvalValue::Number(2.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("2"))),
                CallArgValue::Eval(EvalValue::Number(2.0)),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(2.0)));
    }

    #[test]
    fn switch_uses_default_or_na() {
        let got = eval_switch_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(2.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("a"))),
                CallArgValue::Eval(EvalValue::Number(3.0)),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("c"))),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("d"))),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Text(ExcelText::from_interop_assignment("d")))
        );

        let got = eval_switch_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(2.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("a"))),
            ],
            &NoResolver,
        );
        assert_eq!(got, Err(SwitchEvalError::NotAvailable));
    }

    #[test]
    fn switch_is_lazy_over_unmatched_pairs_and_results() {
        let got = eval_switch_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(2.0)),
                CallArgValue::Eval(EvalValue::Number(2.0)),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("a"))),
                CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::Div0)),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment("b"))),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Text(ExcelText::from_interop_assignment("a")))
        );
    }
}
