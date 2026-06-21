//! W105 oxf-y2uw.9 — spec-driven generator for the unary-numeric by-index dispatch arms,
//! plus the drift-check and the interpreter-equals-generated equivalence test.
//!
//! ODR-FN-004 "Layer 4" / W105 "Open Lanes" item 4: the by-index dispatch table
//! (`surface_dispatch_by_index_generated.rs`) was historically generated FROM the hand-written
//! `eval_*_surface` string arms in `surface_dispatch.rs`. For the migrated unary-numeric family
//! this module flips the INPUT: the arm is emitted FROM each id's declarative
//! [`UnaryNumericExecSpec`] binding (its pure/fallible kernel + the `ExcelRealPolicy` carried on
//! its `FunctionMeta`), which is exactly the binding each `eval_*_surface` constructs internally.
//! The emitted arm calls the generic executor entry `eval_unary_numeric_via_executor(args,
//! resolver, SPEC)` directly, so the dispatch path for those ids no longer restates a per-id
//! surface function — it derives from the spec.
//!
//! Scope is EXPLICITLY partial. Only the 39 unary-numeric ids whose `eval_*_surface` is a pure
//! delegation to `eval_unary_numeric_via_executor` are migrated. Every other family (and the
//! one unary-shaped id whose surface is NOT a pure executor delegation — `OP_UNARY_PLUS`, which
//! has bespoke type-preserving identity semantics) stays on its hand-written arm until the
//! full-surface rollout (bead .12). See `MIGRATED_UNARY_NUMERIC` and the partition banner in the
//! generated file.
//!
//! XLL Q-SCALAR TABLE — DEFERRED (explicitly). The XLL `Q` path (`eval_surface_q_unary_number`)
//! is left on its hand-written arms, on purpose:
//!   * it is NOT a generated artifact — there is no `include!`d file and no generator for it, so
//!     "flip the generator's INPUT to the spec" (item 1) has no Q analogue to flip;
//!   * its arms are ALREADY pinned to the spec's behaviour bit-exactly by an existing guard:
//!     `unary_numeric_equivalence_law::law1_*` proves `xll_scalar(x) == scalar(x)` for every
//!     Q-eligible id, and `scalar(x)` is now the spec-driven by-index `execute` path — so the Q
//!     surface cannot silently drift from the spec without that law going red; and
//!   * rewriting each Q arm as an inline `execute(UnaryNumericExecSpec::..)` call would add a
//!     THIRD hand-construction of the per-id spec (a fresh drift surface) absent a single runtime
//!     spec registry, which does not exist until a later axis-widening bead — working against the
//!     workset's de-duplication goal. It is deferred to the .12 rollout for that reason.
//!
//! Three things live here, all `#[cfg(test)]` (this module ships NO runtime code):
//!   * [`emit_spec_driven_unary_numeric_arms`] — the generator: spec table -> arm source text.
//!   * `drift_check_committed_table_equals_generator` — fails the build if the committed block
//!     between the BEGIN/END sentinels diverges from the generator's output, so the
//!     "spec-driven / generated" claim cannot rot.
//!   * `regenerate_*` — an `--ignored` helper that rewrites the committed block in place.
//!   * `interpreter_equals_generated_dispatch_for_every_migrated_id` — the
//!     interpreter-equals-generated equivalence test (item 2).

#![cfg(test)]

use crate::function::ExcelRealPolicy;
use crate::functions::binary_numeric::BinaryNumericExecSpec;
use crate::functions::unary_numeric::UnaryNumericExecSpec;

/// One migrated unary-numeric arm. `spec` is the SPEC INPUT the generator reads: it is built
/// from the id's real kernel + its `FunctionMeta::real_result_policy`, identical to the binding
/// the id's `eval_*_surface` constructs. The `*_path` strings are the source-text identifiers
/// the generator needs to *write* the arm (the spec is a value, not source text); the drift
/// check + interpreter-equals test below prove the written arm matches the spec value, so these
/// strings cannot silently disagree with `spec`.
struct MigratedUnaryArm {
    function_id: &'static str,
    /// The declarative executor binding (kernel + policy) — the spec the arm is emitted from.
    spec: UnaryNumericExecSpec,
    /// `crate::functions::<module>` path segment, e.g. `"sin"` or `"operator_arithmetic_family"`.
    module: &'static str,
    /// kernel fn ident, e.g. `"sin_kernel"`.
    kernel: &'static str,
    /// `*_META` ident, e.g. `"SIN_META"`.
    meta: &'static str,
}

/// Build a migrated-arm row. The spec is constructed HERE from the live kernel + policy so the
/// generator's input is the declarative binding, not a transcription. `kernel_is_raw` is read
/// off the spec to choose the constructor — see `kernel_ctor`.
macro_rules! arm {
    ($id:expr, raw, $module:ident, $kernel:ident, $meta:ident) => {
        MigratedUnaryArm {
            function_id: $id,
            spec: UnaryNumericExecSpec::raw(
                crate::functions::$module::$kernel,
                crate::functions::$module::$meta.real_result_policy,
            ),
            module: stringify!($module),
            kernel: stringify!($kernel),
            meta: stringify!($meta),
        }
    };
    ($id:expr, fallible, $module:ident, $kernel:ident, $meta:ident) => {
        MigratedUnaryArm {
            function_id: $id,
            spec: UnaryNumericExecSpec::fallible(
                crate::functions::$module::$kernel,
                crate::functions::$module::$meta.real_result_policy,
            ),
            module: stringify!($module),
            kernel: stringify!($kernel),
            meta: stringify!($meta),
        }
    };
}

