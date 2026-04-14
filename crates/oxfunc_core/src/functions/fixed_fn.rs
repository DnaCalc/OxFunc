use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedArgValue, run_values_only_prepared};
use crate::locale_format::LocaleFormatContext;
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const FIXED_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.FIXED",
    arity: Arity { min: 1, max: 3 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::LocaleProfile,
    surface_fec_dependency_profile: FecDependencyProfile::Composite,
};

#[derive(Debug, Clone, PartialEq)]
pub enum FixedEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

fn coerce_number_arg(
    arg: &PreparedArgValue,
    ctx: &LocaleFormatContext,
) -> Result<f64, FixedEvalError> {
    match arg {
        PreparedArgValue::Eval(EvalValue::Number(n)) => Ok(*n),
        PreparedArgValue::Eval(EvalValue::Logical(b)) => Ok(if *b { 1.0 } else { 0.0 }),
        PreparedArgValue::Eval(EvalValue::Text(text)) => ctx
            .parser
            .parse_value_text(&ctx.profile, ctx.date_system, &text.to_string_lossy())
            .map_err(|_| {
                FixedEvalError::Coercion(CoercionError::NonNumericText(text.to_string_lossy()))
            }),
        PreparedArgValue::Eval(EvalValue::Error(code)) => Err(FixedEvalError::Coercion(
            CoercionError::WorksheetError(*code),
        )),
        PreparedArgValue::EmptyCell => Ok(0.0),
        PreparedArgValue::MissingArg => Err(FixedEvalError::Coercion(CoercionError::MissingArg)),
        PreparedArgValue::Eval(EvalValue::Array(_))
        | PreparedArgValue::Eval(EvalValue::Reference(_))
        | PreparedArgValue::Eval(EvalValue::Lambda(_)) => Err(FixedEvalError::Coercion(
            CoercionError::UnsupportedValueKind("fixed_arg_kind"),
        )),
    }
}

fn coerce_boolish_arg(
    arg: &PreparedArgValue,
    ctx: &LocaleFormatContext,
) -> Result<bool, FixedEvalError> {
    Ok(match arg {
        PreparedArgValue::Eval(EvalValue::Logical(b)) => *b,
        other => coerce_number_arg(other, ctx)? != 0.0,
    })
}

pub fn eval_fixed_adapter_prepared(
    args: &[PreparedArgValue],
    ctx: &LocaleFormatContext,
) -> Result<EvalValue, FixedEvalError> {
    if !FIXED_META.arity.accepts(args.len()) {
        return Err(FixedEvalError::ArityMismatch {
            expected_min: FIXED_META.arity.min,
            expected_max: FIXED_META.arity.max,
            actual: args.len(),
        });
    }
    let value = coerce_number_arg(&args[0], ctx)?;
    let decimals = if let Some(arg) = args.get(1) {
        coerce_number_arg(arg, ctx)?.trunc() as i32
    } else {
        2
    };
    let no_commas = if let Some(arg) = args.get(2) {
        coerce_boolish_arg(arg, ctx)?
    } else {
        false
    };
    let text = ctx
        .formatter
        .render_fixed(&ctx.profile, value, decimals, no_commas)
        .map_err(|_| {
            FixedEvalError::Coercion(CoercionError::UnsupportedValueKind("fixed_format"))
        })?;
    Ok(EvalValue::Text(text))
}

pub fn eval_fixed_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    ctx: &LocaleFormatContext,
) -> Result<EvalValue, FixedEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_fixed_adapter_prepared(prepared, ctx),
        FixedEvalError::Coercion,
    )
}

pub fn map_fixed_error_to_ws(e: &FixedEvalError) -> WorksheetErrorCode {
    match e {
        FixedEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        FixedEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        FixedEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locale_format::test_current_excel_host_context;
    use crate::resolver::{RefResolutionError, ReferenceResolver, ResolverCapabilities};
    use crate::value::{ExcelText, ReferenceLike};

    struct NoResolver;

    impl ReferenceResolver for NoResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            ResolverCapabilities::permissive_local()
        }
        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            Err(RefResolutionError::UnresolvedReference {
                target: reference.target.clone(),
            })
        }
    }

    fn text_arg(s: &str) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
            s.encode_utf16().collect(),
        )))
    }

    #[test]
    fn eval_fixed_current_host_seed_rows() {
        let ctx = test_current_excel_host_context();
        assert_eq!(
            eval_fixed_surface(
                &[
                    CallArgValue::Eval(EvalValue::Number(1234.567)),
                    CallArgValue::Eval(EvalValue::Number(2.0))
                ],
                &NoResolver,
                &ctx
            ),
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "1 234.57".encode_utf16().collect()
            )))
        );
        assert_eq!(
            eval_fixed_surface(
                &[
                    CallArgValue::Eval(EvalValue::Number(1234.567)),
                    CallArgValue::Eval(EvalValue::Number(2.0)),
                    CallArgValue::Eval(EvalValue::Logical(true))
                ],
                &NoResolver,
                &ctx
            ),
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "1234.57".encode_utf16().collect()
            )))
        );
        assert_eq!(
            eval_fixed_surface(
                &[
                    CallArgValue::Eval(EvalValue::Number(-1234.567)),
                    CallArgValue::Eval(EvalValue::Number(2.0))
                ],
                &NoResolver,
                &ctx
            ),
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "-1 234.57".encode_utf16().collect()
            )))
        );
        assert_eq!(
            eval_fixed_surface(
                &[
                    CallArgValue::Eval(EvalValue::Logical(true)),
                    CallArgValue::Eval(EvalValue::Number(2.0))
                ],
                &NoResolver,
                &ctx
            ),
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "1.00".encode_utf16().collect()
            )))
        );
        assert_eq!(
            eval_fixed_surface(
                &[text_arg("123"), CallArgValue::Eval(EvalValue::Number(2.0))],
                &NoResolver,
                &ctx
            ),
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "123.00".encode_utf16().collect()
            )))
        );
        assert!(matches!(
            eval_fixed_surface(
                &[text_arg("x"), CallArgValue::Eval(EvalValue::Number(2.0))],
                &NoResolver,
                &ctx
            ),
            Err(FixedEvalError::Coercion(CoercionError::NonNumericText(_)))
        ));
        assert_eq!(
            eval_fixed_surface(
                &[
                    CallArgValue::EmptyCell,
                    CallArgValue::Eval(EvalValue::Number(2.0))
                ],
                &NoResolver,
                &ctx
            ),
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "0.00".encode_utf16().collect()
            )))
        );
    }
}
