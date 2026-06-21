use crate::coercion::{CoercionError, coerce_calc_scalar_to_number};
use crate::function::ExcelRealPolicy;
use crate::functions::adapters::{
    apply_unary_numeric_scalar_prepared, expand_arg_values_only, prepare_arg_values_only,
    prepare_calc_value_values_only,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::{CalcArray, CoreValue, WorksheetErrorCode};

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryNumericSurfaceError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

// =====================================================================================
//  W105 oxf-y2uw.3 — the generic unary-numeric executor (kernel-publication slice).
//
//  ODR-FN-004 Layer 3: `execute(spec, x)` is the ONE behavioural code path for the
//  unary-numeric family's post-coercion math. Its BOUNDED INTERFACE is exactly the
//  kernel-publication slice the pilot is scoped to: the input is an already-prepared
//  numeric `f64` (coercion / reference resolution / array lift happen in the surface
//  helpers, NOT here), and the output is the `ExcelRealPolicy::publish` shape
//  (`Result<f64, WorksheetErrorCode>`). Reference / context-provider functions are out
//  of scope by construction — there is nowhere to thread a context through this signature.
//
//  Each family member binds its pure kernel and its declared `ExcelRealPolicy` into a
//  `UnaryNumericExecSpec`. The spec is a `Copy`/`Eq` closed value (a function-pointer
//  kernel binding plus the closed-enum policy — no `String`, no free `f64`), matching the
//  `FunctionSpec` discipline so the binding is itself an inspectable value.
// =====================================================================================

/// The two kernel shapes the unary-numeric family uses, as a closed, `Copy`/`Eq` binding.
///
/// * `Raw` is a pure `f64 -> f64` kernel whose Excel publication (argument-domain guard +
///   non-finite handling) is supplied by the `ExcelRealPolicy` — exactly the historic
///   `policy.wrap(raw_kernel)` shape.
/// * `Fallible` is a kernel that owns its own compute and error algebra (domain rejection,
///   `#DIV/0!`, and — where relevant — the `check_arg` circular-trig guard it calls
///   internally), returning the published value directly — exactly the historic shape where
///   such a kernel was handed straight to the surface helper.
#[derive(Clone, Copy, Debug)]
pub enum UnaryNumericKernel {
    /// Pure `f64 -> f64`; publication is owned by the `ExcelRealPolicy`.
    Raw(fn(f64) -> f64),
    /// `f64 -> Result<f64, _>`; the kernel owns its compute and publication.
    Fallible(fn(f64) -> Result<f64, WorksheetErrorCode>),
}

impl PartialEq for UnaryNumericKernel {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Compare function pointers by address (the only `Eq`-able identity a `fn`
            // item has). Keeps the binding a comparable value without a free `f64`.
            (Self::Raw(a), Self::Raw(b)) => std::ptr::fn_addr_eq(*a, *b),
            (Self::Fallible(a), Self::Fallible(b)) => std::ptr::fn_addr_eq(*a, *b),
            _ => false,
        }
    }
}

impl Eq for UnaryNumericKernel {}

/// The kernel-publication binding for one unary-numeric family member: its kernel plus the
/// Excel publication policy declared on its `FunctionMeta`. `Copy`/`Eq` so it is an
/// inspectable value, in the `FunctionSpec` mould.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnaryNumericExecSpec {
    kernel: UnaryNumericKernel,
    policy: ExcelRealPolicy,
}

impl UnaryNumericExecSpec {
    /// Bind a pure `f64 -> f64` kernel whose publication is owned by `policy`.
    pub const fn raw(kernel: fn(f64) -> f64, policy: ExcelRealPolicy) -> Self {
        Self {
            kernel: UnaryNumericKernel::Raw(kernel),
            policy,
        }
    }

    /// Bind a `f64 -> Result<f64, _>` kernel that owns its own compute and publication.
    /// The policy is recorded for inspection/equivalence but the kernel is authoritative
    /// (it already applies any argument guard internally), so `execute` calls it directly —
    /// reproducing exactly the historic "hand the fallible kernel to the surface" behaviour.
    pub const fn fallible(
        kernel: fn(f64) -> Result<f64, WorksheetErrorCode>,
        policy: ExcelRealPolicy,
    ) -> Self {
        Self {
            kernel: UnaryNumericKernel::Fallible(kernel),
            policy,
        }
    }