/// The migrated unary-numeric family: every id whose `eval_*_surface` is a pure delegation to
/// `eval_unary_numeric_via_executor`. This is the spec corpus the by-index generator consumes.
///
/// EXPLICITLY EXCLUDED (still hand-arm, pending bead .12):
///   * every non-unary-numeric family (the bulk of the table), and
///   * `OP_UNARY_PLUS` — it carries a `UnaryNumericExecSpec`-shaped meta but its surface is NOT
///     a pure executor delegation (type-preserving identity: text stays text, blank -> 0), so
///     routing it through the executor would change Excel-visible behaviour. It keeps its hand
///     arm.
fn migrated_unary_numeric() -> Vec<MigratedUnaryArm> {
    vec![
        arm!("FUNC.ACOS", fallible, acos, acos_kernel, ACOS_META),
        arm!("FUNC.ACOT", fallible, acot, acot_kernel, ACOT_META),
        arm!("FUNC.ACOSH", fallible, acosh, acosh_kernel, ACOSH_META),
        arm!("FUNC.ACOTH", fallible, acoth, acoth_kernel, ACOTH_META),
        arm!("FUNC.ABS", raw, abs, abs_kernel, ABS_META),
        arm!("FUNC.ASINH", fallible, asinh, asinh_kernel, ASINH_META),
        arm!("FUNC.ATAN", raw, atan, atan_kernel, ATAN_META),
        arm!("FUNC.ATANH", fallible, atanh, atanh_kernel, ATANH_META),
        arm!("FUNC.COS", raw, cos, cos_kernel, COS_META),
        arm!("FUNC.COSH", raw, cosh, cosh_kernel, COSH_META),
        arm!("FUNC.COT", fallible, cot, cot_kernel, COT_META),
        arm!("FUNC.COTH", fallible, coth, coth_kernel, COTH_META),
        arm!("FUNC.CSC", fallible, csc, csc_kernel, CSC_META),
        arm!("FUNC.CSCH", fallible, csch, csch_kernel, CSCH_META),
        arm!("FUNC.DEGREES", raw, degrees, degrees_kernel, DEGREES_META),
        arm!("FUNC.EVEN", fallible, even_fn, even_kernel, EVEN_META),
        arm!("FUNC.EXP", raw, exp_fn, exp_kernel, EXP_META),
        arm!("FUNC.FACT", fallible, fact, fact_kernel, FACT_META),
        arm!(
            "FUNC.FACTDOUBLE",
            fallible,
            factdouble,
            factdouble_kernel,
            FACTDOUBLE_META
        ),
        arm!(
            "FUNC.FISHER",
            fallible,
            fisher_fn,
            fisher_kernel,
            FISHER_META
        ),
        arm!(
            "FUNC.FISHERINV",
            fallible,
            fisherinv_fn,
            fisherinv_kernel,
            FISHERINV_META
        ),
        arm!("FUNC.GAUSS", fallible, gauss_fn, gauss_kernel, GAUSS_META),
        arm!("FUNC.INT", fallible, int_fn, int_kernel, INT_META),
        arm!("FUNC.LN", fallible, ln_fn, ln_kernel, LN_META),
        arm!("FUNC.LOG10", fallible, log10_fn, log10_kernel, LOG10_META),
        arm!("FUNC.ODD", fallible, odd_fn, odd_kernel, ODD_META),
        arm!("FUNC.PHI", fallible, phi_fn, phi_kernel, PHI_META),
        arm!(
            "FUNC.OP_NEGATE",
            fallible,
            operator_arithmetic_family,
            op_negate_kernel,
            OP_NEGATE_META
        ),
        arm!(
            "FUNC.OP_PERCENT",
            fallible,
            operator_arithmetic_family,
            op_percent_kernel,
            OP_PERCENT_META
        ),
        arm!("FUNC.RADIANS", raw, radians, radians_kernel, RADIANS_META),
        arm!("FUNC.SEC", fallible, sec, sec_kernel, SEC_META),
        arm!("FUNC.SECH", fallible, sech, sech_kernel, SECH_META),
        arm!("FUNC.SIGN", fallible, sign_fn, sign_kernel, SIGN_META),
        arm!("FUNC.SIN", raw, sin, sin_kernel, SIN_META),
        arm!("FUNC.SINH", raw, sinh, sinh_kernel, SINH_META),
        arm!("FUNC.SQRT", fallible, sqrt_fn, sqrt_kernel, SQRT_META),
        arm!("FUNC.SQRTPI", fallible, sqrtpi, sqrtpi_kernel, SQRTPI_META),
        arm!("FUNC.TAN", raw, tan, tan_kernel, TAN_META),
        arm!("FUNC.TANH", raw, tanh, tanh_kernel, TANH_META),
    ]
}

