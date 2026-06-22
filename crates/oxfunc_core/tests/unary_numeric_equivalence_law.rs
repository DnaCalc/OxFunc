//! W105 oxf-y2uw.1 — Cross-surface equivalence-law harness for the unary-numeric family.
//!
//! This is the behaviour-preservation safety net that MUST exist *before* any dispatch
//! path is collapsed (ODR-FN-004; W105 "Open Lanes" item 1). It encodes the laws as
//! generic, data-over-the-table tests: the unary-numeric family is enumerated once from a
//! single `unary_family()` table and every law iterates that table, so the harness scales
//! across the whole family with no per-function test code.
//!
//! Three execution surfaces are in play, and they are deliberately NOT all equated on the
//! same footing:
//!
//!   * the scalar `CalcValue` surface           (`eval_surface_value_call`, scalar input),
//!   * the array-lift `CalcValue` surface       (`eval_surface_value_call`, 1x1 array input),
//!   * the XLL `Q` scalar ABI                    (`eval_surface_q_unary_number`, raw f64).
//!
//! The `Q` path takes an already-coerced raw `f64` and runs **no** coercion / reference
//! prep, so it cannot be equated to the `CalcValue` paths on arbitrary `CalcValue` inputs.
//! Hence the split the bead mandates:
//!
//!   * **Law 1 — kernel-publication equivalence** (`law1_*`): on the *post-coercion numeric
//!     payload* only. For each family member and a representative set of already-coerced
//!     `f64` inputs (incl. overflow / NaN / domain-edge values), the published payload is
//!     identical across all three surfaces. Numeric payloads ONLY — never an arbitrary
//!     `CalcValue`.
//!   * **Law 2 — coercion / lift equivalence** (`law2_*`): tested SEPARATELY and ONLY across
//!     the two `CalcValue` surfaces (scalar vs array-lift). Argument coercion, reference
//!     resolution, and broadcast are identical between the scalar and array-lift surfaces.
//!     The `Q` path is explicitly out of scope here.
//!   * **Law 3 — overflow-guard invariant** (`law3_*`): every family kernel whose *raw*
//!     result can be non-finite for a finite argument declares a non-`PASS`
//!     `real_result_policy`.
//!
//! This bead is a test/harness bead only: it asserts the *current* cross-path behaviour and
//! must stay green on the untouched tree. It introduces no production behaviour change.

use oxfunc_core::function::ExcelRealPolicy;
use oxfunc_core::functions::surface_dispatch::{
    eval_surface_q_unary_number, eval_surface_value_call,
};
use oxfunc_core::resolver::{
    ReferenceDereferenceRequest, ReferenceEnumerationRequest, ReferenceResolutionError,
    ReferenceSystemCapabilities, ReferenceSystemProvider, ResolvedReferenceCell,
    ResolvedReferenceExtent, ResolvedReferenceValues, reference_identity_key,
};
use oxfunc_core::value::{
    CalcArray, CalcValue, CoreValue, ExcelText, ReferenceKind, ReferenceLike, WorksheetErrorCode,
};

use std::collections::BTreeMap;

// --- function modules (kernels + metas), reached as DATA for the family table ---------------
use oxfunc_core::functions::{
    abs, acot, asinh, atan, atanh, cos, cosh, cot, coth, csc, csch, degrees, even_fn, exp_fn, fact,
    factdouble, int_fn, ln_fn, log10_fn, odd_fn, operator_arithmetic_family as ops, radians, sec,
    sech, sign_fn, sin, sinh, sqrt_fn, sqrtpi, tan, tanh,
};

/// A normalized, bit-exact, comparable outcome of a single unary evaluation: either a
/// numeric payload (compared on raw IEEE-754 bits so that `NaN`/`-0.0` distinctions are
/// preserved) or a worksheet error code.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Number(u64),
    Error(WorksheetErrorCode),
    /// Anything that is neither a scalar number nor a scalar error (e.g. a non-scalar
    /// array, text passthrough, or `#VALUE!`-shaped surface rejection that produced a
    /// non-error/non-number value). Recorded verbatim so a mismatch is still diagnosable.
    Other(&'static str),
}

impl Outcome {
    fn number(n: f64) -> Self {
        Outcome::Number(n.to_bits())
    }

    fn describe(self) -> String {
        match self {
            Outcome::Number(bits) => {
                let v = f64::from_bits(bits);
                format!("Number({v:?} | bits=0x{bits:016x})")
            }
            Outcome::Error(code) => format!("Error({code:?})"),
            Outcome::Other(what) => format!("Other({what})"),
        }
    }
}