    /// The declared Excel publication policy for this binding.
    pub const fn policy(self) -> ExcelRealPolicy {
        self.policy
    }
}

/// THE generic executor for the unary-numeric kernel-publication slice (ODR-FN-004 Layer 3).
///
/// Bounded interface: `x` is an already-prepared numeric payload; the result is the
/// `ExcelRealPolicy::publish` shape. This is the single behavioural definition every family
/// member's surface evaluation derives from — collapsing the historic
/// calc-dispatch-vs-by-index path drift (the `SIN`-rides-one-table-while-`COS`-rides-another
/// inconsistency) into one path.
pub fn execute(spec: UnaryNumericExecSpec, x: f64) -> Result<f64, WorksheetErrorCode> {
    match spec.kernel {
        UnaryNumericKernel::Raw(kernel) => spec.policy.publish(x, kernel(x)),
        UnaryNumericKernel::Fallible(kernel) => kernel(x),
    }
}

/// Bind a spec into the guarded `Fn(f64) -> Result<f64, _>` kernel the unary surface helpers
/// consume, so every dispatch site routes the post-coercion math through `execute`.
pub fn executor_kernel(
    spec: UnaryNumericExecSpec,
) -> impl Fn(f64) -> Result<f64, WorksheetErrorCode> + Copy {
    move |x| execute(spec, x)
}

/// Evaluate the unary-numeric surface for `args` by routing the post-coercion math through
/// the generic `execute(spec, …)` kernel-publication slice. This is the single entry the
/// family's `eval_*_surface` functions call, so every member shares one behavioural path.
pub fn eval_unary_numeric_via_executor(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    spec: UnaryNumericExecSpec,
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    eval_unary_numeric_surface(args, resolver, executor_kernel(spec))
}

pub fn eval_unary_numeric_surface(
    args: &[crate::value::CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    kernel: impl Fn(f64) -> Result<f64, WorksheetErrorCode> + Copy,
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    if args.len() != 1 {
        return Err(UnaryNumericSurfaceError::ArityMismatch {
            expected: 1,
            actual: args.len(),
        });
    }

    let prepared =
        prepare_arg_values_only(&args[0], resolver).map_err(UnaryNumericSurfaceError::Coercion)?;

    match prepared.core() {
        CoreValue::Array(array) => {
            let mapped = expand_arg_values_only(&args[0], resolver)
                .map_err(UnaryNumericSurfaceError::Coercion)?
                .into_iter()
                .map(|item| map_unary_numeric_item(&item, kernel))
                .collect::<Vec<_>>();
            Ok(CalcValue::array(
                CalcArray::new(array.shape(), mapped).expect("shape preserved"),
            ))
        }
        _ => match apply_unary_numeric_scalar_prepared(&prepared, |n| n) {
            Ok(n) => kernel(n)
                .map(CalcValue::number)
                .map_err(UnaryNumericSurfaceError::Domain),
            Err(err) => Err(UnaryNumericSurfaceError::Coercion(err)),
        },
    }
}

pub fn eval_unary_numeric_calc_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    kernel: impl Fn(f64) -> Result<f64, WorksheetErrorCode> + Copy,
) -> Result<CalcValue, UnaryNumericSurfaceError> {
    if args.len() != 1 {
        return Err(UnaryNumericSurfaceError::ArityMismatch {
            expected: 1,
            actual: args.len(),
        });
    }

    let prepared = prepare_calc_value_values_only(&args[0], resolver)
        .map_err(UnaryNumericSurfaceError::Coercion)?;
    match prepared.core() {
        CoreValue::Array(array) => {
            let cells = array
                .iter_row_major()
                .map(|item| map_unary_numeric_calc_item(item, kernel))
                .collect::<Vec<_>>();
            Ok(CalcValue::array(
                CalcArray::new(array.shape(), cells).expect("shape preserved"),
            ))
        }
        _ => match coerce_calc_scalar_to_number(&prepared) {
            Ok(n) => kernel(n)
                .map(CalcValue::number)
                .map_err(UnaryNumericSurfaceError::Domain),
            Err(err) => Err(UnaryNumericSurfaceError::Coercion(err)),
        },
    }
}

