use std::collections::BTreeMap;
use std::sync::OnceLock;

use crate::capability::{
    CapabilitySetMismatch, ensure_capability_superset, sorted_stable_key_join,
    webimage_producer_capability_set_keys,
};
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::registry_context_seed::registry_metadata_for_id;
use crate::registry_help_seed::{RegistryHelpSeed, registry_help_seed_for_id};
use crate::registry_signature_seed::{SignatureSeed, signature_seed_for_id};
use crate::value::ReferenceLike;
use crate::xll_export_specs;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionSource {
    BuiltIn,
    Udf {
        provenance: Option<String>,
        replaces_builtin: bool,
    },
}

impl FunctionSource {
    fn replaces_builtin(&self) -> bool {
        matches!(
            self,
            FunctionSource::Udf {
                replaces_builtin: true,
                ..
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParameterDescriptor {
    pub name: String,
    pub optional: bool,
    pub repeats: bool,
    /// Authored parameter help text. `None` means OxFunc has not yet frozen a
    /// parameter-description corpus for this parameter.
    pub short_description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureForm {
    pub signature_display: String,
    pub parameters: Vec<ParameterDescriptor>,
    pub trailing_repeats: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryFunctionMeta {
    pub function_id: String,
    pub arity: Arity,
    pub determinism: DeterminismClass,
    pub volatility: VolatilityClass,
    pub host_interaction: HostInteractionClass,
    pub thread_safety: ThreadSafetyClass,
    pub arg_preparation_profile: ArgPreparationProfile,
    pub coercion_lift_profile: CoercionLiftProfile,
    pub kernel_signature_class: KernelSignatureClass,
    pub fec_dependency_profile: FecDependencyProfile,
    pub surface_fec_dependency_profile: FecDependencyProfile,
    pub semantic_kernel_metadata: SemanticKernelMetadata,
    pub semantic_kernel_metadata_version: String,
    pub arg_admission_metadata: ArgAdmissionMetadata,
    pub arg_admission_metadata_version: String,
    pub rich_value_usage: RichValueUsage,
    pub producer_capability_set_keys: Vec<String>,
}

impl From<FunctionMeta> for RegistryFunctionMeta {
    fn from(meta: FunctionMeta) -> Self {
        let semantic_kernel_metadata = semantic_kernel_metadata_for_id(meta.function_id);
        let arg_admission_metadata =
            ArgAdmissionMetadata::from_arg_preparation_profile(meta.arg_preparation_profile);
        Self {
            function_id: meta.function_id.to_string(),
            arity: meta.arity,
            determinism: meta.determinism,
            volatility: meta.volatility,
            host_interaction: meta.host_interaction,
            thread_safety: meta.thread_safety,
            arg_preparation_profile: meta.arg_preparation_profile,
            coercion_lift_profile: meta.coercion_lift_profile,
            kernel_signature_class: meta.kernel_signature_class,
            fec_dependency_profile: meta.fec_dependency_profile,
            surface_fec_dependency_profile: meta.surface_fec_dependency_profile,
            semantic_kernel_metadata_version: semantic_kernel_metadata.version_key(),
            semantic_kernel_metadata,
            arg_admission_metadata_version: arg_admission_metadata.version_key(),
            arg_admission_metadata,
            rich_value_usage: rich_value_usage_for_id(meta.function_id),
            producer_capability_set_keys: producer_capability_set_keys_for_id(meta.function_id),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RichValueUsage {
    RichBlind,
    ProducesPresentation,
    ProducesRichObject,
    ProducesErrorMetadata,
}

impl RichValueUsage {
    pub const fn version_key(self) -> &'static str {
        match self {
            Self::RichBlind => "rich_value_usage.v1;rich_blind",
            Self::ProducesPresentation => "rich_value_usage.v1;produces_presentation",
            Self::ProducesRichObject => "rich_value_usage.v1;produces_rich_object",
            Self::ProducesErrorMetadata => "rich_value_usage.v1;produces_error_metadata",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticKernelMetadata {
    pub reduction_sensitive: bool,
    pub error_collapse_sensitive: bool,
    pub numerical_reduction_policy: Option<String>,
    pub error_algebra: Option<String>,
}

impl SemanticKernelMetadata {
    pub fn version_key(&self) -> String {
        format!(
            "semantic_kernel_metadata.v1;reduction_sensitive={};error_collapse_sensitive={};numerical_reduction_policy={};error_algebra={}",
            self.reduction_sensitive,
            self.error_collapse_sensitive,
            self.numerical_reduction_policy.as_deref().unwrap_or("none"),
            self.error_algebra.as_deref().unwrap_or("none")
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgAdmissionMetadata {
    ExistingArgPreparation {
        profile: ArgPreparationProfile,
    },
    RichArgAccepted {
        required_capability_set_keys: Vec<String>,
    },
    SparseRangeAccepted {
        extent_class: String,
        cardinality_class: String,
    },
}

impl ArgAdmissionMetadata {
    pub fn from_arg_preparation_profile(profile: ArgPreparationProfile) -> Self {
        Self::ExistingArgPreparation { profile }
    }

    pub fn version_key(&self) -> String {
        match self {
            Self::ExistingArgPreparation { profile } => {
                format!(
                    "arg_admission_metadata.v1;existing_arg_preparation={}",
                    arg_preparation_profile_key(*profile)
                )
            }
            Self::RichArgAccepted {
                required_capability_set_keys,
            } => {
                format!(
                    "arg_admission_metadata.v1;rich_arg_accepted={}",
                    sorted_stable_key_join(required_capability_set_keys)
                )
            }
            Self::SparseRangeAccepted {
                extent_class,
                cardinality_class,
            } => format!(
                "arg_admission_metadata.v1;sparse_range_accepted;extent_class={extent_class};cardinality_class={cardinality_class}"
            ),
        }
    }

    pub fn validate_producer_capabilities(
        &self,
        producer_capability_set_keys: &[String],
    ) -> Result<(), CapabilitySetMismatch> {
        match self {
            Self::RichArgAccepted {
                required_capability_set_keys,
            } => ensure_capability_superset(
                required_capability_set_keys,
                producer_capability_set_keys,
            ),
            Self::ExistingArgPreparation { .. } | Self::SparseRangeAccepted { .. } => Ok(()),
        }
    }
}

fn arg_preparation_profile_key(profile: ArgPreparationProfile) -> &'static str {
    match profile {
        ArgPreparationProfile::ValuesOnlyPreAdapter => "values_only_pre_adapter",
        ArgPreparationProfile::RefsVisibleInAdapter => "refs_visible_in_adapter",
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FunctionRegistryMetadata {
    pub lane_id: Option<String>,
    pub entry_kind: Option<String>,
    pub registration_source_kind: Option<String>,
    pub surface_stable_id: Option<String>,
    pub xlcall_builtin_symbol: Option<String>,
    pub xlcall_builtin_code: Option<String>,
    pub canonical_surface_name: Option<String>,
    pub name_resolution_table_ref: Option<String>,
    pub semantic_trait_profile_ref: Option<String>,
    pub gating_profile_ref: Option<String>,
    pub version_marker: Option<String>,
    pub category: Option<String>,
    pub interesting: Option<String>,
    pub metadata_status: Option<String>,
    pub special_interface_kind: Option<String>,
    pub admission_interface_kind: Option<String>,
    pub preparation_owner: Option<String>,
    pub runtime_boundary_kind: Option<String>,
    pub interface_contract_ref: Option<String>,
    pub source_catalog_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionEntry {
    pub meta: RegistryFunctionMeta,
    pub surface_name: String,
    pub display_signature: SignatureForm,
    pub registry_metadata: FunctionRegistryMetadata,
    /// Authored function help summary. `None` means OxFunc has not yet frozen
    /// a function-description corpus for this entry.
    pub short_description: Option<String>,
    /// Authored extended function help. `None` means OxFunc has not yet frozen
    /// a long-form help corpus for this entry.
    pub long_description: Option<String>,
    pub source: FunctionSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UdfSourceKind {
    XllRegisteredFunction,
    VbaPublicModuleFunction,
    JavaScriptCustomFunction,
    AutomationRegisteredFunction,
    RegisteredExternalBridge,
    HostRegisteredExternal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UdfReplacementPolicy {
    RejectOnCollision,
    ReplaceSameSourceRegistrationId,
    AllowBuiltinReplacement,
}

impl Default for UdfReplacementPolicy {
    fn default() -> Self {
        Self::RejectOnCollision
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UdfExecutionProfile {
    pub host_execution_profile_key: Option<String>,
    pub required_capability_set_keys: Vec<String>,
    pub async_invocation: bool,
    pub streaming: bool,
    pub cancellable: bool,
    pub externally_invalidated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UdfOpaqueRuntimeValue {
    ReferenceLike(ReferenceLike),
    HostReference {
        reference_family: String,
        opaque_reference_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UdfInvocationTargetDescriptor {
    Xll {
        module_path: Option<String>,
        export_name: String,
        type_text: Option<String>,
        register_id: Option<String>,
        opaque_runtime_values: Vec<UdfOpaqueRuntimeValue>,
    },
    Vba {
        project_ref: String,
        module_name: String,
        procedure_name: String,
        opaque_runtime_values: Vec<UdfOpaqueRuntimeValue>,
    },
    JavaScript {
        addin_id: String,
        namespace: Option<String>,
        custom_function_id: String,
        runtime_ref: Option<String>,
        opaque_runtime_values: Vec<UdfOpaqueRuntimeValue>,
    },
    Automation {
        prog_id: Option<String>,
        clsid: Option<String>,
        member: String,
        opaque_runtime_values: Vec<UdfOpaqueRuntimeValue>,
    },
    RegisteredExternal {
        stable_registration_id: String,
        opaque_runtime_values: Vec<UdfOpaqueRuntimeValue>,
    },
    HostOpaque {
        host_system: String,
        opaque_target_id: String,
        opaque_runtime_values: Vec<UdfOpaqueRuntimeValue>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UdfRegistrationRequest {
    pub stable_source_registration_id: String,
    pub function_id: String,
    pub surface_name: String,
    pub source_kind: UdfSourceKind,
    pub source_provenance: Option<String>,
    pub arity: Arity,
    pub parameters: Vec<ParameterDescriptor>,
    pub display_signature: Option<String>,
    pub determinism: DeterminismClass,
    pub volatility: VolatilityClass,
    pub host_interaction: HostInteractionClass,
    pub thread_safety: ThreadSafetyClass,
    pub arg_preparation_profile: ArgPreparationProfile,
    pub coercion_lift_profile: CoercionLiftProfile,
    pub kernel_signature_class: KernelSignatureClass,
    pub fec_dependency_profile: FecDependencyProfile,
    pub surface_fec_dependency_profile: FecDependencyProfile,
    pub short_description: Option<String>,
    pub long_description: Option<String>,
    pub category: Option<String>,
    pub execution_profile: Option<UdfExecutionProfile>,
    pub invocation_target: Option<UdfInvocationTargetDescriptor>,
    pub replacement_policy: UdfReplacementPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionRegistrySnapshotIdentity {
    pub snapshot_family: String,
    pub generation: u64,
    pub content_fingerprint: String,
}

impl FunctionRegistrySnapshotIdentity {
    pub fn stable_key(&self) -> String {
        format!(
            "{};generation={};fingerprint={}",
            self.snapshot_family, self.generation, self.content_fingerprint
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryChangeSet {
    pub previous_snapshot_identity: FunctionRegistrySnapshotIdentity,
    pub new_snapshot_identity: FunctionRegistrySnapshotIdentity,
    pub added_function_ids: Vec<String>,
    pub removed_function_ids: Vec<String>,
    pub replaced_function_ids: Vec<String>,
    pub changed_surface_names: Vec<String>,
    pub affected_source_registration_ids: Vec<String>,
    pub descriptor_only_mutation: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UdfRegistrationRejectionCode {
    MissingSourceRegistrationId,
    MissingFunctionId,
    MissingSurfaceName,
    InvalidArity,
    DuplicateFunctionId,
    BuiltInSurfaceCollision,
    UdfSurfaceCollision,
    SourceRegistrationCollision,
    UnknownFunctionId,
    BuiltInUnregister,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UdfRegistrationRejection {
    pub code: UdfRegistrationRejectionCode,
    pub function_id: Option<String>,
    pub surface_name: Option<String>,
    pub source_registration_id: Option<String>,
    pub snapshot_identity: FunctionRegistrySnapshotIdentity,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UdfRegistrationResult {
    Registered {
        function_id: String,
        change_set: RegistryChangeSet,
    },
    Replaced {
        function_id: String,
        change_set: RegistryChangeSet,
    },
    Rejected(UdfRegistrationRejection),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UdfUnregistrationResult {
    Unregistered {
        function_id: String,
        change_set: RegistryChangeSet,
    },
    NoOp {
        source_registration_id: Option<String>,
        snapshot_identity: FunctionRegistrySnapshotIdentity,
    },
    Rejected(UdfRegistrationRejection),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    BuiltInUnregister { function_id: String },
    DuplicateFunctionId { function_id: String },
    SurfaceNameCollision { surface_name: String },
    UnknownFunctionId { function_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionAvailability {
    Available,
    Unavailable { reason: String },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CapabilityOverlay {
    availability_by_function_id: BTreeMap<String, FunctionAvailability>,
}

impl CapabilityOverlay {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_availability(
        &mut self,
        function_id: impl Into<String>,
        availability: FunctionAvailability,
    ) {
        self.availability_by_function_id
            .insert(function_id.into(), availability);
    }

    pub fn deny_function_id(&mut self, function_id: impl Into<String>, reason: impl Into<String>) {
        self.set_availability(
            function_id,
            FunctionAvailability::Unavailable {
                reason: reason.into(),
            },
        );
    }

    pub fn availability_for(&self, function_id: &str) -> FunctionAvailability {
        self.availability_by_function_id
            .iter()
            .find(|(id, _)| id.eq_ignore_ascii_case(function_id))
            .map(|(_, availability)| availability.clone())
            .unwrap_or(FunctionAvailability::Available)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityScopedFunctionEntry<'a> {
    pub entry: &'a FunctionEntry,
    pub availability: FunctionAvailability,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionRegistry {
    entries: Vec<FunctionEntry>,
    snapshot_generation: u64,
    source_registration_ids_by_function_id: BTreeMap<String, String>,
    function_ids_by_source_registration_id: BTreeMap<String, String>,
    udf_invocation_targets_by_function_id: BTreeMap<String, UdfInvocationTargetDescriptor>,
}

impl FunctionRegistry {
    pub fn try_from_entries(entries: Vec<FunctionEntry>) -> Result<Self, RegistryError> {
        let mut registry = Self {
            entries: Vec::with_capacity(entries.len()),
            snapshot_generation: 0,
            source_registration_ids_by_function_id: BTreeMap::new(),
            function_ids_by_source_registration_id: BTreeMap::new(),
            udf_invocation_targets_by_function_id: BTreeMap::new(),
        };
        for entry in entries {
            registry.register_entry(entry)?;
        }
        Ok(registry)
    }

    pub fn built_ins() -> Self {
        let entries = xll_export_specs::function_catalog()
            .iter()
            .copied()
            .map(built_in_entry_from_meta)
            .collect();
        Self {
            entries,
            snapshot_generation: 0,
            source_registration_ids_by_function_id: BTreeMap::new(),
            function_ids_by_source_registration_id: BTreeMap::new(),
            udf_invocation_targets_by_function_id: BTreeMap::new(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &FunctionEntry> {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn lookup_by_surface_name(&self, name: &str) -> Option<&FunctionEntry> {
        self.entries
            .iter()
            .rev()
            .find(|entry| entry.surface_name.eq_ignore_ascii_case(name))
    }

    pub fn lookup_by_id(&self, function_id: &str) -> Option<&FunctionEntry> {
        self.entries
            .iter()
            .rev()
            .find(|entry| entry.meta.function_id.eq_ignore_ascii_case(function_id))
    }

    pub fn snapshot_identity(&self) -> FunctionRegistrySnapshotIdentity {
        FunctionRegistrySnapshotIdentity {
            snapshot_family: "oxfunc.function_registry.v1".to_string(),
            generation: self.snapshot_generation,
            content_fingerprint: stable_registry_fingerprint(self),
        }
    }

    pub fn udf_invocation_target_by_id(
        &self,
        function_id: &str,
    ) -> Option<&UdfInvocationTargetDescriptor> {
        self.udf_invocation_targets_by_function_id
            .iter()
            .find(|(id, _)| id.eq_ignore_ascii_case(function_id))
            .map(|(_, target)| target)
    }

    pub fn source_registration_id_for_function_id(&self, function_id: &str) -> Option<&str> {
        self.source_registration_ids_by_function_id
            .iter()
            .find(|(id, _)| id.eq_ignore_ascii_case(function_id))
            .map(|(_, source_registration_id)| source_registration_id.as_str())
    }

    pub fn register_udf_request(
        &mut self,
        request: UdfRegistrationRequest,
    ) -> UdfRegistrationResult {
        let previous_snapshot_identity = self.snapshot_identity();
        if request.stable_source_registration_id.trim().is_empty() {
            return UdfRegistrationResult::Rejected(typed_rejection(
                UdfRegistrationRejectionCode::MissingSourceRegistrationId,
                Some(request.function_id),
                Some(request.surface_name),
                None,
                previous_snapshot_identity,
                "stable source registration id is required",
            ));
        }
        if request.function_id.trim().is_empty() {
            return UdfRegistrationResult::Rejected(typed_rejection(
                UdfRegistrationRejectionCode::MissingFunctionId,
                None,
                Some(request.surface_name),
                Some(request.stable_source_registration_id),
                previous_snapshot_identity,
                "function id is required",
            ));
        }
        if request.surface_name.trim().is_empty() {
            return UdfRegistrationResult::Rejected(typed_rejection(
                UdfRegistrationRejectionCode::MissingSurfaceName,
                Some(request.function_id),
                None,
                Some(request.stable_source_registration_id),
                previous_snapshot_identity,
                "surface name is required",
            ));
        }
        if request.arity.min > request.arity.max {
            return UdfRegistrationResult::Rejected(typed_rejection(
                UdfRegistrationRejectionCode::InvalidArity,
                Some(request.function_id),
                Some(request.surface_name),
                Some(request.stable_source_registration_id),
                previous_snapshot_identity,
                "arity min must be less than or equal to arity max",
            ));
        }

        let existing_id_index = self.entry_index_by_id(&request.function_id);
        let existing_surface_index = self.entry_index_by_surface_name(&request.surface_name);
        let existing_source_function_id = self
            .function_ids_by_source_registration_id
            .iter()
            .find(|(source_id, _)| {
                source_id.eq_ignore_ascii_case(&request.stable_source_registration_id)
            })
            .map(|(_, function_id)| function_id.clone());

        if let Some(existing_source_function_id) = existing_source_function_id.as_ref()
            && !existing_source_function_id.eq_ignore_ascii_case(&request.function_id)
        {
            return UdfRegistrationResult::Rejected(typed_rejection(
                UdfRegistrationRejectionCode::SourceRegistrationCollision,
                Some(request.function_id),
                Some(request.surface_name),
                Some(request.stable_source_registration_id),
                previous_snapshot_identity,
                "source registration id already belongs to a different function id",
            ));
        }

        let replacing_same_source = existing_source_function_id
            .as_deref()
            .is_some_and(|existing| existing.eq_ignore_ascii_case(&request.function_id))
            && matches!(
                request.replacement_policy,
                UdfReplacementPolicy::ReplaceSameSourceRegistrationId
                    | UdfReplacementPolicy::AllowBuiltinReplacement
            );

        if let Some(index) = existing_id_index
            && matches!(self.entries[index].source, FunctionSource::BuiltIn)
        {
            return UdfRegistrationResult::Rejected(typed_rejection(
                UdfRegistrationRejectionCode::DuplicateFunctionId,
                Some(request.function_id),
                Some(request.surface_name),
                Some(request.stable_source_registration_id),
                previous_snapshot_identity,
                "built-in function id cannot be replaced by UDF registration",
            ));
        } else if existing_id_index.is_some() && !replacing_same_source {
            return UdfRegistrationResult::Rejected(typed_rejection(
                UdfRegistrationRejectionCode::DuplicateFunctionId,
                Some(request.function_id),
                Some(request.surface_name),
                Some(request.stable_source_registration_id),
                previous_snapshot_identity,
                "function id is already registered",
            ));
        }

        if let Some(index) = existing_surface_index {
            let surface_belongs_to_same_function = self.entries[index]
                .meta
                .function_id
                .eq_ignore_ascii_case(&request.function_id);
            match self.entries[index].source {
                FunctionSource::BuiltIn => {
                    if !matches!(
                        request.replacement_policy,
                        UdfReplacementPolicy::AllowBuiltinReplacement
                    ) {
                        return UdfRegistrationResult::Rejected(typed_rejection(
                            UdfRegistrationRejectionCode::BuiltInSurfaceCollision,
                            Some(request.function_id),
                            Some(request.surface_name),
                            Some(request.stable_source_registration_id),
                            previous_snapshot_identity,
                            "built-in surface names are protected by default",
                        ));
                    }
                }
                FunctionSource::Udf { .. } if !surface_belongs_to_same_function => {
                    return UdfRegistrationResult::Rejected(typed_rejection(
                        UdfRegistrationRejectionCode::UdfSurfaceCollision,
                        Some(request.function_id),
                        Some(request.surface_name),
                        Some(request.stable_source_registration_id),
                        previous_snapshot_identity,
                        "surface name is already registered by another UDF",
                    ));
                }
                FunctionSource::Udf { .. } => {}
            }
        }

        let function_id = request.function_id.clone();
        let surface_name = request.surface_name.clone();
        let source_registration_id = request.stable_source_registration_id.clone();
        let invocation_target = request.invocation_target.clone();
        let entry = udf_entry_from_request(request);

        let mut added_function_ids = Vec::new();
        let mut replaced_function_ids = Vec::new();
        let mut changed_surface_names = Vec::new();
        if let Some(index) = existing_id_index.filter(|_| replacing_same_source) {
            if !self.entries[index]
                .surface_name
                .eq_ignore_ascii_case(&surface_name)
            {
                changed_surface_names.push(self.entries[index].surface_name.clone());
                changed_surface_names.push(surface_name.clone());
            }
            self.entries[index] = entry;
            replaced_function_ids.push(function_id.clone());
        } else {
            self.entries.push(entry);
            added_function_ids.push(function_id.clone());
        }

        self.source_registration_ids_by_function_id
            .insert(function_id.clone(), source_registration_id.clone());
        self.function_ids_by_source_registration_id
            .insert(source_registration_id.clone(), function_id.clone());
        match invocation_target {
            Some(target) => {
                self.udf_invocation_targets_by_function_id
                    .insert(function_id.clone(), target);
            }
            None => {
                self.udf_invocation_targets_by_function_id
                    .remove(&function_id);
            }
        }
        self.snapshot_generation += 1;

        let change_set = RegistryChangeSet {
            previous_snapshot_identity,
            new_snapshot_identity: self.snapshot_identity(),
            added_function_ids,
            removed_function_ids: Vec::new(),
            replaced_function_ids,
            changed_surface_names,
            affected_source_registration_ids: vec![source_registration_id],
            descriptor_only_mutation: false,
        };

        if change_set.replaced_function_ids.is_empty() {
            UdfRegistrationResult::Registered {
                function_id,
                change_set,
            }
        } else {
            UdfRegistrationResult::Replaced {
                function_id,
                change_set,
            }
        }
    }

    pub fn register_udf(&mut self, entry: FunctionEntry) -> Result<(), RegistryError> {
        if self.lookup_by_id(&entry.meta.function_id).is_some() {
            return Err(RegistryError::DuplicateFunctionId {
                function_id: entry.meta.function_id.clone(),
            });
        }

        let surface_collision = self.lookup_by_surface_name(&entry.surface_name).is_some();
        if surface_collision && !entry.source.replaces_builtin() {
            return Err(RegistryError::SurfaceNameCollision {
                surface_name: entry.surface_name,
            });
        }

        self.entries.push(entry);
        self.snapshot_generation += 1;
        Ok(())
    }

    fn register_entry(&mut self, entry: FunctionEntry) -> Result<(), RegistryError> {
        if self.lookup_by_id(&entry.meta.function_id).is_some() {
            return Err(RegistryError::DuplicateFunctionId {
                function_id: entry.meta.function_id.clone(),
            });
        }

        if self.lookup_by_surface_name(&entry.surface_name).is_some() {
            return Err(RegistryError::SurfaceNameCollision {
                surface_name: entry.surface_name,
            });
        }

        self.entries.push(entry);
        Ok(())
    }

    pub fn unregister_udf_with_change_set(&mut self, function_id: &str) -> UdfUnregistrationResult {
        let previous_snapshot_identity = self.snapshot_identity();
        let Some(index) = self.entry_index_by_id(function_id) else {
            return UdfUnregistrationResult::Rejected(typed_rejection(
                UdfRegistrationRejectionCode::UnknownFunctionId,
                Some(function_id.to_string()),
                None,
                None,
                previous_snapshot_identity,
                "function id is not registered",
            ));
        };

        if matches!(self.entries[index].source, FunctionSource::BuiltIn) {
            return UdfUnregistrationResult::Rejected(typed_rejection(
                UdfRegistrationRejectionCode::BuiltInUnregister,
                Some(function_id.to_string()),
                Some(self.entries[index].surface_name.clone()),
                None,
                previous_snapshot_identity,
                "built-in entries cannot be unregistered as UDFs",
            ));
        }

        let removed = self.entries.remove(index);
        let removed_function_id = removed.meta.function_id;
        let source_registration_id = self
            .remove_udf_side_tables_for_function_id(&removed_function_id)
            .unwrap_or_else(|| removed_function_id.clone());
        self.snapshot_generation += 1;

        UdfUnregistrationResult::Unregistered {
            function_id: removed_function_id.clone(),
            change_set: RegistryChangeSet {
                previous_snapshot_identity,
                new_snapshot_identity: self.snapshot_identity(),
                added_function_ids: Vec::new(),
                removed_function_ids: vec![removed_function_id],
                replaced_function_ids: Vec::new(),
                changed_surface_names: Vec::new(),
                affected_source_registration_ids: vec![source_registration_id],
                descriptor_only_mutation: false,
            },
        }
    }

    pub fn unregister_udf_by_source_registration_id(
        &mut self,
        source_registration_id: &str,
    ) -> UdfUnregistrationResult {
        let Some(function_id) = self
            .function_ids_by_source_registration_id
            .iter()
            .find(|(id, _)| id.eq_ignore_ascii_case(source_registration_id))
            .map(|(_, function_id)| function_id.clone())
        else {
            return UdfUnregistrationResult::NoOp {
                source_registration_id: Some(source_registration_id.to_string()),
                snapshot_identity: self.snapshot_identity(),
            };
        };

        self.unregister_udf_with_change_set(&function_id)
    }

    pub fn descriptor_only_change_set(
        &self,
        affected_source_registration_ids: Vec<String>,
    ) -> RegistryChangeSet {
        let snapshot_identity = self.snapshot_identity();
        RegistryChangeSet {
            previous_snapshot_identity: snapshot_identity.clone(),
            new_snapshot_identity: snapshot_identity,
            added_function_ids: Vec::new(),
            removed_function_ids: Vec::new(),
            replaced_function_ids: Vec::new(),
            changed_surface_names: Vec::new(),
            affected_source_registration_ids,
            descriptor_only_mutation: true,
        }
    }

    pub fn unregister_udf(&mut self, function_id: &str) -> Result<(), RegistryError> {
        let Some(index) = self
            .entries
            .iter()
            .position(|entry| entry.meta.function_id.eq_ignore_ascii_case(function_id))
        else {
            return Err(RegistryError::UnknownFunctionId {
                function_id: function_id.to_string(),
            });
        };

        if matches!(self.entries[index].source, FunctionSource::BuiltIn) {
            return Err(RegistryError::BuiltInUnregister {
                function_id: function_id.to_string(),
            });
        }

        let removed = self.entries.remove(index);
        self.remove_udf_side_tables_for_function_id(&removed.meta.function_id);
        self.snapshot_generation += 1;
        Ok(())
    }

    pub fn with_capability_overlay<'a>(
        &'a self,
        overlay: &'a CapabilityOverlay,
    ) -> CapabilityScopedRegistry<'a> {
        CapabilityScopedRegistry {
            registry: self,
            overlay,
        }
    }

    fn entry_index_by_id(&self, function_id: &str) -> Option<usize> {
        self.entries
            .iter()
            .rposition(|entry| entry.meta.function_id.eq_ignore_ascii_case(function_id))
    }

    fn entry_index_by_surface_name(&self, surface_name: &str) -> Option<usize> {
        self.entries
            .iter()
            .rposition(|entry| entry.surface_name.eq_ignore_ascii_case(surface_name))
    }

    fn remove_udf_side_tables_for_function_id(&mut self, function_id: &str) -> Option<String> {
        let source_registration_id = self
            .source_registration_ids_by_function_id
            .iter()
            .find(|(id, _)| id.eq_ignore_ascii_case(function_id))
            .map(|(_, source_registration_id)| source_registration_id.clone());
        if let Some(source_registration_id) = source_registration_id.as_ref() {
            self.function_ids_by_source_registration_id
                .remove(source_registration_id);
        }
        self.source_registration_ids_by_function_id
            .remove(function_id);
        self.udf_invocation_targets_by_function_id
            .remove(function_id);
        source_registration_id
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CapabilityScopedRegistry<'a> {
    registry: &'a FunctionRegistry,
    overlay: &'a CapabilityOverlay,
}

impl<'a> CapabilityScopedRegistry<'a> {
    pub fn iter(&'a self) -> impl Iterator<Item = CapabilityScopedFunctionEntry<'a>> + 'a {
        self.registry
            .iter()
            .map(move |entry| scoped_entry(entry, self.overlay))
    }

    pub fn lookup_by_surface_name(&self, name: &str) -> Option<CapabilityScopedFunctionEntry<'a>> {
        self.registry
            .lookup_by_surface_name(name)
            .map(|entry| scoped_entry(entry, self.overlay))
    }

    pub fn lookup_by_id(&self, function_id: &str) -> Option<CapabilityScopedFunctionEntry<'a>> {
        self.registry
            .lookup_by_id(function_id)
            .map(|entry| scoped_entry(entry, self.overlay))
    }
}

pub fn builtin_registry() -> &'static FunctionRegistry {
    static REGISTRY: OnceLock<FunctionRegistry> = OnceLock::new();
    REGISTRY.get_or_init(FunctionRegistry::built_ins)
}

pub fn render_registry_metadata_csv(registry: &FunctionRegistry) -> String {
    let mut out = String::from(
        "function_id,surface_name,semantic_kernel_metadata_version,reduction_sensitive,error_collapse_sensitive,numerical_reduction_policy,error_algebra,arg_admission_metadata_version,arg_admission_profile,rich_required_capability_set_keys,sparse_extent_class,sparse_cardinality_class,rich_value_usage,producer_capability_set_keys\n",
    );

    for entry in registry.iter() {
        let (
            arg_admission_profile,
            rich_required_capability_set_keys,
            sparse_extent_class,
            sparse_cardinality_class,
        ) = arg_admission_export_fields(&entry.meta.arg_admission_metadata);
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            csv_escape(&entry.meta.function_id),
            csv_escape(&entry.surface_name),
            csv_escape(&entry.meta.semantic_kernel_metadata_version),
            entry.meta.semantic_kernel_metadata.reduction_sensitive,
            entry.meta.semantic_kernel_metadata.error_collapse_sensitive,
            csv_escape(
                entry
                    .meta
                    .semantic_kernel_metadata
                    .numerical_reduction_policy
                    .as_deref()
                    .unwrap_or("")
            ),
            csv_escape(
                entry
                    .meta
                    .semantic_kernel_metadata
                    .error_algebra
                    .as_deref()
                    .unwrap_or("")
            ),
            csv_escape(&entry.meta.arg_admission_metadata_version),
            csv_escape(&arg_admission_profile),
            csv_escape(&rich_required_capability_set_keys),
            csv_escape(&sparse_extent_class),
            csv_escape(&sparse_cardinality_class),
            csv_escape(entry.meta.rich_value_usage.version_key()),
            csv_escape(&entry.meta.producer_capability_set_keys.join("|")),
        ));
    }

    out
}

fn arg_admission_export_fields(
    metadata: &ArgAdmissionMetadata,
) -> (String, String, String, String) {
    match metadata {
        ArgAdmissionMetadata::ExistingArgPreparation { profile } => (
            arg_preparation_profile_key(*profile).to_string(),
            String::new(),
            String::new(),
            String::new(),
        ),
        ArgAdmissionMetadata::RichArgAccepted {
            required_capability_set_keys,
        } => (
            "rich_arg_accepted".to_string(),
            sorted_stable_key_join(required_capability_set_keys),
            String::new(),
            String::new(),
        ),
        ArgAdmissionMetadata::SparseRangeAccepted {
            extent_class,
            cardinality_class,
        } => (
            "sparse_range_accepted".to_string(),
            String::new(),
            extent_class.clone(),
            cardinality_class.clone(),
        ),
    }
}

fn csv_escape(field: &str) -> String {
    let needs_quotes = field.contains(',') || field.contains('"') || field.contains('\n');
    if !needs_quotes {
        return field.to_string();
    }

    let escaped = field.replace('"', "\"\"");
    format!("\"{escaped}\"")
}

fn scoped_entry<'a>(
    entry: &'a FunctionEntry,
    overlay: &CapabilityOverlay,
) -> CapabilityScopedFunctionEntry<'a> {
    CapabilityScopedFunctionEntry {
        entry,
        availability: overlay.availability_for(&entry.meta.function_id),
    }
}

fn typed_rejection(
    code: UdfRegistrationRejectionCode,
    function_id: Option<String>,
    surface_name: Option<String>,
    source_registration_id: Option<String>,
    snapshot_identity: FunctionRegistrySnapshotIdentity,
    detail: impl Into<String>,
) -> UdfRegistrationRejection {
    UdfRegistrationRejection {
        code,
        function_id,
        surface_name,
        source_registration_id,
        snapshot_identity,
        detail: detail.into(),
    }
}

fn udf_entry_from_request(request: UdfRegistrationRequest) -> FunctionEntry {
    let function_id = request.function_id;
    let surface_name = request.surface_name;
    let source_registration_id = request.stable_source_registration_id;
    let source_provenance = request.source_provenance;
    let replacement_policy = request.replacement_policy;
    let semantic_kernel_metadata = SemanticKernelMetadata {
        reduction_sensitive: false,
        error_collapse_sensitive: false,
        numerical_reduction_policy: None,
        error_algebra: None,
    };
    let arg_admission_metadata =
        ArgAdmissionMetadata::from_arg_preparation_profile(request.arg_preparation_profile);
    let signature_display = request
        .display_signature
        .unwrap_or_else(|| synthesized_signature_display(&surface_name, &request.parameters));
    let capability_keys = request
        .execution_profile
        .as_ref()
        .map(|profile| profile.required_capability_set_keys.clone())
        .unwrap_or_default();
    let category = request.category.clone();
    let source_kind = format!("{:?}", request.source_kind);
    FunctionEntry {
        meta: RegistryFunctionMeta {
            function_id,
            arity: request.arity,
            determinism: request.determinism,
            volatility: request.volatility,
            host_interaction: request.host_interaction,
            thread_safety: request.thread_safety,
            arg_preparation_profile: request.arg_preparation_profile,
            coercion_lift_profile: request.coercion_lift_profile,
            kernel_signature_class: request.kernel_signature_class,
            fec_dependency_profile: request.fec_dependency_profile,
            surface_fec_dependency_profile: request.surface_fec_dependency_profile,
            semantic_kernel_metadata_version: semantic_kernel_metadata.version_key(),
            semantic_kernel_metadata,
            arg_admission_metadata_version: arg_admission_metadata.version_key(),
            arg_admission_metadata,
            rich_value_usage: RichValueUsage::RichBlind,
            producer_capability_set_keys: capability_keys,
        },
        surface_name: surface_name.clone(),
        display_signature: SignatureForm {
            signature_display,
            parameters: request.parameters,
            trailing_repeats: false,
        },
        registry_metadata: FunctionRegistryMetadata {
            registration_source_kind: Some(source_kind),
            surface_stable_id: Some(source_registration_id),
            canonical_surface_name: Some(surface_name),
            category,
            source_catalog_ref: source_provenance.clone(),
            ..FunctionRegistryMetadata::default()
        },
        short_description: request.short_description,
        long_description: request.long_description,
        source: FunctionSource::Udf {
            provenance: source_provenance,
            replaces_builtin: matches!(
                replacement_policy,
                UdfReplacementPolicy::AllowBuiltinReplacement
            ),
        },
    }
}

fn synthesized_signature_display(surface_name: &str, parameters: &[ParameterDescriptor]) -> String {
    let parameter_names = parameters
        .iter()
        .map(|parameter| parameter.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    format!("{surface_name}({parameter_names})")
}

fn stable_registry_fingerprint(registry: &FunctionRegistry) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for entry in registry.iter() {
        stable_hash_str(&mut hash, &entry.meta.function_id.to_ascii_uppercase());
        stable_hash_str(&mut hash, &entry.surface_name.to_ascii_uppercase());
        stable_hash_str(&mut hash, &entry.meta.arity.min.to_string());
        stable_hash_str(&mut hash, &entry.meta.arity.max.to_string());
        stable_hash_str(&mut hash, &entry.display_signature.signature_display);
        stable_hash_str(
            &mut hash,
            registry
                .source_registration_id_for_function_id(&entry.meta.function_id)
                .unwrap_or(""),
        );
        stable_hash_str(
            &mut hash,
            if registry
                .udf_invocation_target_by_id(&entry.meta.function_id)
                .is_some()
            {
                "target"
            } else {
                "no-target"
            },
        );
    }
    format!("{hash:016x}")
}

fn stable_hash_str(hash: &mut u64, value: &str) {
    for byte in value.as_bytes() {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(0x100000001b3);
    }
    *hash ^= 0xff;
    *hash = hash.wrapping_mul(0x100000001b3);
}

fn built_in_entry_from_meta(meta: FunctionMeta) -> FunctionEntry {
    let seed = signature_seed_for_id(meta.function_id)
        .unwrap_or_else(|| panic!("missing signature seed for {}", meta.function_id));
    let help_seed = registry_help_seed_for_id(meta.function_id);
    FunctionEntry {
        meta: RegistryFunctionMeta::from(meta),
        surface_name: canonical_surface_name(meta.function_id).to_string(),
        display_signature: signature_from_seed(seed, meta, help_seed),
        registry_metadata: registry_metadata_for_id(meta.function_id),
        short_description: help_seed
            .and_then(|seed| seed.short_description)
            .map(str::to_string),
        long_description: help_seed
            .and_then(|seed| seed.long_description)
            .map(str::to_string),
        source: FunctionSource::BuiltIn,
    }
}

fn signature_from_seed(
    seed: &SignatureSeed,
    meta: FunctionMeta,
    help_seed: Option<&RegistryHelpSeed>,
) -> SignatureForm {
    let implied_trailing_repeats = meta.arity.max > seed.parameters.len();
    let last_parameter_index = seed.parameters.len().saturating_sub(1);
    SignatureForm {
        signature_display: seed.signature_display.to_string(),
        parameters: seed
            .parameters
            .iter()
            .enumerate()
            .map(|(index, parameter)| ParameterDescriptor {
                name: parameter.name.to_string(),
                optional: index >= meta.arity.min,
                repeats: parameter.repeats
                    || (implied_trailing_repeats && index == last_parameter_index),
                short_description: parameter_help_description(help_seed, index, parameter.name)
                    .map(str::to_string),
            })
            .collect(),
        trailing_repeats: seed.trailing_repeats || implied_trailing_repeats,
    }
}

fn parameter_help_description<'a>(
    help_seed: Option<&'a RegistryHelpSeed>,
    index: usize,
    parameter_name: &str,
) -> Option<&'a str> {
    help_seed?
        .parameters
        .iter()
        .find(|parameter| {
            parameter.index == index && parameter.name.eq_ignore_ascii_case(parameter_name)
        })
        .and_then(|parameter| parameter.short_description)
}

fn canonical_surface_name(function_id: &str) -> &str {
    function_id.strip_prefix("FUNC.").unwrap_or(function_id)
}

fn semantic_kernel_metadata_for_id(function_id: &str) -> SemanticKernelMetadata {
    let reduction_sensitive = is_reduction_sensitive_function(function_id);
    let error_collapse_sensitive =
        reduction_sensitive || is_error_collapse_sensitive_function(function_id);
    SemanticKernelMetadata {
        reduction_sensitive,
        error_collapse_sensitive,
        numerical_reduction_policy: reduction_sensitive.then(|| "SequentialLeftFold".to_string()),
        error_algebra: error_collapse_sensitive.then(|| "CanonicalExcelLegacy".to_string()),
    }
}

fn is_reduction_sensitive_function(function_id: &str) -> bool {
    matches!(
        function_id,
        "FUNC.AGGREGATE"
            | "FUNC.AVERAGE"
            | "FUNC.AVERAGEA"
            | "FUNC.AVERAGEIF"
            | "FUNC.AVERAGEIFS"
            | "FUNC.BYCOL"
            | "FUNC.BYROW"
            | "FUNC.COUNT"
            | "FUNC.COUNTA"
            | "FUNC.COUNTIF"
            | "FUNC.COUNTIFS"
            | "FUNC.DAVERAGE"
            | "FUNC.DCOUNT"
            | "FUNC.DCOUNTA"
            | "FUNC.DMAX"
            | "FUNC.DMIN"
            | "FUNC.DPRODUCT"
            | "FUNC.DSTDEV"
            | "FUNC.DSTDEVP"
            | "FUNC.DSUM"
            | "FUNC.DVAR"
            | "FUNC.DVARP"
            | "FUNC.GROUPBY"
            | "FUNC.MAP"
            | "FUNC.MAX"
            | "FUNC.MAXA"
            | "FUNC.MAXIFS"
            | "FUNC.MDETERM"
            | "FUNC.MIN"
            | "FUNC.MINA"
            | "FUNC.MINIFS"
            | "FUNC.MINVERSE"
            | "FUNC.MMULT"
            | "FUNC.PIVOTBY"
            | "FUNC.PRODUCT"
            | "FUNC.REDUCE"
            | "FUNC.SCAN"
            | "FUNC.SUBTOTAL"
            | "FUNC.SUM"
            | "FUNC.SUMIF"
            | "FUNC.SUMIFS"
            | "FUNC.SUMPRODUCT"
            | "FUNC.SUMSQ"
            | "FUNC.SUMX2MY2"
            | "FUNC.SUMX2PY2"
            | "FUNC.SUMXMY2"
    )
}

fn is_error_collapse_sensitive_function(function_id: &str) -> bool {
    matches!(
        function_id,
        "FUNC.CHOOSE" | "FUNC.IF" | "FUNC.IFS" | "FUNC.IFERROR" | "FUNC.IFNA" | "FUNC.SWITCH"
    )
}

pub fn producer_capability_set_keys_for_id(function_id: &str) -> Vec<String> {
    match function_id {
        "FUNC.IMAGE" => webimage_producer_capability_set_keys(),
        _ => Vec::new(),
    }
}

pub fn rich_value_usage_for_id(function_id: &str) -> RichValueUsage {
    match function_id {
        "FUNC.IMAGE" => RichValueUsage::ProducesRichObject,
        "FUNC.HYPERLINK" | "FUNC.NOW" | "FUNC.TODAY" => RichValueUsage::ProducesPresentation,
        _ => RichValueUsage::RichBlind,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::ReferenceKind;

    #[test]
    fn builtin_registry_count_matches_function_catalog() {
        assert_eq!(
            builtin_registry().len(),
            xll_export_specs::function_catalog().len()
        );
    }

    #[test]
    fn builtin_registry_round_trips_surface_and_id_lookup() {
        for entry in builtin_registry().iter() {
            let by_id = builtin_registry()
                .lookup_by_id(&entry.meta.function_id)
                .expect("registry id lookup must round-trip");
            assert_eq!(
                by_id.meta.function_id.as_str(),
                entry.meta.function_id.as_str()
            );

            let by_surface = builtin_registry()
                .lookup_by_surface_name(&entry.surface_name)
                .expect("registry surface lookup must round-trip");
            assert_eq!(
                by_surface.meta.function_id.as_str(),
                entry.meta.function_id.as_str()
            );
        }
    }

    #[test]
    fn now_signature_is_zero_argument() {
        let now = builtin_registry()
            .lookup_by_surface_name("NOW")
            .expect("NOW must exist in builtin registry");
        assert_eq!(now.display_signature.signature_display, "NOW()");
        assert!(now.display_signature.parameters.is_empty());
        assert_eq!(now.meta.arity, Arity::exact(0));
    }

    #[test]
    fn builtin_registry_carries_library_context_metadata() {
        let sum = builtin_registry()
            .lookup_by_surface_name("SUM")
            .expect("SUM must exist in builtin registry");
        assert_eq!(
            sum.registry_metadata.surface_stable_id.as_deref(),
            Some("FUNC.SUM")
        );
        assert_eq!(
            sum.registry_metadata.name_resolution_table_ref.as_deref(),
            Some("docs/function-lane/W28_FUNCTION_NAME_LOCALIZATION_LIBRARY_SEED.csv")
        );
        assert_eq!(
            sum.registry_metadata.special_interface_kind.as_deref(),
            Some("ordinary")
        );
        assert_eq!(
            sum.registry_metadata.admission_interface_kind.as_deref(),
            Some("ordinary_call")
        );
    }

    #[test]
    fn reducer_registry_metadata_publishes_semantic_kernel_version() {
        let sum = builtin_registry()
            .lookup_by_surface_name("SUM")
            .expect("SUM must exist in builtin registry");

        assert!(sum.meta.semantic_kernel_metadata.reduction_sensitive);
        assert!(sum.meta.semantic_kernel_metadata.error_collapse_sensitive);
        assert_eq!(
            sum.meta
                .semantic_kernel_metadata
                .numerical_reduction_policy
                .as_deref(),
            Some("SequentialLeftFold")
        );
        assert_eq!(
            sum.meta.semantic_kernel_metadata.error_algebra.as_deref(),
            Some("CanonicalExcelLegacy")
        );
        assert!(
            sum.meta
                .semantic_kernel_metadata_version
                .contains("numerical_reduction_policy=SequentialLeftFold")
        );
    }

    #[test]
    fn selector_registry_metadata_publishes_error_algebra_without_reduction_policy() {
        let if_entry = builtin_registry()
            .lookup_by_surface_name("IF")
            .expect("IF must exist in builtin registry");

        assert!(!if_entry.meta.semantic_kernel_metadata.reduction_sensitive);
        assert!(
            if_entry
                .meta
                .semantic_kernel_metadata
                .error_collapse_sensitive
        );
        assert_eq!(
            if_entry
                .meta
                .semantic_kernel_metadata
                .numerical_reduction_policy,
            None
        );
        assert_eq!(
            if_entry
                .meta
                .semantic_kernel_metadata
                .error_algebra
                .as_deref(),
            Some("CanonicalExcelLegacy")
        );
    }

    #[test]
    fn semantic_kernel_version_changes_when_selector_metadata_changes() {
        let baseline = semantic_kernel_metadata_for_id("FUNC.SUM");
        let mut changed = baseline.clone();
        changed.numerical_reduction_policy = Some("PairwiseTree".to_string());

        assert_ne!(baseline.version_key(), changed.version_key());

        let mut changed = baseline.clone();
        changed.error_algebra = Some("CustomTestAlgebra".to_string());
        assert_ne!(baseline.version_key(), changed.version_key());
    }

    #[test]
    fn arg_admission_version_changes_when_admission_metadata_changes() {
        let values_only = ArgAdmissionMetadata::from_arg_preparation_profile(
            ArgPreparationProfile::ValuesOnlyPreAdapter,
        );
        let refs_visible = ArgAdmissionMetadata::from_arg_preparation_profile(
            ArgPreparationProfile::RefsVisibleInAdapter,
        );
        let rich = ArgAdmissionMetadata::RichArgAccepted {
            required_capability_set_keys: vec![
                "Materialisable(target_class=published_fallback_text)".to_string(),
                "Indexable(rank=1,index_type=rich_value_key,element_value_class=rich_value_data)"
                    .to_string(),
            ],
        };

        assert_ne!(values_only.version_key(), refs_visible.version_key());
        assert_ne!(values_only.version_key(), rich.version_key());
    }

    #[test]
    fn rich_arg_admission_metadata_validates_required_capability_keys() {
        let rich = ArgAdmissionMetadata::RichArgAccepted {
            required_capability_set_keys: vec![
                "Materialisable(target_class=published_fallback_text)".to_string(),
                "Indexable(rank=1,index_type=rich_value_key,element_value_class=rich_value_data)"
                    .to_string(),
            ],
        };
        let producer = producer_capability_set_keys_for_id("FUNC.IMAGE");

        assert_eq!(rich.validate_producer_capabilities(&producer), Ok(()));

        let missing = rich
            .validate_producer_capabilities(&[
                "Shaped(extent_class=webimage_kvp_record)".to_string()
            ])
            .expect_err("missing required capability key");
        assert_eq!(
            missing.missing_capability_set_keys,
            vec![
                "Indexable(rank=1,index_type=rich_value_key,element_value_class=rich_value_data)"
                    .to_string(),
                "Materialisable(target_class=published_fallback_text)".to_string(),
            ]
        );
    }

    #[test]
    fn image_registry_entry_publishes_webimage_producer_capabilities() {
        let image = builtin_registry()
            .lookup_by_surface_name("IMAGE")
            .expect("IMAGE must exist in builtin registry");

        assert_eq!(
            image.meta.arg_admission_metadata,
            ArgAdmissionMetadata::ExistingArgPreparation {
                profile: ArgPreparationProfile::ValuesOnlyPreAdapter
            }
        );
        assert!(
            image
                .meta
                .producer_capability_set_keys
                .iter()
                .any(|key| key.starts_with("Materialisable("))
        );
        assert!(
            image
                .meta
                .producer_capability_set_keys
                .iter()
                .any(|key| key.starts_with("Indexable("))
        );
        assert!(
            image
                .meta
                .producer_capability_set_keys
                .iter()
                .any(|key| key.starts_with("Shaped("))
        );
    }

    #[test]
    fn registry_metadata_csv_exports_version_and_capability_columns() {
        let csv = render_registry_metadata_csv(builtin_registry());
        let header = csv.lines().next().expect("csv header");
        assert_eq!(
            header,
            "function_id,surface_name,semantic_kernel_metadata_version,reduction_sensitive,error_collapse_sensitive,numerical_reduction_policy,error_algebra,arg_admission_metadata_version,arg_admission_profile,rich_required_capability_set_keys,sparse_extent_class,sparse_cardinality_class,rich_value_usage,producer_capability_set_keys"
        );
        assert!(
            csv.contains("FUNC.SUM,SUM,semantic_kernel_metadata.v1;reduction_sensitive=true;error_collapse_sensitive=true;numerical_reduction_policy=SequentialLeftFold;error_algebra=CanonicalExcelLegacy,true,true,SequentialLeftFold,CanonicalExcelLegacy,arg_admission_metadata.v1;existing_arg_preparation=values_only_pre_adapter,values_only_pre_adapter"),
            "SUM row must publish semantic kernel metadata and versions"
        );
        assert!(
            csv.contains("FUNC.IMAGE,IMAGE,semantic_kernel_metadata.v1;reduction_sensitive=false;error_collapse_sensitive=false;numerical_reduction_policy=none;error_algebra=none,false,false,,,arg_admission_metadata.v1;existing_arg_preparation=values_only_pre_adapter,values_only_pre_adapter"),
            "IMAGE row must remain ordinary arg admission"
        );
        assert!(
            csv.contains("FUNC.SUM,SUM,semantic_kernel_metadata.v1;reduction_sensitive=true;error_collapse_sensitive=true;numerical_reduction_policy=SequentialLeftFold;error_algebra=CanonicalExcelLegacy,true,true,SequentialLeftFold,CanonicalExcelLegacy,arg_admission_metadata.v1;existing_arg_preparation=values_only_pre_adapter,values_only_pre_adapter,,,,rich_value_usage.v1;rich_blind,"),
            "SUM row must publish rich-blind metadata"
        );
        assert!(
            csv.contains("FUNC.IMAGE,IMAGE,semantic_kernel_metadata.v1;reduction_sensitive=false;error_collapse_sensitive=false;numerical_reduction_policy=none;error_algebra=none,false,false,,,arg_admission_metadata.v1;existing_arg_preparation=values_only_pre_adapter,values_only_pre_adapter,,,,rich_value_usage.v1;produces_rich_object,"),
            "IMAGE row must publish rich-object metadata"
        );
        assert!(
            csv.contains("Materialisable(target_class=published_fallback_text)"),
            "IMAGE row must publish webimage producer capabilities"
        );
    }

    #[test]
    fn every_builtin_registry_metadata_matches_runtime_function_id() {
        for entry in builtin_registry().iter() {
            assert_eq!(
                entry.registry_metadata.surface_stable_id.as_deref(),
                Some(entry.meta.function_id.as_str()),
                "{} metadata must be keyed by runtime function id",
                entry.meta.function_id
            );
        }
    }

    #[test]
    fn every_builtin_signature_is_consistent_with_arity() {
        for entry in builtin_registry().iter() {
            let parameters = &entry.display_signature.parameters;
            let required_count = parameters
                .iter()
                .filter(|parameter| !parameter.optional)
                .count();
            assert!(
                required_count <= entry.meta.arity.min,
                "{} has more required parameters than arity min",
                entry.meta.function_id
            );
            assert!(
                entry.meta.arity.min <= parameters.len()
                    || entry.display_signature.trailing_repeats,
                "{} has too few parameters for arity min",
                entry.meta.function_id
            );
            assert!(
                entry.meta.arity.max <= parameters.len()
                    || entry.display_signature.trailing_repeats,
                "{} needs trailing repeat metadata to cover arity max",
                entry.meta.function_id
            );
            assert!(
                !parameters
                    .iter()
                    .any(|parameter| parameter.name.starts_with("arg")
                        && parameter.name[3..].chars().all(|ch| ch.is_ascii_digit())),
                "{} still has synthesized argN parameter names",
                entry.meta.function_id
            );
        }
    }

    #[test]
    fn udf_registration_round_trips_after_builtins() {
        let mut registry = builtin_registry().clone();
        let entry = test_udf_entry("FUNC.UDF.MYFUNC", "MYFUNC");
        registry.register_udf(entry).expect("UDF registration");

        let last = registry.iter().last().expect("registered UDF");
        assert_eq!(last.meta.function_id.as_str(), "FUNC.UDF.MYFUNC");
        assert_eq!(
            registry
                .lookup_by_surface_name("MYFUNC")
                .expect("surface lookup")
                .meta
                .function_id
                .as_str(),
            "FUNC.UDF.MYFUNC"
        );

        registry
            .unregister_udf("FUNC.UDF.MYFUNC")
            .expect("UDF unregister");
        assert!(registry.lookup_by_surface_name("MYFUNC").is_none());
    }

    #[test]
    fn udf_registration_does_not_mutate_builtin_registry_singleton() {
        let builtin_before = builtin_registry().snapshot_identity();
        let mut host_registry = builtin_registry().clone();
        host_registry
            .register_udf(test_udf_entry("FUNC.UDF.MYFUNC", "MYFUNC"))
            .expect("UDF registration");

        assert!(host_registry.lookup_by_surface_name("MYFUNC").is_some());
        assert!(
            builtin_registry()
                .lookup_by_surface_name("MYFUNC")
                .is_none()
        );
        assert_eq!(builtin_registry().snapshot_identity(), builtin_before);
    }

    #[test]
    fn duplicate_udf_surface_requires_explicit_builtin_replacement() {
        let mut registry = builtin_registry().clone();
        let collision = test_udf_entry("FUNC.UDF.NOW", "NOW");
        assert!(matches!(
            registry.register_udf(collision),
            Err(RegistryError::SurfaceNameCollision { .. })
        ));

        let mut replacement = test_udf_entry("FUNC.UDF.NOW.REPLACEMENT", "NOW");
        replacement.source = FunctionSource::Udf {
            provenance: Some("test replacement".to_string()),
            replaces_builtin: true,
        };
        registry
            .register_udf(replacement)
            .expect("explicit replacement registration");
        assert_eq!(
            registry
                .lookup_by_surface_name("NOW")
                .expect("replacement lookup")
                .meta
                .function_id
                .as_str(),
            "FUNC.UDF.NOW.REPLACEMENT"
        );
    }

    #[test]
    fn capability_overlay_projects_without_mutating_registry() {
        let registry = builtin_registry();
        let mut overlay = CapabilityOverlay::new();
        overlay.deny_function_id("FUNC.RTD", "provider unavailable");

        let scoped = registry.with_capability_overlay(&overlay);
        assert!(matches!(
            scoped
                .lookup_by_id("FUNC.RTD")
                .expect("RTD scoped entry")
                .availability,
            FunctionAvailability::Unavailable { .. }
        ));
        assert_eq!(
            registry
                .with_capability_overlay(&CapabilityOverlay::new())
                .lookup_by_id("FUNC.RTD")
                .expect("RTD baseline entry")
                .availability,
            FunctionAvailability::Available
        );
    }

    #[test]
    fn udf_registration_request_emits_snapshot_change_set() {
        let mut registry = builtin_registry().clone();
        let before = registry.snapshot_identity();
        let result = registry.register_udf_request(test_udf_registration_request(
            "SRC.xll.book1.myfunc",
            "FUNC.UDF.MYFUNC",
            "MYFUNC",
        ));

        let change_set = match result {
            UdfRegistrationResult::Registered {
                function_id,
                change_set,
            } => {
                assert_eq!(function_id, "FUNC.UDF.MYFUNC");
                change_set
            }
            other => panic!("unexpected registration result: {other:?}"),
        };

        assert_eq!(change_set.previous_snapshot_identity, before);
        assert_ne!(
            change_set.previous_snapshot_identity,
            change_set.new_snapshot_identity
        );
        assert_eq!(
            change_set.added_function_ids,
            vec!["FUNC.UDF.MYFUNC".to_string()]
        );
        assert_eq!(
            change_set.affected_source_registration_ids,
            vec!["SRC.xll.book1.myfunc".to_string()]
        );
        assert!(!change_set.descriptor_only_mutation);
        assert_eq!(
            registry.source_registration_id_for_function_id("FUNC.UDF.MYFUNC"),
            Some("SRC.xll.book1.myfunc")
        );
    }

    #[test]
    fn rejected_udf_registration_does_not_advance_snapshot_identity() {
        let mut registry = builtin_registry().clone();
        let before = registry.snapshot_identity();
        let result = registry.register_udf_request(test_udf_registration_request(
            "SRC.xll.book1.now",
            "FUNC.UDF.NOW",
            "NOW",
        ));

        match result {
            UdfRegistrationResult::Rejected(rejection) => {
                assert_eq!(
                    rejection.code,
                    UdfRegistrationRejectionCode::BuiltInSurfaceCollision
                );
                assert_eq!(rejection.snapshot_identity, before);
            }
            other => panic!("unexpected registration result: {other:?}"),
        }
        assert_eq!(registry.snapshot_identity(), before);
        assert_eq!(
            registry
                .lookup_by_surface_name("NOW")
                .expect("built-in NOW")
                .meta
                .function_id,
            "FUNC.NOW"
        );
    }

    #[test]
    fn same_source_registration_update_replaces_surface_metadata() {
        let mut registry = builtin_registry().clone();
        let source_id = "SRC.vba.book1.module1.myfunc";
        let first = test_udf_registration_request(source_id, "FUNC.UDF.MYFUNC", "MYFUNC");
        assert!(matches!(
            registry.register_udf_request(first),
            UdfRegistrationResult::Registered { .. }
        ));

        let mut update = test_udf_registration_request(source_id, "FUNC.UDF.MYFUNC", "MYFUNC2");
        update.replacement_policy = UdfReplacementPolicy::ReplaceSameSourceRegistrationId;
        update.parameters.push(ParameterDescriptor {
            name: "extra".to_string(),
            optional: true,
            repeats: false,
            short_description: None,
        });
        update.display_signature = Some("MYFUNC2(value, [extra])".to_string());
        let result = registry.register_udf_request(update);

        let change_set = match result {
            UdfRegistrationResult::Replaced {
                function_id,
                change_set,
            } => {
                assert_eq!(function_id, "FUNC.UDF.MYFUNC");
                change_set
            }
            other => panic!("unexpected update result: {other:?}"),
        };
        assert_eq!(
            change_set.replaced_function_ids,
            vec!["FUNC.UDF.MYFUNC".to_string()]
        );
        assert_eq!(
            change_set.changed_surface_names,
            vec!["MYFUNC".to_string(), "MYFUNC2".to_string()]
        );
        let entry = registry
            .lookup_by_id("FUNC.UDF.MYFUNC")
            .expect("updated UDF entry");
        assert_eq!(entry.surface_name, "MYFUNC2");
        assert_eq!(
            entry.display_signature.signature_display,
            "MYFUNC2(value, [extra])"
        );
    }

    #[test]
    fn same_source_registration_update_rejects_other_udf_surface_collision() {
        let mut registry = builtin_registry().clone();
        assert!(matches!(
            registry.register_udf_request(test_udf_registration_request(
                "SRC.xll.book1.first",
                "FUNC.UDF.FIRST",
                "FIRSTUDF",
            )),
            UdfRegistrationResult::Registered { .. }
        ));
        assert!(matches!(
            registry.register_udf_request(test_udf_registration_request(
                "SRC.xll.book1.second",
                "FUNC.UDF.SECOND",
                "SECONDUDF",
            )),
            UdfRegistrationResult::Registered { .. }
        ));
        let before = registry.snapshot_identity();

        let mut update =
            test_udf_registration_request("SRC.xll.book1.first", "FUNC.UDF.FIRST", "SECONDUDF");
        update.replacement_policy = UdfReplacementPolicy::ReplaceSameSourceRegistrationId;

        match registry.register_udf_request(update) {
            UdfRegistrationResult::Rejected(rejection) => {
                assert_eq!(
                    rejection.code,
                    UdfRegistrationRejectionCode::UdfSurfaceCollision
                );
                assert_eq!(rejection.snapshot_identity, before);
            }
            other => panic!("unexpected update result: {other:?}"),
        }
        assert_eq!(registry.snapshot_identity(), before);
        assert_eq!(
            registry
                .lookup_by_surface_name("SECONDUDF")
                .expect("second UDF")
                .meta
                .function_id,
            "FUNC.UDF.SECOND"
        );
    }

    #[test]
    fn unregister_udf_emits_snapshot_change_set_and_removes_target() {
        let mut registry = builtin_registry().clone();
        let request = test_udf_registration_request(
            "SRC.js.addin.ns.func",
            "FUNC.UDF.NS.MYFUNC",
            "NS.MYFUNC",
        );
        assert!(matches!(
            registry.register_udf_request(request),
            UdfRegistrationResult::Registered { .. }
        ));
        let before = registry.snapshot_identity();
        let result = registry.unregister_udf_by_source_registration_id("SRC.js.addin.ns.func");

        let change_set = match result {
            UdfUnregistrationResult::Unregistered {
                function_id,
                change_set,
            } => {
                assert_eq!(function_id, "FUNC.UDF.NS.MYFUNC");
                change_set
            }
            other => panic!("unexpected unregister result: {other:?}"),
        };
        assert_eq!(change_set.previous_snapshot_identity, before);
        assert_eq!(
            change_set.removed_function_ids,
            vec!["FUNC.UDF.NS.MYFUNC".to_string()]
        );
        assert!(registry.lookup_by_id("FUNC.UDF.NS.MYFUNC").is_none());
        assert!(
            registry
                .udf_invocation_target_by_id("FUNC.UDF.NS.MYFUNC")
                .is_none()
        );
    }

    #[test]
    fn invocation_target_preserves_opaque_reference_and_host_values() {
        let mut registry = builtin_registry().clone();
        let mut request =
            test_udf_registration_request("SRC.host.opaque", "FUNC.UDF.OPAQUE", "OPAQUEUDF");
        let reference = ReferenceLike::new(ReferenceKind::MultiArea, "(A1:A2,C1)").normalized();
        request.invocation_target = Some(UdfInvocationTargetDescriptor::HostOpaque {
            host_system: "generic-host".to_string(),
            opaque_target_id: "target-42".to_string(),
            opaque_runtime_values: vec![
                UdfOpaqueRuntimeValue::ReferenceLike(reference.clone()),
                UdfOpaqueRuntimeValue::HostReference {
                    reference_family: "macro-project".to_string(),
                    opaque_reference_id: "vba:Book1.Module1".to_string(),
                },
            ],
        });

        assert!(matches!(
            registry.register_udf_request(request),
            UdfRegistrationResult::Registered { .. }
        ));

        match registry
            .udf_invocation_target_by_id("FUNC.UDF.OPAQUE")
            .expect("target descriptor")
        {
            UdfInvocationTargetDescriptor::HostOpaque {
                host_system,
                opaque_target_id,
                opaque_runtime_values,
            } => {
                assert_eq!(host_system, "generic-host");
                assert_eq!(opaque_target_id, "target-42");
                assert_eq!(
                    opaque_runtime_values[0],
                    UdfOpaqueRuntimeValue::ReferenceLike(reference)
                );
                assert!(matches!(
                    &opaque_runtime_values[1],
                    UdfOpaqueRuntimeValue::HostReference {
                        reference_family,
                        opaque_reference_id
                    } if reference_family == "macro-project"
                        && opaque_reference_id == "vba:Book1.Module1"
                ));
            }
            other => panic!("unexpected target descriptor: {other:?}"),
        }
    }

    #[test]
    fn descriptor_only_change_set_does_not_advance_snapshot_identity() {
        let registry = builtin_registry().clone();
        let before = registry.snapshot_identity();
        let change_set =
            registry.descriptor_only_change_set(vec!["REG.worksheet.tickcount".to_string()]);

        assert_eq!(change_set.previous_snapshot_identity, before);
        assert_eq!(change_set.new_snapshot_identity, before);
        assert!(change_set.descriptor_only_mutation);
        assert!(change_set.added_function_ids.is_empty());
        assert!(change_set.removed_function_ids.is_empty());
    }

    fn test_udf_entry(function_id: &str, surface_name: &str) -> FunctionEntry {
        FunctionEntry {
            meta: RegistryFunctionMeta {
                function_id: function_id.to_string(),
                arity: Arity::exact(1),
                determinism: DeterminismClass::Deterministic,
                volatility: VolatilityClass::NonVolatile,
                host_interaction: HostInteractionClass::None,
                thread_safety: ThreadSafetyClass::SafePure,
                arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
                coercion_lift_profile: CoercionLiftProfile::Custom,
                kernel_signature_class: KernelSignatureClass::Custom,
                fec_dependency_profile: FecDependencyProfile::None,
                surface_fec_dependency_profile: FecDependencyProfile::None,
                semantic_kernel_metadata: SemanticKernelMetadata {
                    reduction_sensitive: false,
                    error_collapse_sensitive: false,
                    numerical_reduction_policy: None,
                    error_algebra: None,
                },
                semantic_kernel_metadata_version: SemanticKernelMetadata {
                    reduction_sensitive: false,
                    error_collapse_sensitive: false,
                    numerical_reduction_policy: None,
                    error_algebra: None,
                }
                .version_key(),
                arg_admission_metadata: ArgAdmissionMetadata::from_arg_preparation_profile(
                    ArgPreparationProfile::ValuesOnlyPreAdapter,
                ),
                arg_admission_metadata_version: ArgAdmissionMetadata::from_arg_preparation_profile(
                    ArgPreparationProfile::ValuesOnlyPreAdapter,
                )
                .version_key(),
                rich_value_usage: RichValueUsage::RichBlind,
                producer_capability_set_keys: Vec::new(),
            },
            surface_name: surface_name.to_string(),
            display_signature: SignatureForm {
                signature_display: format!("{surface_name}(value)"),
                parameters: vec![ParameterDescriptor {
                    name: "value".to_string(),
                    optional: false,
                    repeats: false,
                    short_description: None,
                }],
                trailing_repeats: false,
            },
            registry_metadata: FunctionRegistryMetadata::default(),
            short_description: None,
            long_description: None,
            source: FunctionSource::Udf {
                provenance: Some("test".to_string()),
                replaces_builtin: false,
            },
        }
    }

    fn test_udf_registration_request(
        source_registration_id: &str,
        function_id: &str,
        surface_name: &str,
    ) -> UdfRegistrationRequest {
        UdfRegistrationRequest {
            stable_source_registration_id: source_registration_id.to_string(),
            function_id: function_id.to_string(),
            surface_name: surface_name.to_string(),
            source_kind: UdfSourceKind::XllRegisteredFunction,
            source_provenance: Some("test provenance".to_string()),
            arity: Arity::exact(1),
            parameters: vec![ParameterDescriptor {
                name: "value".to_string(),
                optional: false,
                repeats: false,
                short_description: None,
            }],
            display_signature: None,
            determinism: DeterminismClass::Deterministic,
            volatility: VolatilityClass::NonVolatile,
            host_interaction: HostInteractionClass::None,
            thread_safety: ThreadSafetyClass::SafePure,
            arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
            coercion_lift_profile: CoercionLiftProfile::Custom,
            kernel_signature_class: KernelSignatureClass::Custom,
            fec_dependency_profile: FecDependencyProfile::None,
            surface_fec_dependency_profile: FecDependencyProfile::None,
            short_description: Some("test UDF".to_string()),
            long_description: None,
            category: Some("User Defined".to_string()),
            execution_profile: Some(UdfExecutionProfile {
                host_execution_profile_key: Some("test-host".to_string()),
                required_capability_set_keys: vec!["udf.runtime.test".to_string()],
                async_invocation: false,
                streaming: false,
                cancellable: false,
                externally_invalidated: false,
            }),
            invocation_target: Some(UdfInvocationTargetDescriptor::Xll {
                module_path: Some("Book1.xll".to_string()),
                export_name: "MyFunc".to_string(),
                type_text: Some("Q".to_string()),
                register_id: None,
                opaque_runtime_values: Vec::new(),
            }),
            replacement_policy: UdfReplacementPolicy::RejectOnCollision,
        }
    }
}
