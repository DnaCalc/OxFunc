use crate::coercion::CoercionError;
use crate::function::ExcelRealPolicy;
use crate::functions::adapters::{
    BroadcastPreparedPair, coerce_prepared_to_number, expand_binary_broadcast_grid,
    prepare_args_values_only, prepare_calc_values_only, prepared_from_calc_value,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::{CalcArray, WorksheetErrorCode};

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryNumericSurfaceError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

// =====================================================================================
//  W105 oxf-y2uw.12.1 — the generic binary-numeric executor (2-arg kernel-publication slice).
//
//  This mirrors the `.3` unary executor (`unary_numeric::execute`) for the binary-arithmetic
//  operator family (MOD, OP_ADD, OP_SUBTRACT, OP_MULTIPLY, OP_DIVIDE, OP_POWER / POWER), per
//  the W105-D2 rescope: operators -> a 2-arg numeric kernel pattern that extends the pilot.
//
//  BOUNDED INTERFACE — exactly the kernel-publication slice the pilot is scoped to, one arity
//  up: the inputs are two ALREADY-PREPARED numeric `f64`s (coercion / reference resolution /
//  array broadcast happen in the surface helpers below, NOT here), and the output is the
//  `Result<f64, WorksheetErrorCode>` publication shape. There is nowhere to thread a context
//  through this signature, so reference / context-provider functions are out of scope by
//  construction — exactly as in the unary pilot.
//
//  Each family member binds its pure kernel and its declared `ExcelRealPolicy` into a
//  `BinaryNumericExecSpec`. The spec is a `Copy`/`Eq` closed value (a function-pointer kernel
//  binding plus the closed-enum policy — no `String`, no free `f64`), matching the
//  `FunctionSpec` discipline so the binding is itself an inspectable value.
// =====================================================================================

/// The two kernel shapes the binary-arithmetic family uses, as a closed, `Copy`/`Eq` binding —
/// the 2-arg analogue of [`crate::functions::unary_numeric::UnaryNumericKernel`].
///
/// * `Raw` is a pure `(f64, f64) -> f64` kernel whose result is published verbatim as `Ok(..)`
///   — exactly the historic `OP_ADD` shape (`Ok(op_add_kernel(lhs, rhs))`), the one operator
///   whose kernel cannot fail.
/// * `Fallible` is a kernel that owns its own compute and error algebra (`#DIV/0!`, `#NUM!`,
///   the underflow-to-zero / integer-exponent publication it applies internally), returning the
///   published value directly — exactly the historic shape where such a kernel was handed
///   straight to the binary surface helper. POWER rides this arm so its declared
///   `IntegerExponentPublication` precision quirk (W105 .8), applied inside `power_kernel`,
///   keeps applying bit-exactly.
#[derive(Clone, Copy, Debug)]
pub enum BinaryNumericKernel {
    /// Pure `(f64, f64) -> f64`; the result is published verbatim as `Ok(..)`.
    Raw(fn(f64, f64) -> f64),
    /// `(f64, f64) -> Result<f64, _>`; the kernel owns its compute and publication.
    Fallible(fn(f64, f64) -> Result<f64, WorksheetErrorCode>),
}

impl PartialEq for BinaryNumericKernel {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Compare function pointers by address (the only `Eq`-able identity a `fn` item
            // has). Keeps the binding a comparable value without a free `f64`.
            (Self::Raw(a), Self::Raw(b)) => std::ptr::fn_addr_eq(*a, *b),
            (Self::Fallible(a), Self::Fallible(b)) => std::ptr::fn_addr_eq(*a, *b),
            _ => false,
        }
    }
}

impl Eq for BinaryNumericKernel {}

/// The kernel-publication binding for one binary-arithmetic family member: its kernel plus the
/// Excel publication policy declared on its `FunctionMeta`. `Copy`/`Eq` so it is an inspectable
/// value, in the `FunctionSpec` mould — the 2-arg analogue of
/// [`crate::functions::unary_numeric::UnaryNumericExecSpec`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BinaryNumericExecSpec {
    kernel: BinaryNumericKernel,
    policy: ExcelRealPolicy,
}

impl BinaryNumericExecSpec {
    /// Bind a pure `(f64, f64) -> f64` kernel whose result is published verbatim as `Ok(..)`.
    pub const fn raw(kernel: fn(f64, f64) -> f64, policy: ExcelRealPolicy) -> Self {
        Self {
            kernel: BinaryNumericKernel::Raw(kernel),
            policy,
        }
    }

