use std::collections::BTreeMap;
use std::sync::OnceLock;

use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::registry_context_seed::registry_metadata_for_id;
use crate::registry_signature_seed::{SignatureSeed, signature_seed_for_id};
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
}

impl From<FunctionMeta> for RegistryFunctionMeta {
    fn from(meta: FunctionMeta) -> Self {
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
        }
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
}

impl FunctionRegistry {
    pub fn try_from_entries(entries: Vec<FunctionEntry>) -> Result<Self, RegistryError> {
        let mut registry = Self {
            entries: Vec::with_capacity(entries.len()),
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
        Self { entries }
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

        self.entries.remove(index);
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

fn scoped_entry<'a>(
    entry: &'a FunctionEntry,
    overlay: &CapabilityOverlay,
) -> CapabilityScopedFunctionEntry<'a> {
    CapabilityScopedFunctionEntry {
        entry,
        availability: overlay.availability_for(&entry.meta.function_id),
    }
}

fn built_in_entry_from_meta(meta: FunctionMeta) -> FunctionEntry {
    let seed = signature_seed_for_id(meta.function_id)
        .unwrap_or_else(|| panic!("missing signature seed for {}", meta.function_id));
    FunctionEntry {
        meta: RegistryFunctionMeta::from(meta),
        surface_name: canonical_surface_name(meta.function_id).to_string(),
        display_signature: signature_from_seed(seed, meta),
        registry_metadata: registry_metadata_for_id(meta.function_id),
        short_description: None,
        long_description: None,
        source: FunctionSource::BuiltIn,
    }
}

fn signature_from_seed(seed: &SignatureSeed, meta: FunctionMeta) -> SignatureForm {
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
                short_description: None,
            })
            .collect(),
        trailing_repeats: seed.trailing_repeats || implied_trailing_repeats,
    }
}

fn canonical_surface_name(function_id: &str) -> &str {
    function_id.strip_prefix("FUNC.").unwrap_or(function_id)
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
