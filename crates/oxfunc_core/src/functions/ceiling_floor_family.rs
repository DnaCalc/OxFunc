use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    BroadcastPreparedGroup, coerce_prepared_to_number, expand_prepared_broadcast_grid,
    run_values_only_prepared,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::{CalcArray, WorksheetErrorCode};

const OPTIONAL_ARITY_2: Arity = Arity { min: 1, max: 2 };
const OPTIONAL_ARITY_3: Arity = Arity { min: 1, max: 3 };

pub const CEILING_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.CEILING",
    arity: Arity::exact(2),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
};

pub const CEILING_MATH_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.CEILING.MATH",
    arity: OPTIONAL_ARITY_3,
    ..CEILING_META
};

pub const CEILING_PRECISE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.CEILING.PRECISE",
    arity: OPTIONAL_ARITY_2,
    ..CEILING_META
};

pub const ISO_CEILING_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ISO.CEILING",
    arity: OPTIONAL_ARITY_2,
    ..CEILING_META
};

pub const FLOOR_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.FLOOR",
    ..CEILING_META
};

pub const FLOOR_MATH_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.FLOOR.MATH",
    arity: OPTIONAL_ARITY_3,
    ..CEILING_META
};

pub const FLOOR_PRECISE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.FLOOR.PRECISE",
    arity: OPTIONAL_ARITY_2,
    ..CEILING_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum CeilingFloorEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

fn normalize_zero(x: f64) -> f64 {
    if x == 0.0 { 0.0 } else { x }
}