/// Collapse a `Q`-path `Result<f64, _>` into the comparable outcome.
fn outcome_from_q(result: Result<f64, WorksheetErrorCode>) -> Outcome {
    match result {
        Ok(n) => Outcome::number(n),
        Err(code) => Outcome::Error(code),
    }
}

/// Collapse a scalar `CalcValue`-surface `Result<CalcValue, _>` into the comparable outcome.
fn outcome_from_calc_scalar(result: &Result<CalcValue, WorksheetErrorCode>) -> Outcome {
    match result {
        Ok(value) => match value.core() {
            CoreValue::Number(n) => Outcome::number(*n),
            CoreValue::Error(code) => Outcome::Error(*code),
            CoreValue::Array(_) => Outcome::Other("array"),
            CoreValue::Text(_) => Outcome::Other("text"),
            CoreValue::Logical(_) => Outcome::Other("logical"),
            CoreValue::Empty => Outcome::Other("empty"),
            CoreValue::Missing => Outcome::Other("missing"),
            CoreValue::Reference(_) => Outcome::Other("reference"),
        },
        Err(code) => Outcome::Error(*code),
    }
}

/// Collapse a single cell of an array-lift `CalcValue`-surface result into the comparable
/// outcome. An array surface maps element errors into per-cell `CoreValue::Error`, so cell
/// extraction has to fold those into `Outcome::Error` exactly as the scalar path folds a
/// top-level `Err`.
fn outcome_from_cell(cell: &CalcValue) -> Outcome {
    match cell.core() {
        CoreValue::Number(n) => Outcome::number(*n),
        CoreValue::Error(code) => Outcome::Error(*code),
        CoreValue::Array(_) => Outcome::Other("array"),
        CoreValue::Text(_) => Outcome::Other("text"),
        CoreValue::Logical(_) => Outcome::Other("logical"),
        CoreValue::Empty => Outcome::Other("empty"),
        CoreValue::Missing => Outcome::Other("missing"),
        CoreValue::Reference(_) => Outcome::Other("reference"),
    }
}

/// A resolver that resolves no references (the scalar / array CalcValue surfaces consult it
/// only for `Reference` operands; Law 1 / Law 3 feed only numbers, so it is never hit).
struct NoResolver;

impl ReferenceSystemProvider for NoResolver {
    fn capabilities(&self) -> ReferenceSystemCapabilities {
        ReferenceSystemCapabilities::permissive_local()
    }

    fn dereference(
        &self,
        request: &ReferenceDereferenceRequest,
    ) -> Result<CalcValue, ReferenceResolutionError> {
        Err(ReferenceResolutionError::UnresolvedReference {
            target: request.reference.target().to_string(),
        })
    }
}

/// A resolver that maps each registered reference target to a fixed resolved-values grid via
/// `enumerate_values` — the path both `prepare_arg_values_only` and `expand_arg_values_only`
/// consult first. Used by Law 2 to drive reference resolution / broadcast through the two
/// CalcValue surfaces identically.
struct GridResolver {
    grids: BTreeMap<oxfunc_core::resolver::ReferenceIdentityKey, ResolvedReferenceValues>,
}

impl GridResolver {
    fn with(target: &str, rows: usize, cols: usize, cells: Vec<(usize, usize, CalcValue)>) -> Self {
        let key = reference_identity_key(&ReferenceLike::new(ReferenceKind::A1, target));
        let values = ResolvedReferenceValues::new(
            ResolvedReferenceExtent::new(rows, cols),
            cells
                .into_iter()
                .map(|(r, c, v)| ResolvedReferenceCell::new(r, c, v))
                .collect(),
            Some(format!("test:grid:{rows}x{cols}")),
        );
        let mut grids = BTreeMap::new();
        grids.insert(key, values);
        Self { grids }
    }
}

impl ReferenceSystemProvider for GridResolver {
    fn capabilities(&self) -> ReferenceSystemCapabilities {
        ReferenceSystemCapabilities::permissive_local()
    }

    fn dereference(
        &self,
        request: &ReferenceDereferenceRequest,
    ) -> Result<CalcValue, ReferenceResolutionError> {
        Err(ReferenceResolutionError::UnresolvedReference {
            target: request.reference.target().to_string(),
        })
    }

    fn enumerate_values(
        &self,
        request: &ReferenceEnumerationRequest,
    ) -> Result<Option<ResolvedReferenceValues>, ReferenceResolutionError> {
        Ok(self
            .grids
            .get(&reference_identity_key(&request.reference))
            .cloned())
    }
}

