use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{coerce_prepared_to_text, run_values_only_prepared};
use crate::host_info::{
    HostInfoError, HostInfoProvider, WidthConversionFunction, WidthConversionMode,
};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, ExcelText, WorksheetErrorCode};

const TEXT_COMPAT_LOCALE_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.TEXT_COMPAT_LOCALE_BASE",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::ApplicationState,
    thread_safety: ThreadSafetyClass::HostSerialized,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::None,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::Composite,
    surface_fec_dependency_profile: FecDependencyProfile::Composite,
};

pub const ASC_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ASC",
    ..TEXT_COMPAT_LOCALE_BASE_META
};
pub const DBCS_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DBCS",
    ..TEXT_COMPAT_LOCALE_BASE_META
};
pub const JIS_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.JIS",
    ..TEXT_COMPAT_LOCALE_BASE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum TextCompatLocaleEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    HostInfoProviderMissing(&'static str),
    HostInfo(HostInfoError),
}

fn narrow_basic_width(unit: u16) -> Option<u16> {
    match unit {
        0x3000 => Some(0x0020),
        0xFF01..=0xFF5E => Some(unit - 0xFEE0),
        _ => None,
    }
}

fn widen_basic_width(unit: u16) -> Option<u16> {
    match unit {
        0x0020 => Some(0x3000),
        0x0021..=0x007E => Some(unit + 0xFEE0),
        _ => None,
    }
}

fn halfwidth_katakana_to_fullwidth(unit: u16) -> Option<u16> {
    match unit {
        0xFF61 => Some(0x3002),
        0xFF62 => Some(0x300C),
        0xFF63 => Some(0x300D),
        0xFF64 => Some(0x3001),
        0xFF65 => Some(0x30FB),
        0xFF66 => Some(0x30F2),
        0xFF67 => Some(0x30A1),
        0xFF68 => Some(0x30A3),
        0xFF69 => Some(0x30A5),
        0xFF6A => Some(0x30A7),
        0xFF6B => Some(0x30A9),
        0xFF6C => Some(0x30E3),
        0xFF6D => Some(0x30E5),
        0xFF6E => Some(0x30E7),
        0xFF6F => Some(0x30C3),
        0xFF70 => Some(0x30FC),
        0xFF71 => Some(0x30A2),
        0xFF72 => Some(0x30A4),
        0xFF73 => Some(0x30A6),
        0xFF74 => Some(0x30A8),
        0xFF75 => Some(0x30AA),
        0xFF76 => Some(0x30AB),
        0xFF77 => Some(0x30AD),
        0xFF78 => Some(0x30AF),
        0xFF79 => Some(0x30B1),
        0xFF7A => Some(0x30B3),
        0xFF7B => Some(0x30B5),
        0xFF7C => Some(0x30B7),
        0xFF7D => Some(0x30B9),
        0xFF7E => Some(0x30BB),
        0xFF7F => Some(0x30BD),
        0xFF80 => Some(0x30BF),
        0xFF81 => Some(0x30C1),
        0xFF82 => Some(0x30C4),
        0xFF83 => Some(0x30C6),
        0xFF84 => Some(0x30C8),
        0xFF85 => Some(0x30CA),
        0xFF86 => Some(0x30CB),
        0xFF87 => Some(0x30CC),
        0xFF88 => Some(0x30CD),
        0xFF89 => Some(0x30CE),
        0xFF8A => Some(0x30CF),
        0xFF8B => Some(0x30D2),
        0xFF8C => Some(0x30D5),
        0xFF8D => Some(0x30D8),
        0xFF8E => Some(0x30DB),
        0xFF8F => Some(0x30DE),
        0xFF90 => Some(0x30DF),
        0xFF91 => Some(0x30E0),
        0xFF92 => Some(0x30E1),
        0xFF93 => Some(0x30E2),
        0xFF94 => Some(0x30E4),
        0xFF95 => Some(0x30E6),
        0xFF96 => Some(0x30E8),
        0xFF97 => Some(0x30E9),
        0xFF98 => Some(0x30EA),
        0xFF99 => Some(0x30EB),
        0xFF9A => Some(0x30EC),
        0xFF9B => Some(0x30ED),
        0xFF9C => Some(0x30EF),
        0xFF9D => Some(0x30F3),
        0xFF9E => Some(0x309B),
        0xFF9F => Some(0x309C),
        _ => None,
    }
}

