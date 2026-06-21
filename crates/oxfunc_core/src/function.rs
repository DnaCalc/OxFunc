// Re-exported so the real-result publication vocabulary lives alongside the other
// `FunctionMeta` declarative profile types. The policy itself is defined next to the
// kernel helpers that consume it (`crate::functions::excel_numeric`).
pub use crate::functions::excel_numeric::{ExcelRealPolicy, NonFinite};

// Re-exported so the error-algebra vocabulary lives alongside the other `FunctionMeta`
// declarative profile types. The algebra itself is defined next to the runtime collapse
// helper that consumes it (`crate::semantic_kernel::collapse_worksheet_errors`); the
// `ErrorCollapseProfile` axis below carries it and is the single declared source the
// `SemanticKernelMetadata` projection derives its `error_algebra` string from.
pub use crate::semantic_kernel::ErrorAlgebra;

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

/// The argument-position mask carried by [`LiftBroadcastProfile::ByIndexScalarArrayLift`]:
/// the (small, ordered) set of argument slots over which the by-index dispatch layer applies
/// its scalar-array broadcast. A `&'static [usize]` IS `Copy`/`Eq`, so this keeps
/// [`FunctionMeta`] a cheap comparable value (the property the quirk-algebra and equivalence
/// laws rest on) while still modelling the genuinely irreducible per-function structure.
///
/// DESIGN NOTE (named-behaviours-vs-position-mask tension, recorded per the W105 oxf-y2uw.6
/// growth-discipline charter): the other behavioural axes (`real_result_policy`,
/// `arg_preparation_profile`, `coercion_lift_profile`) are *closed enums of named behaviours*
/// because a finite vocabulary captures every function. The scalar-array-lift behaviour is
/// different in kind: each function's lifting argument set is a distinct positional structure
/// (`ADDRESS` lifts `[0,1,2,3,4]`, `SWITCH` lifts `[0,1,3]`, the inverse-distribution surfaces
/// lift `[0,1,2]`, …), so there is no small named vocabulary to enumerate — the position list
/// IS the irreducible datum. We therefore model it as a `Copy`/`Eq` mask wrapped in a named
/// ADT variant (so the *axis* is still a named behaviour and a future checker can quantify
/// over it) rather than pretending it factors into magic-free named variants it does not have.
/// This is the principled middle the charter blesses for irreducible structural metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LiftPositionMask(pub &'static [usize]);

impl LiftPositionMask {
    /// The argument positions that participate in the by-index scalar-array broadcast.
    pub const fn positions(self) -> &'static [usize] {
        self.0
    }
}

/// How a function's by-index surface lifts a scalar kernel over array arguments / broadcasts
/// multi-argument shapes — the lift/broadcast behavioural axis (ODR-FN-004 Layer 2, the axis
/// widened under W105 oxf-y2uw.6). A CLOSED `Copy`/`Eq` ADT carried on [`FunctionMeta`] and
/// read at the SINGLE lift/broadcast dispatch site, so the lift rule for a function cannot be
/// restated (and drift) per path.
///
/// GROWTH DISCIPLINE (same shape as [`ArgPreparationProfile`]): a [`FunctionMeta`] field with a
/// `DEFAULT_*` associated const for the value the majority carry, variants that name a real
/// observed Excel behaviour, no free `f64`, [`FunctionMeta`] stays `Copy`/`Eq`. The
/// non-default variant carries a [`LiftPositionMask`] because the lifting argument set is
/// irreducible per-function structure (see [`LiftPositionMask`]'s design note), not a small
/// named vocabulary.
///
/// RECONCILIATION (why this is a distinct field, not an extension of [`CoercionLiftProfile`]):
/// `coercion_lift_profile` already names the *coercion/lift kernel-shape category* a function
/// belongs to (unary-numeric scalar-or-array, aggregate dual-policy, lookup/match, custom, …)
/// and is consumed by the XLL `U`-lift gating and the registry projection. The
/// scalar-array-lift *positions* are an orthogonal concern — a per-argument broadcast mask the
/// by-index dispatch fallback applies — that does not map onto those categories (e.g. `ADDRESS`,
/// `SWITCH`, `IFS`, the `IM*` surfaces, and the inverse-distribution surfaces carry assorted
/// coercion profiles but each its own distinct position mask). Folding a `&'static [usize]`
/// payload into `CoercionLiftProfile::Custom` would overload that enum with data unrelated to
/// the categories it names and would NOT yield one clean source. A dedicated axis keeps each
/// concern a single, separately-declared source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiftBroadcastProfile {
    /// The majority shape: the function's own surface evaluation performs whatever lifting /
    /// broadcast it needs (native elementwise lift, e.g. the unary-numeric family, or none),
    /// so the by-index dispatch layer applies no extra positional scalar-array broadcast.
    SurfaceNative,
    /// The function's by-index surface is scalar-shaped; when an argument in the masked
    /// positions is an array, the dispatch layer broadcasts the scalar kernel over the array
    /// shape elementwise. The [`LiftPositionMask`] names exactly which argument positions
    /// participate. This is the observed Excel scalar-array-lift behaviour previously encoded
    /// in the hand-maintained `observed_scalar_array_lift_positions` id-table, now declared
    /// once here. Verified live Excel 16.0 build 20026.
    ByIndexScalarArrayLift(LiftPositionMask),
}

