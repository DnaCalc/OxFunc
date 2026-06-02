use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedValue, coerce_prepared_to_number, run_values_only_prepared,
    run_values_only_prepared_lifted,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{
    FunctionArg, FunctionArray, FunctionArrayCell, FunctionValue, WorksheetErrorCode,
};

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

fn is_error_cell(cell: &FunctionArrayCell) -> FunctionArrayCell {
    match cell {
        FunctionArrayCell::Error(_) => FunctionArrayCell::Logical(true),
        FunctionArrayCell::Number(_)
        | FunctionArrayCell::Text(_)
        | FunctionArrayCell::Logical(_)
        | FunctionArrayCell::EmptyCell => FunctionArrayCell::Logical(false),
    }
}

fn is_na_cell(cell: &FunctionArrayCell) -> FunctionArrayCell {
    match cell {
        FunctionArrayCell::Error(WorksheetErrorCode::NA) => FunctionArrayCell::Logical(true),
        FunctionArrayCell::Error(_)
        | FunctionArrayCell::Number(_)
        | FunctionArrayCell::Text(_)
        | FunctionArrayCell::Logical(_)
        | FunctionArrayCell::EmptyCell => FunctionArrayCell::Logical(false),
    }
}

fn eval_boolean_predicate_surface(
    meta: &FunctionMeta,
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    predicate: impl Fn(&PreparedValue) -> bool,
) -> Result<FunctionValue, InformationPredicateEvalError> {
    // Type predicates classify each element (Excel spills over an array
    // argument), so lift the per-cell classifier elementwise.
    run_values_only_prepared_lifted(
        args,
        resolver,
        |prepared| {
            if !meta.arity.accepts(prepared.len()) {
                return Err(arity_error(meta, prepared.len()));
            }
            Ok(FunctionValue::Logical(predicate(&prepared[0])))
        },
        map_information_predicate_error_to_ws,
        InformationPredicateEvalError::Preparation,
    )
}

fn coerce_isodd_number(arg: &PreparedValue) -> Result<f64, CoercionError> {
    match arg {
        PreparedValue::MissingArg | PreparedValue::EmptyCell => Ok(0.0),
        PreparedValue::Eval(FunctionValue::Logical(_)) => {
            Err(CoercionError::UnsupportedValueKind("logical"))
        }
        _ => coerce_prepared_to_number(arg),
    }
}

pub fn isodd_kernel(n: f64) -> bool {
    (n.trunc() as i64).rem_euclid(2) != 0
}

pub fn eval_isblank_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, InformationPredicateEvalError> {
    eval_boolean_predicate_surface(&ISBLANK_META, args, resolver, |arg| {
        matches!(arg, PreparedValue::EmptyCell)
    })
}

pub fn eval_iserr_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, InformationPredicateEvalError> {
    eval_boolean_predicate_surface(&ISERR_META, args, resolver, |arg| {
        matches!(
            arg,
            PreparedValue::Eval(FunctionValue::Error(code)) if *code != WorksheetErrorCode::NA
        )
    })
}

pub fn eval_iserror_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, InformationPredicateEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            if !ISERROR_META.arity.accepts(prepared.len()) {
                return Err(arity_error(&ISERROR_META, prepared.len()));
            }
            match &prepared[0] {
                PreparedValue::Eval(FunctionValue::Array(array)) => {
                    let cells = array.iter_row_major().map(is_error_cell).collect();
                    Ok(FunctionValue::Array(
                        FunctionArray::new(array.shape(), cells)
                            .expect("input array shape is valid"),
                    ))
                }
                _ => Ok(FunctionValue::Logical(matches!(
                    prepared[0],
                    PreparedValue::Eval(FunctionValue::Error(_))
                ))),
            }
        },
        InformationPredicateEvalError::Preparation,
    )
}

pub fn eval_islogical_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, InformationPredicateEvalError> {
    eval_boolean_predicate_surface(&ISLOGICAL_META, args, resolver, |arg| {
        matches!(arg, PreparedValue::Eval(FunctionValue::Logical(_)))
    })
}

