use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, run_values_only_prepared,
};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, ExcelText, WorksheetErrorCode};

pub const VALUETOTEXT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.VALUETOTEXT",
    arity: Arity { min: 1, max: 2 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ValueToTextEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    InvalidFormat(f64),
}

fn worksheet_error_literal(code: WorksheetErrorCode) -> &'static str {
    match code {
        WorksheetErrorCode::Null => "#NULL!",
        WorksheetErrorCode::Div0 => "#DIV/0!",
        WorksheetErrorCode::Value => "#VALUE!",
        WorksheetErrorCode::Ref => "#REF!",
        WorksheetErrorCode::Name => "#NAME?",
        WorksheetErrorCode::Num => "#NUM!",
        WorksheetErrorCode::NA => "#N/A",
        WorksheetErrorCode::Busy => "#BUSY!",
        WorksheetErrorCode::GettingData => "#GETTING_DATA",
        WorksheetErrorCode::Spill => "#SPILL!",
        WorksheetErrorCode::Calc => "#CALC!",
        WorksheetErrorCode::Field => "#FIELD!",
        WorksheetErrorCode::Blocked => "#BLOCKED!",
        WorksheetErrorCode::Connect => "#CONNECT!",
    }
}

fn parse_format_flag(prepared: Option<&PreparedArgValue>) -> Result<bool, ValueToTextEvalError> {
    match prepared {
        None | Some(PreparedArgValue::MissingArg) | Some(PreparedArgValue::EmptyCell) => Ok(false),
        Some(arg) => {
            let raw = coerce_prepared_to_number(arg).map_err(ValueToTextEvalError::Coercion)?;
            if !raw.is_finite() {
                return Err(ValueToTextEvalError::InvalidFormat(raw));
            }
            match raw.trunc() {
                0.0 => Ok(false),
                1.0 => Ok(true),
                other => Err(ValueToTextEvalError::InvalidFormat(other)),
            }
        }
    }
}

fn value_concise(value: &PreparedArgValue) -> String {
    match value {
        PreparedArgValue::Eval(EvalValue::Number(n)) => format!("{n}"),
        PreparedArgValue::Eval(EvalValue::Text(t)) => t.to_string_lossy(),
        PreparedArgValue::Eval(EvalValue::Logical(b)) => {
            if *b { "TRUE" } else { "FALSE" }.to_string()
        }
        PreparedArgValue::Eval(EvalValue::Error(code)) => {
            worksheet_error_literal(*code).to_string()
        }
        PreparedArgValue::EmptyCell | PreparedArgValue::MissingArg => String::new(),
        PreparedArgValue::Eval(EvalValue::Array(_)) => String::new(),
        PreparedArgValue::Eval(EvalValue::Reference(_)) => String::new(),
        PreparedArgValue::Eval(EvalValue::Lambda(_)) => String::new(),
    }
}

fn value_strict(value: &PreparedArgValue) -> String {
    match value {
        PreparedArgValue::Eval(EvalValue::Number(n)) => format!("{n}"),
        PreparedArgValue::Eval(EvalValue::Text(t)) => {
            let escaped = t.to_string_lossy().replace('"', "\"\"");
            format!("\"{escaped}\"")
        }
        PreparedArgValue::Eval(EvalValue::Logical(b)) => {
            if *b { "TRUE" } else { "FALSE" }.to_string()
        }
        PreparedArgValue::Eval(EvalValue::Error(code)) => {
            worksheet_error_literal(*code).to_string()
        }
        PreparedArgValue::EmptyCell | PreparedArgValue::MissingArg => String::new(),
        PreparedArgValue::Eval(EvalValue::Array(_)) => String::new(),
        PreparedArgValue::Eval(EvalValue::Reference(_)) => String::new(),
        PreparedArgValue::Eval(EvalValue::Lambda(_)) => String::new(),
    }
}

pub fn eval_valuetotext_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, ValueToTextEvalError> {
    if !VALUETOTEXT_META.arity.accepts(args.len()) {
        return Err(ValueToTextEvalError::ArityMismatch {
            expected_min: VALUETOTEXT_META.arity.min,
            expected_max: VALUETOTEXT_META.arity.max,
            actual: args.len(),
        });
    }

    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            let strict = parse_format_flag(prepared.get(1))?;
            let rendered = if strict {
                value_strict(&prepared[0])
            } else {
                value_concise(&prepared[0])
            };
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                rendered.encode_utf16().collect(),
            )))
        },
        ValueToTextEvalError::Coercion,
    )
}

