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

/// The precision/rounding quirk Excel applies when publishing a function's numeric result —
/// a *deviation* from the obvious floating-point math, NOT a function whose core purpose is
/// rounding. (`ROUND`/`MROUND`/`TRUNC`/`INT`/`CEILING`/`FLOOR` are not modelled here: rounding
/// is their defined behaviour, not a separable precision quirk.) This is a behavioural axis in
/// the `FunctionSpec` mould (ODR-FN-004 Layer 2, the axis widened under W105 oxf-y2uw.8): a
/// CLOSED enum whose variants name a real, observed Excel precision behaviour, carried on
/// [`FunctionMeta`] and read by the kernel that owns the quirk, so the rule is declared once and
/// the kernel cannot carry a second private copy of it. Functions Excel publishes with the plain
/// IEEE-754 kernel result carry [`FunctionMeta::DEFAULT_PRECISION_ROUNDING_PROFILE`]; only a
/// genuine precision deviation names a non-default variant.
///
/// GROWTH DISCIPLINE (the same pattern as [`ArgPreparationProfile`] / [`LiftBroadcastProfile`] /
/// [`ErrorCollapseProfile`]): a [`FunctionMeta`] field with a `DEFAULT_*` associated const for the
/// value the overwhelming majority carry, variants that name a behaviour (never a number), no free
/// `f64`/`String` on the meta, [`FunctionMeta`] stays `Copy`/`Eq`. ANY numeric threshold (what
/// counts as an exact integer, the binary-exponentiation algorithm) lives in the ONE impl that
/// interprets the variant — `crate::functions::power_fn` — never as data on the meta. A variant is
/// added only for a precision quirk some real function exhibits; speculative variants are not
/// introduced. Today the honest finding is a small axis: one real separable publication-time
/// precision deviation (`POWER`/`^`), so a mostly-`Default` axis with a single named variant is the
/// correct outcome rather than a manufactured taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrecisionRoundingProfile {
    /// The majority shape: Excel publishes the kernel's plain IEEE-754 result; the function
    /// applies no special precision/rounding deviation of its own.
    Default,
    /// `POWER(x, n)` and the `^` operator: when the exponent is an *exact integer*, Excel computes
    /// `x^n` by repeated multiplication (binary exponentiation) rather than `powf`/`exp(n·ln x)`,
    /// producing a bit-different — and "rounder", Excel-matching — result than the libm transcendental
    /// path. This is a genuine precision deviation (not a domain guard or non-finite rule, which the
    /// `real_result_policy` axis owns), so it is its own axis. The integer-detection tolerance and the
    /// repeated-multiplication algorithm live in `crate::functions::power_fn` (the impl that
    /// interprets this variant); only the *name* of the behaviour is carried here. Verified live
    /// Excel 16.0 build 20026 (`POWER(1.05,10)=1.6288946267774416`, etc.).
    IntegerExponentPublication,
}

