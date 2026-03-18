use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_text, run_values_only_prepared,
};
use crate::locale_format::{FormatFailure, LocaleFormatContext};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, ExcelText, WorksheetErrorCode};

pub const TEXT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.TEXT",
    arity: Arity::exact(2),
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
pub enum TextEvalError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
    Format(FormatFailure),
}

fn logical_text(value: bool) -> ExcelText {
    ExcelText::from_utf16_code_units(
        (if value { "TRUE" } else { "FALSE" })
            .encode_utf16()
            .collect(),
    )
}

pub fn eval_text_adapter_prepared(
    args: &[PreparedArgValue],
    ctx: &LocaleFormatContext,
) -> Result<EvalValue, TextEvalError> {
    if !TEXT_META.arity.accepts(args.len()) {
        return Err(TextEvalError::ArityMismatch {
            expected: TEXT_META.arity.min,
            actual: args.len(),
        });
    }

    let format_code = coerce_prepared_to_text(&args[1]).map_err(TextEvalError::Coercion)?;
    let format_code_string = format_code.to_string_lossy();

    let rendered = match &args[0] {
        PreparedArgValue::Eval(EvalValue::Number(n)) => ctx
            .formatter
            .render_with_code(&ctx.profile, ctx.date_system, *n, &format_code_string)
            .map_err(TextEvalError::Format)?,
        PreparedArgValue::Eval(EvalValue::Text(text)) => {
            let raw = text.to_string_lossy();
            match ctx
                .parser
                .parse_value_text(&ctx.profile, ctx.date_system, &raw)
            {
                Ok(parsed) => ctx
                    .formatter
                    .render_with_code(&ctx.profile, ctx.date_system, parsed, &format_code_string)
                    .map_err(TextEvalError::Format)?,
                Err(_) => text.clone(),
            }
        }
        PreparedArgValue::Eval(EvalValue::Logical(b)) => logical_text(*b),
        PreparedArgValue::Eval(EvalValue::Error(code)) => return Ok(EvalValue::Error(*code)),
        PreparedArgValue::EmptyCell => ctx
            .formatter
            .render_with_code(&ctx.profile, ctx.date_system, 0.0, &format_code_string)
            .map_err(TextEvalError::Format)?,
        PreparedArgValue::MissingArg => {
            return Err(TextEvalError::Coercion(CoercionError::MissingArg));
        }
        PreparedArgValue::Eval(EvalValue::Array(_))
        | PreparedArgValue::Eval(EvalValue::Reference(_))
        | PreparedArgValue::Eval(EvalValue::Lambda(_)) => {
            return Err(TextEvalError::Coercion(
                CoercionError::UnsupportedValueKind("text_arg_kind"),
            ));
        }
    };

    Ok(EvalValue::Text(rendered))
}

pub fn eval_text_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    ctx: &LocaleFormatContext,
) -> Result<EvalValue, TextEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_text_adapter_prepared(prepared, ctx),
        TextEvalError::Coercion,
    )
}

pub fn map_text_error_to_ws(e: &TextEvalError) -> WorksheetErrorCode {
    match e {
        TextEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        TextEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        TextEvalError::Coercion(_) | TextEvalError::Format(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locale_format::current_excel_host_context;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::ReferenceLike;

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
    fn text_current_host_seed_rows() {
        let ctx = current_excel_host_context();
        let got = eval_text_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(0.125)),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                    "0%".encode_utf16().collect(),
                ))),
            ],
            &NoResolver,
            &ctx,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "13%".encode_utf16().collect()
            )))
        );

        let got_text = eval_text_surface(
            &[
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                    "x".encode_utf16().collect(),
                ))),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                    "0".encode_utf16().collect(),
                ))),
            ],
            &NoResolver,
            &ctx,
        );
        assert_eq!(
            got_text,
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "x".encode_utf16().collect()
            )))
        );

        let got_logical = eval_text_surface(
            &[
                CallArgValue::Eval(EvalValue::Logical(true)),
                CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                    "0".encode_utf16().collect(),
                ))),
            ],
            &NoResolver,
            &ctx,
        );
        assert_eq!(
            got_logical,
            Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "TRUE".encode_utf16().collect()
            )))
        );
    }
}
