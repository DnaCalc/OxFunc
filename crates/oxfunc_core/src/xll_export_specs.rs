use crate::functions::surface_dispatch::{FUNC_ID_ABS, FUNC_ID_PI};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XllEntryKind {
    UUnary,
    QUnaryNumber,
    QNullaryNumber,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XllULiftPolicy {
    ScalarOnly,
    UnaryScalarOrArrayElementwise,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XllExportSpec {
    pub export_name: &'static str,
    pub worksheet_name: &'static str,
    pub type_text: &'static str,
    pub arg_names: &'static str,
    pub function_id: &'static str,
    pub entry_kind: XllEntryKind,
    pub u_lift_policy: Option<XllULiftPolicy>,
}

pub const OXFUNC_XLL_EXPORT_SPECS: &[XllExportSpec] = &[
    XllExportSpec {
        export_name: "OX_ABS",
        worksheet_name: "ox_ABS",
        type_text: "QU",
        arg_names: "value",
        function_id: FUNC_ID_ABS,
        entry_kind: XllEntryKind::UUnary,
        u_lift_policy: Some(XllULiftPolicy::UnaryScalarOrArrayElementwise),
    },
    XllExportSpec {
        export_name: "OX_ABS_Q",
        worksheet_name: "ox_ABS_Q",
        type_text: "BB",
        arg_names: "value",
        function_id: FUNC_ID_ABS,
        entry_kind: XllEntryKind::QUnaryNumber,
        u_lift_policy: None,
    },
    XllExportSpec {
        export_name: "OX_PI",
        worksheet_name: "ox_PI",
        type_text: "B",
        arg_names: "",
        function_id: FUNC_ID_PI,
        entry_kind: XllEntryKind::QNullaryNumber,
        u_lift_policy: None,
    },
];

pub fn render_export_specs_csv() -> String {
    let mut out = String::from(
        "export_name,worksheet_name,type_text,arg_names,function_id,entry_kind,u_lift_policy\n",
    );
    for spec in OXFUNC_XLL_EXPORT_SPECS {
        let entry_kind = match spec.entry_kind {
            XllEntryKind::UUnary => "u_unary",
            XllEntryKind::QUnaryNumber => "q_unary_number",
            XllEntryKind::QNullaryNumber => "q_nullary_number",
        };
        let u_lift = match spec.u_lift_policy {
            Some(XllULiftPolicy::ScalarOnly) => "scalar_only",
            Some(XllULiftPolicy::UnaryScalarOrArrayElementwise) => {
                "unary_scalar_or_array_elementwise"
            }
            None => "",
        };
        out.push_str(&format!(
            "{},{},{},{},{},{},{}\n",
            spec.export_name,
            spec.worksheet_name,
            spec.type_text,
            spec.arg_names,
            spec.function_id,
            entry_kind,
            u_lift
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
        for spec in OXFUNC_XLL_EXPORT_SPECS {
            assert!(exports.insert(spec.export_name));
            assert!(worksheets.insert(spec.worksheet_name));
        }
    }

    #[test]
    fn csv_header_and_row_count_are_stable() {
        let csv = render_export_specs_csv();
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(
            lines.first().copied(),
            Some(
                "export_name,worksheet_name,type_text,arg_names,function_id,entry_kind,u_lift_policy"
            )
        );
        assert_eq!(lines.len(), OXFUNC_XLL_EXPORT_SPECS.len() + 1);
    }
}
