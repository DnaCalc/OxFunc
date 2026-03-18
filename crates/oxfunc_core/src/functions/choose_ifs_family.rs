use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, prepare_arg_values_only,
};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const CHOOSE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.CHOOSE",
    arity: Arity { min: 2, max: 255 },
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

pub const IFS_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IFS",
    arity: Arity { min: 2, max: 254 },
    ..CHOOSE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum ChooseIfsEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    PairStructureMismatch {
        actual: usize,
    },
    IndexCoercion(CoercionError),
    ConditionCoercion(CoercionError),
    SelectedPreparation(CoercionError),
}

fn materialize_selected(prepared: PreparedArgValue) -> EvalValue {
    match prepared {
        PreparedArgValue::Eval(value) => value,
        PreparedArgValue::MissingArg => EvalValue::Error(WorksheetErrorCode::Value),
        PreparedArgValue::EmptyCell => EvalValue::Number(0.0),
    }
}

fn choose_index_from_number(
    choice_count: usize,
    index_num: f64,
) -> Result<usize, WorksheetErrorCode> {
    if !index_num.is_finite() {
        return Err(WorksheetErrorCode::Value);
    }

    let truncated = index_num.trunc();
    if truncated < 1.0 || truncated > choice_count as f64 {
        return Err(WorksheetErrorCode::Value);
    }

    Ok(truncated as usize - 1)
}

fn prepared_condition_truthy(prepared: &PreparedArgValue) -> Result<bool, CoercionError> {
    match prepared {
        PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell => Ok(false),
        PreparedArgValue::Eval(EvalValue::Logical(value)) => Ok(*value),
        PreparedArgValue::Eval(EvalValue::Number(value)) => Ok(*value != 0.0),
        PreparedArgValue::Eval(EvalValue::Text(text)) => {
            Err(CoercionError::NonNumericText(text.to_string_lossy()))
        }
        PreparedArgValue::Eval(EvalValue::Error(code)) => Err(CoercionError::WorksheetError(*code)),
        _ => Ok(coerce_prepared_to_number(prepared)? != 0.0),
    }
}

pub fn eval_choose_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, ChooseIfsEvalError> {
    if !CHOOSE_META.arity.accepts(args.len()) {
        return Err(ChooseIfsEvalError::ArityMismatch {
            expected_min: CHOOSE_META.arity.min,
            expected_max: CHOOSE_META.arity.max,
            actual: args.len(),
        });
    }

    let prepared_index =
        prepare_arg_values_only(&args[0], resolver).map_err(ChooseIfsEvalError::IndexCoercion)?;
    let index_num =
        coerce_prepared_to_number(&prepared_index).map_err(ChooseIfsEvalError::IndexCoercion)?;
    let selected_index = match choose_index_from_number(args.len() - 1, index_num) {
        Ok(index) => index,
        Err(code) => return Ok(EvalValue::Error(code)),
    };

    let selected = prepare_arg_values_only(&args[selected_index + 1], resolver)
        .map_err(ChooseIfsEvalError::SelectedPreparation)?;
    Ok(materialize_selected(selected))
}

pub fn eval_ifs_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, ChooseIfsEvalError> {
    if !IFS_META.arity.accepts(args.len()) {
        return Err(ChooseIfsEvalError::ArityMismatch {
            expected_min: IFS_META.arity.min,
            expected_max: IFS_META.arity.max,
            actual: args.len(),
        });
    }
    if args.len() % 2 != 0 {
        return Err(ChooseIfsEvalError::PairStructureMismatch { actual: args.len() });
    }

    for pair in args.chunks_exact(2) {
        let prepared_condition = prepare_arg_values_only(&pair[0], resolver)
            .map_err(ChooseIfsEvalError::ConditionCoercion)?;
        if prepared_condition_truthy(&prepared_condition)
            .map_err(ChooseIfsEvalError::ConditionCoercion)?
        {
            let selected = prepare_arg_values_only(&pair[1], resolver)
                .map_err(ChooseIfsEvalError::SelectedPreparation)?;
            return Ok(materialize_selected(selected));
        }
    }

    Ok(EvalValue::Error(WorksheetErrorCode::NA))
}

