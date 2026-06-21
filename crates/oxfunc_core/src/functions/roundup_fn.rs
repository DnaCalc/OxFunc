use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::binary_numeric::{BinaryNumericSurfaceError, eval_binary_numeric_surface};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const ROUNDUP_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ROUNDUP",
    arity: Arity::exact(2),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    lift_broadcast_profile: FunctionMeta::DEFAULT_LIFT_BROADCAST_PROFILE,
    kernel_signature_class: KernelSignatureClass::NumsToNum,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
};

#[derive(Debug, Clone, PartialEq)]
pub enum RoundUpEvalError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

impl From<BinaryNumericSurfaceError> for RoundUpEvalError {
    fn from(value: BinaryNumericSurfaceError) -> Self {
        match value {
            BinaryNumericSurfaceError::ArityMismatch { expected, actual } => {
                Self::ArityMismatch { expected, actual }
            }
            BinaryNumericSurfaceError::Coercion(error) => Self::Coercion(error),
            BinaryNumericSurfaceError::Domain(code) => Self::Domain(code),
        }
    }
}

pub fn roundup_kernel(n: f64, digits: i32) -> f64 {
    if n == 0.0 {
        return 0.0;
    }
    if digits >= 308 {
        return n;
    }
    if digits <= -308 {
        return if n.is_sign_negative() { -0.0 } else { 0.0 };
    }

    let sign = if n.is_sign_negative() { -1.0 } else { 1.0 };
    let abs_n = n.abs();
    if digits >= 0 {
        let factor = 10f64.powi(digits);
        sign * ((abs_n * factor).ceil() / factor)
    } else {
        let factor = 10f64.powi(-digits);
        sign * ((abs_n / factor).ceil() * factor)
    }
}

pub fn eval_roundup_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, RoundUpEvalError> {
    eval_binary_numeric_surface(args, resolver, |value, digits| {
        Ok(roundup_kernel(value, digits.trunc() as i32))
    })
    .map_err(RoundUpEvalError::from)
}

pub fn map_roundup_error_to_ws(e: &RoundUpEvalError) -> WorksheetErrorCode {
    match e {
        RoundUpEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        RoundUpEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        RoundUpEvalError::Coercion(_) => WorksheetErrorCode::Value,
        RoundUpEvalError::Domain(code) => *code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundup_kernel_matches_excel_probe() {
        assert_eq!(roundup_kernel(3.14159, 3), 3.142);
        assert_eq!(roundup_kernel(-3.14159, 3), -3.142);
        assert_eq!(roundup_kernel(314.159, -2), 400.0);
    }
}