pub fn ceiling_kernel(number: f64, significance: f64) -> Result<f64, WorksheetErrorCode> {
    if significance == 0.0 {
        return Ok(0.0);
    }
    if number > 0.0 && significance < 0.0 {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(normalize_zero(
        significance * (number / significance).ceil(),
    ))
}

pub fn floor_kernel(number: f64, significance: f64) -> Result<f64, WorksheetErrorCode> {
    if significance == 0.0 {
        return Err(WorksheetErrorCode::Div0);
    }
    if number > 0.0 && significance < 0.0 {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(normalize_zero(
        significance * (number / significance).floor(),
    ))
}

pub fn ceiling_math_kernel(
    number: f64,
    significance: f64,
    mode: f64,
) -> Result<f64, WorksheetErrorCode> {
    let significance = significance.abs();
    if significance == 0.0 {
        return Ok(0.0);
    }
    let result = if number >= 0.0 {
        significance * (number / significance).ceil()
    } else if mode != 0.0 {
        -significance * ((-number) / significance).ceil()
    } else {
        -significance * ((-number) / significance).floor()
    };
    Ok(normalize_zero(result))
}

pub fn floor_math_kernel(
    number: f64,
    significance: f64,
    mode: f64,
) -> Result<f64, WorksheetErrorCode> {
    let significance = significance.abs();
    if significance == 0.0 {
        return Ok(0.0);
    }
    let result = if number >= 0.0 {
        significance * (number / significance).floor()
    } else if mode != 0.0 {
        -significance * ((-number) / significance).floor()
    } else {
        -significance * ((-number) / significance).ceil()
    };
    Ok(normalize_zero(result))
}

pub fn ceiling_precise_kernel(number: f64, significance: f64) -> Result<f64, WorksheetErrorCode> {
    ceiling_math_kernel(number, significance, 0.0)
}

pub fn iso_ceiling_kernel(number: f64, significance: f64) -> Result<f64, WorksheetErrorCode> {
    ceiling_math_kernel(number, significance, 0.0)
}

pub fn floor_precise_kernel(number: f64, significance: f64) -> Result<f64, WorksheetErrorCode> {
    floor_math_kernel(number, significance, 0.0)
}

fn arity_error(meta: &FunctionMeta, actual: usize) -> CeilingFloorEvalError {
    CeilingFloorEvalError::ArityMismatch {
        expected_min: meta.arity.min,
        expected_max: meta.arity.max,
        actual,
    }
}

fn eval_prepared_exact2(
    meta: &FunctionMeta,
    args: &[CalcValue],
    kernel: fn(f64, f64) -> Result<f64, WorksheetErrorCode>,
) -> Result<CalcValue, CeilingFloorEvalError> {
    if !meta.arity.accepts(args.len()) {
        return Err(arity_error(meta, args.len()));
    }
    if let Some((shape, cells)) = expand_prepared_broadcast_grid(args) {
        let mapped = cells
            .into_iter()
            .map(|cell| match cell {
                BroadcastPreparedGroup::Values(values) => map_exact2_item(&values, kernel),
                BroadcastPreparedGroup::MissingCoordinate => {
                    CalcValue::error(WorksheetErrorCode::NA)
                }
            })
            .collect();
        return Ok(CalcValue::array(
            CalcArray::new(shape, mapped).expect("shape preserved"),
        ));
    }
    let number = coerce_prepared_to_number(&args[0]).map_err(CeilingFloorEvalError::Coercion)?;
    let significance =
        coerce_prepared_to_number(&args[1]).map_err(CeilingFloorEvalError::Coercion)?;
    match kernel(number, significance) {
        Ok(value) => Ok(CalcValue::number(value)),
        Err(code) => Ok(CalcValue::error(code)),
    }
}

fn eval_prepared_optional2(
    meta: &FunctionMeta,
    args: &[CalcValue],
    kernel: fn(f64, f64) -> Result<f64, WorksheetErrorCode>,
) -> Result<CalcValue, CeilingFloorEvalError> {
    if !meta.arity.accepts(args.len()) {
        return Err(arity_error(meta, args.len()));
    }
    if let Some((shape, cells)) = expand_prepared_broadcast_grid(args) {
        let mapped = cells
            .into_iter()
            .map(|cell| match cell {
                BroadcastPreparedGroup::Values(values) => map_optional2_item(&values, kernel),
                BroadcastPreparedGroup::MissingCoordinate => {
                    CalcValue::error(WorksheetErrorCode::NA)
                }
            })
            .collect();
        return Ok(CalcValue::array(
            CalcArray::new(shape, mapped).expect("shape preserved"),
        ));
    }
    let number = coerce_prepared_to_number(&args[0]).map_err(CeilingFloorEvalError::Coercion)?;
    let significance = if args.len() >= 2 {
        coerce_prepared_to_number(&args[1]).map_err(CeilingFloorEvalError::Coercion)?
    } else {
        1.0
    };
    match kernel(number, significance) {
        Ok(value) => Ok(CalcValue::number(value)),
        Err(code) => Ok(CalcValue::error(code)),
    }
}

fn eval_prepared_optional3(
    meta: &FunctionMeta,
    args: &[CalcValue],
    kernel: fn(f64, f64, f64) -> Result<f64, WorksheetErrorCode>,
) -> Result<CalcValue, CeilingFloorEvalError> {
    if !meta.arity.accepts(args.len()) {
        return Err(arity_error(meta, args.len()));
    }
    if let Some((shape, cells)) = expand_prepared_broadcast_grid(args) {
        let mapped = cells
            .into_iter()
            .map(|cell| match cell {
                BroadcastPreparedGroup::Values(values) => map_optional3_item(&values, kernel),
                BroadcastPreparedGroup::MissingCoordinate => {
                    CalcValue::error(WorksheetErrorCode::NA)
                }
            })
            .collect();
        return Ok(CalcValue::array(
            CalcArray::new(shape, mapped).expect("shape preserved"),
        ));
    }
    let number = coerce_prepared_to_number(&args[0]).map_err(CeilingFloorEvalError::Coercion)?;
    let significance = if args.len() >= 2 {
        coerce_prepared_to_number(&args[1]).map_err(CeilingFloorEvalError::Coercion)?
    } else {
        1.0
    };
    let mode = if args.len() >= 3 {
        coerce_prepared_to_number(&args[2]).map_err(CeilingFloorEvalError::Coercion)?
    } else {
        0.0
    };
    match kernel(number, significance, mode) {
        Ok(value) => Ok(CalcValue::number(value)),
        Err(code) => Ok(CalcValue::error(code)),
    }
}

fn coerce_number_cell(arg: &CalcValue) -> Result<f64, CalcValue> {
    coerce_prepared_to_number(arg).map_err(|error| match error {
        CoercionError::WorksheetError(code) => CalcValue::error(code),
        _ => CalcValue::error(WorksheetErrorCode::Value),
    })
}

fn kernel_result_cell(result: Result<f64, WorksheetErrorCode>) -> CalcValue {
    match result {
        Ok(value) => CalcValue::number(value),
        Err(code) => CalcValue::error(code),
    }
}

fn map_exact2_item(
    args: &[CalcValue],
    kernel: fn(f64, f64) -> Result<f64, WorksheetErrorCode>,
) -> CalcValue {
    let number = match coerce_number_cell(&args[0]) {
        Ok(value) => value,
        Err(cell) => return cell,
    };
    let significance = match coerce_number_cell(&args[1]) {
        Ok(value) => value,
        Err(cell) => return cell,
    };
    kernel_result_cell(kernel(number, significance))
}

fn map_optional2_item(
    args: &[CalcValue],
    kernel: fn(f64, f64) -> Result<f64, WorksheetErrorCode>,
) -> CalcValue {
    let number = match coerce_number_cell(&args[0]) {
        Ok(value) => value,
        Err(cell) => return cell,
    };
    let significance = if args.len() >= 2 {
        match coerce_number_cell(&args[1]) {
            Ok(value) => value,
            Err(cell) => return cell,
        }
    } else {
        1.0
    };
    kernel_result_cell(kernel(number, significance))
}

fn map_optional3_item(
    args: &[CalcValue],
    kernel: fn(f64, f64, f64) -> Result<f64, WorksheetErrorCode>,
) -> CalcValue {
    let number = match coerce_number_cell(&args[0]) {
        Ok(value) => value,
        Err(cell) => return cell,
    };
    let significance = if args.len() >= 2 {
        match coerce_number_cell(&args[1]) {
            Ok(value) => value,
            Err(cell) => return cell,
        }
    } else {
        1.0
    };
    let mode = if args.len() >= 3 {
        match coerce_number_cell(&args[2]) {
            Ok(value) => value,
            Err(cell) => return cell,
        }
    } else {
        0.0
    };
    kernel_result_cell(kernel(number, significance, mode))
}

pub fn eval_ceiling_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, CeilingFloorEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_prepared_exact2(&CEILING_META, prepared, ceiling_kernel),
        CeilingFloorEvalError::Coercion,
    )
}

