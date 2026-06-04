use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::prepare_arg_values_only;
use crate::resolver::ReferenceSystemProvider;
use crate::value::WorksheetErrorCode;
use crate::value::{CalcValue, CoreValue};

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
    left: &CalcValue,
    right: &CalcValue,
    _resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<bool, CoercionError> {
    match (left.core(), right.core()) {
        (CoreValue::Text(a), CoreValue::Text(b)) => Ok(a
            .to_string_lossy()
            .eq_ignore_ascii_case(&b.to_string_lossy())),
        (CoreValue::Number(a), CoreValue::Number(b)) => Ok(a == b),
        (CoreValue::Logical(a), CoreValue::Logical(b)) => Ok(a == b),
        (CoreValue::Error(a), CoreValue::Error(b)) => Ok(a == b),
        (CoreValue::Missing, CoreValue::Missing) | (CoreValue::Empty, CoreValue::Empty) => Ok(true),
        // Excel matches numeric text only as text, not against numbers.
        (CoreValue::Reference(_), _) | (_, CoreValue::Reference(_)) => {
            Err(CoercionError::UnsupportedValueKind("reference_like"))
        }
        (CoreValue::Array(_), _) | (_, CoreValue::Array(_)) => {
            Err(CoercionError::UnsupportedValueKind("array"))
        }
        _ => Ok(false),
    }
}

fn materialize_switch_result(prepared: CalcValue) -> CalcValue {
    match prepared.core() {
        CoreValue::Missing => CalcValue::error(WorksheetErrorCode::NA),
        CoreValue::Empty => CalcValue::number(0.0),
        _ => prepared,
    }
}

pub fn eval_switch_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, SwitchEvalError> {
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
            return Ok(materialize_switch_result(prepared));
        }
        idx += 2;
    }
    if has_default {
        let prepared = prepare_arg_values_only(args.last().expect("default exists"), resolver)
            .map_err(SwitchEvalError::ResultPreparation)?;
        return Ok(materialize_switch_result(prepared));
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
    fn switch_matches_case_insensitive_text_and_exact_types() {
        let got = eval_switch_surface(
            &[
                (CalcValue::text(ExcelText::from_interop_assignment("a"))),
                (CalcValue::text(ExcelText::from_interop_assignment("A"))),
                (CalcValue::number(1.0)),
                (CalcValue::text(ExcelText::from_interop_assignment("a"))),
                (CalcValue::number(2.0)),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(CalcValue::number(1.0)));

        let got = eval_switch_surface(
            &[
                (CalcValue::text(ExcelText::from_interop_assignment("2"))),
                (CalcValue::number(2.0)),
                (CalcValue::number(1.0)),
                (CalcValue::text(ExcelText::from_interop_assignment("2"))),
                (CalcValue::number(2.0)),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(CalcValue::number(2.0)));
    }

    #[test]
    fn switch_uses_default_or_na() {
        let got = eval_switch_surface(
            &[
                (CalcValue::number(2.0)),
                (CalcValue::number(1.0)),
                (CalcValue::text(ExcelText::from_interop_assignment("a"))),
                (CalcValue::number(3.0)),
                (CalcValue::text(ExcelText::from_interop_assignment("c"))),
                (CalcValue::text(ExcelText::from_interop_assignment("d"))),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::text(ExcelText::from_interop_assignment("d")))
        );

        let got = eval_switch_surface(
            &[
                (CalcValue::number(2.0)),
                (CalcValue::number(1.0)),
                (CalcValue::text(ExcelText::from_interop_assignment("a"))),
            ],
            &NoResolver,
        );
        assert_eq!(got, Err(SwitchEvalError::NotAvailable));
    }

    #[test]
    fn switch_is_lazy_over_unmatched_pairs_and_results() {
        let got = eval_switch_surface(
            &[
                (CalcValue::number(2.0)),
                (CalcValue::number(2.0)),
                (CalcValue::text(ExcelText::from_interop_assignment("a"))),
                (CalcValue::error(WorksheetErrorCode::Div0)),
                (CalcValue::text(ExcelText::from_interop_assignment("b"))),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::text(ExcelText::from_interop_assignment("a")))
        );
    }
}
