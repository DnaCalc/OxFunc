use crate::coercion::{CoercionError, coerce_eval_to_number};
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, ErrorCollapseProfile,
    FecDependencyProfile, FunctionMeta, HostInteractionClass, KernelSignatureClass,
    ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::prepare_arg_values_only;
use crate::resolver::{ReferenceSystemProvider, resolve_eval_value};
use crate::value::{ArrayShape, CalcArray, WorksheetErrorCode};
use crate::value::{CalcValue, CoreValue};

pub const IF_META: FunctionMeta = function_spec! {
    function_id: "FUNC.IF",
    arity: Arity { min: 2, max: 3 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    error_collapse_profile: ErrorCollapseProfile::SelectorBranch,
};

#[derive(Debug, Clone, PartialEq)]
pub enum IfEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    ConditionCoercion(CoercionError),
    BranchPreparation(CoercionError),
}

fn prepared_to_eval_value(prepared: CalcValue) -> CalcValue {
    match prepared.core() {
        CoreValue::Missing => CalcValue::logical(false),
        CoreValue::Empty => CalcValue::number(0.0),
        _ => prepared,
    }
}

fn eval_condition_cell(cell: &CalcValue) -> Result<bool, CoercionError> {
    match cell.core() {
        CoreValue::Logical(b) => Ok(*b),
        CoreValue::Number(n) => Ok(*n != 0.0),
        CoreValue::Error(code) => Err(CoercionError::WorksheetError(*code)),
        CoreValue::Text(text) => Err(CoercionError::NonNumericText(text.to_string_lossy())),
        CoreValue::Empty | CoreValue::Missing => Ok(false),
        CoreValue::Array(_) | CoreValue::Reference(_) => {
            Err(CoercionError::UnsupportedValueKind("if_condition_cell"))
        }
    }
}

fn scalar_cell_from_eval_value(value: &CalcValue) -> Result<CalcValue, CoercionError> {
    match value.core() {
        CoreValue::Number(n) => Ok(CalcValue::number(*n)),
        CoreValue::Text(t) => Ok(CalcValue::text(t.clone())),
        CoreValue::Logical(b) => Ok(CalcValue::logical(*b)),
        CoreValue::Error(code) => Ok(CalcValue::error(*code)),
        CoreValue::Array(_) | CoreValue::Reference(_) => {
            Err(CoercionError::UnsupportedValueKind("if_branch_scalar"))
        }
        CoreValue::Empty | CoreValue::Missing => {
            Err(CoercionError::UnsupportedValueKind("if_branch_scalar"))
        }
    }
}

fn materialize_branch_for_shape(
    value: &CalcValue,
    shape: ArrayShape,
) -> Result<CalcArray, CoercionError> {
    match value.core() {
        CoreValue::Array(array) if array.shape() == shape => Ok(array.clone()),
        CoreValue::Array(_) => Err(CoercionError::UnsupportedValueKind("if_branch_shape")),
        _ => {
            let cell = scalar_cell_from_eval_value(value)?;
            CalcArray::new(shape, vec![cell; shape.rows * shape.cols])
                .ok_or(CoercionError::UnsupportedValueKind("if_branch_shape"))
        }
    }
}

