use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    coerce_prepared_to_number, coerce_prepared_to_text, prepare_arg_values_only,
};
use crate::host_info::{HostInfoError, HostInfoProvider, SheetCountSpec, SheetIdentitySpec};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{CalcValue, CoreValue, ExcelText, ReferenceLike, WorksheetErrorCode};

pub const ADDRESS_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ADDRESS",
    arity: Arity { min: 2, max: 5 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::None,
};

pub const AREAS_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.AREAS",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::RefOnly,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

pub const FORMULATEXT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.FORMULATEXT",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::WorkbookState,
    thread_safety: ThreadSafetyClass::HostSerialized,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::RefOnly,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

pub const SHEET_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SHEET",
    arity: Arity { min: 0, max: 1 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::WorkbookState,
    thread_safety: ThreadSafetyClass::HostSerialized,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::Composite,
    surface_fec_dependency_profile: FecDependencyProfile::Composite,
};

pub const SHEETS_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SHEETS",
    arity: Arity { min: 0, max: 1 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::WorkbookState,
    thread_safety: ThreadSafetyClass::HostSerialized,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::RefOnly,
    surface_fec_dependency_profile: FecDependencyProfile::Composite,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ReferenceMetadataEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    InvalidReferenceArg,
    InvalidAddressCoordinate,
    InvalidAbsNum,
    HostInfoProviderMissing(&'static str),
    HostInfo(HostInfoError),
}

fn parse_reference_arg(arg: &CalcValue) -> Result<ReferenceLike, ReferenceMetadataEvalError> {
    arg.as_reference()
        .cloned()
        .ok_or(ReferenceMetadataEvalError::InvalidReferenceArg)
}

fn coerce_required_positive_index(
    arg: &CalcValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<usize, ReferenceMetadataEvalError> {
    let prepared =
        prepare_arg_values_only(arg, resolver).map_err(ReferenceMetadataEvalError::Coercion)?;
    let number =
        coerce_prepared_to_number(&prepared).map_err(ReferenceMetadataEvalError::Coercion)?;
    let truncated = number.trunc();
    if !truncated.is_finite() || truncated < 1.0 {
        return Err(ReferenceMetadataEvalError::InvalidAddressCoordinate);
    }
    Ok(truncated as usize)
}

fn coerce_optional_abs_num(
    arg: Option<&CalcValue>,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<usize, ReferenceMetadataEvalError> {
    let Some(arg) = arg else { return Ok(1) };
    if matches!(arg.core(), CoreValue::Missing | CoreValue::Empty) {
        return Ok(1);
    }
    let prepared =
        prepare_arg_values_only(arg, resolver).map_err(ReferenceMetadataEvalError::Coercion)?;
    let number =
        coerce_prepared_to_number(&prepared).map_err(ReferenceMetadataEvalError::Coercion)?;
    let truncated = number.trunc();
    match truncated as i64 {
        1..=4 if truncated.is_finite() => Ok(truncated as usize),
        _ => Err(ReferenceMetadataEvalError::InvalidAbsNum),
    }
}

fn coerce_optional_a1_flag(
    arg: Option<&CalcValue>,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<bool, ReferenceMetadataEvalError> {
    let Some(arg) = arg else { return Ok(true) };
    match arg.core() {
        CoreValue::Missing | CoreValue::Empty => return Ok(true),
        CoreValue::Logical(b) => return Ok(*b),
        _ => {}
    }
    let prepared =
        prepare_arg_values_only(arg, resolver).map_err(ReferenceMetadataEvalError::Coercion)?;
    match prepared.core() {
        CoreValue::Logical(b) => Ok(*b),
        _ => {
            if let Ok(number) = coerce_prepared_to_number(&prepared) {
                return Ok(number != 0.0);
            }
            let text = coerce_prepared_to_text(&prepared)
                .map_err(ReferenceMetadataEvalError::Coercion)?
                .to_string_lossy()
                .trim()
                .to_ascii_uppercase();
            match text.as_str() {
                "TRUE" => Ok(true),
                "FALSE" => Ok(false),
                _ => Err(ReferenceMetadataEvalError::Coercion(
                    CoercionError::NonNumericText(text),
                )),
            }
        }
    }
}

fn coerce_optional_sheet_text(
    arg: Option<&CalcValue>,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<Option<String>, ReferenceMetadataEvalError> {
    let Some(arg) = arg else { return Ok(None) };
    if arg.is_missing() {
        return Ok(None);
    }
    let prepared =
        prepare_arg_values_only(arg, resolver).map_err(ReferenceMetadataEvalError::Coercion)?;
    let text = coerce_prepared_to_text(&prepared)
        .map_err(ReferenceMetadataEvalError::Coercion)?
        .to_string_lossy();
    Ok(Some(text))
}

fn quote_sheet_text_if_needed(sheet_text: &str) -> String {
    let simple = !sheet_text.is_empty()
        && sheet_text
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '.');
    if simple {
        sheet_text.to_string()
    } else {
        format!("'{}'", sheet_text.replace('\'', "''"))
    }
}

fn column_label_from_index(mut col: usize) -> Option<String> {
    if col == 0 {
        return None;
    }
    let mut chars = Vec::new();
    while col > 0 {
        let rem = (col - 1) % 26;
        chars.push((b'A' + rem as u8) as char);
        col = (col - 1) / 26;
    }
    chars.reverse();
    Some(chars.into_iter().collect())
}

fn format_address_body(
    row: usize,
    col: usize,
    abs_num: usize,
    a1_style: bool,
) -> Result<String, ReferenceMetadataEvalError> {
    let address = if a1_style {
        let col_part = column_label_from_index(col)
            .ok_or(ReferenceMetadataEvalError::InvalidAddressCoordinate)?;
        let row_part = row.to_string();
        let col_text = match abs_num {
            1 | 3 => format!("${col_part}"),
            2 | 4 => col_part.clone(),
            _ => return Err(ReferenceMetadataEvalError::InvalidAbsNum),
        };
        let row_text = match abs_num {
            1 | 2 => format!("${row_part}"),
            3 | 4 => row_part.clone(),
            _ => return Err(ReferenceMetadataEvalError::InvalidAbsNum),
        };
        format!("{col_text}{row_text}")
    } else {
        let row_text = match abs_num {
            1 | 2 => format!("R{row}"),
            3 | 4 => format!("R[{row}]"),
            _ => return Err(ReferenceMetadataEvalError::InvalidAbsNum),
        };
        let col_text = match abs_num {
            1 | 3 => format!("C{col}"),
            2 | 4 => format!("C[{col}]"),
            _ => return Err(ReferenceMetadataEvalError::InvalidAbsNum),
        };
        format!("{row_text}{col_text}")
    };
    Ok(address)
}

fn has_legacy_multi_area_carrier(reference: &ReferenceLike) -> bool {
    !reference.target().is_empty()
        && reference.target().trim().starts_with('(')
        && reference.target().trim().ends_with(')')
        && reference.multi_area_targets().is_none()
}

fn count_reference_areas(reference: &ReferenceLike) -> Result<usize, ReferenceMetadataEvalError> {
    if has_legacy_multi_area_carrier(reference) {
        return Err(ReferenceMetadataEvalError::InvalidReferenceArg);
    }
    Ok(reference.area_count())
}

pub fn eval_address_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, ReferenceMetadataEvalError> {
    if !ADDRESS_META.arity.accepts(args.len()) {
        return Err(ReferenceMetadataEvalError::ArityMismatch {
            expected_min: ADDRESS_META.arity.min,
            expected_max: ADDRESS_META.arity.max,
            actual: args.len(),
        });
    }
    let row = coerce_required_positive_index(&args[0], resolver)?;
    let col = coerce_required_positive_index(&args[1], resolver)?;
    let abs_num = coerce_optional_abs_num(args.get(2), resolver)?;
    let a1_style = coerce_optional_a1_flag(args.get(3), resolver)?;
    let sheet_text = coerce_optional_sheet_text(args.get(4), resolver)?;

    let mut address = format_address_body(row, col, abs_num, a1_style)?;
    if let Some(sheet_text) = sheet_text {
        address = format!("{}!{}", quote_sheet_text_if_needed(&sheet_text), address);
    }
    Ok(CalcValue::text(ExcelText::from_utf16_code_units(
        address.encode_utf16().collect(),
    )))
}

pub fn eval_areas_surface(args: &[CalcValue]) -> Result<CalcValue, ReferenceMetadataEvalError> {
    if !AREAS_META.arity.accepts(args.len()) {
        return Err(ReferenceMetadataEvalError::ArityMismatch {
            expected_min: AREAS_META.arity.min,
            expected_max: AREAS_META.arity.max,
            actual: args.len(),
        });
    }
    let reference = parse_reference_arg(&args[0])?;
    Ok(CalcValue::number(count_reference_areas(&reference)? as f64))
}

pub fn eval_formulatext_surface(
    args: &[CalcValue],
    host_info: Option<&dyn HostInfoProvider>,
) -> Result<CalcValue, ReferenceMetadataEvalError> {
    if !FORMULATEXT_META.arity.accepts(args.len()) {
        return Err(ReferenceMetadataEvalError::ArityMismatch {
            expected_min: FORMULATEXT_META.arity.min,
            expected_max: FORMULATEXT_META.arity.max,
            actual: args.len(),
        });
    }
    let reference = parse_reference_arg(&args[0])?;
    let provider = host_info.ok_or(ReferenceMetadataEvalError::HostInfoProviderMissing(
        "formula_text",
    ))?;
    provider
        .query_formula_text(&reference)
        .map_err(ReferenceMetadataEvalError::HostInfo)
}

pub fn eval_sheet_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    host_info: Option<&dyn HostInfoProvider>,
) -> Result<CalcValue, ReferenceMetadataEvalError> {
    if !SHEET_META.arity.accepts(args.len()) {
        return Err(ReferenceMetadataEvalError::ArityMismatch {
            expected_min: SHEET_META.arity.min,
            expected_max: SHEET_META.arity.max,
            actual: args.len(),
        });
    }
    let provider = host_info.ok_or(ReferenceMetadataEvalError::HostInfoProviderMissing(
        "sheet_index",
    ))?;
    let spec = if args.is_empty() || args[0].is_missing() {
        SheetIdentitySpec::CurrentSheet
    } else if let Ok(reference) = parse_reference_arg(&args[0]) {
        SheetIdentitySpec::Reference(reference)
    } else {
        let prepared = prepare_arg_values_only(&args[0], resolver)
            .map_err(ReferenceMetadataEvalError::Coercion)?;
        let text = coerce_prepared_to_text(&prepared)
            .map_err(ReferenceMetadataEvalError::Coercion)?
            .to_string_lossy();
        SheetIdentitySpec::SheetNameText(text)
    };
    provider
        .query_sheet_index(&spec)
        .map_err(ReferenceMetadataEvalError::HostInfo)
}

pub fn eval_sheets_surface(
    args: &[CalcValue],
    host_info: Option<&dyn HostInfoProvider>,
) -> Result<CalcValue, ReferenceMetadataEvalError> {
    if !SHEETS_META.arity.accepts(args.len()) {
        return Err(ReferenceMetadataEvalError::ArityMismatch {
            expected_min: SHEETS_META.arity.min,
            expected_max: SHEETS_META.arity.max,
            actual: args.len(),
        });
    }
    let provider = host_info.ok_or(ReferenceMetadataEvalError::HostInfoProviderMissing(
        "sheet_count",
    ))?;
    let spec = if args.is_empty() || args[0].is_missing() {
        SheetCountSpec::Workbook
    } else {
        SheetCountSpec::Reference(parse_reference_arg(&args[0])?)
    };
    provider
        .query_sheet_count(&spec)
        .map_err(ReferenceMetadataEvalError::HostInfo)
}

pub fn map_reference_metadata_error_to_ws(e: &ReferenceMetadataEvalError) -> WorksheetErrorCode {
    match e {
        ReferenceMetadataEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        ReferenceMetadataEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        ReferenceMetadataEvalError::InvalidReferenceArg => WorksheetErrorCode::Value,
        ReferenceMetadataEvalError::InvalidAddressCoordinate => WorksheetErrorCode::Value,
        ReferenceMetadataEvalError::InvalidAbsNum => WorksheetErrorCode::Value,
        ReferenceMetadataEvalError::HostInfoProviderMissing(_) => WorksheetErrorCode::Value,
        ReferenceMetadataEvalError::HostInfo(_) => WorksheetErrorCode::Value,
        ReferenceMetadataEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::{ReferenceKind, WorksheetErrorCode};

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
    }

    struct MockProvider;

    impl HostInfoProvider for MockProvider {
        fn query_formula_text(
            &self,
            reference: &ReferenceLike,
        ) -> Result<CalcValue, HostInfoError> {
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                format!("={}", reference.target()).encode_utf16().collect(),
            )))
        }

        fn query_sheet_index(&self, spec: &SheetIdentitySpec) -> Result<CalcValue, HostInfoError> {
            match spec {
                SheetIdentitySpec::CurrentSheet => Ok(CalcValue::number(3.0)),
                SheetIdentitySpec::Reference(reference) if reference.target() == "Beta!A1" => {
                    Ok(CalcValue::number(2.0))
                }
                SheetIdentitySpec::SheetNameText(name) if name == "Alpha" => {
                    Ok(CalcValue::number(3.0))
                }
                SheetIdentitySpec::SheetNameText(_) => Ok(CalcValue::error(WorksheetErrorCode::NA)),
                _ => Err(HostInfoError::ProviderFailure {
                    detail: "unexpected sheet spec".to_string(),
                }),
            }
        }

        fn query_sheet_count(&self, spec: &SheetCountSpec) -> Result<CalcValue, HostInfoError> {
            match spec {
                SheetCountSpec::Workbook => Ok(CalcValue::number(3.0)),
                SheetCountSpec::Reference(reference)
                    if reference.target() == "'Quarter 1':Alpha!A1" =>
                {
                    Ok(CalcValue::number(3.0))
                }
                SheetCountSpec::Reference(reference) if reference.target() == "Beta!A1" => {
                    Ok(CalcValue::number(1.0))
                }
                _ => Err(HostInfoError::ProviderFailure {
                    detail: "unexpected sheet count spec".to_string(),
                }),
            }
        }
    }

    fn number_arg(n: f64) -> CalcValue {
        (CalcValue::number(n))
    }

    fn text_arg(text: &str) -> CalcValue {
        (CalcValue::text(ExcelText::from_utf16_code_units(
            text.encode_utf16().collect(),
        )))
    }

    fn bool_arg(value: bool) -> CalcValue {
        (CalcValue::logical(value))
    }

    fn ref_arg(target: &str) -> CalcValue {
        CalcValue::reference(ReferenceLike::new(ReferenceKind::A1, target.to_string()))
    }

    #[test]
    fn address_default_a1_is_absolute() {
        let got = eval_address_surface(&[number_arg(3.0), number_arg(2.0)], &MockResolver);
        assert_eq!(
            got,
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "$B$3".encode_utf16().collect(),
            )))
        );
    }

    #[test]
    fn address_r1c1_and_quoted_sheet_text_match_seeded_slice() {
        let got = eval_address_surface(
            &[
                number_arg(3.0),
                number_arg(2.0),
                number_arg(4.0),
                bool_arg(false),
                text_arg("Quarter 1"),
            ],
            &MockResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "'Quarter 1'!R[3]C[2]".encode_utf16().collect(),
            )))
        );
    }

    #[test]
    fn areas_rejects_legacy_parenthesized_area_carrier() {
        let got = eval_areas_surface(&[ref_arg("(A1,B2:B3)")]);
        assert_eq!(got, Err(ReferenceMetadataEvalError::InvalidReferenceArg));
    }

    #[test]
    fn areas_counts_first_class_multi_area_members() {
        let got = eval_areas_surface(&[CalcValue::reference(
            ReferenceLike::multi_area(vec!["A1".to_string(), "B2:B3".to_string()]).unwrap(),
        )]);
        assert_eq!(got, Ok(CalcValue::number(2.0)));
    }

    #[test]
    fn formulatext_uses_provider() {
        let got = eval_formulatext_surface(&[ref_arg("A1")], Some(&MockProvider));
        assert_eq!(
            got,
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "=A1".encode_utf16().collect(),
            )))
        );
    }

    #[test]
    fn sheet_supports_current_reference_and_text_spec() {
        assert_eq!(
            eval_sheet_surface(&[], &MockResolver, Some(&MockProvider)),
            Ok(CalcValue::number(3.0))
        );
        assert_eq!(
            eval_sheet_surface(&[ref_arg("Beta!A1")], &MockResolver, Some(&MockProvider)),
            Ok(CalcValue::number(2.0))
        );
        assert_eq!(
            eval_sheet_surface(&[text_arg("Alpha")], &MockResolver, Some(&MockProvider)),
            Ok(CalcValue::number(3.0))
        );
    }

    #[test]
    fn sheets_supports_workbook_and_reference_specs() {
        assert_eq!(
            eval_sheets_surface(&[], Some(&MockProvider)),
            Ok(CalcValue::number(3.0))
        );
        assert_eq!(
            eval_sheets_surface(&[ref_arg("'Quarter 1':Alpha!A1")], Some(&MockProvider)),
            Ok(CalcValue::number(3.0))
        );
    }
}
