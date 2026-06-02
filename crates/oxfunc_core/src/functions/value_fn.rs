use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedValue, run_values_only_prepared};
use crate::locale_format::{LocaleFormatContext, ParseFailure};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{
    FunctionArg, FunctionArray, FunctionArrayCell, FunctionValue, WorksheetErrorCode,
};

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
    cell: &FunctionArrayCell,
    ctx: &LocaleFormatContext,
) -> Result<FunctionArrayCell, ValueEvalError> {
    match cell {
        FunctionArrayCell::Number(n) => Ok(FunctionArrayCell::Number(*n)),
        FunctionArrayCell::Text(text) => match ctx.parser.parse_value_text(
            &ctx.profile,
            ctx.date_system,
            &text.to_string_lossy(),
        ) {
            Ok(parsed) => Ok(FunctionArrayCell::Number(parsed)),
            Err(_) => Ok(FunctionArrayCell::Error(WorksheetErrorCode::Value)),
        },
        FunctionArrayCell::Error(code) => Ok(FunctionArrayCell::Error(*code)),
        FunctionArrayCell::Logical(_) | FunctionArrayCell::EmptyCell => {
            Ok(FunctionArrayCell::Error(WorksheetErrorCode::Value))
        }
    }
}
pub fn eval_value_adapter_prepared(
    args: &[PreparedValue],
    ctx: &LocaleFormatContext,
) -> Result<FunctionValue, ValueEvalError> {
    if !VALUE_META.arity.accepts(args.len()) {
        return Err(ValueEvalError::ArityMismatch {
            expected: VALUE_META.arity.min,
            actual: args.len(),
        });
    }

    match &args[0] {
        PreparedValue::Eval(FunctionValue::Number(n)) => Ok(FunctionValue::Number(*n)),
        PreparedValue::Eval(FunctionValue::Text(text)) => {
            let parsed = ctx
                .parser
                .parse_value_text(&ctx.profile, ctx.date_system, &text.to_string_lossy())
                .map_err(ValueEvalError::Parse)?;
            Ok(FunctionValue::Number(parsed))
        }
        PreparedValue::Eval(FunctionValue::Array(array)) => {
            let cells = array
                .iter_row_major()
                .map(|cell| eval_value_array_cell(cell, ctx))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(FunctionValue::Array(
                FunctionArray::new(array.shape(), cells).expect("input array shape is valid"),
            ))
        }
        PreparedValue::Eval(FunctionValue::Error(code)) => Ok(FunctionValue::Error(*code)),
        PreparedValue::Eval(FunctionValue::Logical(_))
        | PreparedValue::Eval(FunctionValue::Reference(_))
        | PreparedValue::MissingArg
        | PreparedValue::EmptyCell => Err(ValueEvalError::Parse(ParseFailure::UnsupportedText(
            String::new(),
        ))),
        _ => Err(ValueEvalError::Parse(ParseFailure::UnsupportedText(
            String::new(),
        ))),
    }
}

pub fn eval_value_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    ctx: &LocaleFormatContext,
) -> Result<FunctionValue, ValueEvalError> {
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
        ) -> Result<FunctionValue, crate::resolver::ReferenceResolutionError> {
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
            FunctionArg::Eval(FunctionValue::Text(ExcelText::from_utf16_code_units(
                s.encode_utf16().collect(),
            )))
        };
        assert_eq!(
            eval_value_surface(&[mk("1 234.5")], &NoResolver, &ctx),
            Ok(FunctionValue::Number(1234.5))
        );
        assert_eq!(
            eval_value_surface(&[mk("R1 234.57")], &NoResolver, &ctx),
            Ok(FunctionValue::Number(1234.57))
        );
        assert_eq!(
            eval_value_surface(&[mk("12%")], &NoResolver, &ctx),
            Ok(FunctionValue::Number(0.12))
        );
        assert_eq!(
            eval_value_surface(&[mk("2024-02-03")], &NoResolver, &ctx),
            Ok(FunctionValue::Number(45325.0))
        );
        assert!(matches!(
            eval_value_surface(&[mk("1/2/2024")], &NoResolver, &ctx),
            Err(ValueEvalError::Parse(_))
        ));
    }

    #[test]
    fn value_lifts_array_elementwise_and_preserves_errors() {
        let ctx = test_current_excel_host_context();
        let args = [FunctionArg::Eval(FunctionValue::Array(
            FunctionArray::from_rows(vec![vec![
                FunctionArrayCell::Text(ExcelText::from_interop_assignment("12%")),
                FunctionArrayCell::Number(3.0),
                FunctionArrayCell::Error(WorksheetErrorCode::Div0),
                FunctionArrayCell::Text(ExcelText::from_interop_assignment("x")),
            ]])
            .unwrap(),
        ))];
        let got = eval_value_surface(&args, &NoResolver, &ctx);
        assert_eq!(
            got,
            Ok(FunctionValue::Array(
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Number(0.12),
                    FunctionArrayCell::Number(3.0),
                    FunctionArrayCell::Error(WorksheetErrorCode::Div0),
                    FunctionArrayCell::Error(WorksheetErrorCode::Value),
                ]])
                .unwrap()
            ))
        );
    }
}