    /// Bind a `(f64, f64) -> Result<f64, _>` kernel that owns its own compute and publication.
    /// The policy is recorded for inspection/equivalence but the kernel is authoritative (it
    /// already applies its own domain guards and any precision quirk internally — e.g. POWER's
    /// `IntegerExponentPublication`), so `execute` calls it directly, reproducing exactly the
    /// historic "hand the fallible kernel to the binary surface" behaviour.
    pub const fn fallible(
        kernel: fn(f64, f64) -> Result<f64, WorksheetErrorCode>,
        policy: ExcelRealPolicy,
    ) -> Self {
        Self {
            kernel: BinaryNumericKernel::Fallible(kernel),
            policy,
        }
    }

    /// The declared Excel publication policy for this binding.
    pub const fn policy(self) -> ExcelRealPolicy {
        self.policy
    }

    /// `true` when this binding holds a pure `(f64, f64) -> f64` (`Raw`) kernel published
    /// verbatim; `false` for a `Fallible` kernel that owns its own publication. The spec-driven
    /// by-index generator reads THIS — rather than a hand-written per-arm flag — to choose
    /// between emitting `BinaryNumericExecSpec::raw(..)` and `::fallible(..)`, so the generated
    /// constructor reflects the actual declared binding.
    pub const fn kernel_is_raw(self) -> bool {
        matches!(self.kernel, BinaryNumericKernel::Raw(_))
    }
}

/// THE generic executor for the binary-numeric kernel-publication slice — the 2-arg analogue of
/// [`crate::functions::unary_numeric::execute`].
///
/// Bounded interface: `lhs`/`rhs` are already-prepared numeric payloads; the result is the
/// `Result<f64, WorksheetErrorCode>` publication shape. This is the single behavioural
/// definition every binary-arithmetic member's surface evaluation derives from — collapsing the
/// historic calc-dispatch-vs-by-index path drift (these ids were double-served: intercepted by
/// `eval_binary_arithmetic_calc_dispatch` while a shadowed by-index arm also existed) into one
/// path. For the binary family the publication is owned entirely by the kernel (matching the XLL
/// `Q`-scalar path `eval_surface_q_binary_number`, which calls these kernels directly), so a
/// `Raw` kernel publishes `Ok(kernel(..))` and a `Fallible` kernel is called directly.
pub fn execute(spec: BinaryNumericExecSpec, lhs: f64, rhs: f64) -> Result<f64, WorksheetErrorCode> {
    match spec.kernel {
        BinaryNumericKernel::Raw(kernel) => Ok(kernel(lhs, rhs)),
        BinaryNumericKernel::Fallible(kernel) => kernel(lhs, rhs),
    }
}

/// Bind a spec into the `Fn(f64, f64) -> Result<f64, _>` kernel the binary surface helpers
/// consume, so every dispatch site routes the post-coercion math through `execute`.
pub fn executor_kernel(
    spec: BinaryNumericExecSpec,
) -> impl Fn(f64, f64) -> Result<f64, WorksheetErrorCode> + Copy {
    move |lhs, rhs| execute(spec, lhs, rhs)
}

/// Evaluate the binary-numeric surface for `args` by routing the post-coercion math through the
/// generic `execute(spec, …)` kernel-publication slice. This is the single entry the operator
/// family's by-index arms call, so every member shares one behavioural path. It reuses the
/// existing `eval_binary_numeric_surface` prep/lift machinery — coercion/broadcast is NOT
/// reimplemented here.
pub fn eval_binary_numeric_via_executor(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    spec: BinaryNumericExecSpec,
) -> Result<CalcValue, BinaryNumericSurfaceError> {
    eval_binary_numeric_surface(args, resolver, executor_kernel(spec))
}

pub fn eval_binary_numeric_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    kernel: impl Fn(f64, f64) -> Result<f64, WorksheetErrorCode> + Copy,
) -> Result<CalcValue, BinaryNumericSurfaceError> {
    let prepared =
        prepare_args_values_only(args, resolver).map_err(BinaryNumericSurfaceError::Coercion)?;
    eval_binary_numeric_prepared(&prepared, kernel)
}

