use crate::coercion::CoercionError;
use crate::function::{ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass};
use crate::functions::adapters::{PreparedArgValue, run_values_only_prepared};
use crate::locale_format::LocaleFormatContext;
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const DOLLAR_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DOLLAR",
    arity: Arity { min: 1, max: 2 },
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
pub enum DollarEvalError {
    ArityMismatch { expected_min: usize, expected_max: usize, actual: usize },
    Coercion(CoercionError),
}

fn coerce_number_arg(arg: &PreparedArgValue, ctx: &LocaleFormatContext) -> Result<f64, DollarEvalError> {
    match arg {
        PreparedArgValue::Eval(EvalValue::Number(n)) => Ok(*n),
        PreparedArgValue::Eval(EvalValue::Logical(b)) => Ok(if *b { 1.0 } else { 0.0 }),
        PreparedArgValue::Eval(EvalValue::Text(text)) => ctx
            .parser
            .parse_value_text(&ctx.profile, ctx.date_system, &text.to_string_lossy())
            .map_err(|_| DollarEvalError::Coercion(CoercionError::NonNumericText(text.to_string_lossy()))),
        PreparedArgValue::Eval(EvalValue::Error(code)) => Err(DollarEvalError::Coercion(CoercionError::WorksheetError(*code))),
        PreparedArgValue::EmptyCell => Ok(0.0),
        PreparedArgValue::MissingArg => Err(DollarEvalError::Coercion(CoercionError::MissingArg)),
        PreparedArgValue::Eval(EvalValue::Array(_))
        | PreparedArgValue::Eval(EvalValue::Reference(_))
        | PreparedArgValue::Eval(EvalValue::Lambda(_)) => Err(DollarEvalError::Coercion(CoercionError::UnsupportedValueKind("dollar_arg_kind"))),
    }
}

pub fn eval_dollar_adapter_prepared(args: &[PreparedArgValue], ctx: &LocaleFormatContext) -> Result<EvalValue, DollarEvalError> {
    if !DOLLAR_META.arity.accepts(args.len()) {
        return Err(DollarEvalError::ArityMismatch {
            expected_min: DOLLAR_META.arity.min,
            expected_max: DOLLAR_META.arity.max,
            actual: args.len(),
        });
    }
    let value = coerce_number_arg(&args[0], ctx)?;
    let decimals = if let Some(arg) = args.get(1) {
        coerce_number_arg(arg, ctx)?.trunc() as i32
    } else {
        ctx.profile.currency_decimals
    };
    let text = ctx
        .formatter
        .render_currency(&ctx.profile, value, decimals)
        .map_err(|_| DollarEvalError::Coercion(CoercionError::UnsupportedValueKind("currency_format")))?;
    Ok(EvalValue::Text(text))
}

pub fn eval_dollar_surface(args: &[CallArgValue], resolver: &impl ReferenceResolver, ctx: &LocaleFormatContext) -> Result<EvalValue, DollarEvalError> {
    run_values_only_prepared(args, resolver, |prepared| eval_dollar_adapter_prepared(prepared, ctx), DollarEvalError::Coercion)
}

pub fn map_dollar_error_to_ws(e: &DollarEvalError) -> WorksheetErrorCode {
    match e {
        DollarEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        DollarEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        DollarEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locale_format::current_excel_host_context;
    use crate::resolver::{RefResolutionError, ReferenceResolver, ResolverCapabilities};
    use crate::value::{ExcelText, ReferenceLike};

    struct NoResolver;

    impl ReferenceResolver for NoResolver {
        fn capabilities(&self) -> ResolverCapabilities { ResolverCapabilities::permissive_local() }
        fn resolve_reference(&self, reference: &ReferenceLike) -> Result<EvalValue, RefResolutionError> {
            Err(RefResolutionError::UnresolvedReference { target: reference.target.clone() })
        }
    }

    fn text_arg(s: &str) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(s.encode_utf16().collect())))
    }

    #[test]
    fn eval_dollar_current_host_seed_rows() {
        let ctx = current_excel_host_context();
        assert_eq!(
            eval_dollar_surface(&[CallArgValue::Eval(EvalValue::Number(1234.567)), CallArgValue::Eval(EvalValue::Number(2.0))], &NoResolver, &ctx),
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units("R1 234.57".encode_utf16().collect())))
        );
        assert_eq!(
            eval_dollar_surface(&[CallArgValue::Eval(EvalValue::Number(-1234.567)), CallArgValue::Eval(EvalValue::Number(2.0))], &NoResolver, &ctx),
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units("-R1 234.57".encode_utf16().collect())))
        );
        assert_eq!(
            eval_dollar_surface(&[CallArgValue::Eval(EvalValue::Logical(true)), CallArgValue::Eval(EvalValue::Number(2.0))], &NoResolver, &ctx),
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units("R1.00".encode_utf16().collect())))
        );
        assert_eq!(
            eval_dollar_surface(&[text_arg("123"), CallArgValue::Eval(EvalValue::Number(2.0))], &NoResolver, &ctx),
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units("R123.00".encode_utf16().collect())))
        );
        assert!(matches!(
            eval_dollar_surface(&[text_arg("x"), CallArgValue::Eval(EvalValue::Number(2.0))], &NoResolver, &ctx),
            Err(DollarEvalError::Coercion(CoercionError::NonNumericText(_)))
        ));
        assert_eq!(
            eval_dollar_surface(&[CallArgValue::EmptyCell, CallArgValue::Eval(EvalValue::Number(2.0))], &NoResolver, &ctx),
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units("R0.00".encode_utf16().collect())))
        );
    }
}


