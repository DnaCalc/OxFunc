use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::binary_numeric::{BinaryNumericSurfaceError, eval_binary_numeric_surface};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

pub const ROUNDDOWN_META: FunctionMeta = function_spec! {
    function_id: "FUNC.ROUNDDOWN",
    arity: Arity::exact(2),
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
pub enum RoundDownEvalError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

impl From<BinaryNumericSurfaceError> for RoundDownEvalError {
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

pub fn rounddown_kernel(n: f64, digits: i32) -> f64 {
    if digits >= 308 {
        return n;
    }
    if digits <= -308 {
        return if n.is_sign_negative() { -0.0 } else { 0.0 };
    }

    if digits >= 0 {
        let factor = 10f64.powi(digits);
        (n * factor).trunc() / factor
    } else {
        let factor = 10f64.powi(-digits);
        (n / factor).trunc() * factor
    }
}

pub fn eval_rounddown_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, RoundDownEvalError> {
    eval_binary_numeric_surface(args, resolver, |value, digits| {
        Ok(rounddown_kernel(value, digits.trunc() as i32))
    })
    .map_err(RoundDownEvalError::from)
}

pub fn map_rounddown_error_to_ws(e: &RoundDownEvalError) -> WorksheetErrorCode {
    match e {
        RoundDownEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        RoundDownEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        RoundDownEvalError::Coercion(_) => WorksheetErrorCode::Value,
        RoundDownEvalError::Domain(code) => *code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounddown_kernel_matches_excel_probe() {
        assert_eq!(rounddown_kernel(3.14159, 3), 3.141);
        assert_eq!(rounddown_kernel(-3.14159, 3), -3.141);
        assert_eq!(rounddown_kernel(314.159, -2), 300.0);
    }
}