impl LiftBroadcastProfile {
    /// Construct a by-index scalar-array-lift profile over the given argument positions.
    pub const fn lift_at(positions: &'static [usize]) -> Self {
        Self::ByIndexScalarArrayLift(LiftPositionMask(positions))
    }

    /// The argument positions this profile broadcasts the scalar kernel over, or `None` when
    /// the function lifts natively / not at all. The SINGLE accessor every lift/broadcast
    /// dispatch site reads, so no second site can decide a function's lift positions.
    pub const fn scalar_array_lift_positions(self) -> Option<&'static [usize]> {
        match self {
            Self::SurfaceNative => None,
            Self::ByIndexScalarArrayLift(mask) => Some(mask.positions()),
        }
    }
}

/// How a function collapses the Excel error inputs it sees — the error-algebra behavioural axis
/// (ODR-FN-004 Layer 2, the axis widened under W105 oxf-y2uw.7). A CLOSED `Copy`/`Eq` enum
/// carried on [`FunctionMeta`] and read at the SINGLE projection site
/// (`registry.rs::semantic_kernel_metadata_for_id`), so the rule for whether a function is
/// error-collapse sensitive and which [`ErrorAlgebra`] it applies cannot be restated (and drift)
/// in a second id-keyed table. This is the one declared SOURCE the `SemanticKernelMetadata`
/// projection derives `error_collapse_sensitive` and `error_algebra` FROM (and, for the
/// reduction family, the `reduction_sensitive` / `numerical_reduction_policy` facet — see the
/// note on [`ErrorCollapseProfile::ReductionFold`]).
///
/// GROWTH DISCIPLINE (same shape as [`ArgPreparationProfile`] / [`LiftBroadcastProfile`]): a
/// [`FunctionMeta`] field with a `DEFAULT_*` associated const for the value the majority carry,
/// variants that name a real observed Excel behaviour, no free `f64`, [`FunctionMeta`] stays
/// `Copy`/`Eq`. The [`ErrorAlgebra`] a non-default variant applies is a named enum (reused from
/// `crate::semantic_kernel`, the runtime collapse vocabulary), never a free string on the meta.
///
/// RECONCILIATION (why a three-state enum rather than a bare `bool`): the projection distinguishes
/// two reasons a function is error-collapse sensitive that publish differently —
/// reduction/aggregation functions (`SUM`, `MAX`, `COUNTIF`, the `D*` database family, `MMULT`, …)
/// are error-collapse sensitive AND carry a numerical-reduction policy, while the branch-selector
/// functions (`IF`, `IFS`, `CHOOSE`, `IFERROR`, `IFNA`, `SWITCH`) are error-collapse sensitive
/// WITHOUT a reduction policy. A named three-state axis captures both reasons from one declared
/// value, so the projection needs no second id-keyed table to tell the families apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCollapseProfile {
    /// The majority shape: the function performs no Excel error-collapse algebra of its own — an
    /// error input propagates through ordinary value handling, so the function is not
    /// error-collapse sensitive and declares no [`ErrorAlgebra`].
    None,
    /// A reduction / aggregation function that folds many inputs (a range, the database families,
    /// the matrix reducers) into one result and therefore collapses any error inputs by Excel's
    /// error-precedence order ([`ErrorAlgebra::CanonicalExcelLegacy`]). Because such a function
    /// also reduces *numerically*, this variant is additionally the source of the
    /// `reduction_sensitive` / `numerical_reduction_policy` projection facet (current policy
    /// `SequentialLeftFold`). That numerical facet rides on the same family marker today; a later
    /// dedicated numerical-reduction axis may take it over, leaving this axis owning only the
    /// error-collapse semantics. Verified live Excel 16.0 build 20026.
    ReductionFold,
    /// A branch-selector function (`IF`/`IFS`/`CHOOSE`/`IFERROR`/`IFNA`/`SWITCH`) that chooses
    /// among argument branches which may themselves be errors, collapsing them by Excel's
    /// error-precedence order ([`ErrorAlgebra::CanonicalExcelLegacy`]). Error-collapse sensitive
    /// but NOT a numerical reducer, so it carries no numerical-reduction policy. Verified live
    /// Excel 16.0 build 20026.
    SelectorBranch,
}

