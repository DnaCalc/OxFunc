use crate::functions::a1_refs::{A1ReferenceNotation, format_relative_target, parse_a1_reference};
use crate::value::{
    ArrayCellValue, EvalArray, EvalValue, ReferenceKind, ReferenceLike, WorksheetErrorCode,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolverCapabilities {
    pub allow_eval_time_deref: bool,
    pub allow_three_d_refs: bool,
    pub allow_structured_refs: bool,
    pub allow_spill_anchor_refs: bool,
    pub allow_external_refs: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallerContext {
    pub prefix: Option<String>,
    pub row: usize,
    pub col: usize,
}

impl ResolverCapabilities {
    pub const fn permissive_local() -> Self {
        Self {
            allow_eval_time_deref: true,
            allow_three_d_refs: true,
            allow_structured_refs: true,
            allow_spill_anchor_refs: true,
            allow_external_refs: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefResolutionError {
    EvalTimeDerefNotAllowed,
    CapabilityDenied {
        kind: ReferenceKind,
        capability: &'static str,
    },
    UnresolvedReference {
        target: String,
    },
    ProviderFailure {
        detail: String,
    },
}

pub trait ReferenceResolver {
    fn capabilities(&self) -> ResolverCapabilities;
    fn resolve_reference(&self, reference: &ReferenceLike)
    -> Result<EvalValue, RefResolutionError>;

    fn caller_context(&self) -> Option<CallerContext> {
        None
    }
}

pub fn normalize_reference(reference: &ReferenceLike) -> ReferenceLike {
    reference.clone().normalized()
}

fn collect_multi_area_member_references(
    reference: &ReferenceLike,
) -> Result<Option<Vec<ReferenceLike>>, RefResolutionError> {
    if !matches!(reference.kind, ReferenceKind::MultiArea) {
        return Ok(None);
    }

    let parts =
        reference
            .multi_area_targets()
            .ok_or_else(|| RefResolutionError::ProviderFailure {
                detail: "invalid_multi_area_reference".to_string(),
            })?;

    let mut shared_prefix = None;
    let mut members = Vec::new();
    for part in parts {
        collect_multi_area_member_parts(&part, &mut shared_prefix, &mut members)?;
    }

    if members.is_empty() {
        return Err(RefResolutionError::ProviderFailure {
            detail: "multi_area_reference_empty".to_string(),
        });
    }

    Ok(Some(members))
}

fn collect_multi_area_member_parts(
    target: &str,
    shared_prefix: &mut Option<String>,
    members: &mut Vec<ReferenceLike>,
) -> Result<(), RefResolutionError> {
    if let Some(parts) = ReferenceLike::new(ReferenceKind::MultiArea, target).multi_area_targets() {
        for part in parts {
            collect_multi_area_member_parts(&part, shared_prefix, members)?;
        }
        return Ok(());
    }

    let parsed = parse_a1_reference(target).ok_or_else(|| RefResolutionError::ProviderFailure {
        detail: "unsupported_multi_area_reference_part".to_string(),
    })?;

    if !matches!(parsed.notation, A1ReferenceNotation::Rect) {
        return Err(RefResolutionError::ProviderFailure {
            detail: "unsupported_multi_area_reference_part".to_string(),
        });
    }

    match shared_prefix {
        Some(existing) if parsed.prefix.as_ref() != Some(existing) => {
            return Err(RefResolutionError::ProviderFailure {
                detail: "mixed_sheet_multi_area".to_string(),
            });
        }
        None => *shared_prefix = parsed.prefix.clone(),
        _ => {}
    }

    let target =
        format_relative_target(&parsed).ok_or_else(|| RefResolutionError::ProviderFailure {
            detail: "unsupported_multi_area_reference_part".to_string(),
        })?;
    let kind = if parsed.width() == 1 && parsed.height() == 1 {
        ReferenceKind::A1
    } else {
        ReferenceKind::Area
    };
    members.push(ReferenceLike::new(kind, target).normalized());
    Ok(())
}

fn append_materialized_value_cells(cells: &mut Vec<ArrayCellValue>, value: EvalValue) {
    match value {
        EvalValue::Array(array) => cells.extend(array.iter_row_major().cloned()),
        EvalValue::Number(number) => cells.push(ArrayCellValue::Number(number)),
        EvalValue::Text(text) => cells.push(ArrayCellValue::Text(text)),
        EvalValue::Logical(value) => cells.push(ArrayCellValue::Logical(value)),
        EvalValue::Error(code) => cells.push(ArrayCellValue::Error(code)),
        EvalValue::Reference(_) | EvalValue::Lambda(_) => {
            cells.push(ArrayCellValue::Error(WorksheetErrorCode::Value))
        }
    }
}

fn materialize_multi_area_eval_value(
    resolver: &impl ReferenceResolver,
    reference: &ReferenceLike,
) -> Result<EvalValue, RefResolutionError> {
    let Some(members) = collect_multi_area_member_references(reference)? else {
        return resolver.resolve_reference(reference);
    };

    let mut cells = Vec::new();
    for member in members {
        let value = resolve_eval_value(resolver, &member)?;
        append_materialized_value_cells(&mut cells, value);
    }

    let array =
        EvalArray::from_rows(vec![cells]).ok_or_else(|| RefResolutionError::ProviderFailure {
            detail: "multi_area_reference_shape_invalid".to_string(),
        })?;
    Ok(EvalValue::Array(array))
}

pub fn resolve_eval_value(
    resolver: &impl ReferenceResolver,
    reference: &ReferenceLike,
) -> Result<EvalValue, RefResolutionError> {
    let normalized = normalize_reference(reference);
    let caps = resolver.capabilities();

    if !caps.allow_eval_time_deref {
        return Err(RefResolutionError::EvalTimeDerefNotAllowed);
    }

    match normalized.kind {
        ReferenceKind::ThreeD if !caps.allow_three_d_refs => {
            return Err(RefResolutionError::CapabilityDenied {
                kind: normalized.kind,
                capability: "allow_three_d_refs",
            });
        }
        ReferenceKind::MultiArea => {}
        ReferenceKind::Structured if !caps.allow_structured_refs => {
            return Err(RefResolutionError::CapabilityDenied {
                kind: normalized.kind,
                capability: "allow_structured_refs",
            });
        }
        ReferenceKind::SpillAnchor if !caps.allow_spill_anchor_refs => {
            return Err(RefResolutionError::CapabilityDenied {
                kind: normalized.kind,
                capability: "allow_spill_anchor_refs",
            });
        }
        _ => {}
    }

    if !caps.allow_external_refs && normalized.target.contains('[') {
        return Err(RefResolutionError::CapabilityDenied {
            kind: normalized.kind,
            capability: "allow_external_refs",
        });
    }

    if matches!(normalized.kind, ReferenceKind::MultiArea) {
        return materialize_multi_area_eval_value(resolver, &normalized);
    }

    resolver.resolve_reference(&normalized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::{ArrayCellValue, EvalArray, EvalValue, ReferenceKind, ReferenceLike};
    use std::collections::BTreeMap;

    struct MockResolver {
        caps: ResolverCapabilities,
        resolved: Option<EvalValue>,
        by_target: BTreeMap<String, EvalValue>,
    }

    impl ReferenceResolver for MockResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            self.caps
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            if let Some(value) = self.by_target.get(&reference.target) {
                return Ok(value.clone());
            }
            match &self.resolved {
                Some(v) => Ok(v.clone()),
                None => Err(RefResolutionError::UnresolvedReference {
                    target: reference.target.clone(),
                }),
            }
        }
    }

    #[test]
    fn normalize_reference_trims_target() {
        let input = ReferenceLike {
            kind: ReferenceKind::A1,
            target: "  Sheet1!A1  ".to_string(),
        };
        let got = normalize_reference(&input);
        assert_eq!(got.target, "Sheet1!A1");
    }

    #[test]
    fn resolve_rejects_three_d_when_capability_disabled() {
        let resolver = MockResolver {
            caps: ResolverCapabilities {
                allow_eval_time_deref: true,
                allow_three_d_refs: false,
                allow_structured_refs: true,
                allow_spill_anchor_refs: true,
                allow_external_refs: false,
            },
            resolved: Some(EvalValue::Number(1.0)),
            by_target: BTreeMap::new(),
        };

        let input = ReferenceLike {
            kind: ReferenceKind::ThreeD,
            target: "Sheet1:Sheet2!A1".to_string(),
        };

        let got = resolve_eval_value(&resolver, &input);
        assert_eq!(
            got,
            Err(RefResolutionError::CapabilityDenied {
                kind: ReferenceKind::ThreeD,
                capability: "allow_three_d_refs"
            })
        );
    }

    #[test]
    fn resolve_rejects_external_reference_when_disallowed() {
        let resolver = MockResolver {
            caps: ResolverCapabilities::permissive_local(),
            resolved: Some(EvalValue::Number(1.0)),
            by_target: BTreeMap::new(),
        };
        let input = ReferenceLike {
            kind: ReferenceKind::A1,
            target: "[External.xlsx]Sheet1!A1".to_string(),
        };

        let got = resolve_eval_value(&resolver, &input);
        assert_eq!(
            got,
            Err(RefResolutionError::CapabilityDenied {
                kind: ReferenceKind::A1,
                capability: "allow_external_refs"
            })
        );
    }

    #[test]
    fn resolve_passes_normalized_reference_to_provider() {
        let resolver = MockResolver {
            caps: ResolverCapabilities::permissive_local(),
            resolved: Some(EvalValue::Number(3.0)),
            by_target: BTreeMap::new(),
        };

        let input = ReferenceLike {
            kind: ReferenceKind::A1,
            target: "  A1 ".to_string(),
        };

        let got = resolve_eval_value(&resolver, &input);
        assert_eq!(got, Ok(EvalValue::Number(3.0)));
    }

    #[test]
    fn resolve_materializes_same_sheet_multi_area_in_member_order() {
        let mut by_target = BTreeMap::new();
        by_target.insert(
            "Alpha!A1:A2".to_string(),
            EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(7.0)],
                    vec![ArrayCellValue::Number(11.0)],
                ])
                .unwrap(),
            ),
        );
        by_target.insert("Alpha!B2".to_string(), EvalValue::Number(13.0));
        let resolver = MockResolver {
            caps: ResolverCapabilities::permissive_local(),
            resolved: None,
            by_target,
        };

        let got = resolve_eval_value(
            &resolver,
            &ReferenceLike::new(ReferenceKind::MultiArea, "(Alpha!A1:A2,Alpha!B2)"),
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(7.0),
                    ArrayCellValue::Number(11.0),
                    ArrayCellValue::Number(13.0),
                ]])
                .unwrap()
            ))
        );
    }

    #[test]
    fn resolve_preserves_error_cells_while_materializing_multi_area() {
        let mut by_target = BTreeMap::new();
        by_target.insert(
            "A1:A2".to_string(),
            EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(7.0)],
                    vec![ArrayCellValue::Error(WorksheetErrorCode::Div0)],
                ])
                .unwrap(),
            ),
        );
        by_target.insert("C1".to_string(), EvalValue::Number(13.0));
        let resolver = MockResolver {
            caps: ResolverCapabilities::permissive_local(),
            resolved: None,
            by_target,
        };

        let got = resolve_eval_value(
            &resolver,
            &ReferenceLike::new(ReferenceKind::MultiArea, "(A1:A2,C1)"),
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(7.0),
                    ArrayCellValue::Error(WorksheetErrorCode::Div0),
                    ArrayCellValue::Number(13.0),
                ]])
                .unwrap()
            ))
        );
    }

    #[test]
    fn resolve_rejects_mixed_sheet_multi_area_materialization() {
        let resolver = MockResolver {
            caps: ResolverCapabilities::permissive_local(),
            resolved: None,
            by_target: BTreeMap::new(),
        };

        let got = resolve_eval_value(
            &resolver,
            &ReferenceLike::new(ReferenceKind::MultiArea, "(Alpha!A1:A2,Beta!B2)"),
        );
        assert_eq!(
            got,
            Err(RefResolutionError::ProviderFailure {
                detail: "mixed_sheet_multi_area".to_string(),
            })
        );
    }
}
