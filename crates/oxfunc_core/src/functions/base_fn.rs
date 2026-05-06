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
use crate::value::{
    ArrayCellValue, CallArgValue, EvalArray, EvalValue, ExcelText, WorksheetErrorCode,
};

pub const BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.BASE",
    arity: Arity { min: 2, max: 3 },
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
pub enum BaseEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

fn int_to_base(mut value: i64, radix: i64) -> String {
    const DIGITS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    if value == 0 {
        return "0".to_string();
    }
    let mut out = Vec::new();
    while value > 0 {
        let digit = (value % radix) as usize;
        out.push(DIGITS[digit] as char);
        value /= radix;
    }
    out.into_iter().rev().collect()
}

pub fn base_kernel(
    number: f64,
    radix: f64,
    min_length: Option<f64>,
) -> Result<ExcelText, WorksheetErrorCode> {
    let number = number.trunc();
    let radix = radix.trunc();
    let min_length = min_length.unwrap_or(0.0).trunc();
    if number < 0.0 || !(2.0..=36.0).contains(&radix) || min_length < 0.0 {
        return Err(WorksheetErrorCode::Num);
    }
    let mut out = int_to_base(number as i64, radix as i64);
    while out.len() < min_length as usize {
        out.insert(0, '0');
    }
    Ok(ExcelText::from_utf16_code_units(
        out.encode_utf16().collect(),
    ))
}

pub fn eval_base_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, BaseEvalError> {
    let prepared = prepare_args_values_only(args, resolver).map_err(BaseEvalError::Coercion)?;
    if !BASE_META.arity.accepts(prepared.len()) {
        return Err(BaseEvalError::ArityMismatch {
            expected_min: BASE_META.arity.min,
            expected_max: BASE_META.arity.max,
            actual: prepared.len(),
        });
    }
    if let Some((shape, cells)) = expand_prepared_broadcast_grid(&prepared) {
        let mapped = cells
            .into_iter()
            .map(|cell| match cell {
                BroadcastPreparedGroup::Values(values) => map_base_item(&values),
                BroadcastPreparedGroup::MissingCoordinate => {
                    ArrayCellValue::Error(WorksheetErrorCode::NA)
                }
            })
            .collect();
        return Ok(EvalValue::Array(
            EvalArray::new(shape, mapped).expect("shape preserved"),
        ));
    }
    let number = coerce_prepared_to_number(&prepared[0]).map_err(BaseEvalError::Coercion)?;
    let radix = coerce_prepared_to_number(&prepared[1]).map_err(BaseEvalError::Coercion)?;
    let min_length = if prepared.len() > 2 {
        Some(coerce_prepared_to_number(&prepared[2]).map_err(BaseEvalError::Coercion)?)
    } else {
        None
    };
    base_kernel(number, radix, min_length)
        .map(EvalValue::Text)
        .map_err(BaseEvalError::Domain)
}

fn map_base_item(args: &[PreparedArgValue]) -> ArrayCellValue {
    let number = match coerce_prepared_to_number(&args[0]) {
        Ok(value) => value,
        Err(CoercionError::WorksheetError(code)) => return ArrayCellValue::Error(code),
        Err(_) => return ArrayCellValue::Error(WorksheetErrorCode::Value),
    };
    let radix = match coerce_prepared_to_number(&args[1]) {
        Ok(value) => value,
        Err(CoercionError::WorksheetError(code)) => return ArrayCellValue::Error(code),
        Err(_) => return ArrayCellValue::Error(WorksheetErrorCode::Value),
    };
    let min_length = if args.len() > 2 {
        match coerce_prepared_to_number(&args[2]) {
            Ok(value) => Some(value),
            Err(CoercionError::WorksheetError(code)) => return ArrayCellValue::Error(code),
            Err(_) => return ArrayCellValue::Error(WorksheetErrorCode::Value),
        }
    } else {
        None
    };

    match base_kernel(number, radix, min_length) {
        Ok(text) => ArrayCellValue::Text(text),
        Err(code) => ArrayCellValue::Error(code),
    }
}

pub fn map_base_error_to_ws(e: &BaseEvalError) -> WorksheetErrorCode {
    match e {
        BaseEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        BaseEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        BaseEvalError::Coercion(_) => WorksheetErrorCode::Value,
        BaseEvalError::Domain(code) => *code,
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
    fn base_kernel_matches_excel_seed_rows() {
        let plain = base_kernel(31.0, 16.0, None).unwrap();
        assert_eq!(String::from_utf16_lossy(plain.utf16_code_units()), "1F");
        let padded = base_kernel(31.9, 16.0, Some(4.8)).unwrap();
        assert_eq!(String::from_utf16_lossy(padded.utf16_code_units()), "001F");
        assert_eq!(base_kernel(-1.0, 16.0, None), Err(WorksheetErrorCode::Num));
        assert_eq!(base_kernel(31.0, 1.0, None), Err(WorksheetErrorCode::Num));
    }

    #[test]
    fn eval_base_spills_array_arguments() {
        let got = eval_base_surface(
            &[
                CallArgValue::Eval(EvalValue::Array(
                    EvalArray::from_rows(vec![vec![
                        ArrayCellValue::Number(15.0),
                        ArrayCellValue::Number(16.0),
                    ]])
                    .unwrap(),
                )),
                CallArgValue::Eval(EvalValue::Number(16.0)),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Text(ExcelText::from_interop_assignment("F")),
                    ArrayCellValue::Text(ExcelText::from_interop_assignment("10")),
                ]])
                .unwrap()
            ))
        );
    }
}