fn cell_ref(target: &str) -> CalcValue {
    CalcValue::reference(ReferenceLike::new(ReferenceKind::A1, target))
}

fn numeric_text(s: &str) -> CalcValue {
    CalcValue::text(ExcelText::from_utf16_code_units(s.encode_utf16().collect()))
}

/// Drive the *real* scalar CalcValue surface for `function_id` over a single argument.
fn scalar_surface(
    function_id: &str,
    arg: CalcValue,
    resolver: &dyn ReferenceSystemProvider,
) -> Result<CalcValue, WorksheetErrorCode> {
    eval_surface_value_call(function_id, &[arg], resolver, None, None, None, None)
}

/// Drive the *real* array-lift CalcValue surface for `function_id` by wrapping the argument
/// in a 1x1 array and evaluating; returns the (single) result cell as an `Outcome`.
fn array_lift_outcome(
    function_id: &str,
    arg: CalcValue,
    resolver: &dyn ReferenceSystemProvider,
) -> Result<Outcome, String> {
    let array =
        CalcValue::array(CalcArray::from_rows(vec![vec![arg]]).expect("1x1 array constructs"));
    let result = eval_surface_value_call(function_id, &[array], resolver, None, None, None, None);
    match result {
        Ok(value) => match value.core() {
            CoreValue::Array(array) => {
                let cells: Vec<&CalcValue> = array.iter_row_major().collect();
                if cells.len() != 1 {
                    return Err(format!(
                        "array-lift surface returned {} cells, expected 1",
                        cells.len()
                    ));
                }
                Ok(outcome_from_cell(cells[0]))
            }
            // A scalar input wrapped 1x1 might be scalarized by some surfaces; fold the
            // scalar shape identically to the scalar path so the comparison stays apples
            // to apples.
            _ => Ok(outcome_from_calc_scalar(&Ok(value))),
        },
        Err(code) => Ok(Outcome::Error(code)),
    }
}

// =====================================================================================
//  The single family table — enumerated ONCE; every law iterates it.
// =====================================================================================

/// The raw underlying f64 math of a family member, *before* result publication, exposed as
/// data for Law 3. `None` models an argument the kernel rejects with `Err` (no raw value to
/// inspect). For `f64 -> f64` kernels this is the bare kernel; for `Result`-returning
/// kernels (which own their own publication) it is the kernel's `Ok` value.
type RawProbe = fn(f64) -> Option<f64>;

struct FamilyMember {
    /// Canonical function id, e.g. `"FUNC.SIN"`.
    function_id: &'static str,
    /// The declared Excel result-publication policy carried on the function's `FunctionMeta`.
    policy: ExcelRealPolicy,
    /// Probe of the raw (pre-publication) math; see `RawProbe`.
    raw: RawProbe,
    /// `true` for kernels (FACT / FACTDOUBLE) whose cost is `O(truncated arg)` — they build
    /// the full factorial product by iteration before publishing, so feeding an astronomical
    /// magnitude (`2^27`, `1e308`, `f64::MAX`) would loop for billions of steps. Their
    /// overflow-to-`#NUM!` behaviour is already exercised by moderate magnitudes (FACT
    /// overflows at arg >= 171), so the laws cap their probe inputs via `MAX_ITERATING_ARG`.
    /// This is a per-row data property, not per-function test logic.
    iterates_over_arg: bool,
    /// `Some(reason)` for kernels that ALREADY violate the Law-3 invariant on the untouched
    /// tree (a real, pre-existing drift this harness surfaced). This is a test/harness bead:
    /// per the W105 charter it must NOT change production here, so the offender is recorded
    /// as a *documented* known-violation (printed by Law 3) rather than silently treated as
    /// correct, leaving a green safety net for the repair to land in a later bead. See the
    /// bead summary for the divergence detail.
    known_law3_violation: Option<&'static str>,
}

/// Inputs with `|x|` above this are not fed to `iterates_over_arg` kernels (see field doc).
/// Comfortably above FACT's (171) and FACTDOUBLE's (~300) overflow thresholds, so the
/// overflow path is still covered, while keeping the integer loop bounded.
const MAX_ITERATING_ARG: f64 = 1000.0;

fn ok<T>(r: Result<f64, T>) -> Option<f64> {
    r.ok()
}

/// Construct a family row that does NOT iterate over its argument magnitude.
fn m(function_id: &'static str, policy: ExcelRealPolicy, raw: RawProbe) -> FamilyMember {
    FamilyMember {
        function_id,
        policy,
        raw,
        iterates_over_arg: false,
        known_law3_violation: None,
    }
}

