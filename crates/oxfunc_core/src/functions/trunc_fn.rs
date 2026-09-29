use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::prepare_args_values_only;
use crate::functions::binary_numeric::BinaryNumericSurfaceError;
use crate::functions::round_fn::eval_rounding_prepared;
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

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

    let values = [
        args[0].clone(),
        args.get(1)
            .cloned()
            .unwrap_or_else(|| CalcValue::number(0.0)),
    ];
    eval_rounding_prepared(&values, |number, count| {
        let result = trunc_kernel(
            number,
            crate::functions::round_fn::directed_digit_count(count),
        );
        if result.is_finite() {
            Ok(result)
        } else {
            Err(WorksheetErrorCode::Num)
        }
    })
    .map_err(|error| match error {
        BinaryNumericSurfaceError::Coercion(error) => TruncEvalError::Coercion(error),
        BinaryNumericSurfaceError::Domain(code) => {
            TruncEvalError::Coercion(CoercionError::WorksheetError(code))
        }
        BinaryNumericSurfaceError::ArityMismatch { actual, .. } => TruncEvalError::ArityMismatch {
            expected_min: 1,
            expected_max: 2,
            actual,
        },
    })
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
