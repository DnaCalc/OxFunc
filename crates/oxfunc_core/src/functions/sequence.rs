use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedArgValue, coerce_prepared_to_number, run_values_only_prepared};
use crate::resolver::ReferenceResolver;
use crate::value::{ArrayShape, CallArgValue, EvalValue, WorksheetErrorCode};

pub const SEQUENCE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SEQUENCE",
    arity: Arity { min: 1, max: 4 },
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

#[derive(Debug, Clone, PartialEq)]
pub enum SequenceEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    InvalidDimension {
        arg_index: usize,
        value: f64,
    },
}

fn parse_dimension(raw: f64, arg_index: usize) -> Result<usize, SequenceEvalError> {
    if !raw.is_finite() || raw <= 0.0 || raw.fract() != 0.0 {
        return Err(SequenceEvalError::InvalidDimension {
            arg_index,
            value: raw,
        });
    }
    Ok(raw as usize)
}

pub fn eval_sequence_adapter_prepared(
    args: &[PreparedArgValue],
) -> Result<EvalValue, SequenceEvalError> {
    let argc = args.len();
    if !SEQUENCE_META.arity.accepts(argc) {
        return Err(SequenceEvalError::ArityMismatch {
            expected_min: SEQUENCE_META.arity.min,
            expected_max: SEQUENCE_META.arity.max,
            actual: argc,
        });
    }

    let rows = parse_dimension(
        coerce_prepared_to_number(&args[0]).map_err(SequenceEvalError::Coercion)?,
        1,
    )?;

    let cols = if argc >= 2 {
        parse_dimension(
            coerce_prepared_to_number(&args[1]).map_err(SequenceEvalError::Coercion)?,
            2,
        )?
    } else {
        1
    };

    // `start` and `step` are parsed to enforce seed coercion policy,
    // but array payload generation is intentionally out of this model scope.
    if argc >= 3 {
        let _ = coerce_prepared_to_number(&args[2]).map_err(SequenceEvalError::Coercion)?;
    }
    if argc >= 4 {
        let _ = coerce_prepared_to_number(&args[3]).map_err(SequenceEvalError::Coercion)?;
    }

    Ok(EvalValue::Array(ArrayShape { rows, cols }))
}

pub fn eval_sequence_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, SequenceEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_sequence_adapter_prepared,
        SequenceEvalError::Coercion,
    )
}

pub fn map_sequence_error_to_ws(e: &SequenceEvalError) -> WorksheetErrorCode {
    match e {
        SequenceEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        SequenceEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        SequenceEvalError::InvalidDimension { .. } => WorksheetErrorCode::Value,
        SequenceEvalError::Coercion(_) => WorksheetErrorCode::Value,
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

    #[test]
    fn eval_sequence_rows_only_defaults_cols_to_one() {
        let args = [CallArgValue::Eval(EvalValue::Number(3.0))];
        let got = eval_sequence_surface(&args, &NoResolver);
        assert_eq!(got, Ok(EvalValue::Array(ArrayShape { rows: 3, cols: 1 })));
    }

    #[test]
    fn eval_sequence_parses_full_arity() {
        let args = [
            CallArgValue::Eval(EvalValue::Number(2.0)),
            CallArgValue::Eval(EvalValue::Number(3.0)),
            CallArgValue::Eval(EvalValue::Number(10.0)),
            CallArgValue::Eval(EvalValue::Number(2.0)),
        ];
        let got = eval_sequence_surface(&args, &NoResolver);
        assert_eq!(got, Ok(EvalValue::Array(ArrayShape { rows: 2, cols: 3 })));
    }

    #[test]
    fn eval_sequence_numeric_text_dimension_is_allowed() {
        let args = [CallArgValue::Eval(EvalValue::Text(
            ExcelText::from_utf16_code_units("4".encode_utf16().collect()),
        ))];
        let got = eval_sequence_surface(&args, &NoResolver);
        assert_eq!(got, Ok(EvalValue::Array(ArrayShape { rows: 4, cols: 1 })));
    }

    #[test]
    fn eval_sequence_rejects_zero_dimension() {
        let args = [CallArgValue::Eval(EvalValue::Number(0.0))];
        let got = eval_sequence_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Err(SequenceEvalError::InvalidDimension {
                arg_index: 1,
                value: 0.0,
            })
        );
    }
}