impl PrecisionRoundingProfile {
    /// Whether this profile selects Excel's exact-integer-exponent publication (compute `x^n` by
    /// repeated multiplication when `n` is an exact integer). The SINGLE accessor the owning
    /// kernel reads, so the decision to apply the quirk is driven by the declared variant rather
    /// than a private copy of the rule inside the kernel. The integer-detection tolerance and the
    /// binary-exponentiation algorithm remain in the interpreting impl (`crate::functions::power_fn`).
    pub const fn uses_integer_exponent_publication(self) -> bool {
        matches!(self, Self::IntegerExponentPublication)
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
    /// The precision/rounding quirk Excel applies when publishing this function's numeric result
    /// (see [`PrecisionRoundingProfile`]). The SINGLE declared source the owning kernel reads, so a
    /// precision deviation cannot be hand-coded as a second private copy inside the kernel.
    /// Functions Excel publishes with the plain IEEE-754 kernel result carry
    /// [`FunctionMeta::DEFAULT_PRECISION_ROUNDING_PROFILE`]; only a genuine separable precision
    /// deviation (today: `POWER`/`^` integer-exponent publication) names a non-default variant.
    pub precision_rounding_profile: PrecisionRoundingProfile,
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

    /// No special precision/rounding deviation: Excel publishes the kernel's plain IEEE-754 result.
    /// This is the value the overwhelming majority of functions carry; only a function with a real
    /// separable publication-time precision quirk (today: `POWER`/`^`, which publishes an exact
    /// integer exponent via repeated multiplication) overrides it with a non-default
    /// [`PrecisionRoundingProfile`]. Referenced by name (rather than spelled
    /// `PrecisionRoundingProfile::Default`) so a default literal needs no extra import beyond
    /// `FunctionMeta` itself — the same growth-discipline shape as the `DEFAULT_*` consts above.
    pub const DEFAULT_PRECISION_ROUNDING_PROFILE: PrecisionRoundingProfile =
        PrecisionRoundingProfile::Default;

    /// The default-fill base the [`function_spec!`] macro draws every *omitted* DEFAULTABLE axis
    /// from. Each of the five defaultable axes is set to its `DEFAULT_*` here; the ten intrinsic
    /// per-function fields carry placeholder values that the macro caller ALWAYS shadows (every
    /// `function_spec!` invocation states all ten intrinsic fields by name, so the placeholders
    /// are never observed in a generated meta — they exist only so this is a complete, valid
    /// `const FunctionMeta`).
    ///
    /// THIS IS THE ONE PLACE A NEW DEFAULTABLE AXIS IS DEFAULTED. Adding a behavioural axis with a
    /// `DEFAULT_*` const is a single new line here (`new_axis: Self::DEFAULT_NEW_AXIS,`); every meta
    /// that carries the default then needs NO edit (the macro fills it from this base), and only the
    /// deviating functions add a one-line `new_axis: …,` override to their `function_spec!` call —
    /// retiring the all-literal field-add sweep the arg-prep / lift-broadcast / error-collapse /
    /// precision-rounding axis beads each had to perform over every full literal.
    pub const DEFAULTS_BASE: FunctionMeta = FunctionMeta {
        // Intrinsic placeholders — ALWAYS overridden by the `function_spec!` caller (see above).
        function_id: "",
        arity: Arity { min: 0, max: 0 },
        determinism: DeterminismClass::Deterministic,
        volatility: VolatilityClass::NonVolatile,
        host_interaction: HostInteractionClass::None,
        thread_safety: ThreadSafetyClass::SafePure,
        coercion_lift_profile: CoercionLiftProfile::None,
        kernel_signature_class: KernelSignatureClass::Custom,
        fec_dependency_profile: FecDependencyProfile::None,
        surface_fec_dependency_profile: FecDependencyProfile::None,
        // Defaultable axes — the macro fills any of these the caller omits FROM HERE.
        arg_preparation_profile: Self::DEFAULT_ARG_PREPARATION_PROFILE,
        lift_broadcast_profile: Self::DEFAULT_LIFT_BROADCAST_PROFILE,
        real_result_policy: Self::DEFAULT_REAL_RESULT_POLICY,
        error_collapse_profile: Self::DEFAULT_ERROR_COLLAPSE_PROFILE,
        precision_rounding_profile: Self::DEFAULT_PRECISION_ROUNDING_PROFILE,
    };
}

/// Build a [`FunctionMeta`] const, filling every DEFAULTABLE behavioural axis with its
/// `FunctionMeta::DEFAULT_*` and letting a call site override only the non-default axes BY NAME.
///
/// Usage — state the ten intrinsic per-function fields (which have no single default:
/// `function_id`, `arity`, `determinism`, `volatility`, `host_interaction`, `thread_safety`,
/// `coercion_lift_profile`, `kernel_signature_class`, `fec_dependency_profile`,
/// `surface_fec_dependency_profile`) and, optionally, any of the five DEFAULTABLE axes that
/// deviate from the default (`arg_preparation_profile`, `lift_broadcast_profile`,
/// `real_result_policy`, `error_collapse_profile`, `precision_rounding_profile`). Fields may be
/// written in any order; any defaultable axis NOT named is filled from
/// [`FunctionMeta::DEFAULTS_BASE`]:
///
/// ```ignore
/// pub const ABS_META: FunctionMeta = function_spec! {
///     function_id: "FUNC.ABS",
///     arity: Arity::exact(1),
///     determinism: DeterminismClass::Deterministic,
///     volatility: VolatilityClass::NonVolatile,
///     host_interaction: HostInteractionClass::None,
///     thread_safety: ThreadSafetyClass::SafePure,
///     coercion_lift_profile: CoercionLiftProfile::UnaryNumericScalarOrArrayElementwise,
///     kernel_signature_class: KernelSignatureClass::NumToNum,
///     fec_dependency_profile: FecDependencyProfile::None,
///     surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
///     // arg_preparation_profile / lift_broadcast_profile / real_result_policy /
///     // error_collapse_profile / precision_rounding_profile all OMITTED → default.
/// };
/// ```
///
/// HOW DEFAULTS ARE FILLED: the macro expands to `FunctionMeta { <named fields>,
/// ..FunctionMeta::DEFAULTS_BASE }`. Rust's functional struct-update fills every field the caller
/// did not name from the base; the base sets each defaultable axis to its `DEFAULT_*`, so an
/// omitted axis gets its default and a named axis gets the override. This is a plain `const`
/// expression (no allocation, no trait dispatch), so it works wherever a `FunctionMeta` literal
/// did — every `*_META` is `pub const X_META: FunctionMeta`.
///
/// SAFETY NOTE: because `..DEFAULTS_BASE` also carries intrinsic placeholders, a *forgotten*
/// intrinsic field would silently take a placeholder (e.g. `function_id: ""`) rather than fail to
/// compile. That class of mistake is caught by the per-function meta tests, the catalog
/// conformance test, and the `function_spec_metas_are_bit_equal_to_pre_macro_golden` golden
/// equality test (every generated meta is asserted bit-equal to its pre-macro value), not by the
/// type system — state every intrinsic field on every call.
///
/// Scope: defined in `crate::function` and hoisted crate-wide by `#[macro_use] pub mod function;`
/// in `lib.rs`, so every `*_META` site (all under `crate::functions`, declared after `function`)
/// calls it by bare name with no per-file import. It is an internal authoring helper, not part of
/// the crate's public API.
macro_rules! function_spec {
    ( $( $field:ident : $value:expr ),+ $(,)? ) => {
        $crate::function::FunctionMeta {
            $( $field: $value, )+
            ..$crate::function::FunctionMeta::DEFAULTS_BASE
        }
    };
}

#[cfg(test)]
mod function_spec_macro_tests {
    use super::*;