/// Resolve a migrated id to its `catalog_index` exactly as the runtime dispatch does, so the
/// generated arm key is derived (not hand-numbered). Panics with a clear message if an id is not
/// in the catalog — that is a real defect (a migrated id must be dispatchable).
fn catalog_index_for(function_id: &str) -> usize {
    crate::functions::surface_dispatch::resolve_surface_dispatch_key(function_id)
        .unwrap_or_else(|| panic!("migrated unary-numeric id {function_id} is not in the catalog"))
        .catalog_index()
}

/// The constructor token (`raw` / `fallible`) DERIVED from the spec's kernel variant — not a
/// hand-written per-arm flag — so the emitted source reflects the actual declared binding.
fn kernel_ctor(spec: UnaryNumericExecSpec) -> &'static str {
    if spec.kernel_is_raw() {
        "raw"
    } else {
        "fallible"
    }
}

/// THE generator. Emits the spec-driven by-index arms for the migrated unary-numeric family,
/// sorted by `catalog_index`, as the exact source text that lives between the BEGIN/END
/// sentinels in `surface_dispatch_by_index_generated.rs`. Every arm is derived from the
/// [`UnaryNumericExecSpec`] binding: the kernel/meta paths and the `raw`/`fallible` constructor
/// all come from the spec row, and the index comes from `resolve_surface_dispatch_key`.
fn emit_spec_driven_unary_numeric_arms() -> String {
    let mut rows: Vec<(usize, MigratedUnaryArm)> = migrated_unary_numeric()
        .into_iter()
        .map(|arm| (catalog_index_for(arm.function_id), arm))
        .collect();
    rows.sort_by_key(|(idx, _)| *idx);

    let mut out = String::new();
    for (idx, arm) in &rows {
        let ctor = kernel_ctor(arm.spec);
        out.push_str(&format!(
            "    // {fid}  [spec-driven: unary-numeric family, emitted from UnaryNumericExecSpec]\n",
            fid = arm.function_id
        ));
        out.push_str(&format!(
            "    {idx} => crate::functions::unary_numeric::eval_unary_numeric_via_executor(\n"
        ));
        out.push_str("        args,\n");
        out.push_str("        resolver,\n");
        out.push_str(&format!(
            "        crate::functions::unary_numeric::UnaryNumericExecSpec::{ctor}(\n"
        ));
        out.push_str(&format!(
            "            crate::functions::{module}::{kernel},\n",
            module = arm.module,
            kernel = arm.kernel
        ));
        out.push_str(&format!(
            "            crate::functions::{module}::{meta}.real_result_policy,\n",
            module = arm.module,
            meta = arm.meta
        ));
        out.push_str("        ),\n");
        out.push_str("    )\n");
        out.push_str("    .map_err(|e| crate::functions::unary_numeric::map_unary_numeric_error_to_ws(&e)),\n");
    }
    // Trim the trailing newline so the block compares cleanly against the slice between markers.
    out.pop();
    out
}

const BEGIN_MARKER: &str = "    // >>> BEGIN spec-driven unary-numeric arms (oxf-y2uw.9) -- generator-owned; do not hand-edit";
const END_MARKER: &str = "    // <<< END spec-driven unary-numeric arms (oxf-y2uw.9)";

// =====================================================================================
//  W105 oxf-y2uw.12.1 — spec-driven generator for the binary-arithmetic operator family.
//
//  Same shape as the unary block above, one arity up: each migrated id's by-index arm is
//  emitted FROM its declarative `BinaryNumericExecSpec` binding (its raw/fallible 2-arg kernel +
//  the `ExcelRealPolicy` carried on its `FunctionMeta`), calling the generic executor entry
//  `binary_numeric::eval_binary_numeric_via_executor(args, resolver, SPEC)` directly. These ids
//  were previously double-served (intercepted by the now-deleted
//  `eval_binary_arithmetic_calc_dispatch` while a shadowed hand arm also existed); after this
//  bead they are served by exactly one path — these spec-driven arms.
// =====================================================================================

const BINARY_BEGIN_MARKER: &str = "    // >>> BEGIN spec-driven binary-arithmetic arms (oxf-y2uw.12.1) -- generator-owned; do not hand-edit";
const BINARY_END_MARKER: &str = "    // <<< END spec-driven binary-arithmetic arms (oxf-y2uw.12.1)";

fn generated_table_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/functions/surface_dispatch_by_index_generated.rs")
}

/// Extract the source slice strictly between the given BEGIN and END sentinels (exclusive), with
/// a trailing newline trimmed, so it compares apples-to-apples with the generator output.
fn committed_block_between(file: &str, begin_marker: &str, end_marker: &str) -> String {
    let begin = file
        .find(begin_marker)
        .expect("BEGIN sentinel present in generated table");
    let after_begin = begin + begin_marker.len();
    let end_rel = file[after_begin..]
        .find(end_marker)
        .expect("END sentinel present in generated table");
    let block = &file[after_begin..after_begin + end_rel];
    // `block` starts with the newline after BEGIN and ends with the newline before END.
    // Normalize CRLF -> LF so the check is robust if `core.autocrlf` rewrites the checkout
    // (the generator always emits LF), then trim the framing newlines.
    block.replace("\r\n", "\n").trim_matches('\n').to_string()
}

