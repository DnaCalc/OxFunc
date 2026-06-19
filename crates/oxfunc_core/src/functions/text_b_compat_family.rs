use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{coerce_prepared_to_text, run_values_only_prepared};
use crate::functions::text_search_replace_family::{
    TextSearchReplaceEvalError, eval_find_surface, eval_replace_surface, eval_search_surface,
};
use crate::functions::text_slice_family::{
    TextSliceEvalError, eval_left_surface, eval_mid_surface, eval_right_surface,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

const TEXT_B_COMPAT_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.TEXT_B_COMPAT_BASE",
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
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
};

pub const FINDB_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.FINDB",
    arity: Arity { min: 2, max: 3 },
    ..TEXT_B_COMPAT_BASE_META
};
pub const LEFTB_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.LEFTB",
    arity: Arity { min: 1, max: 2 },
    ..TEXT_B_COMPAT_BASE_META
};
pub const LENB_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.LENB",
    ..TEXT_B_COMPAT_BASE_META
};
pub const MIDB_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.MIDB",
    arity: Arity::exact(3),
    ..TEXT_B_COMPAT_BASE_META
};
pub const REPLACEB_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.REPLACEB",
    arity: Arity::exact(4),
    ..TEXT_B_COMPAT_BASE_META
};
pub const RIGHTB_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.RIGHTB",
    arity: Arity { min: 1, max: 2 },
    ..TEXT_B_COMPAT_BASE_META
};
pub const SEARCHB_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SEARCHB",
    arity: Arity { min: 2, max: 3 },
    ..TEXT_B_COMPAT_BASE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum TextBCompatEvalError {
    Slice(TextSliceEvalError),
    Search(TextSearchReplaceEvalError),
}

pub fn eval_findb_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, TextBCompatEvalError> {
    eval_find_surface(args, resolver).map_err(TextBCompatEvalError::Search)
}

pub fn eval_leftb_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, TextBCompatEvalError> {
    eval_left_surface(args, resolver).map_err(TextBCompatEvalError::Slice)
}

pub fn eval_lenb_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, TextBCompatEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            if !LENB_META.arity.accepts(prepared.len()) {
                return Err(TextBCompatEvalError::Slice(
                    TextSliceEvalError::ArityMismatch {
                        expected_min: LENB_META.arity.min,
                        expected_max: LENB_META.arity.max,
                        actual: prepared.len(),
                    },
                ));
            }
            let text = coerce_prepared_to_text(&prepared[0])
                .map_err(|e| TextBCompatEvalError::Slice(TextSliceEvalError::Coercion(e)))?;
            Ok(CalcValue::number(text.len_utf16_code_units() as f64))
        },
        |e| TextBCompatEvalError::Slice(TextSliceEvalError::Coercion(e)),
    )
}

pub fn eval_midb_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, TextBCompatEvalError> {
    eval_mid_surface(args, resolver).map_err(TextBCompatEvalError::Slice)
}

pub fn eval_replaceb_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, TextBCompatEvalError> {
    eval_replace_surface(args, resolver).map_err(TextBCompatEvalError::Search)
}

pub fn eval_rightb_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, TextBCompatEvalError> {
    eval_right_surface(args, resolver).map_err(TextBCompatEvalError::Slice)
}

pub fn eval_searchb_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, TextBCompatEvalError> {
    eval_search_surface(args, resolver).map_err(TextBCompatEvalError::Search)
}

pub fn map_text_b_compat_error_to_ws(error: &TextBCompatEvalError) -> WorksheetErrorCode {
    match error {
        TextBCompatEvalError::Slice(TextSliceEvalError::ArityMismatch { .. })
        | TextBCompatEvalError::Search(TextSearchReplaceEvalError::ArityMismatch { .. }) => {
            WorksheetErrorCode::Value
        }
        TextBCompatEvalError::Slice(TextSliceEvalError::Coercion(
            CoercionError::WorksheetError(code),
        ))
        | TextBCompatEvalError::Search(TextSearchReplaceEvalError::Coercion(
            CoercionError::WorksheetError(code),
        )) => *code,
        TextBCompatEvalError::Slice(TextSliceEvalError::Coercion(_))
        | TextBCompatEvalError::Search(TextSearchReplaceEvalError::Coercion(_)) => {
            WorksheetErrorCode::Value
        }
        TextBCompatEvalError::Slice(TextSliceEvalError::Domain(code))
        | TextBCompatEvalError::Search(TextSearchReplaceEvalError::Domain(code)) => *code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::ExcelText;

    struct NoResolver;

    impl ReferenceSystemProvider for NoResolver {
        fn capabilities(&self) -> crate::resolver::ReferenceSystemCapabilities {
            crate::resolver::ReferenceSystemCapabilities::permissive_local()
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

    fn txt(s: &str) -> CalcValue {
        CalcValue::text(ExcelText::from_utf16_code_units(s.encode_utf16().collect()))
    }

    fn num(n: f64) -> CalcValue {
        CalcValue::number(n)
    }

    #[test]
    fn b_compat_metadata_shapes_match_delegate_family() {
        assert_eq!(LENB_META.function_id, "FUNC.LENB");
        assert_eq!(LEFTB_META.arity.max, 2);
        assert_eq!(MIDB_META.arity.min, 3);
        assert_eq!(
            SEARCHB_META.arg_preparation_profile,
            ArgPreparationProfile::ValuesOnlyPreAdapter
        );
    }

    #[test]
    fn b_compat_delegates_match_unicode_baseline() {
        let resolver = NoResolver;
        assert_eq!(
            eval_lenb_surface(&[txt("A😀B")], &resolver),
            Ok(CalcValue::number(4.0))
        );
        assert_eq!(
            eval_leftb_surface(&[txt("abcdef"), num(3.0)], &resolver),
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "abc".encode_utf16().collect(),
            )))
        );
        assert_eq!(
            eval_rightb_surface(&[txt("abcdef"), num(2.0)], &resolver),
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "ef".encode_utf16().collect(),
            )))
        );
        assert_eq!(
            eval_midb_surface(&[txt("abcdef"), num(2.0), num(3.0)], &resolver),
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "bcd".encode_utf16().collect(),
            )))
        );
        assert_eq!(
            eval_findb_surface(&[txt("cd"), txt("abcdef")], &resolver),
            Ok(CalcValue::number(3.0))
        );
        assert_eq!(
            eval_searchb_surface(&[txt("CD"), txt("abcdef")], &resolver),
            Ok(CalcValue::number(3.0))
        );
        assert_eq!(
            eval_replaceb_surface(&[txt("abcdef"), num(2.0), num(3.0), txt("ZZ")], &resolver),
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "aZZef".encode_utf16().collect(),
            )))
        );
    }

    #[test]
    fn b_compat_error_mapping_is_passthrough() {
        let resolver = NoResolver;
        assert_eq!(
            eval_leftb_surface(&[txt("abc"), num(-1.0)], &resolver),
            Err(TextBCompatEvalError::Slice(TextSliceEvalError::Domain(
                WorksheetErrorCode::Value,
            )))
        );
        assert_eq!(
            map_text_b_compat_error_to_ws(&TextBCompatEvalError::Search(
                TextSearchReplaceEvalError::Domain(WorksheetErrorCode::Value),
            )),
            WorksheetErrorCode::Value
        );
    }
}