pub fn eval_binary_numeric_calc_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    kernel: impl Fn(f64, f64) -> Result<f64, WorksheetErrorCode> + Copy,
) -> Result<CalcValue, BinaryNumericSurfaceError> {
    let prepared_calc =
        prepare_calc_values_only(args, resolver).map_err(BinaryNumericSurfaceError::Coercion)?;
    let prepared = prepared_calc
        .iter()
        .map(prepared_from_calc_value)
        .collect::<Vec<_>>();
    eval_binary_numeric_prepared(&prepared, kernel).map(CalcValue::from)
}

pub fn eval_binary_numeric_prepared(
    args: &[CalcValue],
    kernel: impl Fn(f64, f64) -> Result<f64, WorksheetErrorCode> + Copy,
) -> Result<CalcValue, BinaryNumericSurfaceError> {
    if args.len() != 2 {
        return Err(BinaryNumericSurfaceError::ArityMismatch {
            expected: 2,
            actual: args.len(),
        });
    }

    if let Some((shape, cells)) = expand_binary_broadcast_grid(&args[0], &args[1]) {
        let mapped = cells
            .into_iter()
            .map(|cell| match cell {
                BroadcastPreparedPair::Pair(lhs_value, rhs_value) => {
                    map_binary_numeric_item(&lhs_value, &rhs_value, kernel)
                }
                BroadcastPreparedPair::MissingCoordinate => {
                    CalcValue::error(WorksheetErrorCode::NA)
                }
            })
            .collect();
        Ok(CalcValue::array(
            CalcArray::new(shape, mapped).expect("shape preserved"),
        ))
    } else {
        eval_binary_numeric_scalars(&args[0], &args[1], kernel)
    }
}

pub fn map_binary_numeric_error_to_ws(e: &BinaryNumericSurfaceError) -> WorksheetErrorCode {
    match e {
        BinaryNumericSurfaceError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        BinaryNumericSurfaceError::Coercion(CoercionError::WorksheetError(code)) => *code,
        BinaryNumericSurfaceError::Coercion(_) => WorksheetErrorCode::Value,
        BinaryNumericSurfaceError::Domain(code) => *code,
    }
}

fn map_binary_numeric_item(
    lhs: &CalcValue,
    rhs: &CalcValue,
    kernel: impl Fn(f64, f64) -> Result<f64, WorksheetErrorCode> + Copy,
) -> CalcValue {
    let lhs = match coerce_prepared_to_number(lhs) {
        Ok(lhs) => lhs,
        Err(CoercionError::WorksheetError(code)) => return CalcValue::error(code),
        Err(_) => return CalcValue::error(WorksheetErrorCode::Value),
    };
    let rhs = match coerce_prepared_to_number(rhs) {
        Ok(rhs) => rhs,
        Err(CoercionError::WorksheetError(code)) => return CalcValue::error(code),
        Err(_) => return CalcValue::error(WorksheetErrorCode::Value),
    };

    match kernel(lhs, rhs) {
        Ok(value) => CalcValue::number(value),
        Err(code) => CalcValue::error(code),
    }
}

