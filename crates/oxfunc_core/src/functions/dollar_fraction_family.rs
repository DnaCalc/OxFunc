use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedArgValue, run_values_only_prepared};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const DOLLARDE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DOLLARDE",
    arity: Arity::exact(2),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

pub const DOLLARFR_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DOLLARFR",
    ..DOLLARDE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum DollarFractionEvalError {
    ArityMismatch { expected: usize, actual: usize },
    MissingArg,
    Coercion(CoercionError),
}

fn parse_numeric_text(text: &str) -> Option<f64> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    let parsed = trimmed.parse::<f64>().ok()?;
    if parsed.is_finite() {
        Some(parsed)
    } else {
        None
    }
}

fn coerce_arg_number(arg: &PreparedArgValue) -> Result<f64, DollarFractionEvalError> {
    match arg {
        PreparedArgValue::Eval(EvalValue::Number(n)) => Ok(*n),
        PreparedArgValue::Eval(EvalValue::Text(text)) => {
            let raw = text.to_string_lossy();
            parse_numeric_text(&raw).ok_or_else(|| {
                DollarFractionEvalError::Coercion(CoercionError::NonNumericText(raw))
            })
        }
        PreparedArgValue::Eval(EvalValue::Error(code)) => Err(DollarFractionEvalError::Coercion(
            CoercionError::WorksheetError(*code),
        )),
        PreparedArgValue::MissingArg => Err(DollarFractionEvalError::MissingArg),
        PreparedArgValue::EmptyCell => Ok(0.0),
        PreparedArgValue::Eval(EvalValue::Logical(_)) => Err(DollarFractionEvalError::Coercion(
            CoercionError::UnsupportedValueKind("logical_not_admitted"),
        )),
        PreparedArgValue::Eval(EvalValue::Array(_))
        | PreparedArgValue::Eval(EvalValue::Reference(_))
        | PreparedArgValue::Eval(EvalValue::Lambda(_)) => Err(DollarFractionEvalError::Coercion(
            CoercionError::UnsupportedValueKind("dollar_fraction_arg_kind"),
        )),
    }
}

fn normalized_fraction_denominator(fraction: f64) -> Result<i32, WorksheetErrorCode> {
    if fraction < 0.0 {
        return Err(WorksheetErrorCode::Num);
    }
    let truncated = fraction.trunc();
    if truncated == 0.0 {
        return Err(WorksheetErrorCode::Div0);
    }
    Ok(truncated as i32)
}

fn decimal_scale(denominator: i32) -> f64 {
    let digits = denominator.abs().to_string().len() as i32;
    10f64.powi(digits)
}

pub fn dollarde_kernel(number: f64, fraction: f64) -> Result<f64, WorksheetErrorCode> {
    let denominator = normalized_fraction_denominator(fraction)?;
    let scale = decimal_scale(denominator);
    let whole = number.trunc();
    let fractional = number - whole;
    Ok(whole + fractional * scale / denominator as f64)
}

pub fn dollarfr_kernel(number: f64, fraction: f64) -> Result<f64, WorksheetErrorCode> {
    let denominator = normalized_fraction_denominator(fraction)?;
    let scale = decimal_scale(denominator);
    let whole = number.trunc();
    let fractional = number - whole;
    Ok(whole + fractional * denominator as f64 / scale)
}

fn eval_family_prepared(
    args: &[PreparedArgValue],
    function_meta: &FunctionMeta,
    kernel: fn(f64, f64) -> Result<f64, WorksheetErrorCode>,
) -> Result<EvalValue, DollarFractionEvalError> {
    if !function_meta.arity.accepts(args.len()) {
        return Err(DollarFractionEvalError::ArityMismatch {
            expected: function_meta.arity.min,
            actual: args.len(),
        });
    }
    if args
        .iter()
        .any(|arg| matches!(arg, PreparedArgValue::MissingArg))
    {
        return Ok(EvalValue::Error(WorksheetErrorCode::NA));
    }
    let number = coerce_arg_number(&args[0])?;
    let fraction = coerce_arg_number(&args[1])?;
    match kernel(number, fraction) {
        Ok(value) => Ok(EvalValue::Number(value)),
        Err(code) => Ok(EvalValue::Error(code)),
    }
}

pub fn eval_dollarde_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DollarFractionEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_family_prepared(prepared, &DOLLARDE_META, dollarde_kernel),
        DollarFractionEvalError::Coercion,
    )
}

pub fn eval_dollarfr_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DollarFractionEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_family_prepared(prepared, &DOLLARFR_META, dollarfr_kernel),
        DollarFractionEvalError::Coercion,
    )
}

