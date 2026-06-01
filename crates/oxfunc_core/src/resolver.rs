use crate::functions::a1_refs::{format_relative_target, parse_a1_reference, A1ReferenceNotation};
use crate::value::{
    ArrayCellValue, EvalArray, EvalValue, ReferenceIdentity, ReferenceKind, ReferenceLike,
    ReferenceSystemId, WorksheetErrorCode,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceTextResolutionMode {
    Indirect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceTextResolutionRequest {
    pub text: String,
    pub mode: ReferenceTextResolutionMode,
    pub a1_style: Option<bool>,
    pub caller_context: Option<CallerContext>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceTextResolutionError {
    Unsupported,
    InvalidReferenceText { text: String },
    ProviderFailure { detail: String },
}

pub trait ReferenceTextResolver {
    fn resolve_reference_text(
        &self,
        request: &ReferenceTextResolutionRequest,
    ) -> Result<ReferenceLike, ReferenceTextResolutionError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceSystemOperation {
    Dereference,
    EnumerateValues,
    ResolveText,
    Facts,
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
    fn dereference(
        &self,
        request: &ReferenceDereferenceRequest,
    ) -> Result<EvalValue, ReferenceSystemError>;

    fn enumerate_values(
        &self,
        _request: &ReferenceEnumerationRequest,
    ) -> Result<Option<ResolvedReferenceValues>, ReferenceSystemError> {
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
}

impl<T: ReferenceSystemProvider + ?Sized> ReferenceSystemProvider for &T {
    fn dereference(
        &self,
        request: &ReferenceDereferenceRequest,
    ) -> Result<EvalValue, ReferenceSystemError> {
        (**self).dereference(request)
    }

    fn enumerate_values(
        &self,
        request: &ReferenceEnumerationRequest,
    ) -> Result<Option<ResolvedReferenceValues>, ReferenceSystemError> {
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

// W099 migration-only adapter for legacy FEC call sites. W099-009 moves
// reference-sensitive functions to ReferenceSystemProvider directly, and
// W099-015 owns deleting this adapter with the old resolver traits.
pub struct LegacyReferenceSystemProvider<'a> {
    resolver: &'a dyn ReferenceResolver,
    text_resolver: Option<&'a dyn ReferenceTextResolver>,
}

impl<'a> LegacyReferenceSystemProvider<'a> {
    pub fn new(resolver: &'a dyn ReferenceResolver) -> Self {
        Self {
            resolver,
            text_resolver: None,
        }
    }

    pub fn with_text_resolver(mut self, text_resolver: &'a dyn ReferenceTextResolver) -> Self {
        self.text_resolver = Some(text_resolver);
        self
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

pub trait ReferenceResolver {
    fn capabilities(&self) -> ResolverCapabilities;
    fn resolve_reference(&self, reference: &ReferenceLike)
        -> Result<EvalValue, RefResolutionError>;

    fn resolve_reference_values(
        &self,
        _reference: &ReferenceLike,
    ) -> Result<Option<ResolvedReferenceValues>, RefResolutionError> {
        Ok(None)
    }

    fn caller_context(&self) -> Option<CallerContext> {
        None
    }
}

impl<T: ReferenceResolver + ?Sized> ReferenceResolver for &T {
    fn capabilities(&self) -> ResolverCapabilities {
        (**self).capabilities()
    }

    fn resolve_reference(
        &self,
        reference: &ReferenceLike,
    ) -> Result<EvalValue, RefResolutionError> {
        (**self).resolve_reference(reference)
    }

    fn resolve_reference_values(
        &self,
        reference: &ReferenceLike,
    ) -> Result<Option<ResolvedReferenceValues>, RefResolutionError> {
        (**self).resolve_reference_values(reference)
    }

    fn caller_context(&self) -> Option<CallerContext> {
        (**self).caller_context()
    }
}

impl ReferenceSystemProvider for LegacyReferenceSystemProvider<'_> {
    fn dereference(
        &self,
        request: &ReferenceDereferenceRequest,
    ) -> Result<EvalValue, ReferenceSystemError> {
        self.resolver
            .resolve_reference(&request.reference)
            .map_err(|error| {
                reference_resolution_error_to_system_error(
                    error,
                    ReferenceSystemOperation::Dereference,
                    &request.reference,
                )
            })
    }

    fn enumerate_values(
        &self,
        request: &ReferenceEnumerationRequest,
    ) -> Result<Option<ResolvedReferenceValues>, ReferenceSystemError> {
        self.resolver
            .resolve_reference_values(&request.reference)
            .map_err(|error| {
                reference_resolution_error_to_system_error(
                    error,
                    ReferenceSystemOperation::EnumerateValues,
                    &request.reference,
                )
            })
    }

    fn resolve_text(
        &self,
        request: &ReferenceTextResolveRequest,
    ) -> Result<ReferenceLike, ReferenceSystemError> {
        let Some(text_resolver) = self.text_resolver else {
            return Err(ReferenceSystemError::Unsupported {
                operation: ReferenceSystemOperation::ResolveText,
            });
        };
        text_resolver
            .resolve_reference_text(&ReferenceTextResolutionRequest {
                text: request.text.clone(),
                mode: request.mode,
                a1_style: request.a1_style,
                caller_context: request.caller_context.clone(),
            })
            .map_err(reference_text_resolution_error_to_system_error)
    }
}

fn reference_resolution_error_to_system_error(
    error: RefResolutionError,
    operation: ReferenceSystemOperation,
    reference: &ReferenceLike,
) -> ReferenceSystemError {
    match error {
        RefResolutionError::EvalTimeDerefNotAllowed => ReferenceSystemError::CapabilityDenied {
            operation,
            detail: "eval_time_deref_not_allowed".to_string(),
        },
        RefResolutionError::CapabilityDenied { capability, .. } => {
            ReferenceSystemError::CapabilityDenied {
                operation,
                detail: capability.to_string(),
            }
        }
        RefResolutionError::UnresolvedReference { .. } => {
            ReferenceSystemError::UnresolvedReference {
                system: reference.system.clone(),
                identity_class: reference_identity_class(reference),
            }
        }
        RefResolutionError::ProviderFailure { detail } => {
            ReferenceSystemError::ProviderFailure { detail }
        }
    }
}

fn reference_text_resolution_error_to_system_error(
    error: ReferenceTextResolutionError,
) -> ReferenceSystemError {
    match error {
        ReferenceTextResolutionError::Unsupported => ReferenceSystemError::Unsupported {
            operation: ReferenceSystemOperation::ResolveText,
        },
        ReferenceTextResolutionError::InvalidReferenceText { text } => {
            ReferenceSystemError::InvalidReferenceText { text }
        }
        ReferenceTextResolutionError::ProviderFailure { detail } => {
            ReferenceSystemError::ProviderFailure { detail }
        }
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
    resolver: &(impl ReferenceResolver + ?Sized),
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
    resolver: &(impl ReferenceResolver + ?Sized),
    reference: &ReferenceLike,
) -> Result<EvalValue, RefResolutionError> {
    let normalized = normalize_reference(reference);
    ensure_reference_resolution_allowed(&normalized, resolver.capabilities())?;

    if matches!(normalized.kind, ReferenceKind::MultiArea) {
        return materialize_multi_area_eval_value(resolver, &normalized);
    }

    resolver.resolve_reference(&normalized)
}

pub fn resolve_reference_values(
    resolver: &(impl ReferenceResolver + ?Sized),
    reference: &ReferenceLike,
) -> Result<Option<ResolvedReferenceValues>, RefResolutionError> {
    let normalized = normalize_reference(reference);
    ensure_reference_resolution_allowed(&normalized, resolver.capabilities())?;

    if matches!(normalized.kind, ReferenceKind::MultiArea) {
        return Ok(None);
    }

    resolver.resolve_reference_values(&normalized)
}

pub fn materialize_resolved_reference_values(
    values: &ResolvedReferenceValues,
) -> Result<EvalArray, RefResolutionError> {
    let shape = crate::value::ArrayShape {
        rows: values.declared_extent.rows,
        cols: values.declared_extent.cols,
    };
    if shape.rows == 0 || shape.cols == 0 {
        return Err(RefResolutionError::ProviderFailure {
            detail: "sparse_reference_shape_invalid".to_string(),
        });
    }
    let cell_count =
        shape
            .rows
            .checked_mul(shape.cols)
            .ok_or_else(|| RefResolutionError::ProviderFailure {
                detail: "sparse_reference_shape_invalid".to_string(),
            })?;

    let mut cells = vec![ArrayCellValue::EmptyCell; cell_count];
    for cell in &values.defined_cells {
        if cell.row == 0
            || cell.col == 0
            || cell.row > values.declared_extent.rows
            || cell.col > values.declared_extent.cols
        {
            return Err(RefResolutionError::ProviderFailure {
                detail: "sparse_reference_cell_out_of_bounds".to_string(),
            });
        }
        let index = (cell.row - 1)
            .checked_mul(shape.cols)
            .and_then(|base| base.checked_add(cell.col - 1))
            .ok_or_else(|| RefResolutionError::ProviderFailure {
                detail: "sparse_reference_shape_invalid".to_string(),
            })?;
        cells[index] = cell.value.clone();
    }

    EvalArray::new(shape, cells).ok_or_else(|| RefResolutionError::ProviderFailure {
        detail: "sparse_reference_shape_invalid".to_string(),
    })
}

fn ensure_reference_resolution_allowed(
    normalized: &ReferenceLike,
    caps: ResolverCapabilities,
) -> Result<(), RefResolutionError> {
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

    if !caps.allow_external_refs
        && !matches!(normalized.kind, ReferenceKind::Structured)
        && normalized.target.contains('[')
    {
        Err(RefResolutionError::CapabilityDenied {
            kind: normalized.kind,
            capability: "allow_external_refs",
        })
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

    struct MockTextResolver;

    impl ReferenceTextResolver for MockTextResolver {
        fn resolve_reference_text(
            &self,
            request: &ReferenceTextResolutionRequest,
        ) -> Result<ReferenceLike, ReferenceTextResolutionError> {
            Ok(ReferenceLike::new(ReferenceKind::A1, request.text.clone()))
        }
    }

    struct NativeProvider {
        deref: EvalValue,
    }

    impl ReferenceSystemProvider for NativeProvider {
        fn dereference(
            &self,
            request: &ReferenceDereferenceRequest,
        ) -> Result<EvalValue, ReferenceSystemError> {
            assert_eq!(request.reference.system, ReferenceSystemId::excel_grid_v1());
            Ok(self.deref.clone())
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
    fn legacy_reference_system_provider_adapts_dereference() {
        let resolver = MockResolver {
            caps: ResolverCapabilities::permissive_local(),
            resolved: Some(EvalValue::Number(42.0)),
            by_target: BTreeMap::new(),
        };
        let provider = LegacyReferenceSystemProvider::new(&resolver);

        let got = provider.dereference(&ReferenceDereferenceRequest {
            reference: ReferenceLike::new(ReferenceKind::A1, "A1"),
        });

        assert_eq!(got, Ok(EvalValue::Number(42.0)));
    }

    #[test]
    fn legacy_reference_system_provider_maps_unresolved_to_typed_identity_error() {
        let resolver = MockResolver {
            caps: ResolverCapabilities::permissive_local(),
            resolved: None,
            by_target: BTreeMap::new(),
        };
        let provider = LegacyReferenceSystemProvider::new(&resolver);

        let got = provider.dereference(&ReferenceDereferenceRequest {
            reference: ReferenceLike::opaque(
                ReferenceSystemId("host.opaque.v1".to_string()),
                ReferenceHandle {
                    id: ReferenceHandleId::from_bytes([1]),
                },
                Some(ReferenceDisplay {
                    text: ExcelText::from_interop_assignment("display only"),
                }),
            ),
        });

        assert_eq!(
            got,
            Err(ReferenceSystemError::UnresolvedReference {
                system: ReferenceSystemId("host.opaque.v1".to_string()),
                identity_class: ReferenceIdentityClass::Opaque,
            })
        );
    }

    #[test]
    fn legacy_reference_system_provider_adapts_text_resolution() {
        let resolver = MockResolver {
            caps: ResolverCapabilities::permissive_local(),
            resolved: Some(EvalValue::Number(1.0)),
            by_target: BTreeMap::new(),
        };
        let text_resolver = MockTextResolver;
        let provider =
            LegacyReferenceSystemProvider::new(&resolver).with_text_resolver(&text_resolver);

        let got = provider
            .resolve_text(&ReferenceTextResolveRequest {
                text: "B2".to_string(),
                mode: ReferenceTextResolutionMode::Indirect,
                a1_style: Some(true),
                caller_context: None,
            })
            .unwrap();

        assert_eq!(got.target, "B2");
        assert_eq!(
            reference_facts(&got).identity_class,
            ReferenceIdentityClass::Textual
        );
    }

    #[test]
    fn native_reference_system_provider_can_dereference_textual_identity() {
        let provider = NativeProvider {
            deref: EvalValue::Text(ExcelText::from_interop_assignment("ok")),
        };

        let got = provider.dereference(&ReferenceDereferenceRequest {
            reference: ReferenceLike::new(ReferenceKind::A1, "A1"),
        });

        assert_eq!(
            got,
            Ok(EvalValue::Text(ExcelText::from_interop_assignment("ok")))
        );
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

        let input = ReferenceLike::new(ReferenceKind::ThreeD, "Sheet1:Sheet2!A1".to_string());

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
        let input = ReferenceLike::new(ReferenceKind::A1, "[External.xlsx]Sheet1!A1".to_string());

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

        let input = ReferenceLike::new(ReferenceKind::A1, "  A1 ".to_string());

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
