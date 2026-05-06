use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedArgValue, prepare_arg_values_only};
use crate::functions::average::{eval_average_surface, map_average_error_to_ws};
use crate::functions::count::{eval_count_surface, map_count_error_to_ws};
use crate::functions::counta::{eval_counta_surface, map_counta_error_to_ws};
use crate::functions::large_fn::{eval_large_surface, map_large_error_to_ws};
use crate::functions::max_fn::{eval_max_surface, map_max_error_to_ws};
use crate::functions::median_fn::{eval_median_surface, map_median_error_to_ws};
use crate::functions::min_fn::{eval_min_surface, map_min_error_to_ws};
use crate::functions::mode_sngl_fn::{eval_mode_sngl_surface, map_mode_sngl_error_to_ws};
use crate::functions::percentile_exc_fn::{
    eval_percentile_exc_surface, map_percentile_exc_error_to_ws,
};
use crate::functions::percentile_inc_fn::{
    eval_percentile_inc_surface, map_percentile_inc_error_to_ws,
};
use crate::functions::product::{eval_product_surface, map_product_error_to_ws};
use crate::functions::quartile_exc_fn::{eval_quartile_exc_surface, map_quartile_exc_error_to_ws};
use crate::functions::quartile_inc_fn::{eval_quartile_inc_surface, map_quartile_inc_error_to_ws};
use crate::functions::small_fn::{eval_small_surface, map_small_error_to_ws};
use crate::functions::stdev_p_fn::{eval_stdev_p_surface, map_stdev_p_error_to_ws};
use crate::functions::stdev_s_fn::{eval_stdev_s_surface, map_stdev_s_error_to_ws};
use crate::functions::sum::{eval_sum_surface, map_sum_error_to_ws};
use crate::functions::var_p_fn::{eval_var_p_surface, map_var_p_error_to_ws};
use crate::functions::var_s_fn::{eval_var_s_surface, map_var_s_error_to_ws};
use crate::host_info::{AggregateReferenceContext, HostInfoError, HostInfoProvider};
use crate::resolver::{
    RefResolutionError, ReferenceResolver, ResolverCapabilities, resolve_eval_value,
};
use crate::value::{
    ArrayCellValue, ArrayShape, CallArgValue, EvalArray, EvalValue, ReferenceLike,
    WorksheetErrorCode,
};

pub const SUBTOTAL_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SUBTOTAL",
    arity: Arity { min: 2, max: 255 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::WorkbookState,
    thread_safety: ThreadSafetyClass::HostSerialized,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::Composite,
    surface_fec_dependency_profile: FecDependencyProfile::Composite,
};