/// Extract the committed unary-numeric spec-driven block.
fn committed_spec_block(file: &str) -> String {
    committed_block_between(file, BEGIN_MARKER, END_MARKER)
}

/// One migrated binary-arithmetic arm — the 2-arg analogue of [`MigratedUnaryArm`]. `spec` is
/// the SPEC INPUT the generator reads, built from the id's real kernel + its
/// `FunctionMeta::real_result_policy`, identical to the binding the calc-dispatch / by-index
/// surface would construct. The `*_path` strings are the source-text identifiers the generator
/// needs to *write* the arm; the drift check + interpreter-equals test prove the written arm
/// matches the spec value. Because the kernel and the meta can live in different modules
/// (OP_POWER binds `power_fn::power_kernel` but carries `operator_arithmetic_family::OP_POWER_META`),
/// the kernel-module and meta-module paths are recorded separately.
struct MigratedBinaryArm {
    function_id: &'static str,
    /// The declarative executor binding (kernel + policy) — the spec the arm is emitted from.
    spec: BinaryNumericExecSpec,
    /// `crate::functions::<module>` segment for the KERNEL, e.g. `"power_fn"`.
    kernel_module: &'static str,
    /// kernel fn ident, e.g. `"power_kernel"`.
    kernel: &'static str,
    /// `crate::functions::<module>` segment for the `*_META`, e.g. `"operator_arithmetic_family"`.
    meta_module: &'static str,
    /// `*_META` ident, e.g. `"OP_POWER_META"`.
    meta: &'static str,
}

/// Build a migrated binary arm. The spec is constructed HERE from the live kernel + policy so the
/// generator's input is the declarative binding, not a transcription. `kernel_is_raw` is read off
/// the spec to choose the constructor — see `binary_kernel_ctor`.
macro_rules! binary_arm {
    ($id:expr, raw, $kmod:ident, $kernel:ident, $mmod:ident, $meta:ident) => {
        MigratedBinaryArm {
            function_id: $id,
            spec: BinaryNumericExecSpec::raw(
                crate::functions::$kmod::$kernel,
                crate::functions::$mmod::$meta.real_result_policy,
            ),
            kernel_module: stringify!($kmod),
            kernel: stringify!($kernel),
            meta_module: stringify!($mmod),
            meta: stringify!($meta),
        }
    };
    ($id:expr, fallible, $kmod:ident, $kernel:ident, $mmod:ident, $meta:ident) => {
        MigratedBinaryArm {
            function_id: $id,
            spec: BinaryNumericExecSpec::fallible(
                crate::functions::$kmod::$kernel,
                crate::functions::$mmod::$meta.real_result_policy,
            ),
            kernel_module: stringify!($kmod),
            kernel: stringify!($kernel),
            meta_module: stringify!($mmod),
            meta: stringify!($meta),
        }
    };
}

/// The migrated binary-arithmetic family: the 6 operator ids previously served by the deleted
/// `eval_binary_arithmetic_calc_dispatch`. OP_ADD is the one `Raw` (infallible) kernel; the rest
/// own their compute + publication (`Fallible`). OP_POWER and POWER both bind `power_kernel`, so
/// POWER's declared `IntegerExponentPublication` precision quirk applies identically through both
/// arms.
fn migrated_binary_numeric() -> Vec<MigratedBinaryArm> {
    vec![
        binary_arm!("FUNC.MOD", fallible, mod_fn, mod_kernel, mod_fn, MOD_META),
        binary_arm!(
            "FUNC.OP_ADD",
            raw,
            op_add,
            op_add_kernel,
            op_add,
            OP_ADD_META
        ),
        binary_arm!(
            "FUNC.OP_SUBTRACT",
            fallible,
            operator_arithmetic_family,
            op_subtract_kernel,
            operator_arithmetic_family,
            OP_SUBTRACT_META
        ),
        binary_arm!(
            "FUNC.OP_MULTIPLY",
            fallible,
            operator_arithmetic_family,
            op_multiply_kernel,
            operator_arithmetic_family,
            OP_MULTIPLY_META
        ),
        binary_arm!(
            "FUNC.OP_DIVIDE",
            fallible,
            operator_arithmetic_family,
            op_divide_kernel,
            operator_arithmetic_family,
            OP_DIVIDE_META
        ),
        binary_arm!(
            "FUNC.OP_POWER",
            fallible,
            power_fn,
            power_kernel,
            operator_arithmetic_family,
            OP_POWER_META
        ),
        binary_arm!(
            "FUNC.POWER",
            fallible,
            power_fn,
            power_kernel,
            power_fn,
            POWER_META
        ),
    ]
}

/// The constructor token (`raw` / `fallible`) DERIVED from the spec's kernel variant.
fn binary_kernel_ctor(spec: BinaryNumericExecSpec) -> &'static str {
    if spec.kernel_is_raw() {
        "raw"
    } else {
        "fallible"
    }
}

