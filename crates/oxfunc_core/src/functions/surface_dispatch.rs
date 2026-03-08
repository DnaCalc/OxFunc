use crate::coercion::CoercionError;
use crate::functions::abs::{AbsEvalError, abs_kernel};
use crate::functions::abs_surface::eval_abs_scalar_value;
use crate::functions::pi::eval_pi;
use crate::resolver::RefResolutionError;
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalError, EvalValue, Value, WorksheetErrorCode};

pub const FUNC_ID_ABS: &str = "FUNC.ABS";
pub const FUNC_ID_PI: &str = "FUNC.PI";

fn map_ref_resolution_to_ws(e: &RefResolutionError) -> WorksheetErrorCode {
    match e {
        RefResolutionError::CapabilityDenied { .. } => WorksheetErrorCode::Ref,
        RefResolutionError::UnresolvedReference { .. } => WorksheetErrorCode::Ref,
        RefResolutionError::EvalTimeDerefNotAllowed => WorksheetErrorCode::Ref,
        RefResolutionError::ProviderFailure { .. } => WorksheetErrorCode::Value,
    }
}

fn map_coercion_to_ws(e: &CoercionError) -> WorksheetErrorCode {
    match e {
        CoercionError::WorksheetError(code) => *code,
        CoercionError::RefResolution(err) => map_ref_resolution_to_ws(err),
        CoercionError::MissingArg => WorksheetErrorCode::Value,
        CoercionError::EmptyCell => WorksheetErrorCode::Value,
        CoercionError::NonNumericText(_) => WorksheetErrorCode::Value,
        CoercionError::UnsupportedValueKind(_) => WorksheetErrorCode::Value,
    }
}

fn map_abs_error_to_ws(e: &AbsEvalError) -> WorksheetErrorCode {
    match e {
        AbsEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        AbsEvalError::Coercion(err) => map_coercion_to_ws(err),
    }
}

fn map_eval_error_to_ws(e: &EvalError) -> WorksheetErrorCode {
    match e {
        EvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
    }
}

pub fn eval_surface_unary_scalar_value(
    function_id: &str,
    arg: &CallArgValue,
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, WorksheetErrorCode> {
    match function_id {
        FUNC_ID_ABS => {
            let args = [arg.clone()];
            eval_abs_scalar_value(&args, resolver).map_err(|e| map_abs_error_to_ws(&e))
        }
        _ => Err(WorksheetErrorCode::Value),
    }
}

pub fn eval_surface_q_unary_number(
    function_id: &str,
    value: f64,
) -> Result<f64, WorksheetErrorCode> {
    match function_id {
        FUNC_ID_ABS => Ok(abs_kernel(value)),
        _ => Err(WorksheetErrorCode::Value),
    }
}

pub fn eval_surface_q_nullary_number(function_id: &str) -> Result<f64, WorksheetErrorCode> {
    match function_id {
        FUNC_ID_PI => match eval_pi(&[]) {
            Ok(Value::Number(n)) => Ok(n),
            Ok(_) => Err(WorksheetErrorCode::Value),
            Err(e) => Err(map_eval_error_to_ws(&e)),
        },
        _ => Err(WorksheetErrorCode::Value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ExcelText, ReferenceLike};

    struct NoReferenceResolver;

    impl ReferenceResolver for NoReferenceResolver {
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
    fn eval_surface_unary_scalar_value_abs_accepts_text_numeric() {
        let arg = CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
            " -2 ".encode_utf16().collect(),
        )));
        let got = eval_surface_unary_scalar_value(FUNC_ID_ABS, &arg, &NoReferenceResolver);
        assert_eq!(got, Ok(EvalValue::Number(2.0)));
    }

    #[test]
    fn eval_surface_unary_scalar_value_rejects_unknown_id() {
        let arg = CallArgValue::Eval(EvalValue::Number(1.0));
        let got = eval_surface_unary_scalar_value("FUNC.UNKNOWN", &arg, &NoReferenceResolver);
        assert_eq!(got, Err(WorksheetErrorCode::Value));
    }

    #[test]
    fn eval_surface_q_unary_number_abs_calls_kernel() {
        let got = eval_surface_q_unary_number(FUNC_ID_ABS, -3.0);
        assert_eq!(got, Ok(3.0));
    }

    #[test]
    fn eval_surface_q_nullary_number_pi_returns_constant() {
        let got = eval_surface_q_nullary_number(FUNC_ID_PI);
        assert_eq!(got, Ok(std::f64::consts::PI));
    }
}