fn voiced_halfwidth_pair_to_fullwidth(base: u16, mark: u16) -> Option<u16> {
    match (base, mark) {
        (0xFF73, 0xFF9E) => Some(0x30F4),
        (0xFF76, 0xFF9E) => Some(0x30AC),
        (0xFF77, 0xFF9E) => Some(0x30AE),
        (0xFF78, 0xFF9E) => Some(0x30B0),
        (0xFF79, 0xFF9E) => Some(0x30B2),
        (0xFF7A, 0xFF9E) => Some(0x30B4),
        (0xFF7B, 0xFF9E) => Some(0x30B6),
        (0xFF7C, 0xFF9E) => Some(0x30B8),
        (0xFF7D, 0xFF9E) => Some(0x30BA),
        (0xFF7E, 0xFF9E) => Some(0x30BC),
        (0xFF7F, 0xFF9E) => Some(0x30BE),
        (0xFF80, 0xFF9E) => Some(0x30C0),
        (0xFF81, 0xFF9E) => Some(0x30C2),
        (0xFF82, 0xFF9E) => Some(0x30C5),
        (0xFF83, 0xFF9E) => Some(0x30C7),
        (0xFF84, 0xFF9E) => Some(0x30C9),
        (0xFF8A, 0xFF9E) => Some(0x30D0),
        (0xFF8B, 0xFF9E) => Some(0x30D3),
        (0xFF8C, 0xFF9E) => Some(0x30D6),
        (0xFF8D, 0xFF9E) => Some(0x30D9),
        (0xFF8E, 0xFF9E) => Some(0x30DC),
        (0xFF8A, 0xFF9F) => Some(0x30D1),
        (0xFF8B, 0xFF9F) => Some(0x30D4),
        (0xFF8C, 0xFF9F) => Some(0x30D7),
        (0xFF8D, 0xFF9F) => Some(0x30DA),
        (0xFF8E, 0xFF9F) => Some(0x30DD),
        (0xFF9C, 0xFF9E) => Some(0x30F7),
        _ => None,
    }
}