/// THE binary generator. Emits the spec-driven by-index arms for the migrated binary-arithmetic
/// family, sorted by `catalog_index`, as the exact source text that lives between the binary
/// BEGIN/END sentinels in `surface_dispatch_by_index_generated.rs`. Every arm is derived from the
/// [`BinaryNumericExecSpec`] binding.
fn emit_spec_driven_binary_numeric_arms() -> String {
    let mut rows: Vec<(usize, MigratedBinaryArm)> = migrated_binary_numeric()
        .into_iter()
        .map(|arm| (catalog_index_for(arm.function_id), arm))
        .collect();
    rows.sort_by_key(|(idx, _)| *idx);

    let mut out = String::new();
    for (idx, arm) in &rows {
        let ctor = binary_kernel_ctor(arm.spec);
        out.push_str(&format!(
            "    // {fid}  [spec-driven: binary-arithmetic family, emitted from BinaryNumericExecSpec]\n",
            fid = arm.function_id
        ));
        out.push_str(&format!(
            "    {idx} => crate::functions::binary_numeric::eval_binary_numeric_via_executor(\n"
        ));
        out.push_str("        args,\n");
        out.push_str("        resolver,\n");
        out.push_str(&format!(
            "        crate::functions::binary_numeric::BinaryNumericExecSpec::{ctor}(\n"
        ));
        out.push_str(&format!(
            "            crate::functions::{kmod}::{kernel},\n",
            kmod = arm.kernel_module,
            kernel = arm.kernel
        ));
        out.push_str(&format!(
            "            crate::functions::{mmod}::{meta}.real_result_policy,\n",
            mmod = arm.meta_module,
            meta = arm.meta
        ));
        out.push_str("        ),\n");
        out.push_str("    )\n");
        out.push_str("    .map_err(|e| crate::functions::binary_numeric::map_binary_numeric_error_to_ws(&e)),\n");
    }
    // Trim the trailing newline so the block compares cleanly against the slice between markers.
    out.pop();
    out
}

mod tests {
    use super::*;
    use crate::functions::binary_numeric::execute as binary_execute;
    use crate::functions::surface_dispatch::{
        eval_surface_q_binary_number, eval_surface_value_call,
    };
    use crate::functions::unary_numeric::execute;
    use crate::resolver::{
        ReferenceDereferenceRequest, ReferenceResolutionError, ReferenceSystemCapabilities,
        ReferenceSystemProvider,
    };
    use crate::value::{CalcValue, CoreValue, WorksheetErrorCode};

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

    /// DRIFT CHECK. The committed spec-driven block must equal exactly what the generator emits
    /// today. This is what makes the "generated from the spec" claim non-rotting: if anyone
    /// hand-edits the block, adds/removes a migrated id, or changes a kernel/policy binding
    /// without re-running the generator, the committed file and the freshly-emitted text diverge
    /// and the build fails. (Because the generator builds each arm's `UnaryNumericExecSpec` from
    /// the live kernel + meta, the emitted `raw`/`fallible` constructor is the real binding, not
    /// a copy of the file.)
    #[test]
    fn drift_check_committed_table_equals_generator() {
        let file = std::fs::read_to_string(generated_table_path())
            .expect("read committed generated dispatch table");
        let committed = committed_spec_block(&file);
        let generated = emit_spec_driven_unary_numeric_arms();
        assert_eq!(
            committed, generated,
            "the committed spec-driven unary-numeric block has drifted from the generator. \
             Re-run: cargo test -p oxfunc_core -- --ignored \
             regenerate_surface_dispatch_unary_numeric_arms"
        );
    }

    /// Regeneration helper (run with `--ignored`): rewrite the spec-driven block in place from
    /// the generator. Kept out of the default run so it never mutates files during a normal
    /// `cargo test`; the drift check is the enforcing guard.
    #[test]
    #[ignore = "writes surface_dispatch_by_index_generated.rs; run explicitly to regenerate"]
    fn regenerate_surface_dispatch_unary_numeric_arms() {
        let path = generated_table_path();
        let file = std::fs::read_to_string(&path).expect("read committed generated dispatch table");
        let begin = file.find(BEGIN_MARKER).expect("BEGIN sentinel");
        let after_begin = begin + BEGIN_MARKER.len();
        let end_rel = file[after_begin..].find(END_MARKER).expect("END sentinel");
        let mut rebuilt = String::new();
        rebuilt.push_str(&file[..after_begin]);
        rebuilt.push('\n');
        rebuilt.push_str(&emit_spec_driven_unary_numeric_arms());
        rebuilt.push('\n');
        rebuilt.push_str(&file[after_begin + end_rel..]);
        std::fs::write(&path, rebuilt).expect("write regenerated dispatch table");
    }