pub fn map_choose_ifs_error_to_ws(error: &ChooseIfsEvalError) -> WorksheetErrorCode {
    match error {
        ChooseIfsEvalError::ArityMismatch { .. }
        | ChooseIfsEvalError::PairStructureMismatch { .. } => WorksheetErrorCode::Value,
        ChooseIfsEvalError::IndexCoercion(CoercionError::WorksheetError(code))
        | ChooseIfsEvalError::ConditionCoercion(CoercionError::WorksheetError(code))
        | ChooseIfsEvalError::SelectedPreparation(CoercionError::WorksheetError(code)) => *code,
        ChooseIfsEvalError::IndexCoercion(_)
        | ChooseIfsEvalError::ConditionCoercion(_)
        | ChooseIfsEvalError::SelectedPreparation(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ExcelText, ReferenceKind, ReferenceLike};

    struct MockResolver;

    impl ReferenceResolver for MockResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            ResolverCapabilities::permissive_local()
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            match reference.target.as_str() {
                "BLANK" => Ok(EvalValue::Array(crate::value::EvalArray::from_scalar(
                    crate::value::ArrayCellValue::EmptyCell,
                ))),
                "POISON" => Err(RefResolutionError::UnresolvedReference {
                    target: reference.target.clone(),
                }),
                other => Err(RefResolutionError::UnresolvedReference {
                    target: other.to_string(),
                }),
            }
        }
    }

    fn text_arg(s: &str) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
            s.encode_utf16().collect(),
        )))
    }

    fn number_arg(n: f64) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Number(n))
    }

    #[test]
    fn choose_truncates_index_and_does_not_touch_unselected_poison() {
        let got = eval_choose_surface(
            &[
                number_arg(2.9),
                CallArgValue::Reference(ReferenceLike {
                    kind: ReferenceKind::A1,
                    target: "POISON".to_string(),
                }),
                text_arg("picked"),
                CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::Div0)),
            ],
            &MockResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "picked".encode_utf16().collect(),
            )))
        );
    }

    #[test]
    fn choose_rejects_zero_and_out_of_range_indices() {
        assert_eq!(
            eval_choose_surface(
                &[number_arg(0.9), number_arg(10.0), number_arg(20.0)],
                &MockResolver
            ),
            Ok(EvalValue::Error(WorksheetErrorCode::Value))
        );
        assert_eq!(
            eval_choose_surface(
                &[
                    number_arg(4.0),
                    number_arg(10.0),
                    number_arg(20.0),
                    number_arg(30.0)
                ],
                &MockResolver
            ),
            Ok(EvalValue::Error(WorksheetErrorCode::Value))
        );
    }

    #[test]
    fn choose_materializes_selected_blank_reference_as_zero() {
        let got = eval_choose_surface(
            &[
                number_arg(1.0),
                CallArgValue::Reference(ReferenceLike {
                    kind: ReferenceKind::A1,
                    target: "BLANK".to_string(),
                }),
                number_arg(7.0),
            ],
            &MockResolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(0.0)));
    }

    #[test]
    fn ifs_scans_pairs_left_to_right_and_short_circuits() {
        let got = eval_ifs_surface(
            &[
                CallArgValue::Eval(EvalValue::Logical(false)),
                CallArgValue::Reference(ReferenceLike {
                    kind: ReferenceKind::A1,
                    target: "POISON".to_string(),
                }),
                number_arg(2.0),
                text_arg("hit"),
                CallArgValue::Reference(ReferenceLike {
                    kind: ReferenceKind::A1,
                    target: "POISON".to_string(),
                }),
                number_arg(99.0),
            ],
            &MockResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "hit".encode_utf16().collect(),
            )))
        );
    }

    #[test]
    fn ifs_returns_na_when_no_condition_matches() {
        let got = eval_ifs_surface(
            &[
                CallArgValue::Eval(EvalValue::Logical(false)),
                number_arg(1.0),
                number_arg(0.0),
                number_arg(2.0),
                CallArgValue::EmptyCell,
                number_arg(3.0),
            ],
            &MockResolver,
        );
        assert_eq!(got, Ok(EvalValue::Error(WorksheetErrorCode::NA)));
    }

    #[test]
    fn ifs_rejects_odd_pair_structure_and_propagates_condition_errors() {
        let odd = eval_ifs_surface(
            &[number_arg(1.0), number_arg(2.0), number_arg(3.0)],
            &MockResolver,
        );
        assert_eq!(
            odd,
            Err(ChooseIfsEvalError::PairStructureMismatch { actual: 3 })
        );

        let condition_error = eval_ifs_surface(
            &[
                CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::Div0)),
                number_arg(1.0),
                CallArgValue::Eval(EvalValue::Logical(true)),
                number_arg(2.0),
            ],
            &MockResolver,
        );
        assert_eq!(
            condition_error,
            Err(ChooseIfsEvalError::ConditionCoercion(
                CoercionError::WorksheetError(WorksheetErrorCode::Div0)
            ))
        );

        let text_condition = eval_ifs_surface(&[text_arg("2"), number_arg(1.0)], &MockResolver);
        assert_eq!(
            text_condition,
            Err(ChooseIfsEvalError::ConditionCoercion(
                CoercionError::NonNumericText("2".to_string())
            ))
        );
    }
}
