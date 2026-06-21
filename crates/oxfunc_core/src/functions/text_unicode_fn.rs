use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    coerce_prepared_to_number, coerce_prepared_to_text, run_values_only_prepared_lifted,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::{ExcelText, WorksheetErrorCode};

pub const UNICHAR_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.UNICHAR",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    lift_broadcast_profile: FunctionMeta::DEFAULT_LIFT_BROADCAST_PROFILE,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
};

pub const UNICODE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.UNICODE",
    kernel_signature_class: KernelSignatureClass::Custom,
    ..UNICHAR_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum TextUnicodeEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

fn truncate_toward_zero(n: f64) -> f64 {
    n.trunc()
}

fn encode_unicode_scalar(scalar: char) -> ExcelText {
    ExcelText::from_utf16_code_units(scalar.to_string().encode_utf16().collect())
}

pub fn unichar_kernel(n: f64) -> Result<ExcelText, WorksheetErrorCode> {
    let n = truncate_toward_zero(n);
    if !n.is_finite() || !(1.0..=0x10FFFF as f64).contains(&n) {
        return Err(WorksheetErrorCode::Value);
    }

    let codepoint = n as u32;
    if (0xD800..=0xDFFF).contains(&codepoint) {
        return Err(WorksheetErrorCode::NA);
    }

    let scalar = char::from_u32(codepoint).ok_or(WorksheetErrorCode::Value)?;
    Ok(encode_unicode_scalar(scalar))
}

pub fn unicode_kernel(text: &ExcelText) -> Result<f64, WorksheetErrorCode> {
    let mut decoded = std::char::decode_utf16(text.utf16_code_units().iter().copied());
    match decoded.next() {
        Some(Ok(scalar)) => Ok(scalar as u32 as f64),
        Some(Err(_)) | None => Err(WorksheetErrorCode::Value),
    }
}

pub fn eval_unichar_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, TextUnicodeEvalError> {
    run_values_only_prepared_lifted(
        args,
        resolver,
        |prepared| {
            if !UNICHAR_META.arity.accepts(prepared.len()) {
                return Err(TextUnicodeEvalError::ArityMismatch {
                    expected_min: UNICHAR_META.arity.min,
                    expected_max: UNICHAR_META.arity.max,
                    actual: prepared.len(),
                });
            }
            let number =
                coerce_prepared_to_number(&prepared[0]).map_err(TextUnicodeEvalError::Coercion)?;
            unichar_kernel(number)
                .map(CalcValue::text)
                .map_err(TextUnicodeEvalError::Domain)
        },
        map_text_unicode_error_to_ws,
        TextUnicodeEvalError::Coercion,
    )
}

pub fn eval_unicode_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, TextUnicodeEvalError> {
    run_values_only_prepared_lifted(
        args,
        resolver,
        |prepared| {
            if !UNICODE_META.arity.accepts(prepared.len()) {
                return Err(TextUnicodeEvalError::ArityMismatch {
                    expected_min: UNICODE_META.arity.min,
                    expected_max: UNICODE_META.arity.max,
                    actual: prepared.len(),
                });
            }
            let text =
                coerce_prepared_to_text(&prepared[0]).map_err(TextUnicodeEvalError::Coercion)?;
            unicode_kernel(&text)
                .map(CalcValue::number)
                .map_err(TextUnicodeEvalError::Domain)
        },
        map_text_unicode_error_to_ws,
        TextUnicodeEvalError::Coercion,
    )
}