    /// The generator's spec table must cover the same id set the cross-surface law harness +
    /// routing guard cover (33 Q-eligible + the 6 non-Q-eligible served unary ids), i.e. 39
    /// migrated arms. Guards the table from silently shrinking.
    #[test]
    fn migrated_table_is_the_full_pure_delegation_set() {
        let rows = migrated_unary_numeric();
        assert_eq!(
            rows.len(),
            39,
            "expected 39 migrated (pure-executor-delegation) unary-numeric ids"
        );
        // Every migrated id resolves in the catalog (so its arm key is real).
        let mut ids: Vec<&str> = rows.iter().map(|r| r.function_id).collect();
        for id in &ids {
            let _ = catalog_index_for(id);
        }
        ids.sort_unstable();
        let before = ids.len();
        ids.dedup();
        assert_eq!(before, ids.len(), "duplicate id in the migrated table");
    }

    // --- W105 oxf-y2uw.12.1 binary-arithmetic generator guards ----------------------------

    /// DRIFT CHECK for the binary block — the 2-arg analogue of the unary drift check. The
    /// committed binary spec-driven block must equal exactly what the binary generator emits
    /// today, so the "generated from the spec" claim cannot rot.
    #[test]
    fn binary_drift_check_committed_table_equals_generator() {
        let file = std::fs::read_to_string(generated_table_path())
            .expect("read committed generated dispatch table");
        let committed = committed_block_between(&file, BINARY_BEGIN_MARKER, BINARY_END_MARKER);
        let generated = emit_spec_driven_binary_numeric_arms();
        assert_eq!(
            committed, generated,
            "the committed spec-driven binary-arithmetic block has drifted from the generator. \
             Re-run: cargo test -p oxfunc_core -- --ignored \
             regenerate_surface_dispatch_binary_numeric_arms"
        );
    }

    /// Regeneration helper for the binary block (run with `--ignored`).
    #[test]
    #[ignore = "writes surface_dispatch_by_index_generated.rs; run explicitly to regenerate"]
    fn regenerate_surface_dispatch_binary_numeric_arms() {
        let path = generated_table_path();
        let file = std::fs::read_to_string(&path).expect("read committed generated dispatch table");
        let begin = file
            .find(BINARY_BEGIN_MARKER)
            .expect("binary BEGIN sentinel");
        let after_begin = begin + BINARY_BEGIN_MARKER.len();
        let end_rel = file[after_begin..]
            .find(BINARY_END_MARKER)
            .expect("binary END sentinel");
        let mut rebuilt = String::new();
        rebuilt.push_str(&file[..after_begin]);
        rebuilt.push('\n');
        rebuilt.push_str(&emit_spec_driven_binary_numeric_arms());
        rebuilt.push('\n');
        rebuilt.push_str(&file[after_begin + end_rel..]);
        std::fs::write(&path, rebuilt).expect("write regenerated dispatch table");
    }

    /// The binary generator's spec table must cover the 6 migrated operator ids (OP_POWER and
    /// POWER both ride `power_kernel`). Guards the table from silently shrinking.
    #[test]
    fn migrated_binary_table_is_the_full_operator_set() {
        let rows = migrated_binary_numeric();
        assert_eq!(
            rows.len(),
            7,
            "expected 7 migrated binary-arithmetic arms (MOD, OP_ADD, OP_SUBTRACT, OP_MULTIPLY, \
             OP_DIVIDE, OP_POWER, POWER)"
        );
        let mut ids: Vec<&str> = rows.iter().map(|r| r.function_id).collect();
        for id in &ids {
            let _ = catalog_index_for(id);
        }
        ids.sort_unstable();
        let before = ids.len();
        ids.dedup();
        assert_eq!(
            before,
            ids.len(),
            "duplicate id in the migrated binary table"
        );
    }

    /// INTERPRETER ≡ GENERATED for the binary family (bead item 2 / .9 analogue), authored
    /// data-over-the-table. For every migrated operator id and a representative pair-input set
    /// (incl. overflow / domain-edge / div-by-zero / negative-base / non-integer exponent), it
    /// asserts the GENERATED by-index dispatch path's published scalar result equals the direct
    /// `binary_execute(spec, lhs, rhs)` result — a wrong-routing detector for the generated
    /// binary arms (e.g. a swapped kernel or a lost POWER precision quirk would diverge).
    #[test]
    fn binary_interpreter_equals_generated_dispatch_for_every_migrated_id() {
        // Representative already-coercible pairs: signed zero, div-by-zero, large quotients
        // (MOD #NUM! threshold), POWER overflow / underflow / 0^0 / 0^-n / negative-base
        // reciprocal-odd roots and non-integer exponents, plus ordinary arithmetic.
        let pairs: &[(f64, f64)] = &[
            (6.0, 2.0),
            (-6.0, 2.0),
            (6.0, -2.0),
            (1.0, 0.0),
            (0.0, 0.0),
            (-0.0, 0.0),
            (0.0, -1.0),
            (3.0, 2.0),
            (-3.0, 2.0),
            (2.0, 3.0),
            (2.0, -3.0),
            (2.0, -1023.0),
            (10.0, 700.0),
            (0.001, -700.0),
            (10.0, -700.0),
            (-8.0, 1.0 / 3.0),
            (-8.0, 0.5),
            (1.05, 10.0),
            (1.005e14, 1.0),
            (1.0e-308, 1.0e-308),
            (5.0, 0.0),
        ];

        let resolver = NoResolver;
        let mut failures: Vec<String> = Vec::new();

        for arm in migrated_binary_numeric() {
            for &(lhs, rhs) in pairs {
                let direct = binary_execute(arm.spec, lhs, rhs);

                let dispatched = eval_surface_value_call(
                    arm.function_id,
                    &[CalcValue::number(lhs), CalcValue::number(rhs)],
                    &resolver,
                    None,
                    None,
                    None,
                    None,
                );

                let dispatched_norm: Result<u64, WorksheetErrorCode> = match dispatched {
                    Ok(value) => match value.core() {
                        CoreValue::Number(n) => Ok(n.to_bits()),
                        CoreValue::Error(code) => Err(*code),
                        other => {
                            failures.push(format!(
                                "{} @ ({:?},{:?}): generated dispatch produced non-scalar {:?}",
                                arm.function_id, lhs, rhs, other
                            ));
                            continue;
                        }
                    },
                    Err(code) => Err(code),
                };
                let direct_norm: Result<u64, WorksheetErrorCode> = direct.map(|n| n.to_bits());

                if dispatched_norm != direct_norm {
                    failures.push(format!(
                        "{} @ ({:?},{:?}): generated dispatch != binary_execute(spec): \
                         dispatch={:?}, execute={:?}",
                        arm.function_id, lhs, rhs, dispatched_norm, direct_norm,
                    ));
                }
            }
        }

        assert!(
            failures.is_empty(),
            "binary interpreter-equals-generated broke for {} (id @ input) cases:\n{}",
            failures.len(),
            failures.join("\n")
        );
    }