fn fullwidth_katakana_to_halfwidth_seq(unit: u16) -> Option<&'static [u16]> {
    match unit {
        0x3002 => Some(&[0xFF61]),
        0x300C => Some(&[0xFF62]),
        0x300D => Some(&[0xFF63]),
        0x3001 => Some(&[0xFF64]),
        0x30FB => Some(&[0xFF65]),
        0x30F2 => Some(&[0xFF66]),
        0x30A1 => Some(&[0xFF67]),
        0x30A3 => Some(&[0xFF68]),
        0x30A5 => Some(&[0xFF69]),
        0x30A7 => Some(&[0xFF6A]),
        0x30A9 => Some(&[0xFF6B]),
        0x30E3 => Some(&[0xFF6C]),
        0x30E5 => Some(&[0xFF6D]),
        0x30E7 => Some(&[0xFF6E]),
        0x30C3 => Some(&[0xFF6F]),
        0x30FC => Some(&[0xFF70]),
        0x30A2 => Some(&[0xFF71]),
        0x30A4 => Some(&[0xFF72]),
        0x30A6 => Some(&[0xFF73]),
        0x30A8 => Some(&[0xFF74]),
        0x30AA => Some(&[0xFF75]),
        0x30AB => Some(&[0xFF76]),
        0x30AD => Some(&[0xFF77]),
        0x30AF => Some(&[0xFF78]),
        0x30B1 => Some(&[0xFF79]),
        0x30B3 => Some(&[0xFF7A]),
        0x30B5 => Some(&[0xFF7B]),
        0x30B7 => Some(&[0xFF7C]),
        0x30B9 => Some(&[0xFF7D]),
        0x30BB => Some(&[0xFF7E]),
        0x30BD => Some(&[0xFF7F]),
        0x30BF => Some(&[0xFF80]),
        0x30C1 => Some(&[0xFF81]),
        0x30C4 => Some(&[0xFF82]),
        0x30C6 => Some(&[0xFF83]),
        0x30C8 => Some(&[0xFF84]),
        0x30CA => Some(&[0xFF85]),
        0x30CB => Some(&[0xFF86]),
        0x30CC => Some(&[0xFF87]),
        0x30CD => Some(&[0xFF88]),
        0x30CE => Some(&[0xFF89]),
        0x30CF => Some(&[0xFF8A]),
        0x30D2 => Some(&[0xFF8B]),
        0x30D5 => Some(&[0xFF8C]),
        0x30D8 => Some(&[0xFF8D]),
        0x30DB => Some(&[0xFF8E]),
        0x30DE => Some(&[0xFF8F]),
        0x30DF => Some(&[0xFF90]),
        0x30E0 => Some(&[0xFF91]),
        0x30E1 => Some(&[0xFF92]),
        0x30E2 => Some(&[0xFF93]),
        0x30E4 => Some(&[0xFF94]),
        0x30E6 => Some(&[0xFF95]),
        0x30E8 => Some(&[0xFF96]),
        0x30E9 => Some(&[0xFF97]),
        0x30EA => Some(&[0xFF98]),
        0x30EB => Some(&[0xFF99]),
        0x30EC => Some(&[0xFF9A]),
        0x30ED => Some(&[0xFF9B]),
        0x30EF => Some(&[0xFF9C]),
        0x30F3 => Some(&[0xFF9D]),
        0x309B => Some(&[0xFF9E]),
        0x309C => Some(&[0xFF9F]),
        0x30F4 => Some(&[0xFF73, 0xFF9E]),
        0x30AC => Some(&[0xFF76, 0xFF9E]),
        0x30AE => Some(&[0xFF77, 0xFF9E]),
        0x30B0 => Some(&[0xFF78, 0xFF9E]),
        0x30B2 => Some(&[0xFF79, 0xFF9E]),
        0x30B4 => Some(&[0xFF7A, 0xFF9E]),
        0x30B6 => Some(&[0xFF7B, 0xFF9E]),
        0x30B8 => Some(&[0xFF7C, 0xFF9E]),
        0x30BA => Some(&[0xFF7D, 0xFF9E]),
        0x30BC => Some(&[0xFF7E, 0xFF9E]),
        0x30BE => Some(&[0xFF7F, 0xFF9E]),
        0x30C0 => Some(&[0xFF80, 0xFF9E]),
        0x30C2 => Some(&[0xFF81, 0xFF9E]),
        0x30C5 => Some(&[0xFF82, 0xFF9E]),
        0x30C7 => Some(&[0xFF83, 0xFF9E]),
        0x30C9 => Some(&[0xFF84, 0xFF9E]),
        0x30D0 => Some(&[0xFF8A, 0xFF9E]),
        0x30D3 => Some(&[0xFF8B, 0xFF9E]),
        0x30D6 => Some(&[0xFF8C, 0xFF9E]),
        0x30D9 => Some(&[0xFF8D, 0xFF9E]),
        0x30DC => Some(&[0xFF8E, 0xFF9E]),
        0x30D1 => Some(&[0xFF8A, 0xFF9F]),
        0x30D4 => Some(&[0xFF8B, 0xFF9F]),
        0x30D7 => Some(&[0xFF8C, 0xFF9F]),
        0x30DA => Some(&[0xFF8D, 0xFF9F]),
        0x30DD => Some(&[0xFF8E, 0xFF9F]),
        0x30F7 => Some(&[0xFF9C, 0xFF9E]),
        _ => None,
    }
}

fn asc_kernel(text: &ExcelText) -> ExcelText {
    let mut out = Vec::with_capacity(text.len_utf16_code_units());
    for &unit in text.utf16_code_units() {
        if let Some(narrow) = narrow_basic_width(unit) {
            out.push(narrow);
        } else if let Some(seq) = fullwidth_katakana_to_halfwidth_seq(unit) {
            out.extend_from_slice(seq);
        } else {
            out.push(unit);
        }
    }
    ExcelText::from_utf16_code_units(out)
}

fn jis_kernel(text: &ExcelText) -> ExcelText {
    let units = text.utf16_code_units();
    let mut out = Vec::with_capacity(units.len());
    let mut idx = 0;
    while idx < units.len() {
        let unit = units[idx];
        if idx + 1 < units.len() {
            if let Some(full) = voiced_halfwidth_pair_to_fullwidth(unit, units[idx + 1]) {
                out.push(full);
                idx += 2;
                continue;
            }
        }
        if let Some(wide) = widen_basic_width(unit) {
            out.push(wide);
        } else if let Some(full) = halfwidth_katakana_to_fullwidth(unit) {
            out.push(full);
        } else {
            out.push(unit);
        }
        idx += 1;
    }
    ExcelText::from_utf16_code_units(out)
}

fn render_text_for_mode(text: &ExcelText, mode: WidthConversionMode) -> EvalValue {
    match mode {
        WidthConversionMode::PassThrough => EvalValue::Text(text.clone()),
        WidthConversionMode::NarrowBasicWidthAndKana => EvalValue::Text(asc_kernel(text)),
        WidthConversionMode::WidenBasicWidthAndKana => EvalValue::Text(jis_kernel(text)),
        WidthConversionMode::Unavailable => EvalValue::Error(WorksheetErrorCode::Name),
    }
}

