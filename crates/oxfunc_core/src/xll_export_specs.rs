use crate::function::{
    ArgPreparationProfile, CoercionLiftProfile, DeterminismClass, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, VolatilityClass,
};
use crate::functions::{
    abs::ABS_META, asin::ASIN_META, and_fn::AND_META, average::AVERAGE_META, cell::CELL_META, column_fn::COLUMN_META,
    clean_fn::CLEAN_META, count::COUNT_META, counta::COUNTA_META, date_fn::DATE_META,
    dollar_fn::DOLLAR_META,
    exact_fn::EXACT_META, fixed_fn::FIXED_META, hstack::HSTACK_META, if_fn::IF_META, iferror::IFERROR_META,
    index::INDEX_META, indirect::INDIRECT_META, isnumber::ISNUMBER_META, match_fn::MATCH_META,
    n_fn::N_META, now_fn::NOW_META, offset::OFFSET_META, op_add::OP_ADD_META, pi::PI_META,
    rand_fn::RAND_META, round_fn::ROUND_META, row_fn::ROW_META, sequence::SEQUENCE_META, sin::SIN_META,
    sum::SUM_META, t_fn::T_META, text_fn::TEXT_META, textjoin::TEXTJOIN_META, today_fn::TODAY_META,
    type_fn::TYPE_META, value_fn::VALUE_META, xlookup::XLOOKUP_META, xmatch::XMATCH_META,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XllEntryKind {
    UArity(usize),
    QUnaryNumber,
    QBinaryNumber,
    QNullaryNumber,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XllULiftPolicy {
    ScalarOnly,
    UnaryScalarOrArrayElementwise,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XllExportSpec {
    pub export_name: String,
    pub worksheet_name: String,
    pub type_text: String,
    pub arg_names: String,
    pub function_id: &'static str,
    pub min_arity: usize,
    pub entry_kind: XllEntryKind,
    pub u_lift_policy: Option<XllULiftPolicy>,
    pub preserve_refs: bool,
}

const FUNCTION_CATALOG: &[FunctionMeta] = &[
    ABS_META,
    ASIN_META,
    AND_META,
    AVERAGE_META,
    CELL_META,
    COLUMN_META,
    CLEAN_META,
    COUNT_META,
    COUNTA_META,
    DATE_META,
    DOLLAR_META,
    EXACT_META,
    FIXED_META,
    HSTACK_META,
    IF_META,
    IFERROR_META,
    INDEX_META,
    INDIRECT_META,
    ISNUMBER_META,
    MATCH_META,
    N_META,
    NOW_META,
    OFFSET_META,
    OP_ADD_META,
    PI_META,
    RAND_META,
    ROW_META,
    ROUND_META,
    SEQUENCE_META,
    SIN_META,
    SUM_META,
    T_META,
    TEXT_META,
    TEXTJOIN_META,
    TODAY_META,
    TYPE_META,
    VALUE_META,
    XLOOKUP_META,
    XMATCH_META,
];

fn function_suffix(function_id: &str) -> String {
    function_id
        .strip_prefix("FUNC.")
        .unwrap_or(function_id)
        .replace('.', "_")
}

fn csv_escape(field: &str) -> String {
    let needs_quotes = field.contains(',') || field.contains('"') || field.contains('\n');
    if !needs_quotes {
        return field.to_string();
    }

    let escaped = field.replace('"', "\"\"");
    format!("\"{escaped}\"")
}

fn arg_names_for_count(count: usize) -> String {
    if count == 0 {
        return String::new();
    }
    (1..=count)
        .map(|i| format!("arg{i}"))
        .collect::<Vec<_>>()
        .join(",")
}

const MAX_XLL_ARG_NAMES_LEN: usize = 255;

fn capped_arg_names_for_count(count: usize) -> String {
    let arg_names = arg_names_for_count(count);
    if arg_names.len() > MAX_XLL_ARG_NAMES_LEN {
        String::new()
    } else {
        arg_names
    }
}

fn type_text_for_u_arity(count: usize) -> String {
    let mut out = String::from("Q");
    out.push_str(&"U".repeat(count));
    out
}

const MAX_XLL_TYPE_TEXT_LEN: usize = 255;

fn registration_suffix_len(meta: &FunctionMeta) -> usize {
    let mut len = 0;
    if meta.volatility == VolatilityClass::VolatileFull {
        len += 1;
    }
    len
}

fn capped_u_arity(meta: &FunctionMeta) -> usize {
    let max_u_arity = MAX_XLL_TYPE_TEXT_LEN.saturating_sub(1 + registration_suffix_len(meta));
    meta.arity.max.min(max_u_arity)
}

fn apply_registration_suffixes(meta: &FunctionMeta, base_type_text: String) -> String {
    let mut out = base_type_text;
    if meta.volatility == VolatilityClass::VolatileFull {
        out.push('!');
    }
    out
}

fn q_entry_kind_from_profile(meta: &FunctionMeta) -> Option<XllEntryKind> {
    let exact_arity = meta.arity.min == meta.arity.max;
    if !exact_arity {
        return None;
    }

    let profile_allows_q = meta.determinism == DeterminismClass::Deterministic
        && meta.volatility == VolatilityClass::NonVolatile
        && meta.host_interaction == HostInteractionClass::None
        && meta.arg_preparation_profile == ArgPreparationProfile::ValuesOnlyPreAdapter;

    if !profile_allows_q {
        return None;
    }

    match (meta.arity.min, meta.kernel_signature_class) {
        (0, KernelSignatureClass::NullaryConst) => Some(XllEntryKind::QNullaryNumber),
        (1, KernelSignatureClass::NumToNum) => Some(XllEntryKind::QUnaryNumber),
        (2, KernelSignatureClass::NumsToNum) => Some(XllEntryKind::QBinaryNumber),
        _ => None,
    }
}

fn u_lift_policy_from_profile(meta: &FunctionMeta) -> XllULiftPolicy {
    match meta.coercion_lift_profile {
        CoercionLiftProfile::UnaryNumericScalarOrArrayElementwise
            if meta.arity.min == 1 && meta.arity.max == 1 =>
        {
            XllULiftPolicy::UnaryScalarOrArrayElementwise
        }
        _ => XllULiftPolicy::ScalarOnly,
    }
}

pub fn xll_export_specs() -> Vec<XllExportSpec> {
    let mut specs = Vec::new();

    for meta in FUNCTION_CATALOG {
        let suffix = function_suffix(meta.function_id);
        let export_base = format!("OX_{suffix}");
        let worksheet_base = format!("ox_{suffix}");
        let q_kind = q_entry_kind_from_profile(meta);
        let emit_u = meta.arity.max > 0 || q_kind.is_none();
        let u_arity = capped_u_arity(meta);

        if emit_u {
            specs.push(XllExportSpec {
                export_name: export_base.clone(),
                worksheet_name: worksheet_base.clone(),
                type_text: apply_registration_suffixes(meta, type_text_for_u_arity(u_arity)),
                arg_names: capped_arg_names_for_count(u_arity),
                function_id: meta.function_id,
                min_arity: meta.arity.min,
                entry_kind: XllEntryKind::UArity(u_arity),
                u_lift_policy: Some(u_lift_policy_from_profile(meta)),
                preserve_refs: meta.arg_preparation_profile
                    == ArgPreparationProfile::RefsVisibleInAdapter,
            });
        }

        if let Some(kind) = q_kind {
            let (type_text, arg_names) = match kind {
                XllEntryKind::QNullaryNumber => ("B".to_string(), String::new()),
                XllEntryKind::QUnaryNumber => ("BB".to_string(), "value".to_string()),
                XllEntryKind::QBinaryNumber => ("BBB".to_string(), "lhs,rhs".to_string()),
                XllEntryKind::UArity(_) => unreachable!(),
            };

            let export_name = if emit_u {
                format!("{export_base}_Q")
            } else {
                export_base.clone()
            };
            let worksheet_name = if emit_u {
                format!("{worksheet_base}_Q")
            } else {
                worksheet_base.clone()
            };

            specs.push(XllExportSpec {
                export_name,
                worksheet_name,
                type_text,
                arg_names,
                function_id: meta.function_id,
                min_arity: meta.arity.min,
                entry_kind: kind,
                u_lift_policy: None,
                preserve_refs: false,
            });
        }
    }

    specs.sort_by(|a, b| a.export_name.cmp(&b.export_name));
    specs
}

pub fn render_export_specs_csv() -> String {
    let mut out = String::from(
        "export_name,worksheet_name,type_text,arg_names,function_id,min_arity,entry_kind,u_lift_policy,preserve_refs\n",
    );
    for spec in xll_export_specs() {
        let entry_kind = match spec.entry_kind {
            XllEntryKind::UArity(n) => format!("u_arity_{n}"),
            XllEntryKind::QUnaryNumber => "q_unary_number".to_string(),
            XllEntryKind::QBinaryNumber => "q_binary_number".to_string(),
            XllEntryKind::QNullaryNumber => "q_nullary_number".to_string(),
        };
        let u_lift = match spec.u_lift_policy {
            Some(XllULiftPolicy::ScalarOnly) => "scalar_only",
            Some(XllULiftPolicy::UnaryScalarOrArrayElementwise) => {
                "unary_scalar_or_array_elementwise"
            }
            None => "",
        };
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            csv_escape(&spec.export_name),
            csv_escape(&spec.worksheet_name),
            csv_escape(&spec.type_text),
            csv_escape(&spec.arg_names),
            csv_escape(spec.function_id),
            spec.min_arity,
            csv_escape(&entry_kind),
            csv_escape(u_lift),
            if spec.preserve_refs { "true" } else { "false" }
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_specs_have_unique_export_and_worksheet_names() {
        let mut exports = std::collections::BTreeSet::new();
        let mut worksheets = std::collections::BTreeSet::new();
        for spec in xll_export_specs() {
            assert!(exports.insert(spec.export_name));
            assert!(worksheets.insert(spec.worksheet_name));
        }
    }

    #[test]
    fn all_catalog_functions_have_at_least_one_export() {
        let specs = xll_export_specs();
        let mut ids = std::collections::BTreeSet::new();
        for spec in specs {
            ids.insert(spec.function_id);
        }
        for meta in FUNCTION_CATALOG {
            assert!(ids.contains(meta.function_id));
        }
    }

    #[test]
    fn csv_header_and_row_count_are_stable() {
        let csv = render_export_specs_csv();
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(
            lines.first().copied(),
            Some(
                "export_name,worksheet_name,type_text,arg_names,function_id,min_arity,entry_kind,u_lift_policy,preserve_refs"
            )
        );
        assert_eq!(lines.len(), xll_export_specs().len() + 1);
    }

    #[test]
    fn volatile_full_u_exports_receive_bang_suffix() {
        let specs = xll_export_specs();
        for function_id in ["FUNC.NOW", "FUNC.TODAY", "FUNC.RAND"] {
            let spec = specs
                .iter()
                .find(|s| s.function_id == function_id && matches!(s.entry_kind, XllEntryKind::UArity(_)))
                .unwrap_or_else(|| panic!("missing U export for {function_id}"));
            assert!(
                spec.type_text.ends_with('!'),
                "expected volatile U export for {function_id} to end with !, got {}",
                spec.type_text
            );
        }
    }

    #[test]
    fn nonvolatile_u_export_does_not_receive_bang_suffix() {
        let specs = xll_export_specs();
        let spec = specs
            .iter()
            .find(|s| s.function_id == "FUNC.ABS" && matches!(s.entry_kind, XllEntryKind::UArity(_)))
            .expect("missing U export for FUNC.ABS");
        assert!(
            !spec.type_text.ends_with('!'),
            "expected nonvolatile U export for FUNC.ABS to omit !, got {}",
            spec.type_text
        );
    }

    #[test]
    fn u_exports_stay_within_type_text_limit() {
        let specs = xll_export_specs();
        for spec in specs {
            if matches!(spec.entry_kind, XllEntryKind::UArity(_)) {
                assert!(
                    spec.type_text.len() <= MAX_XLL_TYPE_TEXT_LEN,
                    "expected {} to stay within {MAX_XLL_TYPE_TEXT_LEN}, got {}",
                    spec.export_name,
                    spec.type_text.len()
                );
            }
        }
    }

    #[test]
    fn sum_u_export_is_capped_to_callable_arity_limit() {
        let specs = xll_export_specs();
        let spec = specs
            .iter()
            .find(|s| s.function_id == "FUNC.SUM" && matches!(s.entry_kind, XllEntryKind::UArity(_)))
            .expect("missing U export for FUNC.SUM");
        assert_eq!(spec.entry_kind, XllEntryKind::UArity(254));
        assert_eq!(spec.type_text.len(), MAX_XLL_TYPE_TEXT_LEN);
        assert!(spec.arg_names.is_empty());
    }
}





