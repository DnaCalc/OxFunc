use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::run_values_only_prepared;
use crate::locale_format::{LocaleFormatContext, ParseFailure};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{CalcArray, WorksheetErrorCode};
use crate::value::{CalcValue, CoreValue};

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

fn eval_value_array_cell(
    cell: &CalcValue,
    ctx: &LocaleFormatContext,
) -> Result<CalcValue, ValueEvalError> {
    match cell.core() {
        CoreValue::Number(n) => Ok(CalcValue::number(*n)),
        CoreValue::Text(text) => match ctx.parser.parse_value_text(
            &ctx.profile,
            ctx.date_system,
            &text.to_string_lossy(),
        ) {
            Ok(parsed) => Ok(CalcValue::number(parsed)),
            Err(_) => Ok(CalcValue::error(WorksheetErrorCode::Value)),
        },
        CoreValue::Error(code) => Ok(CalcValue::error(*code)),
        CoreValue::Logical(_)
        | CoreValue::Empty
        | CoreValue::Missing
        | CoreValue::Array(_)
        | CoreValue::Reference(_) => Ok(CalcValue::error(WorksheetErrorCode::Value)),
    }
}
pub fn eval_value_adapter_prepared(
    args: &[CalcValue],
    ctx: &LocaleFormatContext,
) -> Result<CalcValue, ValueEvalError> {
    if !VALUE_META.arity.accepts(args.len()) {
        return Err(ValueEvalError::ArityMismatch {
            expected: VALUE_META.arity.min,
            actual: args.len(),
        });
    }

    match args[0].core() {
        CoreValue::Number(n) => Ok(CalcValue::number(*n)),
        CoreValue::Text(text) => {
            let parsed = ctx
                .parser
                .parse_value_text(&ctx.profile, ctx.date_system, &text.to_string_lossy())
                .map_err(ValueEvalError::Parse)?;
            Ok(CalcValue::number(parsed))
        }
        CoreValue::Array(array) => {
            let cells = array
                .iter_row_major()
                .map(|cell| eval_value_array_cell(cell, ctx))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(CalcValue::array(
                CalcArray::new(array.shape(), cells).expect("input array shape is valid"),
            ))
        }
        CoreValue::Error(code) => Ok(CalcValue::error(*code)),
        CoreValue::Logical(_) | CoreValue::Reference(_) | CoreValue::Missing | CoreValue::Empty => {
            Err(ValueEvalError::Parse(ParseFailure::UnsupportedText(
                String::new(),
            )))
        }
    }
}

pub fn eval_value_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    ctx: &LocaleFormatContext,
) -> Result<CalcValue, ValueEvalError> {
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
    use crate::locale_format::test_current_excel_host_context;
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
    fn value_current_host_seed_rows() {
        let ctx = test_current_excel_host_context();
        let mk = |s: &str| {
            (CalcValue::text(ExcelText::from_utf16_code_units(s.encode_utf16().collect())))
        };
        assert_eq!(
            eval_value_surface(&[mk("1 234.5")], &NoResolver, &ctx),
            Ok(CalcValue::number(1234.5))
        );
        assert_eq!(
            eval_value_surface(&[mk("R1 234.57")], &NoResolver, &ctx),
            Ok(CalcValue::number(1234.57))
        );
        assert_eq!(
            eval_value_surface(&[mk("12%")], &NoResolver, &ctx),
            Ok(CalcValue::number(0.12))
        );
        assert_eq!(
            eval_value_surface(&[mk("2024-02-03")], &NoResolver, &ctx),
            Ok(CalcValue::number(45325.0))
        );
        assert!(matches!(
            eval_value_surface(&[mk("1/2/2024")], &NoResolver, &ctx),
            Err(ValueEvalError::Parse(_))
        ));
    }

    #[test]
    fn value_lifts_array_elementwise_and_preserves_errors() {
        let ctx = test_current_excel_host_context();
        let args = [(CalcValue::array(
            CalcArray::from_rows(vec![vec![
                CalcValue::text(ExcelText::from_interop_assignment("12%")),
                CalcValue::number(3.0),
                CalcValue::error(WorksheetErrorCode::Div0),
                CalcValue::text(ExcelText::from_interop_assignment("x")),
            ]])
            .unwrap(),
        ))];
        let got = eval_value_surface(&args, &NoResolver, &ctx);
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(0.12),
                    CalcValue::number(3.0),
                    CalcValue::error(WorksheetErrorCode::Div0),
                    CalcValue::error(WorksheetErrorCode::Value),
                ]])
                .unwrap()
            ))
        );
    }
}