    // A representative full intrinsic field set, repeated in each case below (a macro cannot be
    // expanded inside the `function_spec!` token stream, so the fields are stated inline; this also
    // keeps each case a faithful copy of a real `*_META` call shape). Mirrors a unary-numeric meta.

    /// Omitting every defaultable axis fills each one from its `DEFAULT_*` — the value the
    /// overwhelming majority of metas carry, so the majority of `function_spec!` calls name only
    /// the ten intrinsic fields (cf. `ABS_META`).
    #[test]
    fn omitted_defaultable_axes_take_their_defaults() {
        const M: FunctionMeta = function_spec! {
            function_id: "TEST.FUNC",
            arity: Arity::exact(1),
            determinism: DeterminismClass::Deterministic,
            volatility: VolatilityClass::NonVolatile,
            host_interaction: HostInteractionClass::None,
            thread_safety: ThreadSafetyClass::SafePure,
            coercion_lift_profile: CoercionLiftProfile::UnaryNumericScalarOrArrayElementwise,
            kernel_signature_class: KernelSignatureClass::NumToNum,
            fec_dependency_profile: FecDependencyProfile::None,
            surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
        };
        assert_eq!(
            M.arg_preparation_profile,
            FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE
        );
        assert_eq!(
            M.lift_broadcast_profile,
            FunctionMeta::DEFAULT_LIFT_BROADCAST_PROFILE
        );
        assert_eq!(
            M.real_result_policy,
            FunctionMeta::DEFAULT_REAL_RESULT_POLICY
        );
        assert_eq!(
            M.error_collapse_profile,
            FunctionMeta::DEFAULT_ERROR_COLLAPSE_PROFILE
        );
        assert_eq!(
            M.precision_rounding_profile,
            FunctionMeta::DEFAULT_PRECISION_ROUNDING_PROFILE
        );
        // Intrinsic fields come through verbatim — the placeholders in DEFAULTS_BASE are shadowed.
        assert_eq!(M.function_id, "TEST.FUNC");
        assert_eq!(M.arity, Arity::exact(1));
    }

    /// Naming one defaultable axis overrides exactly that axis and leaves the rest at their
    /// defaults (cf. `SIN_META` overriding only `real_result_policy`). Field order is free — here
    /// the override is written FIRST, before the intrinsic fields.
    #[test]
    fn a_named_override_changes_only_that_axis() {
        const M: FunctionMeta = function_spec! {
            real_result_policy: ExcelRealPolicy::CIRCULAR_TRIG,
            function_id: "TEST.FUNC",
            arity: Arity::exact(1),
            determinism: DeterminismClass::Deterministic,
            volatility: VolatilityClass::NonVolatile,
            host_interaction: HostInteractionClass::None,
            thread_safety: ThreadSafetyClass::SafePure,
            coercion_lift_profile: CoercionLiftProfile::UnaryNumericScalarOrArrayElementwise,
            kernel_signature_class: KernelSignatureClass::NumToNum,
            fec_dependency_profile: FecDependencyProfile::None,
            surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
        };
        assert_eq!(M.real_result_policy, ExcelRealPolicy::CIRCULAR_TRIG);
        // Every other defaultable axis still defaults.
        assert_eq!(
            M.arg_preparation_profile,
            FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE
        );
        assert_eq!(
            M.precision_rounding_profile,
            FunctionMeta::DEFAULT_PRECISION_ROUNDING_PROFILE
        );
    }

