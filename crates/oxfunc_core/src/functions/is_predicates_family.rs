use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, run_values_only_prepared,
};
use crate::resolver::ReferenceResolver;
use crate::value::{ArrayCellValue, CallArgValue, EvalArray, EvalValue, WorksheetErrorCode};

const INFORMATION_PREDICATE_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.INFORMATION_PREDICATE_BASE",
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

pub const ISBLANK_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ISBLANK",
    ..INFORMATION_PREDICATE_BASE_META
};

pub const ISERR_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ISERR",
    ..INFORMATION_PREDICATE_BASE_META
};

pub const ISERROR_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ISERROR",
    ..INFORMATION_PREDICATE_BASE_META
};

pub const ISLOGICAL_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ISLOGICAL",
    ..INFORMATION_PREDICATE_BASE_META
};

pub const ISNA_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ISNA",
    ..INFORMATION_PREDICATE_BASE_META
};

pub const ISNONTEXT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ISNONTEXT",
    ..INFORMATION_PREDICATE_BASE_META
};

pub const ISTEXT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ISTEXT",
    ..INFORMATION_PREDICATE_BASE_META
};

pub const ISODD_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ISODD",
    coercion_lift_profile: CoercionLiftProfile::Custom,
    ..INFORMATION_PREDICATE_BASE_META
};

pub const ISREF_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ISREF",
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    fec_dependency_profile: FecDependencyProfile::RefOnly,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    ..INFORMATION_PREDICATE_BASE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum InformationPredicateEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Preparation(CoercionError),
}

fn arity_error(meta: &FunctionMeta, actual: usize) -> InformationPredicateEvalError {
    InformationPredicateEvalError::ArityMismatch {
        expected_min: meta.arity.min,
        expected_max: meta.arity.max,
        actual,
    }
}

fn is_error_cell(cell: &ArrayCellValue) -> ArrayCellValue {
    match cell {
        ArrayCellValue::Error(_) => ArrayCellValue::Logical(true),
        ArrayCellValue::Number(_)
        | ArrayCellValue::Text(_)
        | ArrayCellValue::Logical(_)
        | ArrayCellValue::EmptyCell => ArrayCellValue::Logical(false),
    }
}

fn eval_boolean_predicate_surface(
    meta: &FunctionMeta,
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    predicate: impl Fn(&PreparedArgValue) -> bool,
) -> Result<EvalValue, InformationPredicateEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            if !meta.arity.accepts(prepared.len()) {
                return Err(arity_error(meta, prepared.len()));
            }
            Ok(EvalValue::Logical(predicate(&prepared[0])))
        },
        InformationPredicateEvalError::Preparation,
    )
}

fn coerce_isodd_number(arg: &PreparedArgValue) -> Result<f64, CoercionError> {
    match arg {
        PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell => Ok(0.0),
        PreparedArgValue::Eval(EvalValue::Logical(_)) => {
            Err(CoercionError::UnsupportedValueKind("logical"))
        }
        _ => coerce_prepared_to_number(arg),
    }
}

pub fn isodd_kernel(n: f64) -> bool {
    (n.trunc() as i64).rem_euclid(2) != 0
}

pub fn eval_isblank_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, InformationPredicateEvalError> {
    eval_boolean_predicate_surface(&ISBLANK_META, args, resolver, |arg| {
        matches!(arg, PreparedArgValue::EmptyCell)
    })
}

pub fn eval_iserr_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, InformationPredicateEvalError> {
    eval_boolean_predicate_surface(&ISERR_META, args, resolver, |arg| {
        matches!(
            arg,
            PreparedArgValue::Eval(EvalValue::Error(code)) if *code != WorksheetErrorCode::NA
        )
    })
}

pub fn eval_iserror_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, InformationPredicateEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            if !ISERROR_META.arity.accepts(prepared.len()) {
                return Err(arity_error(&ISERROR_META, prepared.len()));
            }
            match &prepared[0] {
                PreparedArgValue::Eval(EvalValue::Array(array)) => {
                    let cells = array.iter_row_major().map(is_error_cell).collect();
                    Ok(EvalValue::Array(
                        EvalArray::new(array.shape(), cells)
                            .expect("input array shape is valid"),
                    ))
                }
                _ => Ok(EvalValue::Logical(matches!(
                    prepared[0],
                    PreparedArgValue::Eval(EvalValue::Error(_))
                ))),
            }
        },
        InformationPredicateEvalError::Preparation,
    )
}

