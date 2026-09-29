use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{AggregateArgOrigin, AggregatePreparedItem, expand_aggregate_arg};
use crate::functions::aggregate_common::average_argument_value;
use crate::resolver::ReferenceSystemProvider;
use crate::value::{CalcValue, CoreValue};
use crate::value::WorksheetErrorCode;

pub const HARMEAN_META: FunctionMeta = function_spec! {
    function_id: "FUNC.HARMEAN",
    arity: Arity { min: 1, max: 255 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    coercion_lift_profile: CoercionLiftProfile::AggregateDirectAndRangeDualPolicy,
    kernel_signature_class: KernelSignatureClass::NumsToNum,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

#[derive(Debug, Clone, PartialEq)]
pub enum HarMeanEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

// W111 distinguishes blank cells from explicit omitted scalar arguments.
// Keep reference/array-origin policy in the existing aggregate helper.
fn prepared_number(arg: &AggregatePreparedItem) -> Result<Option<f64>, CoercionError> {
    match arg.0.core() {
        CoreValue::Empty => Ok(None),
        CoreValue::Missing if matches!(arg.1, AggregateArgOrigin::DirectScalar) => Ok(Some(0.0)),
        _ => average_argument_value(arg),
    }
}

fn eval_harmean_aggregate(args: &[AggregatePreparedItem]) -> Result<CalcValue, HarMeanEvalError> {
    // Coercion and explicit worksheet errors precede positivity validation,
    // including a later error after an earlier nonpositive numeric item.
    // Direct scalar coercion errors precede errors in arrays/references.
    // Validate only here: the full pass below keeps numeric accumulation order.
    for arg in args {
        if matches!(arg.1, AggregateArgOrigin::DirectScalar) {
            prepared_number(arg).map_err(HarMeanEvalError::Coercion)?;
        }
    }
    let mut values = Vec::new();
    for arg in args {
        if let Some(value) = prepared_number(arg).map_err(HarMeanEvalError::Coercion)? {
            values.push(value);
        }
    }
    let mut reciprocal_sum = 0.0;
    let mut count = 0usize;
    for value in values {
            if value <= 0.0 {
                return Ok(CalcValue::error(WorksheetErrorCode::Num));
            }
            reciprocal_sum = crate::excel_numeric::excel_x87_add(
                reciprocal_sum, crate::excel_numeric::excel_x87_recip(value));
            count += 1;
    }
    if count == 0 {
        return Ok(CalcValue::error(WorksheetErrorCode::NA));
    }
    if !reciprocal_sum.is_finite() {
        return Ok(CalcValue::error(WorksheetErrorCode::Num));
    }
    // W111 public observations select reciprocal(average(reciprocals));
    // reassociating this as count/sum changes observed output bits.
    let average = crate::excel_numeric::excel_x87_div(reciprocal_sum, count as f64);
    let result = crate::excel_numeric::excel_x87_recip(average);
    Ok(if result.is_finite() { CalcValue::number(result) }
       else { CalcValue::error(WorksheetErrorCode::Num) })
}

pub fn eval_harmean_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, HarMeanEvalError> {
    let argc = args.len();
    if !HARMEAN_META.arity.accepts(argc) {
        return Err(HarMeanEvalError::ArityMismatch {
            expected_min: HARMEAN_META.arity.min,
            expected_max: HARMEAN_META.arity.max,
            actual: argc,
        });
    }
    let mut prepared = Vec::new();
    for arg in args {
        prepared.extend(expand_aggregate_arg(arg, resolver).map_err(HarMeanEvalError::Coercion)?);
    }
    eval_harmean_aggregate(&prepared)
}

pub fn map_harmean_error_to_ws(e: &HarMeanEvalError) -> WorksheetErrorCode {
    match e {
        HarMeanEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        HarMeanEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        HarMeanEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}
