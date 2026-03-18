use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedArgValue, run_values_only_prepared};
use crate::locale_format::{LocaleFormatContext, ParseFailure};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const VALUE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.VALUE",
    arity: Arity::exact(1),
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
pub enum ValueEvalError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
    Parse(ParseFailure),
}

pub fn eval_value_adapter_prepared(
    args: &[PreparedArgValue],
    ctx: &LocaleFormatContext,
) -> Result<EvalValue, ValueEvalError> {
    if !VALUE_META.arity.accepts(args.len()) {
        return Err(ValueEvalError::ArityMismatch {
            expected: VALUE_META.arity.min,
            actual: args.len(),
        });
    }

    match &args[0] {
        PreparedArgValue::Eval(EvalValue::Number(n)) => Ok(EvalValue::Number(*n)),
        PreparedArgValue::Eval(EvalValue::Text(text)) => {
            let parsed = ctx
                .parser
                .parse_value_text(&ctx.profile, ctx.date_system, &text.to_string_lossy())
                .map_err(ValueEvalError::Parse)?;
            Ok(EvalValue::Number(parsed))
        }
        PreparedArgValue::Eval(EvalValue::Error(code)) => Ok(EvalValue::Error(*code)),
        PreparedArgValue::Eval(EvalValue::Logical(_))
        | PreparedArgValue::Eval(EvalValue::Array(_))
        | PreparedArgValue::Eval(EvalValue::Reference(_))
        | PreparedArgValue::Eval(EvalValue::Lambda(_))
        | PreparedArgValue::MissingArg
        | PreparedArgValue::EmptyCell => Err(ValueEvalError::Parse(ParseFailure::UnsupportedText(
            String::new(),
        ))),
    }
}

pub fn eval_value_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    ctx: &LocaleFormatContext,
) -> Result<EvalValue, ValueEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_value_adapter_prepared(prepared, ctx),
        ValueEvalError::Coercion,
    )
}

pub fn map_value_error_to_ws(e: &ValueEvalError) -> WorksheetErrorCode {
    match e {
        ValueEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        ValueEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        ValueEvalError::Coercion(_) | ValueEvalError::Parse(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locale_format::current_excel_host_context;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
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

    #[test]
    fn value_current_host_seed_rows() {
        let ctx = current_excel_host_context();
        let mk = |s: &str| {
            CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                s.encode_utf16().collect(),
            )))
        };
        assert_eq!(
            eval_value_surface(&[mk("1 234.5")], &NoResolver, &ctx),
            Ok(EvalValue::Number(1234.5))
        );
        assert_eq!(
            eval_value_surface(&[mk("R1 234.57")], &NoResolver, &ctx),
            Ok(EvalValue::Number(1234.57))
        );
        assert_eq!(
            eval_value_surface(&[mk("12%")], &NoResolver, &ctx),
            Ok(EvalValue::Number(0.12))
        );
        assert_eq!(
            eval_value_surface(&[mk("2024-02-03")], &NoResolver, &ctx),
            Ok(EvalValue::Number(45325.0))
        );
        assert!(matches!(
            eval_value_surface(&[mk("1/2/2024")], &NoResolver, &ctx),
            Err(ValueEvalError::Parse(_))
        ));
    }
}