pub fn eval_floor_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, CeilingFloorEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_prepared_exact2(&FLOOR_META, prepared, floor_kernel),
        CeilingFloorEvalError::Coercion,
    )
}

pub fn eval_ceiling_math_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, CeilingFloorEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_prepared_optional3(&CEILING_MATH_META, prepared, ceiling_math_kernel),
        CeilingFloorEvalError::Coercion,
    )
}

pub fn eval_floor_math_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, CeilingFloorEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_prepared_optional3(&FLOOR_MATH_META, prepared, floor_math_kernel),
        CeilingFloorEvalError::Coercion,
    )
}

pub fn eval_ceiling_precise_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, CeilingFloorEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_prepared_optional2(&CEILING_PRECISE_META, prepared, ceiling_precise_kernel),
        CeilingFloorEvalError::Coercion,
    )
}

pub fn eval_floor_precise_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, CeilingFloorEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_prepared_optional2(&FLOOR_PRECISE_META, prepared, floor_precise_kernel),
        CeilingFloorEvalError::Coercion,
    )
}

pub fn eval_iso_ceiling_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, CeilingFloorEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_prepared_optional2(&ISO_CEILING_META, prepared, iso_ceiling_kernel),
        CeilingFloorEvalError::Coercion,
    )
}

pub fn map_ceiling_floor_error_to_ws(e: &CeilingFloorEvalError) -> WorksheetErrorCode {
    match e {
        CeilingFloorEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        CeilingFloorEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        CeilingFloorEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;

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
    fn ceiling_legacy_seed_lanes_match_excel_probe() {
        assert_eq!(ceiling_kernel(4.3, 2.0), Ok(6.0));
        assert_eq!(ceiling_kernel(-4.3, 2.0), Ok(-4.0));
        assert_eq!(ceiling_kernel(-4.3, -2.0), Ok(-6.0));
        assert_eq!(ceiling_kernel(4.3, -2.0), Err(WorksheetErrorCode::Num));
        assert_eq!(ceiling_kernel(5.0, 0.0), Ok(0.0));
    }

    #[test]
    fn floor_legacy_seed_lanes_match_excel_probe() {
        assert_eq!(floor_kernel(4.3, 2.0), Ok(4.0));
        assert_eq!(floor_kernel(-4.3, 2.0), Ok(-6.0));
        assert_eq!(floor_kernel(-4.3, -2.0), Ok(-4.0));
        assert_eq!(floor_kernel(4.3, -2.0), Err(WorksheetErrorCode::Num));
        assert_eq!(floor_kernel(5.0, 0.0), Err(WorksheetErrorCode::Div0));
    }

    #[test]
    fn math_and_precise_seed_lanes_match_excel_probe() {
        assert_eq!(ceiling_math_kernel(-4.3, 2.0, 0.0), Ok(-4.0));
        assert_eq!(ceiling_math_kernel(-4.3, 2.0, 1.0), Ok(-6.0));
        assert_eq!(ceiling_math_kernel(4.3, -2.0, 0.0), Ok(6.0));
        assert_eq!(ceiling_math_kernel(5.0, 0.0, 0.0), Ok(0.0));
        assert_eq!(floor_math_kernel(-4.3, 2.0, 0.0), Ok(-6.0));
        assert_eq!(floor_math_kernel(-4.3, 2.0, 1.0), Ok(-4.0));
        assert_eq!(floor_math_kernel(4.3, -2.0, 0.0), Ok(4.0));
        assert_eq!(floor_math_kernel(5.0, 0.0, 0.0), Ok(0.0));
        assert_eq!(ceiling_precise_kernel(-4.3, 2.0), Ok(-4.0));
        assert_eq!(floor_precise_kernel(-4.3, 2.0), Ok(-6.0));
        assert_eq!(iso_ceiling_kernel(-4.3, 2.0), Ok(-4.0));
    }

    #[test]
    fn eval_ceiling_math_spills_optional_array_argument() {
        let got = eval_ceiling_math_surface(
            &[
                (CalcValue::number(-1.2)),
                (CalcValue::number(1.0)),
                (CalcValue::array(
                    CalcArray::from_rows(vec![vec![
                        CalcValue::number(0.0),
                        CalcValue::number(1.0),
                    ]])
                    .unwrap(),
                )),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(-1.0), CalcValue::number(-2.0),]
                ])
                .unwrap()
            ))
        );
    }
}
