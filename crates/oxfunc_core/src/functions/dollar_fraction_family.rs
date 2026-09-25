use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::run_values_only_prepared_lifted;
use crate::resolver::ReferenceSystemProvider;
use crate::value::{CalcValue, CoreValue, WorksheetErrorCode};

pub const DOLLARDE_META: FunctionMeta = function_spec! {
    function_id: "FUNC.DOLLARDE",
    arity: Arity::exact(2),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

// DOLLARFR is scalar-shaped by-index and broadcasts both of its arguments over an array
// (`[0,1]`); DOLLARDE lifts natively (default). Verified live Excel 16.0 build 20026.
pub const DOLLARFR_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DOLLARFR",
    lift_broadcast_profile: FunctionMeta::lift_at(&[0, 1]),
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

fn coerce_arg_number(arg: &CalcValue) -> Result<f64, DollarFractionEvalError> {
    match arg.core() {
        CoreValue::Number(n) => Ok(*n),
        CoreValue::Text(text) => {
            let raw = text.to_string_lossy();
            parse_numeric_text(&raw).ok_or_else(|| {
                DollarFractionEvalError::Coercion(CoercionError::NonNumericText(raw))
            })
        }
        CoreValue::Error(code) => Err(DollarFractionEvalError::Coercion(
            CoercionError::WorksheetError(*code),
        )),
        CoreValue::Missing => Err(DollarFractionEvalError::MissingArg),
        CoreValue::Empty => Ok(0.0),
        CoreValue::Logical(_) => Err(DollarFractionEvalError::Coercion(
            CoercionError::UnsupportedValueKind("logical_not_admitted"),
        )),
        CoreValue::Array(_) | CoreValue::Reference(_) => Err(DollarFractionEvalError::Coercion(
            CoercionError::UnsupportedValueKind("dollar_fraction_arg_kind"),
        )),
    }
}

/// The fraction argument truncated to an integer, kept as a double: an `i32` saturates at
/// 2^31-1 and made `DOLLARDE(1.5, 1e10)` 3.328 where Excel gives 1.5.
fn normalized_fraction_denominator(fraction: f64) -> Result<f64, WorksheetErrorCode> {
    if fraction < 0.0 {
        return Err(WorksheetErrorCode::Num);
    }
    let truncated = fraction.trunc();
    if truncated == 0.0 {
        return Err(WorksheetErrorCode::Div0);
    }
    Ok(truncated)
}

/// The smallest power of ten that is at least the denominator: 10^ceil(log10(d)). Excel uses
/// this, not 10^(digit count): the two differ exactly when d is a power of ten
/// (`DOLLARDE(3.69, 10)` is 3.69, not 9.9; `DOLLARDE(x, 1)` is x). Live Excel 20430, W111-5
/// G8-01.
fn decimal_scale(denominator: f64) -> f64 {
    let d = denominator.abs();
    let mut scale = 1.0;
    while scale < d {
        scale *= 10.0;
    }
    scale
}

pub fn dollarde_kernel(number: f64, fraction: f64) -> Result<f64, WorksheetErrorCode> {
    let denominator = normalized_fraction_denominator(fraction)?;
    let scale = decimal_scale(denominator);
    let whole = number.trunc();
    let fractional = number - whole;
    Ok(whole + fractional * scale / denominator)
}

pub fn dollarfr_kernel(number: f64, fraction: f64) -> Result<f64, WorksheetErrorCode> {
    let denominator = normalized_fraction_denominator(fraction)?;
    let scale = decimal_scale(denominator);
    let whole = number.trunc();
    let fractional = number - whole;
    Ok(whole + fractional * denominator / scale)
}

fn eval_family_prepared(
    args: &[CalcValue],
    function_meta: &FunctionMeta,
    kernel: fn(f64, f64) -> Result<f64, WorksheetErrorCode>,
) -> Result<CalcValue, DollarFractionEvalError> {
    if !function_meta.arity.accepts(args.len()) {
        return Err(DollarFractionEvalError::ArityMismatch {
            expected: function_meta.arity.min,
            actual: args.len(),
        });
    }
    if args.iter().any(CalcValue::is_missing) {
        return Ok(CalcValue::error(WorksheetErrorCode::NA));
    }
    let number = coerce_arg_number(&args[0])?;
    let fraction = coerce_arg_number(&args[1])?;
    match kernel(number, fraction) {
        Ok(value) => Ok(CalcValue::number(value)),
        Err(code) => Ok(CalcValue::error(code)),
    }
}

pub fn eval_dollarde_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, DollarFractionEvalError> {
    run_values_only_prepared_lifted(
        args,
        resolver,
        |prepared| eval_family_prepared(prepared, &DOLLARDE_META, dollarde_kernel),
        map_dollar_fraction_error_to_ws,
        DollarFractionEvalError::Coercion,
    )
}

pub fn eval_dollarfr_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, DollarFractionEvalError> {
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

    /// W111-5 G8-01 live Excel 20430 bits: the decimal scale is 10^ceil(log10(d)), so a
    /// power-of-ten or unit denominator leaves the number unchanged, and a huge denominator
    /// must not saturate.
    #[test]
    fn dollarde_power_of_ten_and_huge_denominators_match_excel_bits() {
        let b = |h: u64| f64::from_bits(h);
        for (number, fraction) in [
            (0x400d_852c_82bc_cb80, 0x4024_0000_0000_0000), // DOLLARDE(3.690026.., 10)
            (0x4021_ed12_f088_5a20, 0x3ff0_0000_0000_0000), // DOLLARDE(8.963035.., 1)
            (0x3ff8_0000_0000_0000, 0x4202_a05f_2000_0000), // DOLLARDE(1.5, 1e10)
        ] {
            assert_eq!(
                dollarde_kernel(b(number), b(fraction)).unwrap().to_bits(),
                number,
                "Excel returns the number unchanged"
            );
        }
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
        let got =
            eval_dollarde_surface(&[text_arg("1.02"), (CalcValue::number(16.0))], &NoResolver);
        assert_eq!(got, Ok(CalcValue::number(1.125)));

        let logical = eval_dollarde_surface(
            &[(CalcValue::logical(true)), (CalcValue::number(16.0))],
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
            &[CalcValue::empty(), (CalcValue::number(16.0))],
            &NoResolver,
        );
        assert_eq!(blank_number, Ok(CalcValue::number(0.0)));

        let blank_denominator = eval_dollarfr_surface(
            &[(CalcValue::number(1.125)), CalcValue::empty()],
            &NoResolver,
        );
        assert_eq!(
            blank_denominator,
            Ok(CalcValue::error(WorksheetErrorCode::Div0))
        );

        let missing = eval_dollarde_surface(
            &[CalcValue::missing(), (CalcValue::number(16.0))],
            &NoResolver,
        );
        assert_eq!(missing, Ok(CalcValue::error(WorksheetErrorCode::NA)));
    }

    #[test]
    fn surface_propagates_non_numeric_text_and_worksheet_errors() {
        let text = eval_dollarfr_surface(&[text_arg("x"), (CalcValue::number(16.0))], &NoResolver);
        assert!(matches!(
            text,
            Err(DollarFractionEvalError::Coercion(
                CoercionError::NonNumericText(_)
            ))
        ));

        let ws_err = eval_dollarde_surface(
            &[
                (CalcValue::error(WorksheetErrorCode::Div0)),
                (CalcValue::number(16.0)),
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