fn eval_width_conversion_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    host_info: Option<&dyn HostInfoProvider>,
    function: WidthConversionFunction,
    meta: &FunctionMeta,
) -> Result<EvalValue, TextCompatLocaleEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            if !meta.arity.accepts(prepared.len()) {
                return Err(TextCompatLocaleEvalError::ArityMismatch {
                    expected_min: meta.arity.min,
                    expected_max: meta.arity.max,
                    actual: prepared.len(),
                });
            }
            let text = coerce_prepared_to_text(&prepared[0])
                .map_err(TextCompatLocaleEvalError::Coercion)?;
            let provider = host_info.ok_or(TextCompatLocaleEvalError::HostInfoProviderMissing(
                "width_conversion_mode",
            ))?;
            let mode = provider
                .query_width_conversion_mode(function)
                .map_err(TextCompatLocaleEvalError::HostInfo)?;
            Ok(render_text_for_mode(&text, mode))
        },
        TextCompatLocaleEvalError::Coercion,
    )
}

pub fn eval_asc_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    host_info: Option<&dyn HostInfoProvider>,
) -> Result<EvalValue, TextCompatLocaleEvalError> {
    eval_width_conversion_surface(
        args,
        resolver,
        host_info,
        WidthConversionFunction::Asc,
        &ASC_META,
    )
}

pub fn eval_dbcs_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    host_info: Option<&dyn HostInfoProvider>,
) -> Result<EvalValue, TextCompatLocaleEvalError> {
    eval_width_conversion_surface(
        args,
        resolver,
        host_info,
        WidthConversionFunction::Dbcs,
        &DBCS_META,
    )
}

pub fn eval_jis_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    host_info: Option<&dyn HostInfoProvider>,
) -> Result<EvalValue, TextCompatLocaleEvalError> {
    eval_width_conversion_surface(
        args,
        resolver,
        host_info,
        WidthConversionFunction::Jis,
        &JIS_META,
    )
}