    /// BIT-EXACTNESS CROSS-PATH CHECK (the binary analogue of the `.1` Law-1
    /// kernel-publication equivalence). For each migrated operator id and a representative pair
    /// set (overflow / NaN-producing / 0 / negative / domain-edge / div-by-zero), the new binary
    /// executor's kernel-publication slice `binary_execute(spec, lhs, rhs)` must equal — bit for
    /// bit on the published `f64` — the UNCHANGED XLL `Q`-scalar reference
    /// `eval_surface_q_binary_number(id, lhs, rhs)`, which calls the same kernels directly and is
    /// the bit-exact reference for these 6 operators. This proves the migration is
    /// behaviour-preserving at the kernel-publication boundary (and, for POWER/OP_POWER, that the
    /// `IntegerExponentPublication` precision quirk applied inside `power_kernel` keeps applying).
    #[test]
    fn binary_executor_is_bit_exact_vs_q_scalar_reference() {
        // Overflow / NaN-producing / signed-zero / div-by-zero / negative-base / large-quotient
        // / integer-publication / underflow pairs, exercising every kernel's error + value edges.
        let pairs: &[(f64, f64)] = &[
            (6.0, 2.0),
            (-6.0, 2.0),
            (6.0, -2.0),
            (1.0, 0.0),
            (0.0, 0.0),
            (-0.0, 0.0),
            (0.0, -1.0),
            (3.0, 2.0),
            (-3.0, 2.0),
            (-3.0, -2.0),
            (2.0, 3.0),
            (2.0, -3.0),
            (2.0, -1023.0),
            (2.0, -1022.0),
            (10.0, 700.0),
            (0.001, -700.0),
            (10.0, -700.0),
            (-8.0, 1.0 / 3.0),
            (-27.0, 1.0 / 3.0),
            (-8.0, 0.5),
            (-8.0, 2.0 / 3.0),
            (1.05, 10.0),
            (1.01, 48.0),
            (1.005e14, 1.0),
            (-4.44e14, 0.288),
            (1.0e-308, 1.0e-308),
            (5.0e-308, 2.0),
            (f64::NAN, 1.0),
            (1.0, f64::NAN),
            (f64::INFINITY, 1.0),
        ];

        let mut failures: Vec<String> = Vec::new();
        for arm in migrated_binary_numeric() {
            for &(lhs, rhs) in pairs {
                let via_executor = binary_execute(arm.spec, lhs, rhs);
                let via_q = eval_surface_q_binary_number(arm.function_id, lhs, rhs);
                let exec_norm: Result<u64, WorksheetErrorCode> = via_executor.map(|n| n.to_bits());
                let q_norm: Result<u64, WorksheetErrorCode> = via_q.map(|n| n.to_bits());
                if exec_norm != q_norm {
                    failures.push(format!(
                        "{} @ ({:?},{:?}): executor != Q-scalar: executor={:?}, q={:?}",
                        arm.function_id, lhs, rhs, exec_norm, q_norm,
                    ));
                }
            }
        }
        assert!(
            failures.is_empty(),
            "binary executor diverged from the Q-scalar reference for {} case(s):\n{}",
            failures.len(),
            failures.join("\n")
        );
    }

