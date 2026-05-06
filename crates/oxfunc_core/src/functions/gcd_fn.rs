use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedArgValue, expand_aggregate_arg};
use crate::functions::factorial_common::trunc_nonnegative;
use crate::functions::gcd_lcm_common::gcd_int;
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const GCD_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.GCD",
    arity: Arity { min: 1, max: 255 },
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
pub enum GcdEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

fn coerce_prepared_to_nonnegative_int(arg: &PreparedArgValue) -> Result<i64, GcdEvalError> {
    let n = crate::functions::adapters::coerce_prepared_to_number(arg)
        .map_err(GcdEvalError::Coercion)?;
    trunc_nonnegative(n).map_err(GcdEvalError::Domain)
}

pub fn gcd_kernel(items: &[i64]) -> f64 {
    items.iter().copied().fold(0, gcd_int) as f64
}

pub fn eval_gcd_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, GcdEvalError> {
    let argc = args.len();
    if !GCD_META.arity.accepts(argc) {
        return Err(GcdEvalError::ArityMismatch {
            expected_min: GCD_META.arity.min,
            expected_max: GCD_META.arity.max,
            actual: argc,
        });
    }
    let mut items = Vec::new();
    for arg in args {
        let expanded = expand_aggregate_arg(arg, resolver).map_err(GcdEvalError::Coercion)?;
        for item in expanded {
            items.push(coerce_prepared_to_nonnegative_int(&item.value)?);
        }
    }
    Ok(EvalValue::Number(gcd_kernel(&items)))
}

pub fn map_gcd_error_to_ws(e: &GcdEvalError) -> WorksheetErrorCode {
    match e {
        GcdEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        GcdEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        GcdEvalError::Coercion(_) => WorksheetErrorCode::Value,
        GcdEvalError::Domain(code) => *code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ReferenceResolver, ResolverCapabilities};
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
    fn gcd_meta_function_id_is_stable() {
        assert_eq!(GCD_META.function_id, "FUNC.GCD");
    }

    #[test]
    fn gcd_kernel_matches_excel_seed_rows() {
        assert_eq!(gcd_kernel(&[24, 36]), 12.0);
        assert_eq!(gcd_kernel(&[0, 5]), 5.0);
        assert_eq!(gcd_kernel(&[0, 0]), 0.0);
    }

    #[test]
    fn ftc_0959_gcd_array_input_reduces_literal_vector_and_scalar_to_one() {
        let got = eval_gcd_surface(
            &[
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Number(2.0),
                        ArrayCellValue::Number(3.0),
                        ArrayCellValue::Number(4.0),
                        ArrayCellValue::Number(5.0),
                        ArrayCellValue::Number(6.0),
                        ArrayCellValue::Number(7.0),
                        ArrayCellValue::Number(8.0),
                        ArrayCellValue::Number(9.0),
                        ArrayCellValue::Number(10.0),
                        ArrayCellValue::Number(11.0),
                        ArrayCellValue::Number(12.0),
                    ]])
                    .unwrap(),
                )),
                CallArgValue::Eval(EvalValue::Number(12.0)),
            ],
            &NoResolver,
        );
        assert_eq!(got, Ok(EvalValue::Number(1.0)));
    }
}