/// Construct a family row whose kernel cost is `O(truncated arg)` (FACT / FACTDOUBLE).
fn m_iter(function_id: &'static str, policy: ExcelRealPolicy, raw: RawProbe) -> FamilyMember {
    FamilyMember {
        iterates_over_arg: true,
        ..m(function_id, policy, raw)
    }
}

/// Construct a family row that already violates the Law-3 invariant on the untouched tree
/// (documented pre-existing drift; see `known_law3_violation`).
///
/// Retained as the Law-3 ratchet constructor even though no row currently uses it: the two
/// historical drifts (ASINH/SQRTPI internal-overflow → non-finite under PASS) have been
/// repaired (oxf-7m1k), so both rows are now plain `m(..)`. This constructor stays so any
/// future drift can be recorded as a *documented* known-violation rather than silently
/// passing — the same ratchet `law3_*` enforces. Not dead machinery; an intentionally
/// kept escape hatch (hence `#[allow(dead_code)]` until the next drift needs it).
#[allow(dead_code)]
fn m_known_law3(
    function_id: &'static str,
    policy: ExcelRealPolicy,
    raw: RawProbe,
    reason: &'static str,
) -> FamilyMember {
    FamilyMember {
        known_law3_violation: Some(reason),
        ..m(function_id, policy, raw)
    }
}

/// The unary-numeric family that is eligible on the XLL `Q` scalar surface
/// (`eval_surface_q_unary_number`). This is the exact intersection the cross-surface laws
/// quantify over: every id here is also served by the scalar / array-lift CalcValue
/// surfaces. Enumerated once; all three laws fold over it.
fn unary_family() -> Vec<FamilyMember> {
    use oxfunc_core::functions::surface_dispatch as sd;
    vec![
        m(sd::FUNC_ID_ABS, abs::ABS_META.real_result_policy, |n| {
            Some(abs::abs_kernel(n))
        }),
        m(sd::FUNC_ID_ACOT, acot::ACOT_META.real_result_policy, |n| {
            ok(acot::acot_kernel(n))
        }),
        m(sd::FUNC_ID_ATAN, atan::ATAN_META.real_result_policy, |n| {
            Some(atan::atan_kernel(n))
        }),
        m(
            sd::FUNC_ID_ASINH,
            asinh::ASINH_META.real_result_policy,
            |n| ok(asinh::asinh_kernel(n)),
        ),
        m(
            sd::FUNC_ID_ATANH,
            atanh::ATANH_META.real_result_policy,
            |n| ok(atanh::atanh_kernel(n)),
        ),
        m(sd::FUNC_ID_COS, cos::COS_META.real_result_policy, |n| {
            Some(cos::cos_kernel(n))
        }),
        m(sd::FUNC_ID_COSH, cosh::COSH_META.real_result_policy, |n| {
            Some(cosh::cosh_kernel(n))
        }),
        m(sd::FUNC_ID_COT, cot::COT_META.real_result_policy, |n| {
            ok(cot::cot_kernel(n))
        }),
        m(sd::FUNC_ID_COTH, coth::COTH_META.real_result_policy, |n| {
            ok(coth::coth_kernel(n))
        }),
        m(sd::FUNC_ID_CSC, csc::CSC_META.real_result_policy, |n| {
            ok(csc::csc_kernel(n))
        }),
        m(sd::FUNC_ID_CSCH, csch::CSCH_META.real_result_policy, |n| {
            ok(csch::csch_kernel(n))
        }),
        m(
            sd::FUNC_ID_DEGREES,
            degrees::DEGREES_META.real_result_policy,
            |n| Some(degrees::degrees_kernel(n)),
        ),
        m(
            sd::FUNC_ID_EVEN,
            even_fn::EVEN_META.real_result_policy,
            |n| ok(even_fn::even_kernel(n)),
        ),
        m(sd::FUNC_ID_EXP, exp_fn::EXP_META.real_result_policy, |n| {
            Some(exp_fn::exp_kernel(n))
        }),
        m_iter(sd::FUNC_ID_FACT, fact::FACT_META.real_result_policy, |n| {
            ok(fact::fact_kernel(n))
        }),
        m_iter(
            sd::FUNC_ID_FACTDOUBLE,
            factdouble::FACTDOUBLE_META.real_result_policy,
            |n| ok(factdouble::factdouble_kernel(n)),
        ),
        m(sd::FUNC_ID_INT, int_fn::INT_META.real_result_policy, |n| {
            ok(int_fn::int_kernel(n))
        }),
        m(sd::FUNC_ID_LN, ln_fn::LN_META.real_result_policy, |n| {
            ok(ln_fn::ln_kernel(n))
        }),
        m(
            sd::FUNC_ID_LOG10,
            log10_fn::LOG10_META.real_result_policy,
            |n| ok(log10_fn::log10_kernel(n)),
        ),
        m(sd::FUNC_ID_ODD, odd_fn::ODD_META.real_result_policy, |n| {
            ok(odd_fn::odd_kernel(n))
        }),
        m(
            sd::FUNC_ID_OP_NEGATE,
            ops::OP_NEGATE_META.real_result_policy,
            |n| ok(ops::op_negate_kernel(n)),
        ),
        m(
            sd::FUNC_ID_OP_PERCENT,
            ops::OP_PERCENT_META.real_result_policy,
            |n| ok(ops::op_percent_kernel(n)),
        ),
        m(
            sd::FUNC_ID_OP_UNARY_PLUS,
            ops::OP_UNARY_PLUS_META.real_result_policy,
            |n| ok(ops::op_unary_plus_kernel(n)),
        ),
        m(
            sd::FUNC_ID_RADIANS,
            radians::RADIANS_META.real_result_policy,
            |n| Some(radians::radians_kernel(n)),
        ),
        m(sd::FUNC_ID_SEC, sec::SEC_META.real_result_policy, |n| {
            ok(sec::sec_kernel(n))
        }),
        m(sd::FUNC_ID_SECH, sech::SECH_META.real_result_policy, |n| {
            ok(sech::sech_kernel(n))
        }),
        m(
            sd::FUNC_ID_SIGN,
            sign_fn::SIGN_META.real_result_policy,
            |n| ok(sign_fn::sign_kernel(n)),
        ),
        m(sd::FUNC_ID_SIN, sin::SIN_META.real_result_policy, |n| {
            Some(sin::sin_kernel(n))
        }),
        m(sd::FUNC_ID_SINH, sinh::SINH_META.real_result_policy, |n| {
            Some(sinh::sinh_kernel(n))
        }),
        m(
            sd::FUNC_ID_SQRT,
            sqrt_fn::SQRT_META.real_result_policy,
            |n| ok(sqrt_fn::sqrt_kernel(n)),
        ),
        m(
            sd::FUNC_ID_SQRTPI,
            sqrtpi::SQRTPI_META.real_result_policy,
            |n| ok(sqrtpi::sqrtpi_kernel(n)),
        ),
        m(sd::FUNC_ID_TAN, tan::TAN_META.real_result_policy, |n| {
            Some(tan::tan_kernel(n))
        }),
        m(sd::FUNC_ID_TANH, tanh::TANH_META.real_result_policy, |n| {
            Some(tanh::tanh_kernel(n))
        }),
    ]
}

