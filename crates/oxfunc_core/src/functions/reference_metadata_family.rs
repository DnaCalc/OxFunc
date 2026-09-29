use crate::coercion::{CoercionError, parse_excel_logical_text};
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

#[path = "address_name_classes.rs"]
mod address_name_classes;

pub const ADDRESS_META: FunctionMeta = function_spec! {
    function_id: "FUNC.ADDRESS",
    arity: Arity { min: 2, max: 5 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    // ADDRESS is scalar-shaped by-index and broadcasts all five of its arguments over an array
    // (`[0,1,2,3,4]`). Verified live Excel 16.0 build 20026.
    lift_broadcast_profile: FunctionMeta::lift_at(&[0, 1, 2, 3, 4]),
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::None,
};

pub const AREAS_META: FunctionMeta = function_spec! {
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

pub const FORMULATEXT_META: FunctionMeta = function_spec! {
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

pub const SHEET_META: FunctionMeta = function_spec! {
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

pub const SHEETS_META: FunctionMeta = function_spec! {
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
    InvalidSheetText,
    HostInfoProviderMissing(&'static str),
    HostInfo(HostInfoError),
}

fn parse_reference_arg(arg: &CalcValue) -> Result<ReferenceLike, ReferenceMetadataEvalError> {
    arg.as_reference()
        .cloned()
        .ok_or(ReferenceMetadataEvalError::InvalidReferenceArg)
}

fn coerce_address_coordinate(
    arg: &CalcValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<i64, ReferenceMetadataEvalError> {
    let prepared =
        prepare_arg_values_only(arg, resolver).map_err(ReferenceMetadataEvalError::Coercion)?;
    let number =
        coerce_prepared_to_number(&prepared).map_err(ReferenceMetadataEvalError::Coercion)?;
    if !number.is_finite() {
        return Err(ReferenceMetadataEvalError::InvalidAddressCoordinate);
    }
    // Bounds depend on the address style and whether this axis is relative.
    // Keep the sign until those selectors have been read. Saturating casts of
    // finite out-of-range values remain outside every permitted coordinate.
    Ok(address_integer(number))
}

fn address_integer(number: f64) -> i64 {
    // On ADDRESS's admitted integer range, Excel's observed conversion is
    // floor(RN53(RN64(number + 2^31)) - 2^31). The two rounding steps admit
    // the lower neighbor of each integer through these inclusive thresholds.
    // This reduced comparison avoids platform-dependent extended precision.
    // W111 retains exact-bit boundary probes for both thresholds and all axes.
    let upper = number.ceil();
    let threshold = if upper > 0.0 {
        2.0_f64.powi(-22) + 2.0_f64.powi(-33)
    } else {
        2.0_f64.powi(-23) + 2.0_f64.powi(-34)
    };
    if upper - number <= threshold {
        upper as i64
    } else {
        number.floor() as i64
    }
}

fn coerce_optional_abs_num(
    arg: Option<&CalcValue>,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<usize, ReferenceMetadataEvalError> {
    let Some(arg) = arg else { return Ok(1) };
    if matches!(arg.core(), CoreValue::Missing) {
        return Ok(1);
    }
    let prepared =
        prepare_arg_values_only(arg, resolver).map_err(ReferenceMetadataEvalError::Coercion)?;
    let number =
        coerce_prepared_to_number(&prepared).map_err(ReferenceMetadataEvalError::Coercion)?;
    // Domain validation follows coercion of every supplied argument. Thus a
    // later worksheet error wins over abs_num=0, but not over bad numeric text.
    Ok(address_integer(number) as usize)
}

fn coerce_optional_a1_flag(
    arg: Option<&CalcValue>,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<bool, ReferenceMetadataEvalError> {
    let Some(arg) = arg else { return Ok(true) };
    match arg.core() {
        CoreValue::Missing => return Ok(true),
        CoreValue::Logical(b) => return Ok(*b),
        _ => {}
    }
    let prepared =
        prepare_arg_values_only(arg, resolver).map_err(ReferenceMetadataEvalError::Coercion)?;
    match prepared.core() {
        CoreValue::Logical(b) => Ok(*b),
        CoreValue::Text(text) => {
            let text = text.to_string_lossy();
            parse_excel_logical_text(&text).ok_or(ReferenceMetadataEvalError::Coercion(
                CoercionError::NonNumericText(text),
            ))
        }
        _ => coerce_prepared_to_number(&prepared)
            .map(|number| number != 0.0)
            .map_err(ReferenceMetadataEvalError::Coercion),
    }
}

fn coerce_optional_sheet_text(
    arg: Option<&CalcValue>,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<Option<ExcelText>, ReferenceMetadataEvalError> {
    let Some(arg) = arg else { return Ok(None) };
    if arg.is_missing() {
        return Ok(None);
    }
    let prepared =
        prepare_arg_values_only(arg, resolver).map_err(ReferenceMetadataEvalError::Coercion)?;
    let text = if let CoreValue::Number(number) = prepared.core() {
        ExcelText::from_utf16_code_units(
            super::numeric_text::finite_number_text(*number)
                .encode_utf16()
                .collect(),
        )
    } else {
        coerce_prepared_to_text(&prepared).map_err(ReferenceMetadataEvalError::Coercion)?
    };
    Ok(Some(text))
}

fn address_name_character(unit: u16, initial: bool) -> bool {
    if unit < 128 {
        let byte = unit as u8;
        byte.is_ascii_alphabetic()
            || matches!(byte, b'_' | b'\\')
            || (!initial && (byte.is_ascii_digit() || matches!(byte, b'.' | b'?')))
    } else if unit < 256 {
        // Excel's legacy name classes are distinct from Unicode alphabetic
        // categories (for example multiplication and division signs qualify).
        matches!(unit, 0xA1 | 0xA4 | 0xA7 | 0xA8 | 0xAA | 0xAD | 0xAF..=0xBA | 0xBC..=0xFF)
    } else if (0xD800..=0xDFFF).contains(&unit) {
        // Classification is by UTF-16 unit, even for isolated surrogates.
        !initial || unit <= 0xDBFF
    } else {
        let ranges = if initial {
            address_name_classes::NAME_START
        } else {
            address_name_classes::NAME_CONTINUE
        };
        let index = ranges.partition_point(|&(_, end)| end < unit);
        ranges.get(index).is_some_and(|&(start, _)| start <= unit)
    }
}

fn address_name_is_reference(units: &[u16]) -> bool {
    use crate::functions::a1_refs::{A1ReferenceNotation, parse_a1_reference};
    // Reference tokens are ASCII. Lossy conversion here affects no emitted
    // text and cannot turn a non-ASCII unit into an ASCII reference character.
    let name = String::from_utf16_lossy(units).to_ascii_uppercase();
    if let Some(reference) = parse_a1_reference(&name) {
        if reference.notation == A1ReferenceNotation::Rect
            && reference.end_row <= 1_048_576
            && reference.end_col <= 16_384
        {
            return true;
        }
    }
    if matches!(name.as_str(), "R" | "C" | "RC" | "TRUE" | "FALSE") {
        return true;
    }
    // Defined-name ambiguity uses a leading R1C1 axis token. A dot immediately
    // after its digits instead makes a dotted name token. The suffix need not
    // itself form a reference (R1foo and C1R0 still require quoting).
    let (rest, limit) = if let Some(rest) = name.strip_prefix("RC") {
        (rest, 16_384)
    } else if let Some(rest) = name.strip_prefix('R') {
        (rest, 1_048_576)
    } else if let Some(rest) = name.strip_prefix('C') {
        (rest, 16_384)
    } else {
        return false;
    };
    let count = rest.bytes().take_while(u8::is_ascii_digit).count();
    count > 0
        && !rest[count..].starts_with('.')
        && rest[..count]
            .parse::<u32>()
            .is_ok_and(|number| (1..=limit).contains(&number))
}

fn quote_sheet_text_if_needed(
    sheet_text: &ExcelText,
) -> Result<Vec<u16>, ReferenceMetadataEvalError> {
    let units = sheet_text.utf16_code_units();
    let escaped_len = units.len() + units.iter().filter(|&&unit| unit == 39).count();
    if units.len() > 255 || escaped_len > 256 {
        return Err(ReferenceMetadataEvalError::InvalidSheetText);
    }
    let simple_token = |token: &[u16]| {
        token
            .iter()
            .enumerate()
            .all(|(index, &unit)| address_name_character(unit, index == 0))
    };
    let simple = if units.is_empty() {
        true
    } else if units[0] == 91 {
        units
            .iter()
            .position(|&unit| unit == 93)
            .is_some_and(|end| {
                end > 1
                    && units[1..end]
                        .iter()
                        .all(|&unit| address_name_character(unit, false))
                    && simple_token(&units[end + 1..])
            })
    } else {
        simple_token(units) && !address_name_is_reference(units)
    };
    if simple {
        return Ok(units.to_vec());
    }
    let mut quoted = Vec::with_capacity(escaped_len + 2);
    quoted.push(39);
    for &unit in units {
        quoted.push(unit);
        if unit == 39 {
            quoted.push(39);
        }
    }
    quoted.push(39);
    Ok(quoted)
}

struct AddressBuffer(Vec<u16>);

impl AddressBuffer {
    fn literal(&mut self, text: &str) {
        let remaining = 258usize.saturating_sub(self.0.len());
        self.0.extend(text.encode_utf16().take(remaining));
    }

    fn integer(&mut self, number: i64) {
        // Signs are literal fragments; magnitude digits append atomically.
        if number < 0 {
            self.literal("-");
        }
        let digits = number.unsigned_abs().to_string();
        if digits.len() <= 258usize.saturating_sub(self.0.len()) {
            self.literal(&digits);
        }
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
    row: i64,
    col: i64,
    abs_num: usize,
    a1_style: bool,
    sheet_text: Option<&ExcelText>,
) -> Result<ExcelText, ReferenceMetadataEvalError> {
    if !(1..=4).contains(&abs_num) {
        return Err(ReferenceMetadataEvalError::InvalidAbsNum);
    }
    let row_relative = !a1_style && matches!(abs_num, 3 | 4);
    let col_relative = !a1_style && matches!(abs_num, 2 | 4);
    let valid_coordinate = |value: i64, limit: i64, relative: bool| {
        if relative {
            (-limit + 1..limit).contains(&value)
        } else {
            (1..=limit).contains(&value)
        }
    };
    if !valid_coordinate(row, 1_048_576, row_relative)
        || !valid_coordinate(col, 16_384, col_relative)
    {
        return Err(ReferenceMetadataEvalError::InvalidAddressCoordinate);
    }
    let prefix = match sheet_text {
        Some(sheet) => {
            let mut prefix = quote_sheet_text_if_needed(sheet)?;
            prefix.push(33);
            prefix
        }
        None => Vec::new(),
    };
    // The sheet prefix is constructed before the bounded body append. An
    // admitted escaped name can make the prefix itself 259 UTF-16 units long.
    let mut address = AddressBuffer(prefix);
    if a1_style {
        let col_part = column_label_from_index(col as usize)
            .ok_or(ReferenceMetadataEvalError::InvalidAddressCoordinate)?;
        if matches!(abs_num, 1 | 3) {
            address.literal("$");
        }
        address.literal(&col_part);
        if matches!(abs_num, 1 | 2) {
            address.literal("$");
        }
        address.integer(row);
    } else {
        for (label, number, relative) in [("R", row, row_relative), ("C", col, col_relative)] {
            address.literal(label);
            if !relative || number != 0 {
                if relative {
                    address.literal("[");
                }
                address.integer(number);
                if relative {
                    address.literal("]");
                }
            }
        }
    }
    Ok(ExcelText::from_utf16_code_units(address.0))
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
    let row = if args[0].is_missing() {
        0
    } else {
        coerce_address_coordinate(&args[0], resolver)?
    };
    let col = coerce_address_coordinate(&args[1], resolver)?;
    let abs_num = coerce_optional_abs_num(args.get(2), resolver)?;
    let a1_style = coerce_optional_a1_flag(args.get(3), resolver)?;
    let sheet_text = coerce_optional_sheet_text(args.get(4), resolver)?;

    Ok(CalcValue::text(format_address_body(
        row,
        col,
        abs_num,
        a1_style,
        sheet_text.as_ref(),
    )?))
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
        ReferenceMetadataEvalError::InvalidSheetText => WorksheetErrorCode::Value,
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
        CalcValue::number(n)
    }

    fn text_arg(text: &str) -> CalcValue {
        CalcValue::text(ExcelText::from_utf16_code_units(
            text.encode_utf16().collect(),
        ))
    }

    fn bool_arg(value: bool) -> CalcValue {
        CalcValue::logical(value)
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