pub fn eval_isna_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, InformationPredicateEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            if !ISNA_META.arity.accepts(prepared.len()) {
                return Err(arity_error(&ISNA_META, prepared.len()));
            }
            match &prepared[0] {
                PreparedValue::Eval(FunctionValue::Array(array)) => {
                    let cells = array.iter_row_major().map(is_na_cell).collect();
                    Ok(FunctionValue::Array(
                        FunctionArray::new(array.shape(), cells)
                            .expect("input array shape is valid"),
                    ))
                }
                _ => Ok(FunctionValue::Logical(matches!(
                    prepared[0],
                    PreparedValue::Eval(FunctionValue::Error(WorksheetErrorCode::NA))
                ))),
            }
        },
        InformationPredicateEvalError::Preparation,
    )
}

pub fn eval_isnontext_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, InformationPredicateEvalError> {
    eval_boolean_predicate_surface(&ISNONTEXT_META, args, resolver, |arg| {
        !matches!(arg, PreparedValue::Eval(FunctionValue::Text(_)))
    })
}

pub fn eval_istext_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, InformationPredicateEvalError> {
    eval_boolean_predicate_surface(&ISTEXT_META, args, resolver, |arg| {
        matches!(arg, PreparedValue::Eval(FunctionValue::Text(_)))
    })
}

pub fn eval_isodd_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, InformationPredicateEvalError> {
    run_values_only_prepared_lifted(
        args,
        resolver,
        |prepared| {
            if !ISODD_META.arity.accepts(prepared.len()) {
                return Err(arity_error(&ISODD_META, prepared.len()));
            }
            Ok(FunctionValue::Logical(isodd_kernel(
                coerce_isodd_number(&prepared[0])
                    .map_err(InformationPredicateEvalError::Preparation)?,
            )))
        },
        map_information_predicate_error_to_ws,
        InformationPredicateEvalError::Preparation,
    )
}