/// Representative *already-coerced* numeric payloads for Law 1. Deliberately broad: normal
/// values, signed zeros, the circular-trig `2^27` boundary, overflow-inducing magnitudes,
/// hyperbolic-saturation magnitudes, domain edges (0, ±1) for log/inverse-hyperbolic
/// kernels, and the non-finite payloads themselves — Law 1 is a pure cross-path equivalence
/// property, valid for *any* f64 payload, so feeding `±Inf` / `NaN` directly is in scope.
fn law1_inputs() -> Vec<f64> {
    vec![
        0.0,
        -0.0,
        1.0,
        -1.0,
        0.5,
        -0.5,
        2.0,
        -2.0,
        3.0,
        -3.0,
        10.0,
        -10.0,
        100.0,
        -100.0,
        0.25,
        1.5,
        // hyperbolic saturation region (cosh/sinh -> Inf, COTH saturates to sign)
        710.0,
        -710.0,
        800.0,
        -800.0,
        // exp / general overflow region
        1000.0,
        -1000.0,
        1.0e308,
        -1.0e308,
        // circular-trig 2^27 guard boundary
        134_217_727.0,
        134_217_728.0,
        -134_217_728.0,
        // non-finite payloads (pure equivalence probe)
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NAN,
    ]
}

/// Finite-only inputs for Law 3. The invariant is "raw result can be non-finite *for a
/// finite argument*", so `±Inf` / `NaN` are intentionally excluded — an identity-shaped
/// kernel (e.g. unary minus) is only non-finite on a non-finite input, which Excel coercion
/// never produces, and must not be flagged.
fn law3_inputs() -> Vec<f64> {
    vec![
        0.0,
        -0.0,
        1.0,
        -1.0,
        0.5,
        -0.5,
        2.0,
        -2.0,
        10.0,
        -10.0,
        170.0,
        -170.0,
        700.0,
        -700.0,
        710.0,
        -710.0,
        800.0,
        -800.0,
        1000.0,
        -1000.0,
        1.0e300,
        -1.0e300,
        1.0e308,
        -1.0e308,
        f64::MAX,
        f64::MIN,
        134_217_727.0,
        134_217_728.0,
    ]
}

