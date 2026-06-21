use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{coerce_prepared_to_number, run_values_only_prepared};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::{CalcArray, CoreValue, ExcelText, WorksheetErrorCode};

pub const VALUETOTEXT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.VALUETOTEXT",
    arity: Arity { min: 1, max: 2 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
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

fn parse_format_flag(prepared: Option<&CalcValue>) -> Result<bool, ValueToTextEvalError> {
    match prepared {
        None => Ok(false),
        Some(arg) if matches!(arg.core(), CoreValue::Missing | CoreValue::Empty) => Ok(false),
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

fn cell_concise(cell: &CalcValue) -> String {
    match cell.core() {
        CoreValue::Number(n) => format!("{n}"),
        CoreValue::Text(t) => t.to_string_lossy(),
        CoreValue::Logical(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
        CoreValue::Error(code) => worksheet_error_literal(*code).to_string(),
        CoreValue::Empty | CoreValue::Missing => String::new(),
        CoreValue::Array(_) | CoreValue::Reference(_) => String::new(),
    }
}

fn cell_strict(cell: &CalcValue) -> String {
    match cell.core() {
        CoreValue::Number(n) => format!("{n}"),
        CoreValue::Text(t) => {
            let escaped = t.to_string_lossy().replace('"', "\"\"");
            format!("\"{escaped}\"")
        }
        CoreValue::Logical(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
        CoreValue::Error(code) => worksheet_error_literal(*code).to_string(),
        CoreValue::Empty | CoreValue::Missing => String::new(),
        CoreValue::Array(_) | CoreValue::Reference(_) => String::new(),
    }
}

fn value_concise(value: &CalcValue) -> String {
    match value.core() {
        CoreValue::Number(n) => format!("{n}"),
        CoreValue::Text(t) => t.to_string_lossy(),
        CoreValue::Logical(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
        CoreValue::Error(code) => worksheet_error_literal(*code).to_string(),
        CoreValue::Empty | CoreValue::Missing => String::new(),
        CoreValue::Array(_) | CoreValue::Reference(_) => String::new(),
    }
}

fn value_strict(value: &CalcValue) -> String {
    match value.core() {
        CoreValue::Number(n) => format!("{n}"),
        CoreValue::Text(t) => {
            let escaped = t.to_string_lossy().replace('"', "\"\"");
            format!("\"{escaped}\"")
        }
        CoreValue::Logical(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
        CoreValue::Error(code) => worksheet_error_literal(*code).to_string(),
        CoreValue::Empty | CoreValue::Missing => String::new(),
        CoreValue::Array(_) | CoreValue::Reference(_) => String::new(),
    }
}

fn render_array_value(array: &CalcArray, strict: bool) -> CalcValue {
    let cells = array
        .iter_row_major()
        .map(|cell| {
            let rendered = if strict {
                cell_strict(cell)
            } else {
                cell_concise(cell)
            };
            CalcValue::text(ExcelText::from_utf16_code_units(
                rendered.encode_utf16().collect(),
            ))
        })
        .collect::<Vec<_>>();
    CalcValue::array(CalcArray::new(array.shape(), cells).expect("shape preserved"))
}

pub fn eval_valuetotext_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, ValueToTextEvalError> {
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
            if let CoreValue::Array(array) = prepared[0].core() {
                return Ok(render_array_value(array, strict));
            }
            let rendered = if strict {
                value_strict(&prepared[0])
            } else {
                value_concise(&prepared[0])
            };
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
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
    use crate::resolver::{CallerContext, ReferenceSystemCapabilities};

    struct MockResolver;
    impl ReferenceSystemProvider for MockResolver {
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
        fn caller_context(&self) -> Option<CallerContext> {
            None
        }
    }

    fn num(v: f64) -> CalcValue {
        CalcValue::number(v)
    }

    fn text(s: &str) -> CalcValue {
        CalcValue::text(ExcelText::from_interop_assignment(s))
    }

    fn text_val(s: &str) -> CalcValue {
        CalcValue::text(ExcelText::from_interop_assignment(s))
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
        let got = eval_valuetotext_surface(&[(CalcValue::logical(true))], &MockResolver);
        assert_eq!(got, Ok(text_val("TRUE")));
    }

    #[test]
    fn valuetotext_logical_false_concise() {
        let got = eval_valuetotext_surface(&[(CalcValue::logical(false))], &MockResolver);
        assert_eq!(got, Ok(text_val("FALSE")));
    }

    #[test]
    fn valuetotext_error_concise() {
        let got =
            eval_valuetotext_surface(&[(CalcValue::error(WorksheetErrorCode::NA))], &MockResolver);
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
        let got = eval_valuetotext_surface(&[(CalcValue::logical(true)), num(1.0)], &MockResolver);
        assert_eq!(got, Ok(text_val("TRUE")));
    }

    #[test]
    fn valuetotext_error_strict() {
        let got = eval_valuetotext_surface(
            &[(CalcValue::error(WorksheetErrorCode::Value)), num(1.0)],
            &MockResolver,
        );
        assert_eq!(got, Ok(text_val("#VALUE!")));
    }

    #[test]
    fn valuetotext_array_strict_returns_quoted_text_array() {
        let got = eval_valuetotext_surface(
            &[
                (CalcValue::array(
                    CalcArray::from_rows(vec![
                        vec![
                            CalcValue::text(ExcelText::from_interop_assignment("a")),
                            CalcValue::text(ExcelText::from_interop_assignment("b")),
                        ],
                        vec![
                            CalcValue::text(ExcelText::from_interop_assignment("c")),
                            CalcValue::text(ExcelText::from_interop_assignment("d")),
                        ],
                    ])
                    .unwrap(),
                )),
                num(1.0),
            ],
            &MockResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![
                        CalcValue::text(ExcelText::from_interop_assignment("\"a\"")),
                        CalcValue::text(ExcelText::from_interop_assignment("\"b\"")),
                    ],
                    vec![
                        CalcValue::text(ExcelText::from_interop_assignment("\"c\"")),
                        CalcValue::text(ExcelText::from_interop_assignment("\"d\"")),
                    ],
                ])
                .unwrap()
            ))
        );
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