fn eval_binary_numeric_scalars(
    lhs: &CalcValue,
    rhs: &CalcValue,
    kernel: impl Fn(f64, f64) -> Result<f64, WorksheetErrorCode> + Copy,
) -> Result<CalcValue, BinaryNumericSurfaceError> {
    let lhs = coerce_prepared_to_number(lhs).map_err(BinaryNumericSurfaceError::Coercion)?;
    let rhs = coerce_prepared_to_number(rhs).map_err(BinaryNumericSurfaceError::Coercion)?;
    kernel(lhs, rhs)
        .map(CalcValue::number)
        .map_err(BinaryNumericSurfaceError::Domain)
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
    fn binary_numeric_surface_accepts_numeric_text() {
        let got = eval_binary_numeric_surface(
            &[
                (CalcValue::text(ExcelText::from_utf16_code_units(
                    "2".encode_utf16().collect(),
                ))),
                (CalcValue::number(3.0)),
            ],
            &NoResolver,
            |lhs, rhs| Ok(lhs + rhs),
        )
        .unwrap();
        assert_eq!(got, CalcValue::number(5.0));
    }

    #[test]
    fn binary_numeric_surface_maps_domain_errors() {
        let got = eval_binary_numeric_surface(
            &[(CalcValue::number(1.0)), (CalcValue::number(0.0))],
            &NoResolver,
            |_lhs, _rhs| Err(WorksheetErrorCode::Div0),
        );
        assert_eq!(
            got,
            Err(BinaryNumericSurfaceError::Domain(WorksheetErrorCode::Div0))
        );
    }

    #[test]
    fn binary_numeric_surface_lifts_array_scalar_elementwise() {
        let got = eval_binary_numeric_surface(
            &[
                (CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::number(1.0),
                        CalcValue::text(ExcelText::from_utf16_code_units(
                            "2".encode_utf16().collect(),
                        )),
                    ]])
                    .unwrap(),
                )),
                (CalcValue::number(10.0)),
            ],
            &NoResolver,
            |lhs, rhs| Ok(lhs + rhs),
        )
        .unwrap();

        assert_eq!(
            got,
            CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(11.0), CalcValue::number(12.0),]
                ])
                .unwrap()
            )
        );
    }

    #[test]
    fn binary_numeric_surface_lifts_scalar_array_elementwise() {
        let got = eval_binary_numeric_surface(
            &[
                (CalcValue::number(2.0)),
                (CalcValue::array(
                    CalcArray::from_rows(vec![
                        vec![CalcValue::number(3.0), CalcValue::logical(true)],
                        vec![CalcValue::empty(), CalcValue::number(4.0)],
                    ])
                    .unwrap(),
                )),
            ],
            &NoResolver,
            |lhs, rhs| Ok(lhs * rhs),
        )
        .unwrap();

        assert_eq!(
            got,
            CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(6.0), CalcValue::number(2.0)],
                    vec![
                        CalcValue::error(WorksheetErrorCode::Value),
                        CalcValue::number(8.0)
                    ],
                ])
                .unwrap()
            )
        );
    }

    #[test]
    fn binary_numeric_surface_lifts_same_shape_arrays_elementwise() {
        let got = eval_binary_numeric_surface(
            &[
                (CalcValue::array(
                    CalcArray::from_rows(vec![
                        vec![CalcValue::number(1.0), CalcValue::number(2.0)],
                        vec![CalcValue::number(6.0), CalcValue::number(8.0)],
                    ])
                    .unwrap(),
                )),
                (CalcValue::array(
                    CalcArray::from_rows(vec![
                        vec![CalcValue::number(1.0), CalcValue::number(0.0)],
                        vec![CalcValue::number(3.0), CalcValue::number(2.0)],
                    ])
                    .unwrap(),
                )),
            ],
            &NoResolver,
            |lhs, rhs| {
                if rhs == 0.0 {
                    Err(WorksheetErrorCode::Div0)
                } else {
                    Ok(lhs / rhs)
                }
            },
        )
        .unwrap();

        assert_eq!(
            got,
            CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![
                        CalcValue::number(1.0),
                        CalcValue::error(WorksheetErrorCode::Div0)
                    ],
                    vec![CalcValue::number(2.0), CalcValue::number(4.0)],
                ])
                .unwrap()
            )
        );
    }

    #[test]
    fn binary_numeric_surface_broadcasts_row_and_column_arrays() {
        let got = eval_binary_numeric_surface(
            &[
                (CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::number(1.0),
                        CalcValue::number(2.0),
                    ]])
                    .unwrap(),
                )),
                (CalcValue::array(
                    CalcArray::from_rows(vec![
                        vec![CalcValue::number(1.0)],
                        vec![CalcValue::number(2.0)],
                    ])
                    .unwrap(),
                )),
            ],
            &NoResolver,
            |lhs, rhs| Ok(lhs + rhs),
        )
        .unwrap();

        assert_eq!(
            got,
            CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(2.0), CalcValue::number(3.0)],
                    vec![CalcValue::number(3.0), CalcValue::number(4.0)],
                ])
                .unwrap()
            )
        );
    }

    // --- W105 oxf-y2uw.12.1 binary executor tests ------------------------------------------

    /// `execute` over a `Raw` spec publishes the kernel result verbatim as `Ok(..)` — exactly
    /// the historic `OP_ADD` shape (`Ok(op_add_kernel(lhs, rhs))`).
    #[test]
    fn execute_raw_spec_publishes_kernel_verbatim() {
        fn add_like(lhs: f64, rhs: f64) -> f64 {
            lhs + rhs
        }
        let spec = BinaryNumericExecSpec::raw(add_like, ExcelRealPolicy::PASS);
        for &(lhs, rhs) in &[(1.5, 2.0), (-3.0, 3.0), (0.0, -0.0), (1e308, 1e308)] {
            assert_eq!(execute(spec, lhs, rhs), Ok(add_like(lhs, rhs)));
        }
    }

    /// `execute` over a `Fallible` spec calls the kernel directly — the kernel owns its own
    /// compute and publication (the historic "hand the kernel to the binary surface" shape).
    #[test]
    fn execute_fallible_spec_calls_kernel_directly() {
        fn guarded_div(lhs: f64, rhs: f64) -> Result<f64, WorksheetErrorCode> {
            if rhs == 0.0 {
                Err(WorksheetErrorCode::Div0)
            } else {
                Ok(lhs / rhs)
            }
        }
        let spec = BinaryNumericExecSpec::fallible(guarded_div, ExcelRealPolicy::PASS);
        assert_eq!(execute(spec, 6.0, 2.0), Ok(3.0));
        assert_eq!(execute(spec, 1.0, 0.0), Err(WorksheetErrorCode::Div0));
    }

    /// The spec binding is a `Copy`/`Eq` value (the `FunctionSpec` discipline). Same kernel
    /// pointer + same policy compares equal; a different kernel or policy does not.
    #[test]
    fn exec_spec_is_a_comparable_value() {
        fn k1(lhs: f64, rhs: f64) -> f64 {
            lhs + rhs
        }
        fn k2(lhs: f64, rhs: f64) -> f64 {
            lhs - rhs
        }
        let a = BinaryNumericExecSpec::raw(k1, ExcelRealPolicy::PASS);
        let b = BinaryNumericExecSpec::raw(k1, ExcelRealPolicy::PASS);
        let c = BinaryNumericExecSpec::raw(k2, ExcelRealPolicy::PASS);
        let d = BinaryNumericExecSpec::raw(k1, ExcelRealPolicy::FINITE);
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_ne!(a, d);
    }

    /// `kernel_is_raw` reports the bound kernel variant, so the by-index generator can derive
    /// the `raw`/`fallible` constructor from the spec rather than a hand-written flag.
    #[test]
    fn kernel_is_raw_reflects_the_binding() {
        fn raw_k(lhs: f64, rhs: f64) -> f64 {
            lhs + rhs
        }
        fn fallible_k(lhs: f64, rhs: f64) -> Result<f64, WorksheetErrorCode> {
            Ok(lhs + rhs)
        }
        assert!(BinaryNumericExecSpec::raw(raw_k, ExcelRealPolicy::PASS).kernel_is_raw());
        assert!(
            !BinaryNumericExecSpec::fallible(fallible_k, ExcelRealPolicy::PASS).kernel_is_raw()
        );
    }

    /// `eval_binary_numeric_via_executor` is a pure delegation to `eval_binary_numeric_surface`
    /// with the executor kernel: routing the post-coercion math through `execute` must give the
    /// same scalar / array-lift / error outcomes as handing the bare kernel to the surface
    /// helper. This is the local proof that the executor entry preserves the surface behaviour.
    #[test]
    fn via_executor_matches_direct_surface_for_the_family() {
        fn numeric_text(s: &str) -> CalcValue {
            CalcValue::text(ExcelText::from_utf16_code_units(s.encode_utf16().collect()))
        }
        fn add_raw(lhs: f64, rhs: f64) -> f64 {
            lhs + rhs
        }
        fn div_fallible(lhs: f64, rhs: f64) -> Result<f64, WorksheetErrorCode> {
            if rhs == 0.0 {
                Err(WorksheetErrorCode::Div0)
            } else {
                Ok(lhs / rhs)
            }
        }
        let specs = [
            (
                BinaryNumericExecSpec::raw(add_raw, ExcelRealPolicy::PASS),
                add_raw as fn(f64, f64) -> f64,
            ),
            (
                BinaryNumericExecSpec::fallible(div_fallible, ExcelRealPolicy::PASS),
                {
                    fn unused(_: f64, _: f64) -> f64 {
                        0.0
                    }
                    unused
                },
            ),
        ];

        // Operand pairs exercising scalar, numeric-text coercion, div-by-zero, and array lift.
        let pairs: Vec<[CalcValue; 2]> = vec![
            [CalcValue::number(6.0), CalcValue::number(2.0)],
            [numeric_text("8"), CalcValue::logical(true)],
            [CalcValue::number(1.0), CalcValue::number(0.0)],
            [
                CalcValue::error(WorksheetErrorCode::NA),
                CalcValue::number(1.0),
            ],
            [
                CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::number(6.0),
                        CalcValue::number(0.0),
                    ]])
                    .unwrap(),
                ),
                CalcValue::number(2.0),
            ],
        ];

        for (spec, _raw) in specs {
            let kernel = executor_kernel(spec);
            for pair in &pairs {
                let via_executor = eval_binary_numeric_via_executor(pair, &NoResolver, spec);
                let via_surface = eval_binary_numeric_surface(pair, &NoResolver, kernel);
                assert_eq!(
                    via_executor, via_surface,
                    "executor entry diverged from the bare-kernel surface on {pair:?}"
                );
            }
        }
    }

    /// The two binary prep helpers are equivalent for the operator family. The migration moves
    /// these ids from the deleted calc-dispatch (which used `eval_binary_numeric_calc_surface`)
    /// onto the executor entry (which uses `eval_binary_numeric_surface`). For the migration to
    /// be behaviour-preserving the two prep helpers must produce bit-identical scalar /
    /// array-lift / broadcast / error outcomes for the same kernel — this is the local proof at
    /// the prep boundary that the path switch preserves behaviour (the binary analogue of the
    /// unary `two_prep_helpers_are_equivalent_for_the_family` and the `.1` Law-2 check).
    #[test]
    fn two_binary_prep_helpers_are_equivalent_for_the_family() {
        fn numeric_text(s: &str) -> CalcValue {
            CalcValue::text(ExcelText::from_utf16_code_units(s.encode_utf16().collect()))
        }
        // A `Raw` add-like kernel and a `Fallible` div-like kernel cover both spec shapes.
        fn add_raw(lhs: f64, rhs: f64) -> Result<f64, WorksheetErrorCode> {
            Ok(lhs + rhs)
        }
        fn div_fallible(lhs: f64, rhs: f64) -> Result<f64, WorksheetErrorCode> {
            if rhs == 0.0 {
                Err(WorksheetErrorCode::Div0)
            } else {
                Ok(lhs / rhs)
            }
        }
        let kernels: [fn(f64, f64) -> Result<f64, WorksheetErrorCode>; 2] = [add_raw, div_fallible];

        // Scalar, numeric-text coercion, logical, error-propagation, blank, and the array-lift /
        // broadcast / non-broadcastable shapes that exercise the prep+lift machinery on both
        // helpers.
        let pairs: Vec<[CalcValue; 2]> = vec![
            [CalcValue::number(6.0), CalcValue::number(2.0)],
            [numeric_text("8"), CalcValue::logical(true)],
            [CalcValue::number(1.0), CalcValue::number(0.0)],
            [
                CalcValue::error(WorksheetErrorCode::NA),
                CalcValue::number(1.0),
            ],
            [CalcValue::empty(), CalcValue::number(3.0)],
            [numeric_text("not-a-number"), CalcValue::number(1.0)],
            [
                CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::number(6.0),
                        CalcValue::number(0.0),
                    ]])
                    .unwrap(),
                ),
                CalcValue::number(2.0),
            ],
            [
                CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::number(1.0),
                        CalcValue::number(2.0),
                    ]])
                    .unwrap(),
                ),
                CalcValue::array(
                    CalcArray::from_rows(vec![
                        vec![CalcValue::number(1.0)],
                        vec![CalcValue::number(2.0)],
                    ])
                    .unwrap(),
                ),
            ],
            [
                CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::number(1.0),
                        CalcValue::number(2.0),
                    ]])
                    .unwrap(),
                ),
                CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::number(1.0),
                        CalcValue::number(2.0),
                        CalcValue::number(3.0),
                    ]])
                    .unwrap(),
                ),
            ],
        ];

        for kernel in kernels {
            for pair in &pairs {
                let via_surface = eval_binary_numeric_surface(pair, &NoResolver, kernel);
                let via_calc = eval_binary_numeric_calc_surface(pair, &NoResolver, kernel);
                assert_eq!(
                    via_surface, via_calc,
                    "binary prep-helper divergence on {pair:?}"
                );
            }
        }
    }

    #[test]
    fn binary_numeric_surface_marks_non_broadcastable_cells_as_na() {
        let got = eval_binary_numeric_surface(
            &[
                (CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::number(1.0),
                        CalcValue::number(2.0),
                    ]])
                    .unwrap(),
                )),
                (CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::number(1.0),
                        CalcValue::number(2.0),
                        CalcValue::number(3.0),
                    ]])
                    .unwrap(),
                )),
            ],
            &NoResolver,
            |lhs, rhs| Ok(lhs + rhs),
        );

        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(2.0),
                    CalcValue::number(4.0),
                    CalcValue::error(WorksheetErrorCode::NA),
                ]])
                .unwrap()
            ))
        );
    }
}