pub fn eval_islogical_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, InformationPredicateEvalError> {
    eval_boolean_predicate_surface(&ISLOGICAL_META, args, resolver, |arg| {
        matches!(arg, PreparedArgValue::Eval(EvalValue::Logical(_)))
    })
}

pub fn eval_isna_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, InformationPredicateEvalError> {
    eval_boolean_predicate_surface(&ISNA_META, args, resolver, |arg| {
        matches!(
            arg,
            PreparedArgValue::Eval(EvalValue::Error(WorksheetErrorCode::NA))
        )
    })
}

pub fn eval_isnontext_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, InformationPredicateEvalError> {
    eval_boolean_predicate_surface(&ISNONTEXT_META, args, resolver, |arg| {
        !matches!(arg, PreparedArgValue::Eval(EvalValue::Text(_)))
    })
}

pub fn eval_istext_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, InformationPredicateEvalError> {
    eval_boolean_predicate_surface(&ISTEXT_META, args, resolver, |arg| {
        matches!(arg, PreparedArgValue::Eval(EvalValue::Text(_)))
    })
}

pub fn eval_isodd_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, InformationPredicateEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            if !ISODD_META.arity.accepts(prepared.len()) {
                return Err(arity_error(&ISODD_META, prepared.len()));
            }
            Ok(EvalValue::Logical(isodd_kernel(
                coerce_isodd_number(&prepared[0])
                    .map_err(InformationPredicateEvalError::Preparation)?,
            )))
        },
        InformationPredicateEvalError::Preparation,
    )
}

pub fn eval_isref_surface(
    args: &[CallArgValue],
    _resolver: &impl ReferenceResolver,
) -> Result<EvalValue, InformationPredicateEvalError> {
    if !ISREF_META.arity.accepts(args.len()) {
        return Err(arity_error(&ISREF_META, args.len()));
    }
    Ok(EvalValue::Logical(matches!(
        &args[0],
        CallArgValue::Reference(_) | CallArgValue::Eval(EvalValue::Reference(_))
    )))
}