pub fn map_text_compat_locale_error_to_ws(error: &TextCompatLocaleEvalError) -> WorksheetErrorCode {
    match error {
        TextCompatLocaleEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        TextCompatLocaleEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        TextCompatLocaleEvalError::HostInfoProviderMissing(_) => WorksheetErrorCode::Value,
        TextCompatLocaleEvalError::HostInfo(HostInfoError::ProviderFailure { .. }) => {
            WorksheetErrorCode::Value
        }
        TextCompatLocaleEvalError::HostInfo(
            HostInfoError::UnsupportedWidthConversionProfileQuery(_),
        ) => WorksheetErrorCode::Value,
        TextCompatLocaleEvalError::HostInfo(HostInfoError::UnsupportedTranslateQuery)
        | TextCompatLocaleEvalError::HostInfo(HostInfoError::UnsupportedImageQuery)
        | TextCompatLocaleEvalError::HostInfo(HostInfoError::UnsupportedCellInfoQuery(_))
        | TextCompatLocaleEvalError::HostInfo(HostInfoError::UnsupportedInfoQuery(_))
        | TextCompatLocaleEvalError::HostInfo(HostInfoError::UnsupportedFormulaTextQuery)
        | TextCompatLocaleEvalError::HostInfo(HostInfoError::UnsupportedSheetIndexQuery)
        | TextCompatLocaleEvalError::HostInfo(HostInfoError::UnsupportedSheetCountQuery)
        | TextCompatLocaleEvalError::HostInfo(
            HostInfoError::UnsupportedAggregateReferenceContextQuery,
        ) => WorksheetErrorCode::Value,
        TextCompatLocaleEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host_info::{WidthConversionFunction, WidthConversionMode};
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::ReferenceLike;

    struct NoResolver;

    impl ReferenceResolver for NoResolver {
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
    }

    struct MockWidthConversionProvider;

    impl HostInfoProvider for MockWidthConversionProvider {
        fn query_width_conversion_mode(
            &self,
            function: WidthConversionFunction,
        ) -> Result<WidthConversionMode, HostInfoError> {
            Ok(match function {
                WidthConversionFunction::Asc => WidthConversionMode::PassThrough,
                WidthConversionFunction::Dbcs => WidthConversionMode::PassThrough,
                WidthConversionFunction::Jis => WidthConversionMode::Unavailable,
            })
        }
    }

    struct ConvertingWidthProvider;

    impl HostInfoProvider for ConvertingWidthProvider {
        fn query_width_conversion_mode(
            &self,
            function: WidthConversionFunction,
        ) -> Result<WidthConversionMode, HostInfoError> {
            Ok(match function {
                WidthConversionFunction::Asc | WidthConversionFunction::Dbcs => {
                    WidthConversionMode::NarrowBasicWidthAndKana
                }
                WidthConversionFunction::Jis => WidthConversionMode::WidenBasicWidthAndKana,
            })
        }
    }

    fn txt(s: &str) -> ExcelText {
        ExcelText::from_utf16_code_units(s.encode_utf16().collect())
    }

    fn text_arg(s: &str) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Text(txt(s)))
    }

    fn number_arg(n: f64) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Number(n))
    }

    #[test]
    fn metadata_shapes_match_bounded_family() {
        assert_eq!(ASC_META.arity, Arity::exact(1));
        assert_eq!(DBCS_META.function_id, "FUNC.DBCS");
        assert_eq!(
            ASC_META.host_interaction,
            HostInteractionClass::ApplicationState
        );
        assert_eq!(
            JIS_META.surface_fec_dependency_profile,
            FecDependencyProfile::Composite
        );
    }

    #[test]
    fn asc_narrows_fullwidth_ascii_space_and_katakana() {
        assert_eq!(asc_kernel(&txt("ＡＢＣ　１２３")), txt("ABC 123"));
        assert_eq!(
            asc_kernel(&txt("ガギグゲゴパピプペポヴヷ")),
            txt("ｶﾞｷﾞｸﾞｹﾞｺﾞﾊﾟﾋﾟﾌﾟﾍﾟﾎﾟｳﾞﾜﾞ")
        );
        assert_eq!(asc_kernel(&txt("アイウあ😀")), txt("ｱｲｳあ😀"));
    }

    #[test]
    fn jis_widens_ascii_space_and_katakana_pairs() {
        assert_eq!(jis_kernel(&txt("ABC 123")), txt("ＡＢＣ　１２３"));
        assert_eq!(
            jis_kernel(&txt("ｶﾞｷﾞｸﾞｹﾞｺﾞﾊﾟﾋﾟﾌﾟﾍﾟﾎﾟｳﾞﾜﾞ")),
            txt("ガギグゲゴパピプペポヴヷ")
        );
        assert_eq!(jis_kernel(&txt("ｱｲｳあ😀")), txt("アイウあ😀"));
    }

    #[test]
    fn dbcs_matches_jis_in_current_baseline_slice() {
        let resolver = NoResolver;
        let args = [text_arg("ABC ｶﾞ")];
        assert_eq!(
            eval_dbcs_surface(&args, &resolver, Some(&MockWidthConversionProvider)),
            Ok(EvalValue::Text(txt("ABC ｶﾞ")))
        );
        assert_eq!(
            eval_jis_surface(&args, &resolver, Some(&MockWidthConversionProvider)),
            Ok(EvalValue::Error(WorksheetErrorCode::Name))
        );
    }

    #[test]
    fn surface_evaluators_follow_host_mode_and_textify_values() {
        let resolver = NoResolver;
        assert_eq!(
            eval_asc_surface(
                &[number_arg(123.0)],
                &resolver,
                Some(&ConvertingWidthProvider)
            ),
            Ok(EvalValue::Text(txt("123")))
        );
        assert_eq!(
            eval_jis_surface(
                &[CallArgValue::Eval(EvalValue::Logical(true))],
                &resolver,
                Some(&ConvertingWidthProvider)
            ),
            Ok(EvalValue::Text(txt("ＴＲＵＥ")))
        );
        assert_eq!(
            eval_dbcs_surface(
                &[CallArgValue::EmptyCell],
                &resolver,
                Some(&ConvertingWidthProvider)
            ),
            Ok(EvalValue::Text(txt("")))
        );
    }

    #[test]
    fn error_mapping_and_arity_lanes_are_pinned() {
        let resolver = NoResolver;
        assert_eq!(
            eval_asc_surface(&[], &resolver, Some(&MockWidthConversionProvider)),
            Err(TextCompatLocaleEvalError::ArityMismatch {
                expected_min: 1,
                expected_max: 1,
                actual: 0,
            })
        );
        assert_eq!(
            map_text_compat_locale_error_to_ws(
                &TextCompatLocaleEvalError::HostInfoProviderMissing("width_conversion_mode")
            ),
            WorksheetErrorCode::Value
        );
        assert_eq!(
            map_text_compat_locale_error_to_ws(&TextCompatLocaleEvalError::Coercion(
                CoercionError::WorksheetError(WorksheetErrorCode::NA)
            )),
            WorksheetErrorCode::NA
        );
    }
}