    /// INTERPRETER ≡ GENERATED (bead item 2), authored data-over-the-table: one generic test
    /// folding the whole migrated family. For every migrated id and a representative input set
    /// (incl. overflow / NaN / domain-edge), it asserts the GENERATED dispatch path's published
    /// scalar result equals the direct `execute(spec, x)` result.
    ///
    /// WHY THIS IS NON-TAUTOLOGICAL: the two sides are produced by genuinely different code
    /// paths. The left side runs the REAL generated by-index table: `eval_surface_value_call`
    /// resolves the id to a `catalog_index` and dispatches through the *committed generated
    /// arm* (the spec-driven `match` arm in `surface_dispatch_by_index_generated.rs`), including
    /// argument coercion. The right side calls `execute(spec, x)` directly, where `spec` is the
    /// row's own `UnaryNumericExecSpec`. If a generated arm routed catalog_index N to the WRONG
    /// kernel or the WRONG policy (e.g. COS's index emitting `sin_kernel`, or EXP's index losing
    /// its `FINITE` policy and leaking +Inf), the dispatched result would differ from
    /// `execute(spec, …)` for some probe input and this test would fail. It is exactly a
    /// wrong-routing detector for the generated table, not a restatement of the executor.
    #[test]
    fn interpreter_equals_generated_dispatch_for_every_migrated_id() {
        // Representative already-coercible numeric inputs: signed zero, the 2^27 circular-trig
        // boundary, overflow magnitudes, hyperbolic saturation, domain edges (0, ±1), and the
        // non-finite payloads. FACT/FACTDOUBLE iterate over the (truncated) argument, so they
        // are not fed astronomical magnitudes; their overflow-to-#NUM! is covered at arg ~ 200.
        let general_inputs: &[f64] = &[
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
            100.0,
            -100.0,
            710.0,
            -710.0,
            1000.0,
            -1000.0,
            1.0e308,
            -1.0e308,
            134_217_727.0,
            134_217_728.0,
            -134_217_728.0,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NAN,
        ];
        // Bounded inputs for the O(arg)-cost factorial kernels (still crosses their overflow
        // threshold: FACT overflows at arg >= 171, FACTDOUBLE around ~300).
        let bounded_inputs: &[f64] = &[
            0.0, -0.0, 1.0, -1.0, 0.5, 2.0, -2.0, 5.0, 10.0, 170.0, 200.0, 400.0,
        ];

        let resolver = NoResolver;
        let mut failures: Vec<String> = Vec::new();

        for arm in migrated_unary_numeric() {
            let iterates = matches!(arm.function_id, "FUNC.FACT" | "FUNC.FACTDOUBLE");
            let inputs = if iterates {
                bounded_inputs
            } else {
                general_inputs
            };

            for &x in inputs {
                // Direct executor result for this id's own spec.
                let direct = execute(arm.spec, x);

                // Generated-dispatch result: through the real by-index table.
                let dispatched = eval_surface_value_call(
                    arm.function_id,
                    &[CalcValue::number(x)],
                    &resolver,
                    None,
                    None,
                    None,
                    None,
                );

                // Fold the dispatched scalar `CalcValue` into the executor's
                // `Result<f64, WorksheetErrorCode>` shape for a bit-exact comparison.
                let dispatched_norm: Result<u64, WorksheetErrorCode> = match dispatched {
                    Ok(value) => match value.core() {
                        CoreValue::Number(n) => Ok(n.to_bits()),
                        CoreValue::Error(code) => Err(*code),
                        other => {
                            failures.push(format!(
                                "{} @ x={:?}: generated dispatch produced non-scalar {:?}",
                                arm.function_id, x, other
                            ));
                            continue;
                        }
                    },
                    Err(code) => Err(code),
                };
                // Compare on raw IEEE-754 bits so NaN / -0.0 distinctions are preserved.
                let direct_norm: Result<u64, WorksheetErrorCode> = direct.map(|n| n.to_bits());

                if dispatched_norm != direct_norm {
                    failures.push(format!(
                        "{} @ x={:?} (bits=0x{:016x}): generated dispatch != execute(spec): \
                         dispatch={:?}, execute={:?}",
                        arm.function_id,
                        x,
                        x.to_bits(),
                        dispatched_norm,
                        direct_norm,
                    ));
                }
            }
        }

        assert!(
            failures.is_empty(),
            "interpreter-equals-generated broke for {} (id @ input) cases:\n{}",
            failures.len(),
            failures.join("\n")
        );
    }

    /// Guard the policy-shape coverage of the interpreter-equals test: the migrated family must
    /// exercise each non-default `ExcelRealPolicy`, so a wrong-policy routing bug in ANY shape
    /// would be caught (not just PASS kernels).
    #[test]
    fn migrated_family_exercises_each_policy_shape() {
        let rows = migrated_unary_numeric();
        let has = |p: ExcelRealPolicy| rows.iter().any(|r| r.spec.policy() == p);
        assert!(has(ExcelRealPolicy::PASS), "expected a PASS member (ABS)");
        assert!(
            has(ExcelRealPolicy::FINITE),
            "expected a FINITE member (EXP)"
        );
        assert!(
            has(ExcelRealPolicy::SATURATE_SIGN),
            "expected a SATURATE_SIGN member (COTH)"
        );
        assert!(
            has(ExcelRealPolicy::CIRCULAR_TRIG),
            "expected a CIRCULAR_TRIG member (SIN)"
        );
    }
}