fn map_unary_numeric_item(
    item: &CalcValue,
    kernel: impl Fn(f64) -> Result<f64, WorksheetErrorCode> + Copy,
) -> CalcValue {
    match apply_unary_numeric_scalar_prepared(item, |n| n) {
        Ok(n) => match kernel(n) {
            Ok(v) => CalcValue::number(v),
            Err(code) => CalcValue::error(code),
        },
        Err(CoercionError::WorksheetError(code)) => CalcValue::error(code),
        Err(_) => CalcValue::error(WorksheetErrorCode::Value),
    }
}

fn map_unary_numeric_calc_item(
    item: &CalcValue,
    kernel: impl Fn(f64) -> Result<f64, WorksheetErrorCode> + Copy,
) -> CalcValue {
    match coerce_calc_scalar_to_number(item) {
        Ok(n) => match kernel(n) {
            Ok(v) => CalcValue::number(v),
            Err(code) => CalcValue::error(code),
        },
        Err(CoercionError::WorksheetError(code)) => CalcValue::error(code),
        Err(_) => CalcValue::error(WorksheetErrorCode::Value),
    }
}

pub fn map_unary_numeric_error_to_ws(e: &UnaryNumericSurfaceError) -> WorksheetErrorCode {
    match e {
        UnaryNumericSurfaceError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        UnaryNumericSurfaceError::Coercion(CoercionError::WorksheetError(code)) => *code,
        UnaryNumericSurfaceError::Coercion(_) => WorksheetErrorCode::Value,
        UnaryNumericSurfaceError::Domain(code) => *code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::ExcelText;

    struct NoResolver;

    impl ReferenceSystemProvider for NoResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            ReferenceSystemCapabilities::permissive_local()
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<CalcValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            Err(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
        }
    }

    #[test]
    fn unary_numeric_surface_accepts_numeric_text() {
        let got = eval_unary_numeric_surface(
            &[(CalcValue::text(ExcelText::from_utf16_code_units(
                "1".encode_utf16().collect(),
            )))],
            &NoResolver,
            |n| Ok(n + 1.0),
        )
        .unwrap();
        assert_eq!(got, CalcValue::number(2.0));
    }

    #[test]
    fn unary_numeric_surface_lifts_arrays_elementwise() {
        let got = eval_unary_numeric_surface(
            &[(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(1.0),
                    CalcValue::text(ExcelText::from_utf16_code_units(
                        "bad".encode_utf16().collect(),
                    )),
                ]])
                .unwrap(),
            ))],
            &NoResolver,
            |n| Ok(n * 2.0),
        )
        .unwrap();
        assert_eq!(
            got,
            CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(2.0),
                    CalcValue::error(WorksheetErrorCode::Value),
                ]])
                .unwrap()
            )
        );
    }

    #[test]
    fn unary_numeric_surface_maps_domain_errors() {
        let got = eval_unary_numeric_surface(&[(CalcValue::number(-1.0))], &NoResolver, |_| {
            Err(WorksheetErrorCode::Num)
        });
        assert_eq!(
            got,
            Err(UnaryNumericSurfaceError::Domain(WorksheetErrorCode::Num))
        );
    }

    // --- W105 oxf-y2uw.3 executor tests ----------------------------------------------------

    /// `execute` over a `Raw` spec reproduces `policy.wrap(kernel)` exactly: the publication
    /// (argument-domain guard + non-finite handling) is applied by the policy.
    #[test]
    fn execute_raw_spec_matches_policy_wrap() {
        fn sin_like(n: f64) -> f64 {
            n.sin()
        }
        let spec = UnaryNumericExecSpec::raw(sin_like, ExcelRealPolicy::CIRCULAR_TRIG);
        let wrapped = ExcelRealPolicy::CIRCULAR_TRIG.wrap(sin_like);
        for &x in &[
            0.0,
            -0.0,
            1.0,
            -1.0,
            134_217_727.0,
            134_217_728.0,
            -134_217_728.0,
        ] {
            assert_eq!(execute(spec, x), wrapped(x), "mismatch at x={x:?}");
        }
    }

    /// `execute` over a `Fallible` spec calls the kernel directly — the kernel owns its own
    /// compute and publication (the historic "hand the kernel to the surface" shape).
    #[test]
    fn execute_fallible_spec_calls_kernel_directly() {
        fn guarded(n: f64) -> Result<f64, WorksheetErrorCode> {
            if n < 0.0 {
                Err(WorksheetErrorCode::Num)
            } else {
                Ok(n.sqrt())
            }
        }
        let spec = UnaryNumericExecSpec::fallible(guarded, ExcelRealPolicy::PASS);
        assert_eq!(execute(spec, 4.0), Ok(2.0));
        assert_eq!(execute(spec, -1.0), Err(WorksheetErrorCode::Num));
    }

    /// The spec binding is a `Copy`/`Eq` value (the `FunctionSpec` discipline). Same kernel
    /// pointer + same policy compares equal; a different kernel does not.
    #[test]
    fn exec_spec_is_a_comparable_value() {
        fn k1(n: f64) -> f64 {
            n
        }
        fn k2(n: f64) -> f64 {
            -n
        }
        let a = UnaryNumericExecSpec::raw(k1, ExcelRealPolicy::PASS);
        let b = UnaryNumericExecSpec::raw(k1, ExcelRealPolicy::PASS);
        let c = UnaryNumericExecSpec::raw(k2, ExcelRealPolicy::PASS);
        let d = UnaryNumericExecSpec::raw(k1, ExcelRealPolicy::FINITE);
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_ne!(a, d);
    }

    /// Bead item 2: the two surface prep helpers — `eval_unary_numeric_surface` (the
    /// by-index path's helper) and `eval_unary_numeric_calc_surface` (the deleted
    /// calc-dispatch path's helper) — are equivalent for the unary-numeric family. With
    /// the same kernel they must produce bit-identical scalar, array-lift, and error
    /// outcomes across representative operands. This is the local proof that collapsing the
    /// two paths onto the by-index helper (which `execute` now backs) is behaviour-preserving
    /// at the prep boundary; the cross-surface `unary_numeric_equivalence_law` harness pins
    /// the end-to-end behaviour.
    #[test]
    fn two_prep_helpers_are_equivalent_for_the_family() {
        fn numeric_text(s: &str) -> CalcValue {
            CalcValue::text(ExcelText::from_utf16_code_units(s.encode_utf16().collect()))
        }
        // A representative spec from each kernel shape: a `Raw` circular-trig-guarded kernel
        // (SIN-like) and a `Fallible` domain-rejecting kernel (SQRT-like).
        fn sin_like(n: f64) -> f64 {
            n.sin()
        }
        fn sqrt_like(n: f64) -> Result<f64, WorksheetErrorCode> {
            if n < 0.0 {
                Err(WorksheetErrorCode::Num)
            } else {
                Ok(n.sqrt())
            }
        }
        let specs = [
            UnaryNumericExecSpec::raw(sin_like, ExcelRealPolicy::CIRCULAR_TRIG),
            UnaryNumericExecSpec::fallible(sqrt_like, ExcelRealPolicy::PASS),
        ];

        let scalar_operands: Vec<CalcValue> = vec![
            CalcValue::number(4.0),
            CalcValue::number(-9.0),
            CalcValue::number(0.0),
            CalcValue::number(134_217_728.0),
            numeric_text("16"),
            numeric_text("-2"),
            numeric_text("not-a-number"),
            CalcValue::logical(true),
            CalcValue::logical(false),
            CalcValue::error(WorksheetErrorCode::Div0),
            CalcValue::new(CoreValue::Empty),
        ];

        for spec in specs {
            let kernel = executor_kernel(spec);
            // Scalar operands.
            for operand in &scalar_operands {
                let via_eval =
                    eval_unary_numeric_surface(std::slice::from_ref(operand), &NoResolver, kernel);
                let via_calc = eval_unary_numeric_calc_surface(
                    std::slice::from_ref(operand),
                    &NoResolver,
                    kernel,
                );
                assert_eq!(
                    via_eval, via_calc,
                    "prep-helper divergence on scalar operand {operand:?}"
                );
            }
            // A multi-cell array operand exercises the elementwise lift on both helpers.
            let array = CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(4.0),
                    CalcValue::number(-9.0),
                    numeric_text("16"),
                    numeric_text("oops"),
                    CalcValue::logical(true),
                    CalcValue::error(WorksheetErrorCode::NA),
                    CalcValue::number(134_217_728.0),
                ]])
                .unwrap(),
            );
            let via_eval =
                eval_unary_numeric_surface(std::slice::from_ref(&array), &NoResolver, kernel);
            let via_calc =
                eval_unary_numeric_calc_surface(std::slice::from_ref(&array), &NoResolver, kernel);
            assert_eq!(
                via_eval, via_calc,
                "prep-helper divergence on the array-lift operand"
            );
        }
    }
}
