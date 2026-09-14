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

pub(crate) fn coercion_error_from_resolution(e: ReferenceResolutionError) -> CoercionError {
    match &e {
        ReferenceResolutionError::UnresolvedName { .. } => {
            CoercionError::WorksheetError(WorksheetErrorCode::Name)
        }
        _ => CoercionError::RefResolution(e),
    }
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
    if value.callable_value().is_some() {
        return Err(CoercionError::UnsupportedValueKind("callable"));
    }

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
                resolve_eval_value(resolver, reference).map_err(coercion_error_from_resolution)?;
            coerce_eval_to_number(&resolved, resolver)
        }
        _ => Err(CoercionError::UnsupportedValueKind("unsupported_value")),
    }
}

pub fn coerce_calc_scalar_to_number(value: &CalcValue) -> Result<f64, CoercionError> {
    if value.callable_value().is_some() {
        return Err(CoercionError::UnsupportedValueKind("callable"));
    }

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

/// SCALAR-context numeric coercion of one prepared value: Excel reads a referenced BLANK cell
/// as the number `0` wherever exactly one number is expected — the arithmetic operators
/// (`=A1+1` is `1`, `=-A1` is `0`), and the single-value numeric functions (`=ABS(A1)`,
/// `=ROUND(A1,2)` are `0`). Pinned end to end by the truth table in
/// `functions::blank_cell_coercion_truth_table` (provenance recorded in its module header).
///
/// This is deliberately a SEPARATE helper from [`coerce_calc_scalar_to_number`] rather than a
/// change to it. The aggregate lane IGNORES a blank — `=MAX(A1,-3)` is `-3`, `=AVERAGE(A1)`
/// is `#DIV/0!`, `=COUNT(A1)` is `0` — and it must never see the scalar-zero rule: the
/// production aggregate policies in `functions::aggregate_common` match `CoreValue::Empty`
/// on each item before any coercion, and [`aggregate_scan_sum`] skips the distinct
/// [`CoercionError::EmptyCell`] signal the low-level helpers keep raising. The scalar-zero
/// rule and the aggregate-ignore rule are two different Excel behaviours and live on two
/// different helpers; a consumer picks the one for its context. A MISSING (omitted) argument
/// is still [`CoercionError::MissingArg`] here, so optional-argument defaulting stays the
/// caller's decision. W110-1 (oxf-xvt5.1).
pub fn coerce_scalar_calc_value_to_number(value: &CalcValue) -> Result<f64, CoercionError> {
    match value.core() {
        CoreValue::Empty => Ok(0.0),
        _ => coerce_calc_scalar_to_number(value),
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
                resolve_eval_value(resolver, reference).map_err(coercion_error_from_resolution)?;
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

    /// The scalar-context helper reads a blank as 0 but leaves every other outcome of the
    /// low-level helper untouched — including the MissingArg signal an omitted argument
    /// carries and the numeric-text / logical / error coercions.
    #[test]
    fn scalar_context_reads_blank_as_zero_and_delegates_everything_else() {
        assert_eq!(
            coerce_scalar_calc_value_to_number(&CalcValue::empty()),
            Ok(0.0)
        );
        assert_eq!(
            coerce_scalar_calc_value_to_number(&CalcValue::missing()),
            Err(CoercionError::MissingArg)
        );
        for value in [
            CalcValue::number(7.0),
            CalcValue::logical(true),
            CalcValue::text(ExcelText::from_utf16_code_units(
                "2.5".encode_utf16().collect(),
            )),
            CalcValue::text(ExcelText::from_utf16_code_units(
                "asd".encode_utf16().collect(),
            )),
            CalcValue::error(WorksheetErrorCode::NA),
            CalcValue::reference(ReferenceLike::new(ReferenceKind::A1, "A1")),
        ] {
            assert_eq!(
                coerce_scalar_calc_value_to_number(&value),
                coerce_calc_scalar_to_number(&value),
                "scalar-context helper diverged from the low-level helper on {value:?}"
            );
        }
    }

    /// The aggregate-facing helpers keep their distinct blank signal: the scalar-zero rule
    /// must not leak into the helpers the ignore-blanks policies read.
    #[test]
    fn scalar_zero_rule_does_not_leak_into_the_aggregate_facing_helpers() {
        assert_eq!(
            coerce_calc_scalar_to_number(&CalcValue::empty()),
            Err(CoercionError::EmptyCell)
        );
        assert_eq!(
            coerce_arg_to_number(&CalcValue::empty(), &resolver()),
            Err(CoercionError::EmptyCell)
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
    fn coercion_error_from_resolution_maps_unresolved_name_to_worksheet_name_error() {
        let got = coercion_error_from_resolution(ReferenceResolutionError::UnresolvedName {
            target: "MyName".to_string(),
        });
        assert_eq!(got, CoercionError::WorksheetError(WorksheetErrorCode::Name));
    }

    #[test]
    fn coercion_error_from_resolution_preserves_prior_behavior_for_other_variants() {
        let got = coercion_error_from_resolution(ReferenceResolutionError::UnresolvedReference {
            target: "A1".to_string(),
        });
        assert_eq!(
            got,
            CoercionError::RefResolution(ReferenceResolutionError::UnresolvedReference {
                target: "A1".to_string()
            })
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