pub fn map_text_unicode_error_to_ws(e: &TextUnicodeEvalError) -> WorksheetErrorCode {
    match e {
        TextUnicodeEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        TextUnicodeEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        TextUnicodeEvalError::Coercion(_) => WorksheetErrorCode::Value,
        TextUnicodeEvalError::Domain(code) => *code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;

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

    fn text_value(units: Vec<u16>) -> CalcValue {
        CalcValue::text(ExcelText::from_utf16_code_units(units))
    }

    fn number_value(n: f64) -> CalcValue {
        CalcValue::number(n)
    }

    fn txt(s: &str) -> ExcelText {
        ExcelText::from_utf16_code_units(s.encode_utf16().collect())
    }

    #[test]
    fn unichar_kernel_matches_probe_rows() {
        assert_eq!(unichar_kernel(65.9), Ok(txt("A")));
        assert_eq!(
            unichar_kernel(128512.0).unwrap().utf16_code_units(),
            &[0xD83D, 0xDE00]
        );
        assert_eq!(unichar_kernel(0.0), Err(WorksheetErrorCode::Value));
        assert_eq!(unichar_kernel(1114112.0), Err(WorksheetErrorCode::Value));
    }

    #[test]
    fn unichar_kernel_rejects_non_finite_and_surrogate_domain() {
        assert_eq!(
            unichar_kernel(f64::INFINITY),
            Err(WorksheetErrorCode::Value)
        );
        assert_eq!(
            unichar_kernel(f64::NEG_INFINITY),
            Err(WorksheetErrorCode::Value)
        );
        assert_eq!(unichar_kernel(f64::NAN), Err(WorksheetErrorCode::Value));
        assert_eq!(unichar_kernel(55296.0), Err(WorksheetErrorCode::NA));
        assert_eq!(unichar_kernel(57343.9), Err(WorksheetErrorCode::NA));
    }

    #[test]
    fn unicode_kernel_matches_probe_rows() {
        assert_eq!(unicode_kernel(&txt("😀")), Ok(128512.0));
        assert_eq!(unicode_kernel(&txt("")), Err(WorksheetErrorCode::Value));
        assert_eq!(
            unicode_kernel(&ExcelText::from_utf16_code_units(vec![0xD83D])),
            Err(WorksheetErrorCode::Value)
        );
        assert_eq!(
            unicode_kernel(&ExcelText::from_utf16_code_units(vec![0xDE00])),
            Err(WorksheetErrorCode::Value)
        );
    }

    #[test]
    fn unicode_kernel_uses_first_scalar_not_first_utf16_unit() {
        assert_eq!(unicode_kernel(&txt("e\u{0301}")), Ok(101.0));
        assert_eq!(unicode_kernel(&txt("\u{00E9}")), Ok(233.0));
        assert_eq!(
            unicode_kernel(&ExcelText::from_utf16_code_units(vec![
                0xD83D, 0xDE00, 0x0041
            ])),
            Ok(128512.0)
        );
    }

    #[test]
    fn eval_unichar_surface_accepts_numeric_text_and_logicals() {
        assert_eq!(
            eval_unichar_surface(&[text_value("65".encode_utf16().collect())], &NoResolver),
            Ok(CalcValue::text(txt("A")))
        );
        assert_eq!(
            eval_unichar_surface(&[(CalcValue::logical(true))], &NoResolver),
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(vec![1])))
        );
    }

    #[test]
    fn eval_unichar_surface_maps_domain_and_arity_errors() {
        assert_eq!(
            eval_unichar_surface(&[], &NoResolver),
            Err(TextUnicodeEvalError::ArityMismatch {
                expected_min: 1,
                expected_max: 1,
                actual: 0,
            })
        );
        assert_eq!(
            eval_unichar_surface(&[number_value(55296.0)], &NoResolver),
            Err(TextUnicodeEvalError::Domain(WorksheetErrorCode::NA))
        );
        assert_eq!(
            map_text_unicode_error_to_ws(&TextUnicodeEvalError::Domain(WorksheetErrorCode::NA)),
            WorksheetErrorCode::NA
        );
    }

    #[test]
    fn eval_unicode_surface_textifies_numbers_logicals_and_blanks() {
        assert_eq!(
            eval_unicode_surface(&[number_value(65.0)], &NoResolver),
            Ok(CalcValue::number(54.0))
        );
        assert_eq!(
            eval_unicode_surface(&[(CalcValue::logical(true))], &NoResolver),
            Ok(CalcValue::number(84.0))
        );
        assert_eq!(
            eval_unicode_surface(&[CalcValue::empty()], &NoResolver),
            Err(TextUnicodeEvalError::Domain(WorksheetErrorCode::Value))
        );
    }

    #[test]
    fn eval_unicode_surface_rejects_invalid_leading_surrogate_shapes() {
        assert_eq!(
            eval_unicode_surface(
                &[
                    text_value(vec![0xD83D]),
                    text_value("x".encode_utf16().collect())
                ],
                &NoResolver,
            ),
            Err(TextUnicodeEvalError::ArityMismatch {
                expected_min: 1,
                expected_max: 1,
                actual: 2,
            })
        );
        assert_eq!(
            eval_unicode_surface(&[text_value(vec![0xD83D])], &NoResolver),
            Err(TextUnicodeEvalError::Domain(WorksheetErrorCode::Value))
        );
        assert_eq!(
            eval_unicode_surface(&[text_value(vec![0xDE00])], &NoResolver),
            Err(TextUnicodeEvalError::Domain(WorksheetErrorCode::Value))
        );
        assert_eq!(
            eval_unicode_surface(&[text_value(vec![0xD83D, 0x0041])], &NoResolver),
            Err(TextUnicodeEvalError::Domain(WorksheetErrorCode::Value))
        );
    }

    #[test]
    fn eval_unicode_surface_preserves_w7_seed_rows() {
        assert_eq!(
            eval_unicode_surface(&[text_value(vec![0xD83D, 0xDE00])], &NoResolver),
            Ok(CalcValue::number(128512.0))
        );
        assert_eq!(
            eval_unicode_surface(
                &[text_value("e\u{0301}".encode_utf16().collect())],
                &NoResolver
            ),
            Ok(CalcValue::number(101.0))
        );
        assert_eq!(
            eval_unicode_surface(
                &[text_value("\u{00E9}".encode_utf16().collect())],
                &NoResolver
            ),
            Ok(CalcValue::number(233.0))
        );
    }
}