    /// The macro is a `const` expression: `M` and `_ID` below are `const`, which only compiles
    /// because `function_spec!` expands to a plain `FunctionMeta { … }` struct literal (no
    /// allocation, no trait dispatch), exactly as the `pub const X_META: FunctionMeta` sites require.
    #[test]
    fn usable_in_const_context() {
        const M: FunctionMeta = function_spec! {
            function_id: "TEST.FUNC",
            arity: Arity::exact(1),
            determinism: DeterminismClass::Deterministic,
            volatility: VolatilityClass::NonVolatile,
            host_interaction: HostInteractionClass::None,
            thread_safety: ThreadSafetyClass::SafePure,
            coercion_lift_profile: CoercionLiftProfile::UnaryNumericScalarOrArrayElementwise,
            kernel_signature_class: KernelSignatureClass::NumToNum,
            fec_dependency_profile: FecDependencyProfile::None,
            surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
        };
        const _ID: &str = M.function_id;
        assert_eq!(_ID, "TEST.FUNC");
    }

    // ----------------------------------------------------------------------------------------
    // NEW-AXIS-ONE-LINE DEMONSTRATION (the bead's acceptance criterion).
    //
    // This is the property `function_spec!` exists to deliver: adding a NEW behavioural axis costs
    // ONE line in the macro defaults (`DEFAULTS_BASE`) plus a one-line override ONLY on the
    // functions that deviate — NOT an all-literal sweep over every `*_META`.
    //
    // We cannot extend the real `FunctionMeta` here (that is the next axis-widening bead's job),
    // so the demonstration uses a faithful STANDALONE replica of the exact pattern: a struct with
    // intrinsic + defaultable fields, a `DEFAULTS_BASE` const, and a `..DEFAULTS_BASE` default-fill
    // macro identical in shape to `function_spec!`. The walkthrough below shows that introducing a
    // brand-new defaultable axis (`new_axis`) touches only:
    //   (1) the new field on the struct + its `DEFAULT_NEW_AXIS` const,
    //   (2) ONE new line in `DEFAULTS_BASE` (`new_axis: Self::DEFAULT_NEW_AXIS,`),
    //   (3) a one-line `new_axis: …` override on ONLY the deviating call site.
    // Every pre-existing call site that takes the default is UNCHANGED — the macro fills the new
    // axis for it. In the real crate, step (2) is the single edit to `FunctionMeta::DEFAULTS_BASE`
    // and step (3) is one line on each deviating `*_META`; the ~237 default-carrying metas need no
    // edit at all.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum NewAxis {
        Default,
        Deviation,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct DemoMeta {
        id: &'static str,
        existing_default_axis: u8,
        new_axis: NewAxis, // (1) the freshly added axis
    }

    impl DemoMeta {
        const DEFAULT_EXISTING: u8 = 0;
        const DEFAULT_NEW_AXIS: NewAxis = NewAxis::Default; // (1) its default const

        const DEFAULTS_BASE: DemoMeta = DemoMeta {
            id: "",
            existing_default_axis: Self::DEFAULT_EXISTING,
            new_axis: Self::DEFAULT_NEW_AXIS, // (2) THE ONE LINE that defaults the new axis everywhere
        };
    }

    macro_rules! demo_spec {
        ( $( $field:ident : $value:expr ),+ $(,)? ) => {
            DemoMeta { $( $field: $value, )+ ..DemoMeta::DEFAULTS_BASE }
        };
    }

    #[test]
    fn new_axis_is_one_default_line_plus_only_the_overriding_call_site() {
        // A call site authored BEFORE the new axis existed — unchanged by the axis addition; the
        // macro fills `new_axis` from DEFAULTS_BASE.
        const PRE_EXISTING_META: DemoMeta = demo_spec! { id: "stays.default" };
        // The ONLY call site that deviates adds exactly ONE line.
        const DEVIATING_META: DemoMeta = demo_spec! {
            id: "deviates",
            new_axis: NewAxis::Deviation, // (3) the one-line override, only here
        };

        assert_eq!(PRE_EXISTING_META.new_axis, NewAxis::Default);
        assert_eq!(DEVIATING_META.new_axis, NewAxis::Deviation);
        // Adding the axis did not disturb the pre-existing axis on the unchanged call site.
        assert_eq!(
            PRE_EXISTING_META.existing_default_axis,
            DemoMeta::DEFAULT_EXISTING
        );
    }
}
