use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    BroadcastPreparedGroup, PreparedArgValue, coerce_prepared_to_number,
    expand_prepared_broadcast_grid, prepare_args_values_only,
};
use crate::resolver::ReferenceResolver;
use crate::value::{ArrayCellValue, CallArgValue, EvalArray, EvalValue, WorksheetErrorCode};

pub const TRUNC_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.TRUNC",
    arity: Arity { min: 1, max: 2 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::NumsToNum,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

#[derive(Debug, Clone, PartialEq)]
pub enum TruncEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

pub fn trunc_kernel(number: f64, digits: i32) -> f64 {
    if digits >= 0 {
        let factor = 10f64.powi(digits);
        (number * factor).trunc() / factor
    } else {
        let factor = 10f64.powi(-digits);
        (number / factor).trunc() * factor
    }
}

pub fn eval_trunc_adapter_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, TruncEvalError> {
    if !TRUNC_META.arity.accepts(args.len()) {
        return Err(TruncEvalError::ArityMismatch {
            expected_min: TRUNC_META.arity.min,
            expected_max: TRUNC_META.arity.max,
            actual: args.len(),
        });
    }

    if let Some((shape, cells)) = expand_prepared_broadcast_grid(args) {
        let mapped = cells
            .into_iter()
            .map(|cell| match cell {
                BroadcastPreparedGroup::Values(values) => map_trunc_item(&values),
                BroadcastPreparedGroup::MissingCoordinate => {
                    ArrayCellValue::Error(WorksheetErrorCode::NA)
                }
            })
            .collect();
        return Ok(EvalValue::Array(
            EvalArray::new(shape, mapped).expect("shape preserved"),
        ));
    }

    let number = coerce_prepared_to_number(&args[0]).map_err(TruncEvalError::Coercion)?;
    let digits = if args.len() == 1 {
        0
    } else {
        coerce_prepared_to_number(&args[1])
            .map_err(TruncEvalError::Coercion)?
            .trunc() as i32
    };
    Ok(EvalValue::Number(trunc_kernel(number, digits)))
}

fn map_trunc_item(args: &[PreparedArgValue]) -> ArrayCellValue {
    let number = match coerce_prepared_to_number(&args[0]) {
        Ok(value) => value,
        Err(CoercionError::WorksheetError(code)) => return ArrayCellValue::Error(code),
        Err(_) => return ArrayCellValue::Error(WorksheetErrorCode::Value),
    };
    let digits = if args.len() == 1 {
        0
    } else {
        match coerce_prepared_to_number(&args[1]) {
            Ok(value) => value.trunc() as i32,
            Err(CoercionError::WorksheetError(code)) => return ArrayCellValue::Error(code),
            Err(_) => return ArrayCellValue::Error(WorksheetErrorCode::Value),
        }
    };
    ArrayCellValue::Number(trunc_kernel(number, digits))
}

pub fn eval_trunc_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, TruncEvalError> {
    let prepared = prepare_args_values_only(args, resolver).map_err(TruncEvalError::Coercion)?;
    eval_trunc_adapter_prepared(&prepared)
}

pub fn map_trunc_error_to_ws(e: &TruncEvalError) -> WorksheetErrorCode {
    match e {
        TruncEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        TruncEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        TruncEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ArrayCellValue, EvalArray, ReferenceLike};

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
    fn eval_trunc_spills_array_with_omitted_digits() {
        let got = eval_trunc_surface(
            &[CallArgValue::Eval(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(1.234)],
                    vec![ArrayCellValue::Number(2.345)],
                ])
                .unwrap(),
            ))],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(1.0)],
                    vec![ArrayCellValue::Number(2.0)],
                ])
                .unwrap()
            ))
        );
    }
}