pub fn map_information_predicate_error_to_ws(
    error: &InformationPredicateEvalError,
) -> WorksheetErrorCode {
    match error {
        InformationPredicateEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        InformationPredicateEvalError::Preparation(CoercionError::WorksheetError(code)) => *code,
        InformationPredicateEvalError::Preparation(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ArrayCellValue, EvalArray, ExcelText, ReferenceKind, ReferenceLike};

    struct MockResolver {
        resolved: Option<EvalValue>,
    }

    impl ReferenceResolver for MockResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            ResolverCapabilities::permissive_local()
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            self.resolved
                .clone()
                .ok_or(RefResolutionError::UnresolvedReference {
                    target: reference.target.clone(),
                })
        }
    }

    fn txt(s: &str) -> ExcelText {
        ExcelText::from_utf16_code_units(s.encode_utf16().collect())
    }

    #[test]
    fn isblank_distinguishes_empty_cell_from_empty_string() {
        let blank_ref = CallArgValue::Reference(ReferenceLike {
            kind: ReferenceKind::A1,
            target: "B1".to_string(),
        });
        let blank_resolver = MockResolver {
            resolved: Some(EvalValue::Array(
                EvalArray::from_rows(vec![vec![ArrayCellValue::EmptyCell]]).unwrap(),
            )),
        };
        assert_eq!(
            eval_isblank_surface(&[blank_ref], &blank_resolver),
            Ok(EvalValue::Logical(true))
        );
        assert_eq!(
            eval_isblank_surface(
                &[CallArgValue::Eval(EvalValue::Text(txt("")))],
                &MockResolver { resolved: None },
            ),
            Ok(EvalValue::Logical(false))
        );
    }

    #[test]
    fn error_predicates_follow_excel_split() {
        assert_eq!(
            eval_iserr_surface(
                &[CallArgValue::Eval(EvalValue::Error(
                    WorksheetErrorCode::Div0
                ))],
                &MockResolver { resolved: None },
            ),
            Ok(EvalValue::Logical(true))
        );
        assert_eq!(
            eval_iserr_surface(
                &[CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::NA))],
                &MockResolver { resolved: None },
            ),
            Ok(EvalValue::Logical(false))
        );
        assert_eq!(
            eval_iserror_surface(
                &[CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::NA))],
                &MockResolver { resolved: None },
            ),
            Ok(EvalValue::Logical(true))
        );
        assert_eq!(
            eval_isna_surface(
                &[CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::NA))],
                &MockResolver { resolved: None },
            ),
            Ok(EvalValue::Logical(true))
        );
    }

    #[test]
    fn iserror_array_lifts_elementwise() {
        let got = eval_iserror_surface(
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Text(txt("Alice")),
                    ArrayCellValue::Number(30.0),
                    ArrayCellValue::Error(WorksheetErrorCode::NA),
                    ArrayCellValue::EmptyCell,
                ]])
                .unwrap(),
            ))],
            &MockResolver { resolved: None },
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Logical(false),
                    ArrayCellValue::Logical(false),
                    ArrayCellValue::Logical(true),
                    ArrayCellValue::Logical(false),
                ]])
                .unwrap()
            ))
        );
    }

    #[test]
    fn type_predicates_match_text_and_logical_rules() {
        assert_eq!(
            eval_islogical_surface(
                &[CallArgValue::Eval(EvalValue::Logical(true))],
                &MockResolver { resolved: None },
            ),
            Ok(EvalValue::Logical(true))
        );
        assert_eq!(
            eval_istext_surface(
                &[CallArgValue::Eval(EvalValue::Text(txt("x")))],
                &MockResolver { resolved: None },
            ),
            Ok(EvalValue::Logical(true))
        );
        assert_eq!(
            eval_isnontext_surface(
                &[CallArgValue::Eval(EvalValue::Number(1.0))],
                &MockResolver { resolved: None },
            ),
            Ok(EvalValue::Logical(true))
        );
        assert_eq!(
            eval_isnontext_surface(
                &[CallArgValue::Eval(EvalValue::Text(txt("x")))],
                &MockResolver { resolved: None },
            ),
            Ok(EvalValue::Logical(false))
        );
    }

    #[test]
    fn isodd_matches_seed_coercion_lanes() {
        assert_eq!(
            eval_isodd_surface(
                &[CallArgValue::Eval(EvalValue::Text(txt("3")))],
                &MockResolver { resolved: None },
            ),
            Ok(EvalValue::Logical(true))
        );
        assert_eq!(
            eval_isodd_surface(
                &[CallArgValue::Reference(ReferenceLike {
                    kind: ReferenceKind::A1,
                    target: "B1".to_string(),
                })],
                &MockResolver {
                    resolved: Some(EvalValue::Array(
                        EvalArray::from_rows(vec![vec![ArrayCellValue::EmptyCell]]).unwrap(),
                    )),
                },
            ),
            Ok(EvalValue::Logical(false))
        );
        assert!(matches!(
            eval_isodd_surface(
                &[CallArgValue::Eval(EvalValue::Logical(true))],
                &MockResolver { resolved: None },
            ),
            Err(InformationPredicateEvalError::Preparation(
                CoercionError::UnsupportedValueKind("logical")
            ))
        ));
    }

    #[test]
    fn isref_sees_reference_like_args_without_dereferencing() {
        assert_eq!(
            eval_isref_surface(
                &[CallArgValue::Reference(ReferenceLike {
                    kind: ReferenceKind::Area,
                    target: "A1:A2".to_string(),
                })],
                &MockResolver { resolved: None },
            ),
            Ok(EvalValue::Logical(true))
        );
        assert_eq!(
            eval_isref_surface(
                &[CallArgValue::Eval(EvalValue::Reference(ReferenceLike {
                    kind: ReferenceKind::A1,
                    target: "A1".to_string(),
                }))],
                &MockResolver { resolved: None },
            ),
            Ok(EvalValue::Logical(true))
        );
        assert_eq!(
            eval_isref_surface(
                &[CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![ArrayCellValue::Number(1.0)]]).unwrap(),
                ))],
                &MockResolver { resolved: None },
            ),
            Ok(EvalValue::Logical(false))
        );
    }
}