pub fn map_dollar_fraction_error_to_ws(e: &DollarFractionEvalError) -> WorksheetErrorCode {
    match e {
        DollarFractionEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        DollarFractionEvalError::MissingArg => WorksheetErrorCode::NA,
        DollarFractionEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        DollarFractionEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    fn text_arg(s: &str) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
            s.encode_utf16().collect(),
        )))
    }

    fn assert_close(got: f64, expected: f64) {
        assert!(
            (got - expected).abs() < 1e-12,
            "expected {expected}, got {got}"
        );
    }

    #[test]
    fn dollarde_kernel_matches_native_seed_rows() {
        assert_close(dollarde_kernel(1.02, 16.0).unwrap(), 1.125);
        assert_close(dollarde_kernel(1.02, 8.0).unwrap(), 1.025);
        assert_close(dollarde_kernel(1.02, 16.9).unwrap(), 1.125);
        assert_close(dollarde_kernel(1.01, 32.0).unwrap(), 1.03125);
        assert_eq!(dollarde_kernel(1.02, 0.0), Err(WorksheetErrorCode::Div0));
        assert_eq!(dollarde_kernel(1.02, 0.9), Err(WorksheetErrorCode::Div0));
        assert_eq!(dollarde_kernel(1.02, -0.1), Err(WorksheetErrorCode::Num));
    }

    #[test]
    fn dollarfr_kernel_matches_native_seed_rows() {
        assert_close(dollarfr_kernel(1.125, 16.0).unwrap(), 1.02);
        assert_close(dollarfr_kernel(1.125, 8.0).unwrap(), 1.1);
        assert_close(dollarfr_kernel(1.125, 16.9).unwrap(), 1.02);
        assert_close(dollarfr_kernel(1.03125, 32.0).unwrap(), 1.01);
        assert_close(dollarfr_kernel(-1.125, 16.0).unwrap(), -1.02);
        assert_eq!(dollarfr_kernel(1.125, 0.0), Err(WorksheetErrorCode::Div0));
        assert_eq!(dollarfr_kernel(1.125, -1.0), Err(WorksheetErrorCode::Num));
    }

    #[test]
    fn surface_accepts_numeric_text_but_rejects_logicals() {
        let got = eval_dollarde_surface(
            &[
                text_arg("1.02"),
                CallArgValue::Eval(EvalValue::Number(16.0)),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(1.125)));

        let logical = eval_dollarde_surface(
            &[
                CallArgValue::Eval(EvalValue::Logical(true)),
                CallArgValue::Eval(EvalValue::Number(16.0)),
            ],
            &NoResolver,
        );
        assert!(matches!(
            logical,
            Err(DollarFractionEvalError::Coercion(
                CoercionError::UnsupportedValueKind("logical_not_admitted")
            ))
        ));
    }

    #[test]
    fn surface_blank_cells_become_zero_and_missing_args_become_na() {
        let blank_number = eval_dollarde_surface(
            &[
                CallArgValue::EmptyCell,
                CallArgValue::Eval(EvalValue::Number(16.0)),
            ],
            &NoResolver,
        );
        assert_eq!(blank_number, Ok(EvalValue::Number(0.0)));

        let blank_denominator = eval_dollarfr_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(1.125)),
                CallArgValue::EmptyCell,
            ],
            &NoResolver,
        );
        assert_eq!(
            blank_denominator,
            Ok(EvalValue::Error(WorksheetErrorCode::Div0))
        );

        let missing = eval_dollarde_surface(
            &[
                CallArgValue::MissingArg,
                CallArgValue::Eval(EvalValue::Number(16.0)),
            ],
            &NoResolver,
        );
        assert_eq!(missing, Ok(EvalValue::Error(WorksheetErrorCode::NA)));
    }

    #[test]
    fn surface_propagates_non_numeric_text_and_worksheet_errors() {
        let text = eval_dollarfr_surface(
            &[text_arg("x"), CallArgValue::Eval(EvalValue::Number(16.0))],
            &NoResolver,
        );
        assert!(matches!(
            text,
            Err(DollarFractionEvalError::Coercion(
                CoercionError::NonNumericText(_)
            ))
        ));

        let ws_err = eval_dollarde_surface(
            &[
                CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::Div0)),
                CallArgValue::Eval(EvalValue::Number(16.0)),
            ],
            &NoResolver,
        );
        assert_eq!(
            ws_err,
            Err(DollarFractionEvalError::Coercion(
                CoercionError::WorksheetError(WorksheetErrorCode::Div0)
            ))
        );
    }
}