pub const AGGREGATE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.AGGREGATE",
    arity: Arity { min: 3, max: 255 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::WorkbookState,
    thread_safety: ThreadSafetyClass::HostSerialized,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::Composite,
    surface_fec_dependency_profile: FecDependencyProfile::Composite,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AggregateOperation {
    Average,
    Count,
    CountA,
    Max,
    Min,
    Product,
    StdevS,
    StdevP,
    Sum,
    VarS,
    VarP,
    Median,
    ModeSngl,
    Large,
    Small,
    PercentileInc,
    QuartileInc,
    PercentileExc,
    QuartileExc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AggregateFilterRules {
    ignore_nested_aggregate: bool,
    ignore_manual_hidden_rows: bool,
    ignore_filtered_rows: bool,
    ignore_errors: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SubtotalAggregateEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    HostInfoProviderMissing(&'static str),
    HostInfo(HostInfoError),
    InvalidFunctionNum,
    InvalidOptions,
    InvalidK,
    InvalidAggregateSyntax,
    AggregateContextShapeMismatch,
    DelegateWorksheetError(WorksheetErrorCode),
}

struct PreparedOnlyResolver;

impl ReferenceResolver for PreparedOnlyResolver {
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

fn coerce_scalar_number(
    arg: &CallArgValue,
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<f64, SubtotalAggregateEvalError> {
    let prepared =
        prepare_arg_values_only(arg, resolver).map_err(SubtotalAggregateEvalError::Coercion)?;
    match prepared {
        PreparedArgValue::Eval(EvalValue::Number(n)) => Ok(n),
        PreparedArgValue::Eval(value) => {
            crate::coercion::coerce_eval_to_number(&value, &PreparedOnlyResolver)
                .map_err(SubtotalAggregateEvalError::Coercion)
        }
        PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell => Err(
            SubtotalAggregateEvalError::Coercion(CoercionError::EmptyCell),
        ),
    }
}

fn coerce_whole_number(
    arg: &CallArgValue,
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<i32, SubtotalAggregateEvalError> {
    let value = coerce_scalar_number(arg, resolver)?;
    let truncated = value.trunc();
    if !truncated.is_finite() || truncated != value {
        return Err(SubtotalAggregateEvalError::InvalidFunctionNum);
    }
    Ok(truncated as i32)
}

fn coerce_k_arg(
    arg: &CallArgValue,
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<CallArgValue, SubtotalAggregateEvalError> {
    let prepared =
        prepare_arg_values_only(arg, resolver).map_err(SubtotalAggregateEvalError::Coercion)?;
    match prepared {
        PreparedArgValue::Eval(value) => Ok(CallArgValue::Eval(value)),
        PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell => {
            Err(SubtotalAggregateEvalError::InvalidK)
        }
    }
}

fn subtotal_rules(
    function_num: i32,
) -> Result<(AggregateOperation, AggregateFilterRules), SubtotalAggregateEvalError> {
    let ignore_manual_hidden_rows = match function_num {
        1..=11 => false,
        101..=111 => true,
        _ => return Err(SubtotalAggregateEvalError::InvalidFunctionNum),
    };
    let normalized = if function_num >= 100 {
        function_num - 100
    } else {
        function_num
    };
    let operation = match normalized {
        1 => AggregateOperation::Average,
        2 => AggregateOperation::Count,
        3 => AggregateOperation::CountA,
        4 => AggregateOperation::Max,
        5 => AggregateOperation::Min,
        6 => AggregateOperation::Product,
        7 => AggregateOperation::StdevS,
        8 => AggregateOperation::StdevP,
        9 => AggregateOperation::Sum,
        10 => AggregateOperation::VarS,
        11 => AggregateOperation::VarP,
        _ => return Err(SubtotalAggregateEvalError::InvalidFunctionNum),
    };
    Ok((
        operation,
        AggregateFilterRules {
            ignore_nested_aggregate: true,
            ignore_manual_hidden_rows,
            ignore_filtered_rows: true,
            ignore_errors: false,
        },
    ))
}

fn aggregate_rules(
    function_num: i32,
    options: i32,
) -> Result<(AggregateOperation, AggregateFilterRules), SubtotalAggregateEvalError> {
    let operation = match function_num {
        1 => AggregateOperation::Average,
        2 => AggregateOperation::Count,
        3 => AggregateOperation::CountA,
        4 => AggregateOperation::Max,
        5 => AggregateOperation::Min,
        6 => AggregateOperation::Product,
        7 => AggregateOperation::StdevS,
        8 => AggregateOperation::StdevP,
        9 => AggregateOperation::Sum,
        10 => AggregateOperation::VarS,
        11 => AggregateOperation::VarP,
        12 => AggregateOperation::Median,
        13 => AggregateOperation::ModeSngl,
        14 => AggregateOperation::Large,
        15 => AggregateOperation::Small,
        16 => AggregateOperation::PercentileInc,
        17 => AggregateOperation::QuartileInc,
        18 => AggregateOperation::PercentileExc,
        19 => AggregateOperation::QuartileExc,
        _ => return Err(SubtotalAggregateEvalError::InvalidFunctionNum),
    };
    let rules = match options {
        0 => AggregateFilterRules {
            ignore_nested_aggregate: true,
            ignore_manual_hidden_rows: false,
            ignore_filtered_rows: false,
            ignore_errors: false,
        },
        1 => AggregateFilterRules {
            ignore_nested_aggregate: true,
            ignore_manual_hidden_rows: true,
            ignore_filtered_rows: true,
            ignore_errors: false,
        },
        2 => AggregateFilterRules {
            ignore_nested_aggregate: true,
            ignore_manual_hidden_rows: false,
            ignore_filtered_rows: false,
            ignore_errors: true,
        },
        3 => AggregateFilterRules {
            ignore_nested_aggregate: true,
            ignore_manual_hidden_rows: true,
            ignore_filtered_rows: true,
            ignore_errors: true,
        },
        4 => AggregateFilterRules {
            ignore_nested_aggregate: false,
            ignore_manual_hidden_rows: false,
            ignore_filtered_rows: false,
            ignore_errors: false,
        },
        5 => AggregateFilterRules {
            ignore_nested_aggregate: false,
            ignore_manual_hidden_rows: true,
            ignore_filtered_rows: true,
            ignore_errors: false,
        },
        6 => AggregateFilterRules {
            ignore_nested_aggregate: false,
            ignore_manual_hidden_rows: false,
            ignore_filtered_rows: false,
            ignore_errors: true,
        },
        7 => AggregateFilterRules {
            ignore_nested_aggregate: false,
            ignore_manual_hidden_rows: true,
            ignore_filtered_rows: true,
            ignore_errors: true,
        },
        _ => return Err(SubtotalAggregateEvalError::InvalidOptions),
    };
    Ok((operation, rules))
}

fn single_empty_array_arg() -> CallArgValue {
    CallArgValue::Eval(EvalValue::Array(EvalArray::from_scalar(
        ArrayCellValue::EmptyCell,
    )))
}

fn filter_reference_cells(
    array: &EvalArray,
    context: &AggregateReferenceContext,
    rules: AggregateFilterRules,
) -> Result<CallArgValue, SubtotalAggregateEvalError> {
    if context.shape != array.shape() {
        return Err(SubtotalAggregateEvalError::AggregateContextShapeMismatch);
    }
    let mut kept = Vec::new();
    for row in 0..array.shape().rows {
        for col in 0..array.shape().cols {
            let cell = array
                .get(row, col)
                .expect("shape-validated array cell access");
            let cell_ctx = context
                .get(row, col)
                .expect("shape-validated context cell access");
            if rules.ignore_filtered_rows && cell_ctx.row_filtered_out {
                continue;
            }
            if rules.ignore_manual_hidden_rows && cell_ctx.row_hidden_manual {
                continue;
            }
            if rules.ignore_nested_aggregate && cell_ctx.nested_subtotal_or_aggregate {
                continue;
            }
            if rules.ignore_errors && matches!(cell, ArrayCellValue::Error(_)) {
                continue;
            }
            kept.push(cell.clone());
        }
    }

    if kept.is_empty() {
        return Ok(single_empty_array_arg());
    }

    let array = EvalArray::new(
        ArrayShape {
            rows: kept.len(),
            cols: 1,
        },
        kept,
    )
    .expect("non-empty filtered aggregate array");
    Ok(CallArgValue::Eval(EvalValue::Array(array)))
}

fn materialize_ref_filtered_arg(
    reference: &ReferenceLike,
    rules: AggregateFilterRules,
    resolver: &(impl ReferenceResolver + ?Sized),
    host_info: &dyn HostInfoProvider,
) -> Result<CallArgValue, SubtotalAggregateEvalError> {
    let resolved = resolve_eval_value(resolver, reference)
        .map_err(CoercionError::RefResolution)
        .map_err(SubtotalAggregateEvalError::Coercion)?;
    let context = host_info
        .query_aggregate_reference_context(reference)
        .map_err(SubtotalAggregateEvalError::HostInfo)?;

    match resolved {
        EvalValue::Array(array) => filter_reference_cells(&array, &context, rules),
        EvalValue::Number(n) => filter_reference_cells(
            &EvalArray::from_scalar(ArrayCellValue::Number(n)),
            &context,
            rules,
        ),
        EvalValue::Text(t) => filter_reference_cells(
            &EvalArray::from_scalar(ArrayCellValue::Text(t)),
            &context,
            rules,
        ),
        EvalValue::Logical(b) => filter_reference_cells(
            &EvalArray::from_scalar(ArrayCellValue::Logical(b)),
            &context,
            rules,
        ),
        EvalValue::Error(code) => filter_reference_cells(
            &EvalArray::from_scalar(ArrayCellValue::Error(code)),
            &context,
            rules,
        ),
        EvalValue::Reference(_) => Err(SubtotalAggregateEvalError::Coercion(
            CoercionError::UnsupportedValueKind("reference_like"),
        )),
        EvalValue::Lambda(_) => Err(SubtotalAggregateEvalError::Coercion(
            CoercionError::UnsupportedValueKind("lambda_value"),
        )),
    }
}

fn prepare_reference_form_args(
    args: &[CallArgValue],
    rules: AggregateFilterRules,
    resolver: &(impl ReferenceResolver + ?Sized),
    host_info: &dyn HostInfoProvider,
) -> Result<Vec<CallArgValue>, SubtotalAggregateEvalError> {
    let mut prepared = Vec::new();
    for arg in args {
        match arg {
            CallArgValue::Reference(reference) => {
                prepared.push(materialize_ref_filtered_arg(
                    reference, rules, resolver, host_info,
                )?);
            }
            CallArgValue::Eval(EvalValue::Reference(reference)) => {
                prepared.push(materialize_ref_filtered_arg(
                    reference, rules, resolver, host_info,
                )?);
            }
            CallArgValue::Eval(EvalValue::Array(array)) => {
                let cells = array
                    .iter_row_major()
                    .filter(|cell| {
                        !(rules.ignore_errors && matches!(cell, ArrayCellValue::Error(_)))
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                if cells.is_empty() {
                    prepared.push(single_empty_array_arg());
                } else {
                    prepared.push(CallArgValue::Eval(EvalValue::Array(
                        EvalArray::new(
                            ArrayShape {
                                rows: cells.len(),
                                cols: 1,
                            },
                            cells,
                        )
                        .expect("non-empty aggregate array"),
                    )));
                }
            }
            CallArgValue::EmptyCell => prepared.push(CallArgValue::EmptyCell),
            CallArgValue::MissingArg => prepared.push(CallArgValue::MissingArg),
            CallArgValue::Eval(EvalValue::Number(_))
            | CallArgValue::Eval(EvalValue::Text(_))
            | CallArgValue::Eval(EvalValue::Logical(_))
            | CallArgValue::Eval(EvalValue::Error(_))
            | CallArgValue::Eval(EvalValue::Lambda(_)) => prepared.push(arg.clone()),
        }
    }
    Ok(prepared)
}

fn dispatch_reference_form(
    operation: AggregateOperation,
    args: &[CallArgValue],
) -> Result<EvalValue, SubtotalAggregateEvalError> {
    match operation {
        AggregateOperation::Average => {
            eval_average_surface(args, &PreparedOnlyResolver).map_err(|e| {
                SubtotalAggregateEvalError::DelegateWorksheetError(map_average_error_to_ws(&e))
            })
        }
        AggregateOperation::Count => eval_count_surface(args, &PreparedOnlyResolver).map_err(|e| {
            SubtotalAggregateEvalError::DelegateWorksheetError(map_count_error_to_ws(&e))
        }),
        AggregateOperation::CountA => {
            eval_counta_surface(args, &PreparedOnlyResolver).map_err(|e| {
                SubtotalAggregateEvalError::DelegateWorksheetError(map_counta_error_to_ws(&e))
            })
        }
        AggregateOperation::Max => eval_max_surface(args, &PreparedOnlyResolver).map_err(|e| {
            SubtotalAggregateEvalError::DelegateWorksheetError(map_max_error_to_ws(&e))
        }),
        AggregateOperation::Min => eval_min_surface(args, &PreparedOnlyResolver).map_err(|e| {
            SubtotalAggregateEvalError::DelegateWorksheetError(map_min_error_to_ws(&e))
        }),
        AggregateOperation::Product => {
            eval_product_surface(args, &PreparedOnlyResolver).map_err(|e| {
                SubtotalAggregateEvalError::DelegateWorksheetError(map_product_error_to_ws(&e))
            })
        }
        AggregateOperation::StdevS => {
            eval_stdev_s_surface(args, &PreparedOnlyResolver).map_err(|e| {
                SubtotalAggregateEvalError::DelegateWorksheetError(map_stdev_s_error_to_ws(&e))
            })
        }
        AggregateOperation::StdevP => {
            eval_stdev_p_surface(args, &PreparedOnlyResolver).map_err(|e| {
                SubtotalAggregateEvalError::DelegateWorksheetError(map_stdev_p_error_to_ws(&e))
            })
        }
        AggregateOperation::Sum => eval_sum_surface(args, &PreparedOnlyResolver).map_err(|e| {
            SubtotalAggregateEvalError::DelegateWorksheetError(map_sum_error_to_ws(&e))
        }),
        AggregateOperation::VarS => eval_var_s_surface(args, &PreparedOnlyResolver).map_err(|e| {
            SubtotalAggregateEvalError::DelegateWorksheetError(map_var_s_error_to_ws(&e))
        }),
        AggregateOperation::VarP => eval_var_p_surface(args, &PreparedOnlyResolver).map_err(|e| {
            SubtotalAggregateEvalError::DelegateWorksheetError(map_var_p_error_to_ws(&e))
        }),
        AggregateOperation::Median => {
            eval_median_surface(args, &PreparedOnlyResolver).map_err(|e| {
                SubtotalAggregateEvalError::DelegateWorksheetError(map_median_error_to_ws(&e))
            })
        }
        AggregateOperation::ModeSngl => eval_mode_sngl_surface(args, &PreparedOnlyResolver)
            .map_err(|e| {
                SubtotalAggregateEvalError::DelegateWorksheetError(map_mode_sngl_error_to_ws(&e))
            }),
        AggregateOperation::Large
        | AggregateOperation::Small
        | AggregateOperation::PercentileInc
        | AggregateOperation::QuartileInc
        | AggregateOperation::PercentileExc
        | AggregateOperation::QuartileExc => {
            Err(SubtotalAggregateEvalError::InvalidAggregateSyntax)
        }
    }
}

fn dispatch_array_form(
    operation: AggregateOperation,
    data_arg: CallArgValue,
    k_arg: CallArgValue,
) -> Result<EvalValue, SubtotalAggregateEvalError> {
    let args = [data_arg, k_arg];
    match operation {
        AggregateOperation::Large => {
            eval_large_surface(&args, &PreparedOnlyResolver).map_err(|e| {
                SubtotalAggregateEvalError::DelegateWorksheetError(map_large_error_to_ws(&e))
            })
        }
        AggregateOperation::Small => {
            eval_small_surface(&args, &PreparedOnlyResolver).map_err(|e| {
                SubtotalAggregateEvalError::DelegateWorksheetError(map_small_error_to_ws(&e))
            })
        }
        AggregateOperation::PercentileInc => {
            eval_percentile_inc_surface(&args, &PreparedOnlyResolver).map_err(|e| {
                SubtotalAggregateEvalError::DelegateWorksheetError(map_percentile_inc_error_to_ws(
                    &e,
                ))
            })
        }
        AggregateOperation::QuartileInc => eval_quartile_inc_surface(&args, &PreparedOnlyResolver)
            .map_err(|e| {
                SubtotalAggregateEvalError::DelegateWorksheetError(map_quartile_inc_error_to_ws(&e))
            }),
        AggregateOperation::PercentileExc => {
            eval_percentile_exc_surface(&args, &PreparedOnlyResolver).map_err(|e| {
                SubtotalAggregateEvalError::DelegateWorksheetError(map_percentile_exc_error_to_ws(
                    &e,
                ))
            })
        }
        AggregateOperation::QuartileExc => eval_quartile_exc_surface(&args, &PreparedOnlyResolver)
            .map_err(|e| {
                SubtotalAggregateEvalError::DelegateWorksheetError(map_quartile_exc_error_to_ws(&e))
            }),
        _ => Err(SubtotalAggregateEvalError::InvalidAggregateSyntax),
    }
}

pub fn eval_subtotal_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
    host_info: Option<&dyn HostInfoProvider>,
) -> Result<EvalValue, SubtotalAggregateEvalError> {
    if !SUBTOTAL_META.arity.accepts(args.len()) {
        return Err(SubtotalAggregateEvalError::ArityMismatch {
            expected_min: SUBTOTAL_META.arity.min,
            expected_max: SUBTOTAL_META.arity.max,
            actual: args.len(),
        });
    }
    let function_num = coerce_whole_number(&args[0], resolver)?;
    let (operation, rules) = subtotal_rules(function_num)?;
    let provider = host_info.ok_or(SubtotalAggregateEvalError::HostInfoProviderMissing(
        "aggregate_reference_context",
    ))?;
    let prepared = prepare_reference_form_args(&args[1..], rules, resolver, provider)?;
    dispatch_reference_form(operation, &prepared)
}

pub fn eval_aggregate_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
    host_info: Option<&dyn HostInfoProvider>,
) -> Result<EvalValue, SubtotalAggregateEvalError> {
    if !AGGREGATE_META.arity.accepts(args.len()) {
        return Err(SubtotalAggregateEvalError::ArityMismatch {
            expected_min: AGGREGATE_META.arity.min,
            expected_max: AGGREGATE_META.arity.max,
            actual: args.len(),
        });
    }
    let function_num = coerce_whole_number(&args[0], resolver)?;
    let options = coerce_whole_number(&args[1], resolver)?;
    let (operation, rules) = aggregate_rules(function_num, options)?;
    match operation {
        AggregateOperation::Large
        | AggregateOperation::Small
        | AggregateOperation::PercentileInc
        | AggregateOperation::QuartileInc
        | AggregateOperation::PercentileExc
        | AggregateOperation::QuartileExc => {
            if args.len() != 4 {
                return Err(SubtotalAggregateEvalError::InvalidAggregateSyntax);
            }
            let provider = host_info.ok_or(SubtotalAggregateEvalError::HostInfoProviderMissing(
                "aggregate_reference_context",
            ))?;
            let mut data_args =
                prepare_reference_form_args(&args[2..3], rules, resolver, provider)?;
            let data_arg = data_args.remove(0);
            let k_arg = coerce_k_arg(&args[3], resolver)?;
            dispatch_array_form(operation, data_arg, k_arg)
        }
        _ => {
            let provider = host_info.ok_or(SubtotalAggregateEvalError::HostInfoProviderMissing(
                "aggregate_reference_context",
            ))?;
            let prepared = prepare_reference_form_args(&args[2..], rules, resolver, provider)?;
            dispatch_reference_form(operation, &prepared)
        }
    }
}

pub fn map_subtotal_aggregate_error_to_ws(e: &SubtotalAggregateEvalError) -> WorksheetErrorCode {
    match e {
        SubtotalAggregateEvalError::ArityMismatch { .. }
        | SubtotalAggregateEvalError::InvalidFunctionNum
        | SubtotalAggregateEvalError::InvalidOptions
        | SubtotalAggregateEvalError::InvalidK
        | SubtotalAggregateEvalError::InvalidAggregateSyntax
        | SubtotalAggregateEvalError::AggregateContextShapeMismatch
        | SubtotalAggregateEvalError::HostInfoProviderMissing(_) => WorksheetErrorCode::Value,
        SubtotalAggregateEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        SubtotalAggregateEvalError::HostInfo(HostInfoError::ProviderFailure { .. }) => {
            WorksheetErrorCode::Value
        }
        SubtotalAggregateEvalError::HostInfo(_) => WorksheetErrorCode::Value,
        SubtotalAggregateEvalError::Coercion(_) => WorksheetErrorCode::Value,
        SubtotalAggregateEvalError::DelegateWorksheetError(code) => *code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host_info::{AggregateCellContext, HostInfoProvider};
    use crate::resolver::ResolverCapabilities;
    use crate::value::{ReferenceKind, WorksheetErrorCode};
    use std::collections::BTreeMap;

    struct MockResolver {
        values: BTreeMap<String, EvalValue>,
    }

    impl ReferenceResolver for MockResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            ResolverCapabilities::permissive_local()
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            self.values.get(&reference.target).cloned().ok_or(
                RefResolutionError::UnresolvedReference {
                    target: reference.target.clone(),
                },
            )
        }
    }

    struct MockHostInfoProvider {
        contexts: BTreeMap<String, AggregateReferenceContext>,
    }

    impl HostInfoProvider for MockHostInfoProvider {
        fn query_aggregate_reference_context(
            &self,
            reference: &ReferenceLike,
        ) -> Result<AggregateReferenceContext, HostInfoError> {
            self.contexts
                .get(&reference.target)
                .cloned()
                .ok_or(HostInfoError::UnsupportedAggregateReferenceContextQuery)
        }
    }

    fn area_ref(target: &str) -> CallArgValue {
        CallArgValue::Reference(ReferenceLike {
            kind: ReferenceKind::Area,
            target: target.to_string(),
        })
    }

    fn vertical_array(values: Vec<ArrayCellValue>) -> EvalValue {
        EvalValue::Array(
            EvalArray::new(
                ArrayShape {
                    rows: values.len(),
                    cols: 1,
                },
                values,
            )
            .unwrap(),
        )
    }

    fn context(rows: Vec<AggregateCellContext>) -> AggregateReferenceContext {
        AggregateReferenceContext::new(
            ArrayShape {
                rows: rows.len(),
                cols: 1,
            },
            rows,
        )
        .unwrap()
    }

    #[test]
    fn subtotal_109_ignores_manual_hidden_and_nested_aggregate() {
        let resolver = MockResolver {
            values: BTreeMap::from([(
                "A2:A5".to_string(),
                vertical_array(vec![
                    ArrayCellValue::Number(10.0),
                    ArrayCellValue::Number(20.0),
                    ArrayCellValue::Number(30.0),
                    ArrayCellValue::Number(40.0),
                ]),
            )]),
        };
        let host = MockHostInfoProvider {
            contexts: BTreeMap::from([(
                "A2:A5".to_string(),
                context(vec![
                    AggregateCellContext {
                        row_hidden_manual: false,
                        row_filtered_out: false,
                        nested_subtotal_or_aggregate: false,
                    },
                    AggregateCellContext {
                        row_hidden_manual: true,
                        row_filtered_out: false,
                        nested_subtotal_or_aggregate: false,
                    },
                    AggregateCellContext {
                        row_hidden_manual: false,
                        row_filtered_out: false,
                        nested_subtotal_or_aggregate: true,
                    },
                    AggregateCellContext {
                        row_hidden_manual: false,
                        row_filtered_out: false,
                        nested_subtotal_or_aggregate: false,
                    },
                ]),
            )]),
        };
        let got = eval_subtotal_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(109.0)),
                area_ref("A2:A5"),
            ],
            &resolver,
            Some(&host),
        );
        assert_eq!(got, Ok(EvalValue::Number(50.0)));
    }

    #[test]
    fn aggregate_options_split_nested_hidden_and_error_behavior() {
        let resolver = MockResolver {
            values: BTreeMap::from([(
                "A2:A8".to_string(),
                vertical_array(vec![
                    ArrayCellValue::Number(10.0),
                    ArrayCellValue::Number(20.0),
                    ArrayCellValue::Number(30.0),
                    ArrayCellValue::Number(40.0),
                    ArrayCellValue::Number(50.0),
                    ArrayCellValue::Error(WorksheetErrorCode::NA),
                ]),
            )]),
        };
        let host = MockHostInfoProvider {
            contexts: BTreeMap::from([(
                "A2:A8".to_string(),
                context(vec![
                    AggregateCellContext {
                        row_hidden_manual: false,
                        row_filtered_out: false,
                        nested_subtotal_or_aggregate: false,
                    },
                    AggregateCellContext {
                        row_hidden_manual: true,
                        row_filtered_out: false,
                        nested_subtotal_or_aggregate: false,
                    },
                    AggregateCellContext {
                        row_hidden_manual: false,
                        row_filtered_out: false,
                        nested_subtotal_or_aggregate: true,
                    },
                    AggregateCellContext {
                        row_hidden_manual: false,
                        row_filtered_out: false,
                        nested_subtotal_or_aggregate: false,
                    },
                    AggregateCellContext {
                        row_hidden_manual: false,
                        row_filtered_out: true,
                        nested_subtotal_or_aggregate: false,
                    },
                    AggregateCellContext {
                        row_hidden_manual: false,
                        row_filtered_out: false,
                        nested_subtotal_or_aggregate: false,
                    },
                ]),
            )]),
        };

        let opt0 = eval_aggregate_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(9.0)),
                CallArgValue::Eval(EvalValue::Number(0.0)),
                area_ref("A2:A8"),
            ],
            &resolver,
            Some(&host),
        );
        assert_eq!(
            opt0,
            Err(SubtotalAggregateEvalError::DelegateWorksheetError(
                WorksheetErrorCode::NA
            ))
        );

        let opt1 = eval_aggregate_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(9.0)),
                CallArgValue::Eval(EvalValue::Number(1.0)),
                area_ref("A2:A8"),
            ],
            &resolver,
            Some(&host),
        );
        assert_eq!(
            opt1,
            Err(SubtotalAggregateEvalError::DelegateWorksheetError(
                WorksheetErrorCode::NA
            ))
        );

        let opt3 = eval_aggregate_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(9.0)),
                CallArgValue::Eval(EvalValue::Number(3.0)),
                area_ref("A2:A8"),
            ],
            &resolver,
            Some(&host),
        );
        assert_eq!(opt3, Ok(EvalValue::Number(50.0)));

        let opt6 = eval_aggregate_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(9.0)),
                CallArgValue::Eval(EvalValue::Number(6.0)),
                area_ref("A2:A8"),
            ],
            &resolver,
            Some(&host),
        );
        assert_eq!(opt6, Ok(EvalValue::Number(150.0)));
    }

    #[test]
    fn aggregate_large_array_form_uses_filtered_reference_payload() {
        let resolver = MockResolver {
            values: BTreeMap::from([(
                "A1:A5".to_string(),
                vertical_array(vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(9.0),
                    ArrayCellValue::Number(3.0),
                    ArrayCellValue::Number(7.0),
                    ArrayCellValue::Number(5.0),
                ]),
            )]),
        };
        let host = MockHostInfoProvider {
            contexts: BTreeMap::from([(
                "A1:A5".to_string(),
                context(vec![
                    AggregateCellContext {
                        row_hidden_manual: false,
                        row_filtered_out: false,
                        nested_subtotal_or_aggregate: false,
                    },
                    AggregateCellContext {
                        row_hidden_manual: true,
                        row_filtered_out: false,
                        nested_subtotal_or_aggregate: false,
                    },
                    AggregateCellContext {
                        row_hidden_manual: false,
                        row_filtered_out: false,
                        nested_subtotal_or_aggregate: false,
                    },
                    AggregateCellContext {
                        row_hidden_manual: false,
                        row_filtered_out: false,
                        nested_subtotal_or_aggregate: false,
                    },
                    AggregateCellContext {
                        row_hidden_manual: false,
                        row_filtered_out: false,
                        nested_subtotal_or_aggregate: false,
                    },
                ]),
            )]),
        };
        let got = eval_aggregate_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(14.0)),
                CallArgValue::Eval(EvalValue::Number(5.0)),
                area_ref("A1:A5"),
                CallArgValue::Eval(EvalValue::Number(2.0)),
            ],
            &resolver,
            Some(&host),
        );
        assert_eq!(got, Ok(EvalValue::Number(5.0)));
    }

    #[test]
    fn subtotal_requires_host_context_for_reference_args() {
        let resolver = MockResolver {
            values: BTreeMap::new(),
        };
        let got = eval_subtotal_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(9.0)),
                area_ref("A1:A2"),
            ],
            &resolver,
            None,
        );
        assert_eq!(
            got,
            Err(SubtotalAggregateEvalError::HostInfoProviderMissing(
                "aggregate_reference_context"
            ))
        );
    }
}