pub fn map_valuetotext_error_to_ws(e: &ValueToTextEvalError) -> WorksheetErrorCode {
    match e {
        ValueToTextEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        ValueToTextEvalError::Coercion(_) => WorksheetErrorCode::Value,
        ValueToTextEvalError::InvalidFormat(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::function::{DeterminismClass, FecDependencyProfile, VolatilityClass};
    use crate::resolver::{CallerContext, RefResolutionError, ResolverCapabilities};
    use crate::value::ReferenceLike;

    struct MockResolver;
    impl ReferenceResolver for MockResolver {
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
        fn caller_context(&self) -> Option<CallerContext> {
            None
        }
    }

    fn num(v: f64) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Number(v))
    }

    fn text(s: &str) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment(s)))
    }

    fn text_val(s: &str) -> EvalValue {
        EvalValue::Text(ExcelText::from_interop_assignment(s))
    }

    // --- Meta tests ---

    #[test]
    fn valuetotext_meta_arity() {
        assert_eq!(VALUETOTEXT_META.arity.min, 1);
        assert_eq!(VALUETOTEXT_META.arity.max, 2);
    }

    #[test]
    fn valuetotext_meta_deterministic() {
        assert_eq!(
            VALUETOTEXT_META.determinism,
            DeterminismClass::Deterministic
        );
        assert_eq!(VALUETOTEXT_META.volatility, VolatilityClass::NonVolatile);
    }

    #[test]
    fn valuetotext_meta_fec_none() {
        assert_eq!(
            VALUETOTEXT_META.fec_dependency_profile,
            FecDependencyProfile::None
        );
    }

    // --- Arity tests ---

    #[test]
    fn valuetotext_rejects_zero_args() {
        let got = eval_valuetotext_surface(&[], &MockResolver);
        assert!(matches!(
            got,
            Err(ValueToTextEvalError::ArityMismatch { .. })
        ));
    }

    #[test]
    fn valuetotext_rejects_three_args() {
        let got = eval_valuetotext_surface(&[num(1.0), num(0.0), num(0.0)], &MockResolver);
        assert!(matches!(
            got,
            Err(ValueToTextEvalError::ArityMismatch { .. })
        ));
    }

    // --- Concise mode (default, format=0) ---

    #[test]
    fn valuetotext_number_concise() {
        let got = eval_valuetotext_surface(&[num(42.5)], &MockResolver);
        assert_eq!(got, Ok(text_val("42.5")));
    }

    #[test]
    fn valuetotext_text_concise() {
        let got = eval_valuetotext_surface(&[text("hello")], &MockResolver);
        assert_eq!(got, Ok(text_val("hello")));
    }

    #[test]
    fn valuetotext_logical_true_concise() {
        let got = eval_valuetotext_surface(
            &[CallArgValue::Eval(EvalValue::Logical(true))],
            &MockResolver,
        );
        assert_eq!(got, Ok(text_val("TRUE")));
    }

    #[test]
    fn valuetotext_logical_false_concise() {
        let got = eval_valuetotext_surface(
            &[CallArgValue::Eval(EvalValue::Logical(false))],
            &MockResolver,
        );
        assert_eq!(got, Ok(text_val("FALSE")));
    }

    #[test]
    fn valuetotext_error_concise() {
        let got = eval_valuetotext_surface(
            &[CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::NA))],
            &MockResolver,
        );
        assert_eq!(got, Ok(text_val("#N/A")));
    }

    #[test]
    fn valuetotext_number_concise_explicit_zero() {
        let got = eval_valuetotext_surface(&[num(3.14), num(0.0)], &MockResolver);
        assert_eq!(got, Ok(text_val("3.14")));
    }

    // --- Strict mode (format=1) ---

    #[test]
    fn valuetotext_number_strict() {
        let got = eval_valuetotext_surface(&[num(42.5), num(1.0)], &MockResolver);
        assert_eq!(got, Ok(text_val("42.5")));
    }

    #[test]
    fn valuetotext_text_strict_quoted() {
        let got = eval_valuetotext_surface(&[text("hello"), num(1.0)], &MockResolver);
        assert_eq!(got, Ok(text_val("\"hello\"")));
    }

    #[test]
    fn valuetotext_text_strict_embedded_quotes() {
        let got = eval_valuetotext_surface(&[text("say \"hi\""), num(1.0)], &MockResolver);
        assert_eq!(got, Ok(text_val("\"say \"\"hi\"\"\"")));
    }

    #[test]
    fn valuetotext_logical_strict() {
        let got = eval_valuetotext_surface(
            &[CallArgValue::Eval(EvalValue::Logical(true)), num(1.0)],
            &MockResolver,
        );
        assert_eq!(got, Ok(text_val("TRUE")));
    }

    #[test]
    fn valuetotext_error_strict() {
        let got = eval_valuetotext_surface(
            &[
                CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::Value)),
                num(1.0),
            ],
            &MockResolver,
        );
        assert_eq!(got, Ok(text_val("#VALUE!")));
    }

    // --- Format flag validation ---

    #[test]
    fn valuetotext_invalid_format_flag() {
        let got = eval_valuetotext_surface(&[num(1.0), num(2.0)], &MockResolver);
        assert!(matches!(got, Err(ValueToTextEvalError::InvalidFormat(_))));
    }

    // --- Error mapping ---

    #[test]
    fn valuetotext_error_mapping() {
        assert_eq!(
            map_valuetotext_error_to_ws(&ValueToTextEvalError::InvalidFormat(2.0)),
            WorksheetErrorCode::Value
        );
    }
}
