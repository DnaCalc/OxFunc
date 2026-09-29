use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    BroadcastPreparedGroup, coerce_prepared_to_number, expand_prepared_broadcast_grid,
    prepare_args_values_only,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::{CalcArray, WorksheetErrorCode};

pub const TRUNC_META: FunctionMeta = function_spec! {
    function_id: "FUNC.TRUNC",
    arity: Arity { min: 1, max: 2 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
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
    crate::functions::rounddown_fn::rounddown_kernel(number, digits)
}

pub fn eval_trunc_adapter_prepared(args: &[CalcValue]) -> Result<CalcValue, TruncEvalError> {
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
                    CalcValue::error(WorksheetErrorCode::NA)
                }
            })
            .collect();
        return Ok(CalcValue::array(
            CalcArray::new(shape, mapped).expect("shape preserved"),
        ));
    }

    let number = coerce_prepared_to_number(&args[0]).map_err(TruncEvalError::Coercion)?;
    let digits = if args.len() == 1 {
        0
    } else {
        crate::functions::round_fn::directed_digit_count(coerce_prepared_to_number(&args[1])
            .map_err(TruncEvalError::Coercion)?)
    };
    let result = trunc_kernel(number, digits);
    if result.is_finite() { Ok(CalcValue::number(result)) }
    else { Err(TruncEvalError::Coercion(CoercionError::WorksheetError(WorksheetErrorCode::Num))) }
}

fn map_trunc_item(args: &[CalcValue]) -> CalcValue {
    let number = match coerce_prepared_to_number(&args[0]) {
        Ok(value) => value,
        Err(CoercionError::WorksheetError(code)) => return CalcValue::error(code),
        Err(_) => return CalcValue::error(WorksheetErrorCode::Value),
    };
    let digits = if args.len() == 1 {
        0
    } else {
        match coerce_prepared_to_number(&args[1]) {
            Ok(value) => crate::functions::round_fn::directed_digit_count(value),
            Err(CoercionError::WorksheetError(code)) => return CalcValue::error(code),
            Err(_) => return CalcValue::error(WorksheetErrorCode::Value),
        }
    };
    let result = trunc_kernel(number, digits);
    if result.is_finite() { CalcValue::number(result) }
    else { CalcValue::error(WorksheetErrorCode::Num) }
}

pub fn eval_trunc_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, TruncEvalError> {
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
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::CalcArray;

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
    fn eval_trunc_spills_array_with_omitted_digits() {
        let got = eval_trunc_surface(
            &[(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(1.234)],
                    vec![CalcValue::number(2.345)],
                ])
                .unwrap(),
            ))],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(1.0)],
                    vec![CalcValue::number(2.0)],
                ])
                .unwrap()
            ))
        );
    }
}
