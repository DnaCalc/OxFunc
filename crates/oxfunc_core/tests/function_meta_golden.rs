//! META-EQUALITY proof for the W105 oxf-y2uw.10 `function_spec!` defaulting-macro migration.
//!
//! The migration rewrote ~237 full `FunctionMeta { … }` literals into `function_spec! { … }`
//! calls that omit every DEFAULTABLE axis carrying its default and state only the non-default
//! overrides. The whole risk of that mechanical rewrite is that a field which was NON-default in
//! the literal is accidentally omitted — the macro would then silently fill the default and that
//! meta's value would change (a behaviour change) while still compiling.
//!
//! This test makes that impossible to land undetected: it renders the FULL field set of every
//! meta in the function catalog (`{:?}`, which prints every field of `FunctionMeta`), sorted by
//! `function_id`, and asserts the rendering is byte-identical to a golden snapshot captured from
//! the PRE-macro literal table (`tests/fixtures/function_meta_golden.txt`). Any changed field of
//! any meta — including a silently-defaulted override — is a diff, so it is a test failure rather
//! than a latent bug. The snapshot doubles as a standing regression guard for any future meta edit.
//!
//! To intentionally change a meta value, update the golden snapshot in the same commit; the diff
//! in review then shows exactly which field of which function changed.

use oxfunc_core::xll_export_specs::function_catalog;

/// Render every catalog meta's full field set, one `id => FunctionMeta { … }` line per meta,
/// sorted by id. `FunctionMeta` derives `Debug`, so `{:?}` enumerates every field; equality of
/// this rendering is equality of every field of every meta.
fn render_catalog_dump() -> String {
    let mut lines: Vec<String> = function_catalog()
        .iter()
        .map(|m| format!("{} => {:?}", m.function_id, m))
        .collect();
    lines.sort();
    lines.join("\n")
}

#[test]
fn function_spec_metas_are_bit_equal_to_pre_macro_golden() {
    let golden = include_str!("fixtures/function_meta_golden.txt");
    let actual = render_catalog_dump();
    // Compare line-by-line first so a failure points at the exact function/field that drifted.
    for (i, (g, a)) in golden.lines().zip(actual.lines()).enumerate() {
        assert_eq!(
            g,
            a,
            "function_spec! meta value drifted from the pre-macro golden at line {} \
             (a NON-default axis was likely dropped and silently defaulted):\n  golden: {}\n  actual: {}",
            i + 1,
            g,
            a
        );
    }
    assert_eq!(
        golden.lines().count(),
        actual.lines().count(),
        "catalog meta count changed vs the pre-macro golden snapshot"
    );
    // Normalize line endings: the golden is stored LF, but a Windows checkout with
    // core.autocrlf=true materializes it as CRLF, while render_catalog_dump joins
    // with LF. The per-line comparison above already tolerates this; keep the
    // whole-dump check newline-agnostic so it verifies content, not working-tree EOL.
    assert_eq!(
        golden.replace("\r\n", "\n"),
        actual,
        "function_spec! catalog dump is not byte-identical to the pre-macro golden"
    );
}
