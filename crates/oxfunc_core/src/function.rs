#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeterminismClass {
    Deterministic,
    PseudoRandom,
    TimeDependent,
    ExternalEventDependent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolatilityClass {
    NonVolatile,
    VolatileFull,
    VolatileContextual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostInteractionClass {
    None,
    WorkbookState,
    ApplicationState,
    EnvironmentState,
    ExternalProvider,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadSafetyClass {
    SafePure,
    HostSerialized,
    NotThreadSafe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgPreparationProfile {
    ValuesOnlyPreAdapter,
    RefsVisibleInAdapter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoercionLiftProfile {
    None,
    UnaryNumericScalarOnly,
    UnaryNumericScalarOrArrayElementwise,
    AggregateDirectAndRangeDualPolicy,
    LookupMatchProfile,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelSignatureClass {
    NullaryConst,
    NumToNum,
    NumsToNum,
    TextToText,
    LookupMatch,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FecDependencyProfile {
    None,
    RefOnly,
    CallerContext,
    TimeProvider,
    RandomProvider,
    ExternalProvider,
    LocaleProfile,
    Composite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Arity {
    pub min: usize,
    pub max: usize,
}

impl Arity {
    pub const fn exact(n: usize) -> Self {
        Self { min: n, max: n }
    }

    pub const fn accepts(self, argc: usize) -> bool {
        argc >= self.min && argc <= self.max
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FunctionMeta {
    pub function_id: &'static str,
    pub arity: Arity,
    pub determinism: DeterminismClass,
    pub volatility: VolatilityClass,
    pub host_interaction: HostInteractionClass,
    pub thread_safety: ThreadSafetyClass,
    pub arg_preparation_profile: ArgPreparationProfile,
    pub coercion_lift_profile: CoercionLiftProfile,
    pub kernel_signature_class: KernelSignatureClass,
    // Adapter-level FEC profile.
    pub fec_dependency_profile: FecDependencyProfile,
    // Surface pipeline FEC profile (includes pre-adapter preparation).
    pub surface_fec_dependency_profile: FecDependencyProfile,
}