fn eval_condition_bool(
    arg: &CalcValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<bool, CoercionError> {
    match arg.core() {
        CoreValue::Missing | CoreValue::Empty => Ok(false),
        CoreValue::Reference(r) => {
            let resolved = resolve_eval_value(resolver, r).map_err(CoercionError::RefResolution)?;
            eval_condition_bool(&(resolved), resolver)
        }
        CoreValue::Logical(b) => Ok(*b),
        CoreValue::Number(n) => Ok(*n != 0.0),
        _ => {
            let n = coerce_eval_to_number(arg, resolver)?;
            Ok(n != 0.0)
        }
    }
}

fn eval_if_elementwise_surface(
    condition: &CalcArray,
    true_arg: &CalcValue,
    false_arg: Option<&CalcValue>,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, IfEvalError> {
    let true_value = prepared_to_eval_value(
        prepare_arg_values_only(true_arg, resolver).map_err(IfEvalError::BranchPreparation)?,
    );
    let false_value = match false_arg {
        Some(arg) => prepared_to_eval_value(
            prepare_arg_values_only(arg, resolver).map_err(IfEvalError::BranchPreparation)?,
        ),
        None => CalcValue::logical(false),
    };
    let true_array = materialize_branch_for_shape(&true_value, condition.shape())
        .map_err(IfEvalError::BranchPreparation)?;
    let false_array = materialize_branch_for_shape(&false_value, condition.shape())
        .map_err(IfEvalError::BranchPreparation)?;

    let mut cells = Vec::with_capacity(condition.shape().rows * condition.shape().cols);
    for row in 0..condition.shape().rows {
        for col in 0..condition.shape().cols {
            let keep = eval_condition_cell(
                condition
                    .get(row, col)
                    .expect("validated IF condition shape"),
            )
            .map_err(IfEvalError::ConditionCoercion)?;
            let chosen = if keep {
                true_array.get(row, col).expect("validated IF true shape")
            } else {
                false_array.get(row, col).expect("validated IF false shape")
            };
            cells.push(chosen.clone());
        }
    }

    Ok(CalcValue::array(
        CalcArray::new(condition.shape(), cells).expect("validated IF result shape"),
    ))
}

pub fn eval_if_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, IfEvalError> {
    let argc = args.len();
    if !IF_META.arity.accepts(argc) {
        return Err(IfEvalError::ArityMismatch {
            expected_min: IF_META.arity.min,
            expected_max: IF_META.arity.max,
            actual: argc,
        });
    }

    if let CoreValue::Array(condition) = args[0].core() {
        return eval_if_elementwise_surface(condition, &args[1], args.get(2), resolver);
    }

    let cond = eval_condition_bool(&args[0], resolver).map_err(IfEvalError::ConditionCoercion)?;

    let branch_arg = if cond {
        &args[1]
    } else if argc >= 3 {
        &args[2]
    } else {
        return Ok(CalcValue::logical(false));
    };

    let prepared =
        prepare_arg_values_only(branch_arg, resolver).map_err(IfEvalError::BranchPreparation)?;
    Ok(prepared_to_eval_value(prepared))
}

pub fn map_if_error_to_ws(e: &IfEvalError) -> WorksheetErrorCode {
    match e {
        IfEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        IfEvalError::ConditionCoercion(CoercionError::WorksheetError(code)) => *code,
        IfEvalError::BranchPreparation(CoercionError::WorksheetError(code)) => *code,
        _ => WorksheetErrorCode::Value,
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
    fn eval_if_true_branch_only() {
        let args = vec![
            (CalcValue::logical(true)),
            (CalcValue::number(1.0)),
            (CalcValue::error(WorksheetErrorCode::Div0)),
        ];
        let got = eval_if_surface(&args, &NoResolver);
        assert_eq!(got, Ok(CalcValue::number(1.0)));
    }

    #[test]
    fn eval_if_false_branch_only() {
        let args = vec![
            (CalcValue::logical(false)),
            (CalcValue::error(WorksheetErrorCode::Div0)),
            (CalcValue::number(2.0)),
        ];
        let got = eval_if_surface(&args, &NoResolver);
        assert_eq!(got, Ok(CalcValue::number(2.0)));
    }

    #[test]
    fn eval_if_missing_false_branch_defaults_false() {
        let args = vec![(CalcValue::logical(false)), (CalcValue::number(1.0))];
        let got = eval_if_surface(&args, &NoResolver);
        assert_eq!(got, Ok(CalcValue::logical(false)));
    }

    #[test]
    fn eval_if_empty_text_condition_returns_value_error() {
        let args = vec![
            (CalcValue::text(crate::value::ExcelText::from_interop_assignment(""))),
            (CalcValue::number(1.0)),
            (CalcValue::number(2.0)),
        ];
        let got = eval_if_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Err(IfEvalError::ConditionCoercion(
                CoercionError::NonNumericText("".to_string())
            ))
        );
    }

    #[test]
    fn eval_if_lifts_array_condition_elementwise() {
        let args = vec![
            (CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::logical(true),
                    CalcValue::logical(false),
                    CalcValue::logical(true),
                ]])
                .unwrap(),
            )),
            (CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(1.0),
                    CalcValue::number(2.0),
                    CalcValue::number(3.0),
                ]])
                .unwrap(),
            )),
            (CalcValue::number(0.0)),
        ];
        let got = eval_if_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(1.0),
                    CalcValue::number(0.0),
                    CalcValue::number(3.0),
                ]])
                .unwrap()
            ))
        );
    }
}
