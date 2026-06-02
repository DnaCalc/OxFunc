use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedValue, run_values_only_prepared_lifted};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{FunctionArg, FunctionValue, WorksheetErrorCode};

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

fn coerce_arg_number(arg: &PreparedValue) -> Result<f64, DollarFractionEvalError> {
    match arg {
        PreparedValue::Eval(FunctionValue::Number(n)) => Ok(*n),
        PreparedValue::Eval(FunctionValue::Text(text)) => {
            let raw = text.to_string_lossy();
            parse_numeric_text(&raw).ok_or_else(|| {
                DollarFractionEvalError::Coercion(CoercionError::NonNumericText(raw))
            })
        }
        PreparedValue::Eval(FunctionValue::Error(code)) => Err(DollarFractionEvalError::Coercion(
            CoercionError::WorksheetError(*code),
        )),
        PreparedValue::MissingArg => Err(DollarFractionEvalError::MissingArg),
        PreparedValue::EmptyCell => Ok(0.0),
        PreparedValue::Eval(FunctionValue::Logical(_)) => Err(DollarFractionEvalError::Coercion(
            CoercionError::UnsupportedValueKind("logical_not_admitted"),
        )),
        PreparedValue::Eval(FunctionValue::Array(_))
        | PreparedValue::Eval(FunctionValue::Reference(_)) => {
            Err(DollarFractionEvalError::Coercion(
                CoercionError::UnsupportedValueKind("dollar_fraction_arg_kind"),
            ))
        }
        _ => Err(DollarFractionEvalError::Coercion(
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
    args: &[PreparedValue],
    function_meta: &FunctionMeta,
    kernel: fn(f64, f64) -> Result<f64, WorksheetErrorCode>,
) -> Result<FunctionValue, DollarFractionEvalError> {
    if !function_meta.arity.accepts(args.len()) {
        return Err(DollarFractionEvalError::ArityMismatch {
            expected: function_meta.arity.min,
            actual: args.len(),
        });
    }
    if args
        .iter()
        .any(|arg| matches!(arg, PreparedValue::MissingArg))
    {
        return Ok(FunctionValue::Error(WorksheetErrorCode::NA));
    }
    let number = coerce_arg_number(&args[0])?;
    let fraction = coerce_arg_number(&args[1])?;
    match kernel(number, fraction) {
        Ok(value) => Ok(FunctionValue::Number(value)),
        Err(code) => Ok(FunctionValue::Error(code)),
    }
}

pub fn eval_dollarde_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, DollarFractionEvalError> {
    run_values_only_prepared_lifted(
        args,
        resolver,
        |prepared| eval_family_prepared(prepared, &DOLLARDE_META, dollarde_kernel),
        map_dollar_fraction_error_to_ws,
        DollarFractionEvalError::Coercion,
    )
}

pub fn eval_dollarfr_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, DollarFractionEvalError> {
    run_values_only_prepared_lifted(
        args,
        resolver,
        |prepared| eval_family_prepared(prepared, &DOLLARFR_META, dollarfr_kernel),
        map_dollar_fraction_error_to_ws,
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
        ) -> Result<FunctionValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            Err(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
        }
    }

    fn text_arg(s: &str) -> FunctionArg {
        FunctionArg::Eval(FunctionValue::Text(ExcelText::from_utf16_code_units(
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
                FunctionArg::Eval(FunctionValue::Number(16.0)),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(FunctionValue::Number(1.125)));

        let logical = eval_dollarde_surface(
            &[
                FunctionArg::Eval(FunctionValue::Logical(true)),
                FunctionArg::Eval(FunctionValue::Number(16.0)),
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
                FunctionArg::EmptyCell,
                FunctionArg::Eval(FunctionValue::Number(16.0)),
            ],
            &NoResolver,
        );
        assert_eq!(blank_number, Ok(FunctionValue::Number(0.0)));

        let blank_denominator = eval_dollarfr_surface(
            &[
                FunctionArg::Eval(FunctionValue::Number(1.125)),
                FunctionArg::EmptyCell,
            ],
            &NoResolver,
        );
        assert_eq!(
            blank_denominator,
            Ok(FunctionValue::Error(WorksheetErrorCode::Div0))
        );

        let missing = eval_dollarde_surface(
            &[
                FunctionArg::MissingArg,
                FunctionArg::Eval(FunctionValue::Number(16.0)),
            ],
            &NoResolver,
        );
        assert_eq!(missing, Ok(FunctionValue::Error(WorksheetErrorCode::NA)));
    }

    #[test]
    fn surface_propagates_non_numeric_text_and_worksheet_errors() {
        let text = eval_dollarfr_surface(
            &[
                text_arg("x"),
                FunctionArg::Eval(FunctionValue::Number(16.0)),
            ],
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
                FunctionArg::Eval(FunctionValue::Error(WorksheetErrorCode::Div0)),
                FunctionArg::Eval(FunctionValue::Number(16.0)),
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
