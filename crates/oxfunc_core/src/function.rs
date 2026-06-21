// Re-exported so the real-result publication vocabulary lives alongside the other
// `FunctionMeta` declarative profile types. The policy itself is defined next to the
// kernel helpers that consume it (`crate::functions::excel_numeric`).
pub use crate::functions::excel_numeric::{ExcelRealPolicy, NonFinite};

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

/// How a function's arguments are prepared before its adapter runs — the per-argument
/// coercion + reference-resolution policy. This is a behavioural axis in the
/// `FunctionSpec` mould (ODR-FN-004 Layer 2, the first axis widened under W105 oxf-y2uw.4):
/// a CLOSED enum whose variants name a real, observed Excel preparation behaviour, carried
/// on [`FunctionMeta`] and read at every dispatch / projection site so the rule cannot be
/// restated (and drift) per path. Functions whose Excel argument preparation matches the
/// majority carry [`FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE`]; only a genuine deviation
/// names a non-default variant.
///
/// GROWTH DISCIPLINE (the pattern beads .6/.7/.8 copy for the lift/broadcast, error-algebra,
/// and precision/rounding axes): each new behavioural axis is a closed enum (or small `Copy`
/// ADT) like this one, added as a [`FunctionMeta`] field with a `DEFAULT_*` associated const
/// for the value the majority carry. Variants name behaviours, never numbers — any magic
/// threshold lives in ONE impl that consumes the variant, never as a free `f64` on the meta —
/// so [`FunctionMeta`] stays cheap, `Copy`, and `Eq` (the property the quirk-algebra and the
/// equivalence-law harness rest on). A new variant is added only for a behaviour some real
/// function exhibits; speculative variants are not introduced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgPreparationProfile {
    /// The majority shape: references are resolved and arguments coerced to plain values
    /// *before* the adapter sees them, so the adapter only ever handles materialized values.
    ValuesOnlyPreAdapter,
    /// The deviation: the adapter must see the live reference (it inspects address / shape /
    /// formula identity, not just the dereferenced value), so reference resolution is NOT
    /// performed pre-adapter. Carried by reference-aware surfaces (e.g. `ROW`/`COLUMN`,
    /// `OFFSET`, `CELL`, `INDEX`, the lookup surfaces). Verified live Excel 16.0 build 20026.
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
    /// Per-argument coercion + reference-resolution policy (see [`ArgPreparationProfile`]).
    /// Read at every site that decides whether references are resolved pre-adapter, so the
    /// preparation rule cannot diverge between paths. Functions whose Excel preparation
    /// matches the majority carry [`FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE`].
    pub arg_preparation_profile: ArgPreparationProfile,
    pub coercion_lift_profile: CoercionLiftProfile,
    pub kernel_signature_class: KernelSignatureClass,
    // Adapter-level FEC profile.
    pub fec_dependency_profile: FecDependencyProfile,
    // Surface pipeline FEC profile (includes pre-adapter preparation).
    pub surface_fec_dependency_profile: FecDependencyProfile,
    /// How Excel publishes this function's real-valued kernel result (argument-domain
    /// guard + non-finite handling). Consumed at every dispatch site so the behaviour
    /// cannot diverge between scalar, array-lift, and by-index paths. Functions with no
    /// special real-result behaviour use `FunctionMeta::DEFAULT_REAL_RESULT_POLICY`.
    pub real_result_policy: ExcelRealPolicy,
}

impl FunctionMeta {
    /// Pass-through real-result policy: raw kernel output is published unchanged. This is
    /// the value the overwhelming majority of functions carry; only kernels that overflow,
    /// saturate, or reject part of their argument domain in Excel override it. Referenced
    /// by name (rather than spelled `ExcelRealPolicy::PASS`) so a default literal needs no
    /// extra import beyond `FunctionMeta` itself.
    pub const DEFAULT_REAL_RESULT_POLICY: ExcelRealPolicy = ExcelRealPolicy::PASS;

    /// Values-only argument preparation: references are dereferenced and arguments coerced
    /// to plain values before the adapter runs. This is the value the majority of functions
    /// carry (199 of the 241 catalog `FunctionMeta` entries); only reference-aware surfaces
    /// that must inspect a live reference override it with
    /// `ArgPreparationProfile::RefsVisibleInAdapter`. Referenced by name (rather than spelled
    /// `ArgPreparationProfile::ValuesOnlyPreAdapter`) so a default literal needs no extra
    /// import beyond `FunctionMeta` itself — the same growth-discipline shape as
    /// `DEFAULT_REAL_RESULT_POLICY`.
    pub const DEFAULT_ARG_PREPARATION_PROFILE: ArgPreparationProfile =
        ArgPreparationProfile::ValuesOnlyPreAdapter;
}
