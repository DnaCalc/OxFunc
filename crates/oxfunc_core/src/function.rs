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
    pub fec_dependency_profile: FecDependencyProfile,
}