// =====================================================================================
//  Law 1 — kernel-publication equivalence (post-coercion numeric payload).
// =====================================================================================
#[test]
fn law1_kernel_publication_equivalence_scalar_array_lift_xll() {
    let no_resolver = NoResolver;
    let mut failures: Vec<String> = Vec::new();

    for member in unary_family() {
        for &x in &law1_inputs() {
            // O(arg)-cost kernels (FACT/FACTDOUBLE) are not fed astronomical magnitudes;
            // their overflow path is covered by moderate magnitudes (see field doc).
            if member.iterates_over_arg && x.abs() > MAX_ITERATING_ARG {
                continue;
            }

            let scalar = outcome_from_calc_scalar(&scalar_surface(
                member.function_id,
                CalcValue::number(x),
                &no_resolver,
            ));
            let array =
                match array_lift_outcome(member.function_id, CalcValue::number(x), &no_resolver) {
                    Ok(outcome) => outcome,
                    Err(detail) => {
                        failures.push(format!(
                            "{} @ x={:?}: array-lift surface error: {}",
                            member.function_id, x, detail
                        ));
                        continue;
                    }
                };
            let xll = outcome_from_q(eval_surface_q_unary_number(member.function_id, x));

            if scalar != array || array != xll {
                failures.push(format!(
                    "{} @ x={:?} (bits=0x{:016x}): scalar={}, array_lift={}, xll={}",
                    member.function_id,
                    x,
                    x.to_bits(),
                    scalar.describe(),
                    array.describe(),
                    xll.describe(),
                ));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "Law 1 (kernel-publication equivalence) broke for {} (function @ input) cases:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// =====================================================================================
//  Law 2 — coercion / lift equivalence (the two CalcValue surfaces only).
// =====================================================================================

/// One Law-2 case: an arbitrary `CalcValue` operand the *coercion / reference / broadcast*
/// machinery must treat identically on the scalar surface and on the array-lift surface.
struct CoercionCase {
    label: &'static str,
    make: fn() -> CalcValue,
    /// `None` => use a no-op resolver; `Some(_)` => use the grid resolver builder so a
    /// reference operand resolves.
    grid: Option<fn() -> GridResolver>,
}

fn law2_cases() -> Vec<CoercionCase> {
    vec![
        CoercionCase {
            label: "numeric-text",
            make: || numeric_text("2"),
            grid: None,
        },
        CoercionCase {
            label: "numeric-text-decimal",
            make: || numeric_text("3.5"),
            grid: None,
        },
        CoercionCase {
            label: "non-numeric-text",
            make: || numeric_text("not a number"),
            grid: None,
        },
        CoercionCase {
            label: "logical-true",
            make: || CalcValue::logical(true),
            grid: None,
        },
        CoercionCase {
            label: "logical-false",
            make: || CalcValue::logical(false),
            grid: None,
        },
        CoercionCase {
            label: "empty",
            make: || CalcValue::new(CoreValue::Empty),
            grid: None,
        },
        CoercionCase {
            label: "error-passthrough-div0",
            make: || CalcValue::error(WorksheetErrorCode::Div0),
            grid: None,
        },
        CoercionCase {
            label: "scalar-reference-to-number",
            make: || cell_ref("R1"),
            grid: Some(|| GridResolver::with("R1", 1, 1, vec![(0, 0, CalcValue::number(4.0))])),
        },
        CoercionCase {
            label: "scalar-reference-to-numeric-text",
            make: || cell_ref("R2"),
            grid: Some(|| GridResolver::with("R2", 1, 1, vec![(0, 0, numeric_text("9"))])),
        },
    ]
}

#[test]
fn law2_coercion_and_lift_equivalence_scalar_vs_array_lift() {
    // A representative function from each behavioural shape: a plain f64->f64 kernel (ABS),
    // an overflow-guarded one (EXP), a circular-trig-guarded one (SIN), a Result-returning
    // one (SQRT), and an operator (OP_NEGATE). Coercion / reference / broadcast prep is the
    // shared surface machinery, so a representative cross-section pins it without iterating
    // the whole family (which Law 1 already does on numeric payloads).
    use oxfunc_core::functions::surface_dispatch as sd;
    let functions = [
        sd::FUNC_ID_ABS,
        sd::FUNC_ID_EXP,
        sd::FUNC_ID_SIN,
        sd::FUNC_ID_SQRT,
        sd::FUNC_ID_OP_NEGATE,
    ];

    let mut failures: Vec<String> = Vec::new();

    for function_id in functions {
        for case in law2_cases() {
            let scalar_grid = case.grid.map(|f| f());
            let array_grid = case.grid.map(|f| f());
            let scalar_resolver: &dyn ReferenceSystemProvider = match &scalar_grid {
                Some(g) => g,
                None => &NoResolver,
            };
            let array_resolver: &dyn ReferenceSystemProvider = match &array_grid {
                Some(g) => g,
                None => &NoResolver,
            };

            let scalar = outcome_from_calc_scalar(&scalar_surface(
                function_id,
                (case.make)(),
                scalar_resolver,
            ));
            let array = match array_lift_outcome(function_id, (case.make)(), array_resolver) {
                Ok(outcome) => outcome,
                Err(detail) => {
                    failures.push(format!(
                        "{} / {}: array-lift surface error: {}",
                        function_id, case.label, detail
                    ));
                    continue;
                }
            };

            if scalar != array {
                failures.push(format!(
                    "{} / {}: scalar={}, array_lift={}",
                    function_id,
                    case.label,
                    scalar.describe(),
                    array.describe(),
                ));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "Law 2 (coercion/lift equivalence) broke for {} cases:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn law2_broadcast_equivalence_array_matches_scalar_elementwise() {
    // Broadcast: a multi-cell array operand must map elementwise to exactly what the scalar
    // surface yields for each cell evaluated alone. This pins the array-lift broadcast and
    // per-cell coercion against the scalar coercion path for the same operands.
    use oxfunc_core::functions::surface_dispatch as sd;
    let no_resolver = NoResolver;
    let functions = [sd::FUNC_ID_ABS, sd::FUNC_ID_SQRT, sd::FUNC_ID_SIN];

    let operands = || -> Vec<CalcValue> {
        vec![
            CalcValue::number(4.0),
            CalcValue::number(-9.0),
            numeric_text("16"),
            numeric_text("oops"),
            CalcValue::logical(true),
            CalcValue::error(WorksheetErrorCode::NA),
            CalcValue::number(134_217_728.0),
        ]
    };

    let mut failures: Vec<String> = Vec::new();

    for function_id in functions {
        // Per-cell scalar reference outcomes.
        let scalar_outcomes: Vec<Outcome> = operands()
            .into_iter()
            .map(|operand| {
                outcome_from_calc_scalar(&scalar_surface(function_id, operand, &no_resolver))
            })
            .collect();

        // One array operand (single row) through the array-lift surface.
        let array_arg = CalcValue::array(
            CalcArray::from_rows(vec![operands()]).expect("array operand constructs"),
        );
        let result = eval_surface_value_call(
            function_id,
            &[array_arg],
            &no_resolver,
            None,
            None,
            None,
            None,
        );

        match result {
            Ok(value) => match value.core() {
                CoreValue::Array(array) => {
                    let cells: Vec<Outcome> =
                        array.iter_row_major().map(outcome_from_cell).collect();
                    if cells.len() != scalar_outcomes.len() {
                        failures.push(format!(
                            "{}: broadcast produced {} cells, expected {}",
                            function_id,
                            cells.len(),
                            scalar_outcomes.len()
                        ));
                        continue;
                    }
                    for (idx, (got, want)) in cells.iter().zip(scalar_outcomes.iter()).enumerate() {
                        if got != want {
                            failures.push(format!(
                                "{} cell[{}]: array_lift={}, scalar={}",
                                function_id,
                                idx,
                                got.describe(),
                                want.describe(),
                            ));
                        }
                    }
                }
                other => failures.push(format!(
                    "{}: array operand did not lift to an array (got {:?})",
                    function_id, other
                )),
            },
            Err(code) => failures.push(format!(
                "{}: array operand surface returned top-level error {:?}",
                function_id, code
            )),
        }
    }

    assert!(
        failures.is_empty(),
        "Law 2 (broadcast equivalence) broke for {} cases:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// =====================================================================================
//  Law 3 — overflow-guard invariant.
// =====================================================================================
#[test]
fn law3_overflowing_kernel_declares_non_pass_policy() {
    let mut failures: Vec<String> = Vec::new();
    let mut documented: Vec<String> = Vec::new();

    for member in unary_family() {
        // A finite argument for which the raw (pre-publication) math is non-finite.
        // O(arg)-cost kernels (FACT/FACTDOUBLE) are probed only at bounded magnitudes; their
        // raw product still overflows to Inf well within that bound (FACT at arg >= 171).
        let offending = law3_inputs().into_iter().find(|&x| {
            x.is_finite()
                && !(member.iterates_over_arg && x.abs() > MAX_ITERATING_ARG)
                && matches!((member.raw)(x), Some(v) if !v.is_finite())
        });

        let raw_can_be_non_finite_under_pass =
            offending.is_some() && member.policy == ExcelRealPolicy::PASS;

        match (
            raw_can_be_non_finite_under_pass,
            member.known_law3_violation,
        ) {
            (true, None) => {
                let x = offending.expect("offending input present");
                let raw = (member.raw)(x).expect("offending input has a raw value");
                failures.push(format!(
                    "{}: raw result is non-finite ({:?}) for finite arg {:?}, but declares \
                     PASS real_result_policy (a non-finite value can leak unguarded)",
                    member.function_id, raw, x
                ));
            }
            (true, Some(reason)) => {
                let x = offending.expect("offending input present");
                documented.push(format!(
                    "{} (documented known violation @ arg {:?}): {}",
                    member.function_id, x, reason
                ));
            }
            (false, Some(reason)) => {
                // A documented known-violation that no longer reproduces: the underlying
                // drift was (perhaps inadvertently) repaired. Fail so the stale documentation
                // is removed and the function rejoins the enforced invariant — the honest
                // direction of the ratchet.
                failures.push(format!(
                    "{}: declared a known Law-3 violation ({}) but no finite-arg input now \
                     yields a non-finite raw result under PASS policy. The drift appears fixed; \
                     remove the `m_known_law3` exception so the invariant is enforced again.",
                    member.function_id, reason
                ));
            }
            (false, None) => {}
        }
    }

    // Surface the documented pre-existing violations so they stay visible in test output and
    // cannot rot unnoticed (they do not fail the build — this is a no-production-change bead).
    if !documented.is_empty() {
        eprintln!(
            "Law 3: {} documented pre-existing overflow-guard violation(s) (tracked for a later \
             repair bead, not fixed here):\n{}",
            documented.len(),
            documented.join("\n")
        );
    }

    assert!(
        failures.is_empty(),
        "Law 3 (overflow-guard invariant) broke for {} kernels:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// =====================================================================================
//  Table self-checks: keep the harness honest about what it actually covers.
// =====================================================================================
#[test]
fn family_table_is_the_full_q_eligible_unary_set_and_non_trivial() {
    let family = unary_family();
    // The Q table in surface_dispatch lists exactly these ids today; the harness must keep
    // pace with it. If a unary-numeric id is added to the Q surface, this count guards
    // against the harness silently under-covering the family.
    assert_eq!(
        family.len(),
        33,
        "expected 33 Q-eligible unary-numeric family members, got {}",
        family.len()
    );

    // Every declared family id must actually resolve on the Q surface (not fall through to
    // the `#VALUE!` arm), proving the table is the real Q-eligible set rather than a guess.
    for member in &family {
        let resolved = eval_surface_q_unary_number(member.function_id, 0.5);
        assert!(
            !matches!(resolved, Err(WorksheetErrorCode::Value)),
            "{} is in the family table but is not served by the Q surface",
            member.function_id
        );
    }

    // No duplicate ids.
    let mut ids: Vec<&str> = family.iter().map(|m| m.function_id).collect();
    ids.sort_unstable();
    let before = ids.len();
    ids.dedup();
    assert_eq!(before, ids.len(), "duplicate id in the family table");
}

#[test]
fn family_table_contains_each_policy_shape() {
    // Confidence that Law 1 / Law 3 actually exercise the guarded shapes, not just PASS
    // kernels: the family must include at least one of each non-default policy.
    let family = unary_family();
    let has = |p: ExcelRealPolicy| family.iter().any(|m| m.policy == p);
    assert!(
        has(ExcelRealPolicy::PASS),
        "expected a PASS member (e.g. ABS)"
    );
    assert!(
        has(ExcelRealPolicy::FINITE),
        "expected a FINITE member (e.g. EXP)"
    );
    assert!(
        has(ExcelRealPolicy::SATURATE_SIGN),
        "expected a SATURATE_SIGN member (e.g. COTH)"
    );
    assert!(
        has(ExcelRealPolicy::CIRCULAR_TRIG),
        "expected a CIRCULAR_TRIG member (e.g. SIN)"
    );
}
