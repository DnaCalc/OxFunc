use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::run_values_only_prepared;
use crate::locale_format::LocaleFormatContext;
use crate::resolver::ReferenceSystemProvider;
use crate::value::{CalcValue, CoreValue, WorksheetErrorCode};

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
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

fn coerce_number_arg(arg: &CalcValue, ctx: &LocaleFormatContext) -> Result<f64, DollarEvalError> {
    match arg.core() {
        CoreValue::Number(n) => Ok(*n),
        CoreValue::Logical(b) => Ok(if *b { 1.0 } else { 0.0 }),
        CoreValue::Text(text) => ctx
            .parser
            .parse_value_text(&ctx.profile, ctx.date_system, &text.to_string_lossy())
            .map_err(|_| {
                DollarEvalError::Coercion(CoercionError::NonNumericText(text.to_string_lossy()))
            }),
        CoreValue::Error(code) => Err(DollarEvalError::Coercion(CoercionError::WorksheetError(
            *code,
        ))),
        CoreValue::Empty => Ok(0.0),
        CoreValue::Missing => Err(DollarEvalError::Coercion(CoercionError::MissingArg)),
        CoreValue::Array(_) | CoreValue::Reference(_) => Err(DollarEvalError::Coercion(
            CoercionError::UnsupportedValueKind("dollar_arg_kind"),
        )),
    }
}

pub fn eval_dollar_adapter_prepared(
    args: &[CalcValue],
    ctx: &LocaleFormatContext,
) -> Result<CalcValue, DollarEvalError> {
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
        .map_err(|_| {
            DollarEvalError::Coercion(CoercionError::UnsupportedValueKind("currency_format"))
        })?;
    Ok(CalcValue::text(text))
}

pub fn eval_dollar_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    ctx: &LocaleFormatContext,
) -> Result<CalcValue, DollarEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_dollar_adapter_prepared(prepared, ctx),
        DollarEvalError::Coercion,
    )
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
    use crate::locale_format::test_current_excel_host_context;
    use crate::resolver::{ReferenceSystemCapabilities, ReferenceSystemProvider};
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

    fn text_arg(s: &str) -> CalcValue {
        CalcValue::text(ExcelText::from_utf16_code_units(s.encode_utf16().collect()))
    }

    #[test]
    fn eval_dollar_current_host_seed_rows() {
        let ctx = test_current_excel_host_context();
        assert_eq!(
            eval_dollar_surface(
                &[(CalcValue::number(1234.567)), (CalcValue::number(2.0))],
                &NoResolver,
                &ctx
            ),
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "R1 234.57".encode_utf16().collect()
            )))
        );
        assert_eq!(
            eval_dollar_surface(
                &[(CalcValue::number(-1234.567)), (CalcValue::number(2.0))],
                &NoResolver,
                &ctx
            ),
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "-R1 234.57".encode_utf16().collect()
            )))
        );
        assert_eq!(
            eval_dollar_surface(
                &[(CalcValue::logical(true)), (CalcValue::number(2.0))],
                &NoResolver,
                &ctx
            ),
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "R1.00".encode_utf16().collect()
            )))
        );
        assert_eq!(
            eval_dollar_surface(
                &[text_arg("123"), (CalcValue::number(2.0))],
                &NoResolver,
                &ctx
            ),
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "R123.00".encode_utf16().collect()
            )))
        );
        assert!(matches!(
            eval_dollar_surface(
                &[text_arg("x"), (CalcValue::number(2.0))],
                &NoResolver,
                &ctx
            ),
            Err(DollarEvalError::Coercion(CoercionError::NonNumericText(_)))
        ));
        assert_eq!(
            eval_dollar_surface(
                &[CalcValue::empty(), (CalcValue::number(2.0))],
                &NoResolver,
                &ctx
            ),
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "R0.00".encode_utf16().collect()
            )))
        );
    }
}