impl ErrorCollapseProfile {
    /// Whether this profile makes the function error-collapse sensitive — the SINGLE accessor the
    /// projection reads, so no second site can decide a function's error-collapse sensitivity.
    pub const fn is_error_collapse_sensitive(self) -> bool {
        !matches!(self, Self::None)
    }

    /// The [`ErrorAlgebra`] this profile applies to collapse error inputs, or `None` when the
    /// function performs no error-collapse. Both non-default variants apply Excel's canonical
    /// legacy error-precedence order.
    pub const fn error_algebra(self) -> Option<ErrorAlgebra> {
        match self {
            Self::None => None,
            Self::ReductionFold | Self::SelectorBranch => Some(ErrorAlgebra::CanonicalExcelLegacy),
        }
    }

    /// Whether this profile is a numerical reduction/aggregation fold — the facet the
    /// `reduction_sensitive` / `numerical_reduction_policy` projection derives from while a
    /// dedicated numerical-reduction axis does not yet exist (see [`Self::ReductionFold`]).
    pub const fn is_reduction_fold(self) -> bool {
        matches!(self, Self::ReductionFold)
    }
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
    /// How the by-index surface lifts a scalar kernel over array arguments / broadcasts
    /// multi-argument shapes (see [`LiftBroadcastProfile`]). Read at the SINGLE lift/broadcast
    /// dispatch site so the lift rule cannot diverge between paths. Functions that lift
    /// natively (or not at all) carry [`FunctionMeta::DEFAULT_LIFT_BROADCAST_PROFILE`]; only a
    /// surface that needs the by-index positional scalar-array broadcast names a non-default
    /// [`LiftBroadcastProfile::ByIndexScalarArrayLift`] mask.
    pub lift_broadcast_profile: LiftBroadcastProfile,
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
    /// How this function collapses Excel error inputs and which [`ErrorAlgebra`] it applies
    /// (see [`ErrorCollapseProfile`]). The SINGLE declared source the `SemanticKernelMetadata`
    /// projection derives `error_collapse_sensitive` / `error_algebra` from (and, for the
    /// reduction family, the `reduction_sensitive` / `numerical_reduction_policy` facet), so the
    /// error rule cannot diverge between the meta and a second id-keyed table. Functions that
    /// perform no error-collapse carry [`FunctionMeta::DEFAULT_ERROR_COLLAPSE_PROFILE`].
    pub error_collapse_profile: ErrorCollapseProfile,
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

    /// Surface-native lift/broadcast: the function's own surface evaluation does whatever
    /// lifting it needs (native elementwise lift, or none), so the by-index dispatch layer
    /// applies no extra positional scalar-array broadcast. This is the value the overwhelming
    /// majority of functions carry; only surfaces whose by-index arm is scalar-shaped and rely
    /// on the dispatch layer to broadcast over named argument positions override it with a
    /// [`LiftBroadcastProfile::ByIndexScalarArrayLift`] mask. Referenced by name (rather than
    /// spelled `LiftBroadcastProfile::SurfaceNative`) so a default literal needs no extra
    /// import beyond `FunctionMeta` itself — the same growth-discipline shape as
    /// `DEFAULT_ARG_PREPARATION_PROFILE` and `DEFAULT_REAL_RESULT_POLICY`.
    pub const DEFAULT_LIFT_BROADCAST_PROFILE: LiftBroadcastProfile =
        LiftBroadcastProfile::SurfaceNative;

    /// Construct a [`LiftBroadcastProfile::ByIndexScalarArrayLift`] over the given argument
    /// positions for a `FunctionMeta` literal's `lift_broadcast_profile` field. A thin
    /// re-export of [`LiftBroadcastProfile::lift_at`] so a meta literal that declares its mask
    /// needs no import beyond `FunctionMeta` itself — the same growth-discipline shape as the
    /// `DEFAULT_*` consts above.
    pub const fn lift_at(positions: &'static [usize]) -> LiftBroadcastProfile {
        LiftBroadcastProfile::lift_at(positions)
    }

    /// No error-collapse algebra: error inputs propagate through ordinary value handling. This is
    /// the value the overwhelming majority of functions carry; only reduction/aggregation folds
    /// and the branch-selector functions override it with a non-default
    /// [`ErrorCollapseProfile`]. Referenced by name (rather than spelled
    /// `ErrorCollapseProfile::None`) so a default literal needs no extra import beyond
    /// `FunctionMeta` itself — the same growth-discipline shape as the `DEFAULT_*` consts above.
    pub const DEFAULT_ERROR_COLLAPSE_PROFILE: ErrorCollapseProfile = ErrorCollapseProfile::None;
}
