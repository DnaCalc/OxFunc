use crate::resolver::{ReferenceResolutionError, ReferenceSystemProvider, resolve_eval_value};
use crate::value::{CalcValue, CoreValue, WorksheetErrorCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggregateScanPolicy {
    StrictAllNumeric,
    IgnoreTextAndEmpty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoercionError {
    MissingArg,
    EmptyCell,
    NonNumericText(String),
    UnsupportedValueKind(&'static str),
    WorksheetError(WorksheetErrorCode),
    RefResolution(ReferenceResolutionError),
}

fn parse_excel_number(text: &str) -> Option<f64> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    let parsed = trimmed.parse::<f64>().ok()?;
    if !parsed.is_finite() {
        return None;
    }
    Some(parsed)
}

pub fn coerce_eval_to_number(
    value: &CalcValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<f64, CoercionError> {
    match value.core() {
        CoreValue::Number(n) => Ok(*n),
        CoreValue::Logical(b) => Ok(if *b { 1.0 } else { 0.0 }),
        CoreValue::Text(t) => {
            let raw = t.to_string_lossy();
            parse_excel_number(&raw).ok_or(CoercionError::NonNumericText(raw))
        }
        CoreValue::Error(code) => Err(CoercionError::WorksheetError(*code)),
        CoreValue::Array(_) => Err(CoercionError::UnsupportedValueKind("array")),
        CoreValue::Reference(reference) => {
            let resolved =
                resolve_eval_value(resolver, reference).map_err(CoercionError::RefResolution)?;
            coerce_eval_to_number(&resolved, resolver)
        }
        _ => Err(CoercionError::UnsupportedValueKind("unsupported_value")),
    }
}

pub fn coerce_calc_scalar_to_number(value: &CalcValue) -> Result<f64, CoercionError> {
    match value.core() {
        CoreValue::Number(n) => Ok(*n),
        CoreValue::Logical(b) => Ok(if *b { 1.0 } else { 0.0 }),
        CoreValue::Text(t) => {
            let raw = t.to_string_lossy();
            parse_excel_number(&raw).ok_or(CoercionError::NonNumericText(raw))
        }
        CoreValue::Error(code) => Err(CoercionError::WorksheetError(*code)),
        CoreValue::Empty => Err(CoercionError::EmptyCell),
        CoreValue::Missing => Err(CoercionError::MissingArg),
        CoreValue::Array(_) => Err(CoercionError::UnsupportedValueKind("array")),
        CoreValue::Reference(_) => Err(CoercionError::UnsupportedValueKind("reference")),
    }
}

pub fn coerce_arg_to_number(
    arg: &CalcValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<f64, CoercionError> {
    match arg.core() {
        CoreValue::Missing => Err(CoercionError::MissingArg),
        CoreValue::Empty => Err(CoercionError::EmptyCell),
        CoreValue::Reference(reference) => {
            let resolved =
                resolve_eval_value(resolver, reference).map_err(CoercionError::RefResolution)?;
            coerce_eval_to_number(&resolved, resolver)
        }
        _ => coerce_eval_to_number(arg, resolver),
    }
}

pub fn coerce_direct_args_to_numbers(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<Vec<f64>, CoercionError> {
    args.iter()
        .map(|arg| coerce_arg_to_number(arg, resolver))
        .collect()
}

pub fn aggregate_scan_sum(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    policy: AggregateScanPolicy,
) -> Result<f64, CoercionError> {
    let mut acc = 0.0;
    for arg in args {
        match coerce_arg_to_number(arg, resolver) {
            Ok(n) => acc += n,
            Err(CoercionError::MissingArg | CoercionError::EmptyCell)
                if policy == AggregateScanPolicy::IgnoreTextAndEmpty => {}
            Err(CoercionError::NonNumericText(_))
                if policy == AggregateScanPolicy::IgnoreTextAndEmpty => {}
            Err(err) => return Err(err),
        }
    }
    Ok(acc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{ReferenceSystemCapabilities, ReferenceSystemProvider};
    use crate::value::{CalcValue, ExcelText, ReferenceKind, ReferenceLike};

    struct MockResolver {
        caps: ReferenceSystemCapabilities,
        resolved_value: Option<CalcValue>,
    }

    impl ReferenceSystemProvider for MockResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            self.caps
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<CalcValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            self.resolved_value.clone().ok_or(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
        }
    }

    fn resolver() -> MockResolver {
        MockResolver {
            caps: ReferenceSystemCapabilities::permissive_local(),
            resolved_value: None,
        }
    }

    #[test]
    fn coerce_text_numeric_to_number() {
        let value = CalcValue::text(ExcelText::from_utf16_code_units(
            "1".encode_utf16().collect(),
        ));
        let got = coerce_eval_to_number(&value, &resolver());
        assert_eq!(got, Ok(1.0));
    }

    #[test]
    fn coerce_text_non_numeric_fails() {
        let value = CalcValue::text(ExcelText::from_utf16_code_units(
            "asd".encode_utf16().collect(),
        ));
        let got = coerce_eval_to_number(&value, &resolver());
        assert_eq!(got, Err(CoercionError::NonNumericText("asd".to_string())));
    }

    #[test]
    fn coerce_calc_scalar_to_number_uses_calcvalue_core() {
        assert_eq!(
            coerce_calc_scalar_to_number(&CalcValue::number(7.0)),
            Ok(7.0)
        );
        assert_eq!(
            coerce_calc_scalar_to_number(&CalcValue::logical(true)),
            Ok(1.0)
        );
        assert_eq!(
            coerce_calc_scalar_to_number(&CalcValue::text(ExcelText::from_utf16_code_units(
                "2.5".encode_utf16().collect()
            ))),
            Ok(2.5)
        );
        assert_eq!(
            coerce_calc_scalar_to_number(&CalcValue::missing()),
            Err(CoercionError::MissingArg)
        );
        assert_eq!(
            coerce_calc_scalar_to_number(&CalcValue::empty()),
            Err(CoercionError::EmptyCell)
        );
    }

    #[test]
    fn coerce_calc_scalar_to_number_rejects_unresolved_shapes_without_legacy_construction() {
        assert_eq!(
            coerce_calc_scalar_to_number(&CalcValue::reference(ReferenceLike::new(
                ReferenceKind::A1,
                "A1"
            ))),
            Err(CoercionError::UnsupportedValueKind("reference"))
        );
    }

    #[test]
    fn missing_arg_is_distinct_error() {
        let got = coerce_arg_to_number(&CalcValue::missing(), &resolver());
        assert_eq!(got, Err(CoercionError::MissingArg));
    }

    #[test]
    fn empty_cell_is_distinct_error() {
        let got = coerce_arg_to_number(&CalcValue::empty(), &resolver());
        assert_eq!(got, Err(CoercionError::EmptyCell));
    }

    #[test]
    fn reference_is_dereferenced_via_resolver() {
        let r = MockResolver {
            caps: ReferenceSystemCapabilities::permissive_local(),
            resolved_value: Some(CalcValue::number(2.5)),
        };
        let arg = CalcValue::reference(ReferenceLike::new(ReferenceKind::A1, "A1".to_string()));

        let got = coerce_arg_to_number(&arg, &r);
        assert_eq!(got, Ok(2.5));
    }

    #[test]
    fn unresolved_reference_propagates_resolution_error() {
        let arg = CalcValue::reference(ReferenceLike::new(ReferenceKind::A1, "A1".to_string()));

        let got = coerce_arg_to_number(&arg, &resolver());
        assert_eq!(
            got,
            Err(CoercionError::RefResolution(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: "A1".to_string()
                }
            ))
        );
    }

    #[test]
    fn aggregate_policy_contrast_direct_vs_scan() {
        let args = vec![
            (CalcValue::number(1.0)),
            (CalcValue::text(ExcelText::from_utf16_code_units(
                "asd".encode_utf16().collect(),
            ))),
            (CalcValue::number(2.0)),
        ];

        let strict = aggregate_scan_sum(&args, &resolver(), AggregateScanPolicy::StrictAllNumeric);
        assert_eq!(
            strict,
            Err(CoercionError::NonNumericText("asd".to_string()))
        );

        let relaxed =
            aggregate_scan_sum(&args, &resolver(), AggregateScanPolicy::IgnoreTextAndEmpty);
        assert_eq!(relaxed, Ok(3.0));
    }
}
