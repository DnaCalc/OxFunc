use crate::value::{
    ArrayCellValue, EvalArray, EvalValue, ReferenceIdentity, ReferenceKind, ReferenceLike,
    ReferenceSystemId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReferenceSystemCapabilities {
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

impl ReferenceSystemCapabilities {
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
pub enum ReferenceResolutionError {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceTextResolutionMode {
    Indirect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceSystemOperation {
    Dereference,
    EnumerateValues,
    ResolveText,
    Facts,
    Transform,
    Compose,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceIdentityClass {
    Textual,
    Opaque,
    Composite,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceSystemError {
    CapabilityDenied {
        operation: ReferenceSystemOperation,
        detail: String,
    },
    Unsupported {
        operation: ReferenceSystemOperation,
    },
    InvalidReferenceText {
        text: String,
    },
    UnresolvedReference {
        system: ReferenceSystemId,
        identity_class: ReferenceIdentityClass,
    },
    ProviderFailure {
        detail: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceDereferenceRequest {
    pub reference: ReferenceLike,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceEnumerationRequest {
    pub reference: ReferenceLike,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceTextResolveRequest {
    pub text: String,
    pub mode: ReferenceTextResolutionMode,
    pub a1_style: Option<bool>,
    pub caller_context: Option<CallerContext>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceFactsRequest {
    pub reference: ReferenceLike,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceTrimMode {
    Leading,
    Trailing,
    Both,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceTransformKind {
    Index {
        row: usize,
        col: usize,
        area: usize,
    },
    Offset {
        row_offset: i64,
        col_offset: i64,
        height: Option<usize>,
        width: Option<usize>,
    },
    Trim {
        mode: ReferenceTrimMode,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceTransformRequest {
    pub reference: ReferenceLike,
    pub transform: ReferenceTransformKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceComposeOperation {
    Range,
    Intersection,
    Union,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceComposeRequest {
    pub lhs: ReferenceLike,
    pub rhs: ReferenceLike,
    pub operation: ReferenceComposeOperation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceFacts {
    pub system: ReferenceSystemId,
    pub identity_class: ReferenceIdentityClass,
    pub textual_kind: Option<ReferenceKind>,
    pub display_text: Option<String>,
    pub legacy_kind: ReferenceKind,
    pub legacy_target: String,
}

pub trait ReferenceSystemProvider {
    fn capabilities(&self) -> ReferenceSystemCapabilities {
        ReferenceSystemCapabilities::permissive_local()
    }

    fn dereference(
        &self,
        _request: &ReferenceDereferenceRequest,
    ) -> Result<EvalValue, crate::resolver::ReferenceResolutionError> {
        Err(crate::resolver::ReferenceResolutionError::ProviderFailure {
            detail: "unsupported_reference_system_operation:Dereference".to_string(),
        })
    }

    fn enumerate_values(
        &self,
        _request: &ReferenceEnumerationRequest,
    ) -> Result<Option<ResolvedReferenceValues>, ReferenceResolutionError> {
        Ok(None)
    }

    fn resolve_text(
        &self,
        _request: &ReferenceTextResolveRequest,
    ) -> Result<ReferenceLike, ReferenceSystemError> {
        Err(ReferenceSystemError::Unsupported {
            operation: ReferenceSystemOperation::ResolveText,
        })
    }

    fn facts(
        &self,
        request: &ReferenceFactsRequest,
    ) -> Result<ReferenceFacts, ReferenceSystemError> {
        Ok(reference_facts(&request.reference))
    }

    fn transform_reference(
        &self,
        _request: &ReferenceTransformRequest,
    ) -> Result<ReferenceLike, ReferenceSystemError> {
        Err(ReferenceSystemError::Unsupported {
            operation: ReferenceSystemOperation::Transform,
        })
    }

    fn compose_references(
        &self,
        _request: &ReferenceComposeRequest,
    ) -> Result<ReferenceLike, ReferenceSystemError> {
        Err(ReferenceSystemError::Unsupported {
            operation: ReferenceSystemOperation::Compose,
        })
    }

    fn caller_context(&self) -> Option<CallerContext> {
        None
    }
}

impl<T: ReferenceSystemProvider + ?Sized> ReferenceSystemProvider for &T {
    fn dereference(
        &self,
        request: &ReferenceDereferenceRequest,
    ) -> Result<EvalValue, crate::resolver::ReferenceResolutionError> {
        (**self).dereference(request)
    }

    fn enumerate_values(
        &self,
        request: &ReferenceEnumerationRequest,
    ) -> Result<Option<ResolvedReferenceValues>, ReferenceResolutionError> {
        (**self).enumerate_values(request)
    }

    fn resolve_text(
        &self,
        request: &ReferenceTextResolveRequest,
    ) -> Result<ReferenceLike, ReferenceSystemError> {
        (**self).resolve_text(request)
    }

    fn facts(
        &self,
        request: &ReferenceFactsRequest,
    ) -> Result<ReferenceFacts, ReferenceSystemError> {
        (**self).facts(request)
    }

    fn transform_reference(
        &self,
        request: &ReferenceTransformRequest,
    ) -> Result<ReferenceLike, ReferenceSystemError> {
        (**self).transform_reference(request)
    }

    fn compose_references(
        &self,
        request: &ReferenceComposeRequest,
    ) -> Result<ReferenceLike, ReferenceSystemError> {
        (**self).compose_references(request)
    }

    fn caller_context(&self) -> Option<CallerContext> {
        (**self).caller_context()
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NullReferenceSystemProvider;

pub static NULL_REFERENCE_SYSTEM_PROVIDER: NullReferenceSystemProvider =
    NullReferenceSystemProvider;

impl ReferenceSystemProvider for NullReferenceSystemProvider {
    fn dereference(
        &self,
        _request: &ReferenceDereferenceRequest,
    ) -> Result<EvalValue, crate::resolver::ReferenceResolutionError> {
        Err(crate::resolver::ReferenceResolutionError::ProviderFailure {
            detail: "unsupported_reference_system_operation:Dereference".to_string(),
        })
    }

    fn enumerate_values(
        &self,
        _request: &ReferenceEnumerationRequest,
    ) -> Result<Option<ResolvedReferenceValues>, ReferenceResolutionError> {
        Err(crate::resolver::ReferenceResolutionError::ProviderFailure {
            detail: "unsupported_reference_system_operation:EnumerateValues".to_string(),
        })
    }

    fn resolve_text(
        &self,
        _request: &ReferenceTextResolveRequest,
    ) -> Result<ReferenceLike, ReferenceSystemError> {
        Err(ReferenceSystemError::Unsupported {
            operation: ReferenceSystemOperation::ResolveText,
        })
    }

    fn facts(
        &self,
        _request: &ReferenceFactsRequest,
    ) -> Result<ReferenceFacts, ReferenceSystemError> {
        Err(ReferenceSystemError::Unsupported {
            operation: ReferenceSystemOperation::Facts,
        })
    }
}

pub fn reference_facts(reference: &ReferenceLike) -> ReferenceFacts {
    let identity_class = reference_identity_class(reference);
    let textual_kind = match &reference.identity {
        ReferenceIdentity::Textual(textual) => Some(textual.kind),
        ReferenceIdentity::Opaque(_) | ReferenceIdentity::Composite(_) => None,
    };
    ReferenceFacts {
        system: reference.system.clone(),
        identity_class,
        textual_kind,
        display_text: reference
            .display
            .as_ref()
            .map(|display| display.text.to_string_lossy()),
        legacy_kind: reference.kind,
        legacy_target: reference.target.clone(),
    }
}

pub fn reference_identity_class(reference: &ReferenceLike) -> ReferenceIdentityClass {
    match &reference.identity {
        ReferenceIdentity::Textual(_) => ReferenceIdentityClass::Textual,
        ReferenceIdentity::Opaque(_) => ReferenceIdentityClass::Opaque,
        ReferenceIdentity::Composite(_) => ReferenceIdentityClass::Composite,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedReferenceExtent {
    pub rows: usize,
    pub cols: usize,
}

impl ResolvedReferenceExtent {
    #[must_use]
    pub const fn new(rows: usize, cols: usize) -> Self {
        Self { rows, cols }
    }

    #[must_use]
    pub fn declared_cell_count(self) -> usize {
        self.rows.saturating_mul(self.cols)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedReferenceCell {
    pub row: usize,
    pub col: usize,
    pub value: ArrayCellValue,
}

impl ResolvedReferenceCell {
    #[must_use]
    pub fn new(row: usize, col: usize, value: ArrayCellValue) -> Self {
        Self { row, col, value }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedReferenceValues {
    pub declared_extent: ResolvedReferenceExtent,
    pub defined_cardinality: usize,
    pub defined_cells: Vec<ResolvedReferenceCell>,
    pub reader_identity: Option<String>,
}

impl ResolvedReferenceValues {
    #[must_use]
    pub fn new(
        declared_extent: ResolvedReferenceExtent,
        defined_cells: Vec<ResolvedReferenceCell>,
        reader_identity: Option<String>,
    ) -> Self {
        let defined_cardinality = defined_cells.len();
        Self {
            declared_extent,
            defined_cardinality,
            defined_cells,
            reader_identity,
        }
    }

    #[must_use]
    pub fn declared_cell_count(&self) -> usize {
        self.declared_extent.declared_cell_count()
    }
}

pub fn normalize_reference(reference: &ReferenceLike) -> ReferenceLike {
    reference.clone().normalized()
}

pub fn resolve_eval_value(
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    reference: &ReferenceLike,
) -> Result<EvalValue, crate::resolver::ReferenceResolutionError> {
    let normalized = normalize_reference(reference);
    ensure_reference_resolution_allowed(&normalized, resolver.capabilities())?;

    resolver.dereference(&ReferenceDereferenceRequest {
        reference: normalized,
    })
}

pub fn enumerate_reference_values(
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    reference: &ReferenceLike,
) -> Result<Option<ResolvedReferenceValues>, ReferenceResolutionError> {
    let normalized = normalize_reference(reference);
    ensure_reference_resolution_allowed(&normalized, resolver.capabilities())?;

    resolver.enumerate_values(&ReferenceEnumerationRequest {
        reference: normalized,
    })
}

pub fn materialize_resolved_reference_values(
    values: &ResolvedReferenceValues,
) -> Result<EvalArray, ReferenceResolutionError> {
    let shape = crate::value::ArrayShape {
        rows: values.declared_extent.rows,
        cols: values.declared_extent.cols,
    };
    if shape.rows == 0 || shape.cols == 0 {
        return Err(crate::resolver::ReferenceResolutionError::ProviderFailure {
            detail: "sparse_reference_shape_invalid".to_string(),
        });
    }
    let cell_count = shape.rows.checked_mul(shape.cols).ok_or_else(|| {
        crate::resolver::ReferenceResolutionError::ProviderFailure {
            detail: "sparse_reference_shape_invalid".to_string(),
        }
    })?;

    let mut cells = vec![ArrayCellValue::EmptyCell; cell_count];
    for cell in &values.defined_cells {
        if cell.row == 0
            || cell.col == 0
            || cell.row > values.declared_extent.rows
            || cell.col > values.declared_extent.cols
        {
            return Err(crate::resolver::ReferenceResolutionError::ProviderFailure {
                detail: "sparse_reference_cell_out_of_bounds".to_string(),
            });
        }
        let index = (cell.row - 1)
            .checked_mul(shape.cols)
            .and_then(|base| base.checked_add(cell.col - 1))
            .ok_or_else(
                || crate::resolver::ReferenceResolutionError::ProviderFailure {
                    detail: "sparse_reference_shape_invalid".to_string(),
                },
            )?;
        cells[index] = cell.value.clone();
    }

    EvalArray::new(shape, cells).ok_or_else(|| {
        crate::resolver::ReferenceResolutionError::ProviderFailure {
            detail: "sparse_reference_shape_invalid".to_string(),
        }
    })
}

fn ensure_reference_resolution_allowed(
    normalized: &ReferenceLike,
    caps: ReferenceSystemCapabilities,
) -> Result<(), ReferenceResolutionError> {
    if !caps.allow_eval_time_deref {
        return Err(crate::resolver::ReferenceResolutionError::EvalTimeDerefNotAllowed);
    }

    match normalized.kind {
        ReferenceKind::ThreeD if !caps.allow_three_d_refs => {
            return Err(
                crate::resolver::ReferenceResolutionError::CapabilityDenied {
                    kind: normalized.kind,
                    capability: "allow_three_d_refs",
                },
            );
        }
        ReferenceKind::MultiArea => {}
        ReferenceKind::Structured if !caps.allow_structured_refs => {
            return Err(
                crate::resolver::ReferenceResolutionError::CapabilityDenied {
                    kind: normalized.kind,
                    capability: "allow_structured_refs",
                },
            );
        }
        ReferenceKind::SpillAnchor if !caps.allow_spill_anchor_refs => {
            return Err(
                crate::resolver::ReferenceResolutionError::CapabilityDenied {
                    kind: normalized.kind,
                    capability: "allow_spill_anchor_refs",
                },
            );
        }
        _ => {}
    }

    if !caps.allow_external_refs
        && !matches!(normalized.kind, ReferenceKind::Structured)
        && normalized.target.contains('[')
    {
        Err(
            crate::resolver::ReferenceResolutionError::CapabilityDenied {
                kind: normalized.kind,
                capability: "allow_external_refs",
            },
        )
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::{
        ArrayCellValue, EvalArray, EvalValue, ExcelText, ReferenceDisplay, ReferenceHandle,
        ReferenceHandleId, ReferenceKind, ReferenceLike, ReferenceSystemId,
    };
    use std::collections::BTreeMap;

    struct MockResolver {
        caps: ReferenceSystemCapabilities,
        resolved: Option<EvalValue>,
        by_target: BTreeMap<String, EvalValue>,
    }

    impl ReferenceSystemProvider for MockResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            self.caps
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<EvalValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            if let Some(value) = self.by_target.get(&reference.target) {
                return Ok(value.clone());
            }
            match &self.resolved {
                Some(v) => Ok(v.clone()),
                None => Err(
                    crate::resolver::ReferenceResolutionError::UnresolvedReference {
                        target: reference.target.clone(),
                    },
                ),
            }
        }
    }

    #[test]
    fn normalize_reference_trims_target() {
        let input = ReferenceLike::new(ReferenceKind::A1, "  Sheet1!A1  ".to_string());
        let got = normalize_reference(&input);
        assert_eq!(got.target, "Sheet1!A1");
    }

    #[test]
    fn reference_system_facts_describe_textual_identity() {
        let reference = ReferenceLike::new(ReferenceKind::Area, "Sheet1!A1:B2").normalized();

        let facts = reference_facts(&reference);

        assert_eq!(facts.system, ReferenceSystemId::excel_grid_v1());
        assert_eq!(facts.identity_class, ReferenceIdentityClass::Textual);
        assert_eq!(facts.textual_kind, Some(ReferenceKind::Area));
        assert_eq!(facts.display_text, Some("Sheet1!A1:B2".to_string()));
        assert_eq!(facts.legacy_target, "Sheet1!A1:B2");
    }

    #[test]
    fn reference_system_facts_keep_opaque_identity_separate_from_display() {
        let reference = ReferenceLike::opaque(
            ReferenceSystemId("host.opaque.v1".to_string()),
            ReferenceHandle {
                id: ReferenceHandleId::from_bytes([7, 9]),
            },
            Some(ReferenceDisplay {
                text: ExcelText::from_interop_assignment("visible label"),
            }),
        );

        let facts = reference_facts(&reference);

        assert_eq!(facts.system.0, "host.opaque.v1");
        assert_eq!(facts.identity_class, ReferenceIdentityClass::Opaque);
        assert_eq!(facts.textual_kind, None);
        assert_eq!(facts.display_text, Some("visible label".to_string()));
        assert_eq!(facts.legacy_target, "visible label");
    }

    #[test]
    fn null_reference_system_provider_rejects_reference_capabilities() {
        let reference = ReferenceLike::new(ReferenceKind::A1, "A1");

        assert_eq!(
            NULL_REFERENCE_SYSTEM_PROVIDER.dereference(&ReferenceDereferenceRequest {
                reference: reference.clone(),
            }),
            Err(crate::resolver::ReferenceResolutionError::ProviderFailure {
                detail: "unsupported_reference_system_operation:Dereference".to_string(),
            })
        );
        assert_eq!(
            NULL_REFERENCE_SYSTEM_PROVIDER.enumerate_values(&ReferenceEnumerationRequest {
                reference: reference.clone(),
            }),
            Err(crate::resolver::ReferenceResolutionError::ProviderFailure {
                detail: "unsupported_reference_system_operation:EnumerateValues".to_string(),
            })
        );
        assert_eq!(
            NULL_REFERENCE_SYSTEM_PROVIDER.resolve_text(&ReferenceTextResolveRequest {
                text: "A1".to_string(),
                mode: ReferenceTextResolutionMode::Indirect,
                a1_style: Some(true),
                caller_context: None,
            }),
            Err(ReferenceSystemError::Unsupported {
                operation: ReferenceSystemOperation::ResolveText,
            })
        );
        assert_eq!(
            NULL_REFERENCE_SYSTEM_PROVIDER.facts(&ReferenceFactsRequest { reference }),
            Err(ReferenceSystemError::Unsupported {
                operation: ReferenceSystemOperation::Facts,
            })
        );
    }

    #[test]
    fn resolve_rejects_three_d_when_capability_disabled() {
        let resolver = MockResolver {
            caps: ReferenceSystemCapabilities {
                allow_eval_time_deref: true,
                allow_three_d_refs: false,
                allow_structured_refs: true,
                allow_spill_anchor_refs: true,
                allow_external_refs: false,
            },
            resolved: Some(EvalValue::Number(1.0)),
            by_target: BTreeMap::new(),
        };

        let input = ReferenceLike::new(ReferenceKind::ThreeD, "Sheet1:Sheet2!A1".to_string());

        let got = resolve_eval_value(&resolver, &input);
        assert_eq!(
            got,
            Err(
                crate::resolver::ReferenceResolutionError::CapabilityDenied {
                    kind: ReferenceKind::ThreeD,
                    capability: "allow_three_d_refs"
                }
            )
        );
    }

    #[test]
    fn resolve_rejects_external_reference_when_disallowed() {
        let resolver = MockResolver {
            caps: ReferenceSystemCapabilities::permissive_local(),
            resolved: Some(EvalValue::Number(1.0)),
            by_target: BTreeMap::new(),
        };
        let input = ReferenceLike::new(ReferenceKind::A1, "[External.xlsx]Sheet1!A1".to_string());

        let got = resolve_eval_value(&resolver, &input);
        assert_eq!(
            got,
            Err(
                crate::resolver::ReferenceResolutionError::CapabilityDenied {
                    kind: ReferenceKind::A1,
                    capability: "allow_external_refs"
                }
            )
        );
    }

    #[test]
    fn resolve_passes_normalized_reference_to_provider() {
        let resolver = MockResolver {
            caps: ReferenceSystemCapabilities::permissive_local(),
            resolved: Some(EvalValue::Number(3.0)),
            by_target: BTreeMap::new(),
        };

        let input = ReferenceLike::new(ReferenceKind::A1, "  A1 ".to_string());

        let got = resolve_eval_value(&resolver, &input);
        assert_eq!(got, Ok(EvalValue::Number(3.0)));
    }

    #[test]
    fn resolve_delegates_multi_area_references_to_provider() {
        let mut by_target = BTreeMap::new();
        by_target.insert(
            "(A1:A2,C1)".to_string(),
            EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(7.0),
                    ArrayCellValue::Error(crate::value::WorksheetErrorCode::Div0),
                    ArrayCellValue::Number(13.0),
                ]])
                .unwrap(),
            ),
        );
        let resolver = MockResolver {
            caps: ReferenceSystemCapabilities::permissive_local(),
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
                    ArrayCellValue::Error(crate::value::WorksheetErrorCode::Div0),
                    ArrayCellValue::Number(13.0),
                ]])
                .unwrap()
            ))
        );
    }
}