pub fn eval_isref_surface(
    args: &[FunctionArg],
    _resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, InformationPredicateEvalError> {
    if !ISREF_META.arity.accepts(args.len()) {
        return Err(arity_error(&ISREF_META, args.len()));
    }
    Ok(FunctionValue::Logical(matches!(
        &args[0],
        FunctionArg::Reference(_) | FunctionArg::Eval(FunctionValue::Reference(_))
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
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::{ExcelText, FunctionArray, FunctionArrayCell, ReferenceKind, ReferenceLike};

    struct MockResolver {
        resolved: Option<FunctionValue>,
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
            self.resolved.clone().ok_or(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
        }
    }

    fn txt(s: &str) -> ExcelText {
        ExcelText::from_utf16_code_units(s.encode_utf16().collect())
    }

    #[test]
    fn isblank_distinguishes_empty_cell_from_empty_string() {
        let blank_ref =
            FunctionArg::Reference(ReferenceLike::new(ReferenceKind::A1, "B1".to_string()));
        let blank_resolver = MockResolver {
            resolved: Some(FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![FunctionArrayCell::EmptyCell]]).unwrap(),
            )),
        };
        assert_eq!(
            eval_isblank_surface(&[blank_ref], &blank_resolver),
            Ok(FunctionValue::Logical(true))
        );
        assert_eq!(
            eval_isblank_surface(
                &[FunctionArg::Eval(FunctionValue::Text(txt("")))],
                &MockResolver { resolved: None },
            ),
            Ok(FunctionValue::Logical(false))
        );
    }

    #[test]
    fn error_predicates_follow_excel_split() {
        assert_eq!(
            eval_iserr_surface(
                &[FunctionArg::Eval(FunctionValue::Error(
                    WorksheetErrorCode::Div0
                ))],
                &MockResolver { resolved: None },
            ),
            Ok(FunctionValue::Logical(true))
        );
        assert_eq!(
            eval_iserr_surface(
                &[FunctionArg::Eval(FunctionValue::Error(
                    WorksheetErrorCode::NA
                ))],
                &MockResolver { resolved: None },
            ),
            Ok(FunctionValue::Logical(false))
        );
        assert_eq!(
            eval_iserror_surface(
                &[FunctionArg::Eval(FunctionValue::Error(
                    WorksheetErrorCode::NA
                ))],
                &MockResolver { resolved: None },
            ),
            Ok(FunctionValue::Logical(true))
        );
        assert_eq!(
            eval_isna_surface(
                &[FunctionArg::Eval(FunctionValue::Error(
                    WorksheetErrorCode::NA
                ))],
                &MockResolver { resolved: None },
            ),
            Ok(FunctionValue::Logical(true))
        );
    }

    #[test]
    fn iserror_array_lifts_elementwise() {
        let got = eval_iserror_surface(
            &[FunctionArg::Eval(FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Text(txt("Alice")),
                    FunctionArrayCell::Number(30.0),
                    FunctionArrayCell::Error(WorksheetErrorCode::NA),
                    FunctionArrayCell::EmptyCell,
                ]])
                .unwrap(),
            ))],
            &MockResolver { resolved: None },
        );
        assert_eq!(
            got,
            Ok(FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Logical(false),
                    FunctionArrayCell::Logical(false),
                    FunctionArrayCell::Logical(true),
                    FunctionArrayCell::Logical(false),
                ]])
                .unwrap()
            ))
        );
    }

    #[test]
    fn ftc_0941_and_ftc_0995_isna_array_lifts_xmatch_reduction_mask() {
        let got = eval_isna_surface(
            &[FunctionArg::Eval(FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Error(WorksheetErrorCode::NA),
                    FunctionArrayCell::Number(1.0),
                    FunctionArrayCell::Error(WorksheetErrorCode::NA),
                    FunctionArrayCell::Number(2.0),
                    FunctionArrayCell::Error(WorksheetErrorCode::NA),
                ]])
                .unwrap(),
            ))],
            &MockResolver { resolved: None },
        );
        assert_eq!(
            got,
            Ok(FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Logical(true),
                    FunctionArrayCell::Logical(false),
                    FunctionArrayCell::Logical(true),
                    FunctionArrayCell::Logical(false),
                    FunctionArrayCell::Logical(true),
                ]])
                .unwrap()
            ))
        );
    }

    #[test]
    fn type_predicates_match_text_and_logical_rules() {
        assert_eq!(
            eval_islogical_surface(
                &[FunctionArg::Eval(FunctionValue::Logical(true))],
                &MockResolver { resolved: None },
            ),
            Ok(FunctionValue::Logical(true))
        );
        assert_eq!(
            eval_istext_surface(
                &[FunctionArg::Eval(FunctionValue::Text(txt("x")))],
                &MockResolver { resolved: None },
            ),
            Ok(FunctionValue::Logical(true))
        );
        assert_eq!(
            eval_isnontext_surface(
                &[FunctionArg::Eval(FunctionValue::Number(1.0))],
                &MockResolver { resolved: None },
            ),
            Ok(FunctionValue::Logical(true))
        );
        assert_eq!(
            eval_isnontext_surface(
                &[FunctionArg::Eval(FunctionValue::Text(txt("x")))],
                &MockResolver { resolved: None },
            ),
            Ok(FunctionValue::Logical(false))
        );
    }

    #[test]
    fn isodd_matches_seed_coercion_lanes() {
        assert_eq!(
            eval_isodd_surface(
                &[FunctionArg::Eval(FunctionValue::Text(txt("3")))],
                &MockResolver { resolved: None },
            ),
            Ok(FunctionValue::Logical(true))
        );
        assert_eq!(
            eval_isodd_surface(
                &[FunctionArg::Reference(ReferenceLike::new(
                    ReferenceKind::A1,
                    "B1".to_string()
                ))],
                &MockResolver {
                    resolved: Some(FunctionValue::Array(
                        FunctionArray::from_rows(vec![vec![FunctionArrayCell::EmptyCell]]).unwrap(),
                    )),
                },
            ),
            Ok(FunctionValue::Logical(false))
        );
        assert!(matches!(
            eval_isodd_surface(
                &[FunctionArg::Eval(FunctionValue::Logical(true))],
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
                &[FunctionArg::Reference(ReferenceLike::new(
                    ReferenceKind::Area,
                    "A1:A2".to_string()
                ))],
                &MockResolver { resolved: None },
            ),
            Ok(FunctionValue::Logical(true))
        );
        assert_eq!(
            eval_isref_surface(
                &[FunctionArg::Eval(FunctionValue::Reference(
                    ReferenceLike::new(ReferenceKind::A1, "A1".to_string())
                ))],
                &MockResolver { resolved: None },
            ),
            Ok(FunctionValue::Logical(true))
        );
        assert_eq!(
            eval_isref_surface(
                &[FunctionArg::Eval(FunctionValue::Array(
                    FunctionArray::from_rows(vec![vec![FunctionArrayCell::Number(1.0)]]).unwrap(),
                ))],
                &MockResolver { resolved: None },
            ),
            Ok(FunctionValue::Logical(false))
        );
    }
}
