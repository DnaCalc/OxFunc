use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedArgValue, coerce_prepared_to_number, run_values_only_prepared};
use crate::resolver::ReferenceResolver;
use crate::value::{ArrayCellValue, ArrayShape, CallArgValue, EvalArray, EvalValue, WorksheetErrorCode};

pub const SEQUENCE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SEQUENCE",
    arity: Arity { min: 1, max: 4 },
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
pub enum SequenceEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    ZeroDimension {
        arg_index: usize,
    },
    InvalidDimension {
        arg_index: usize,
        value: f64,
    },
}

fn parse_dimension(raw: f64, arg_index: usize) -> Result<usize, SequenceEvalError> {
    if raw == 0.0 {
        return Err(SequenceEvalError::ZeroDimension { arg_index });
    }
    if !raw.is_finite() || raw < 0.0 || raw.fract() != 0.0 {
        return Err(SequenceEvalError::InvalidDimension {
            arg_index,
            value: raw,
        });
    }
    Ok(raw as usize)
}

fn parse_optional_dimension(
    arg: Option<&PreparedArgValue>,
    arg_index: usize,
    default: usize,
) -> Result<usize, SequenceEvalError> {
    match arg {
        None => Ok(default),
        Some(PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell) => Ok(default),
        Some(other) => parse_dimension(
            coerce_prepared_to_number(other).map_err(SequenceEvalError::Coercion)?,
            arg_index,
        ),
    }
}

fn parse_optional_scalar(
    arg: Option<&PreparedArgValue>,
    default: f64,
) -> Result<f64, SequenceEvalError> {
    match arg {
        None => Ok(default),
        Some(PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell) => Ok(default),
        Some(other) => coerce_prepared_to_number(other).map_err(SequenceEvalError::Coercion),
    }
}

pub fn eval_sequence_adapter_prepared(
    args: &[PreparedArgValue],
) -> Result<EvalValue, SequenceEvalError> {
    let argc = args.len();
    if !SEQUENCE_META.arity.accepts(argc) {
        return Err(SequenceEvalError::ArityMismatch {
            expected_min: SEQUENCE_META.arity.min,
            expected_max: SEQUENCE_META.arity.max,
            actual: argc,
        });
    }

    let rows = parse_optional_dimension(args.first(), 1, 1)?;
    let cols = parse_optional_dimension(args.get(1), 2, 1)?;
    let start = parse_optional_scalar(args.get(2), 1.0)?;
    let step = parse_optional_scalar(args.get(3), 1.0)?;

    let shape = ArrayShape { rows, cols };
    let mut cells = Vec::with_capacity(shape.cell_count());
    for idx in 0..shape.cell_count() {
        cells.push(ArrayCellValue::Number(start + (idx as f64) * step));
    }

    Ok(EvalValue::Array(
        EvalArray::new(shape, cells).expect("sequence dimensions validated"),
    ))
}

pub fn eval_sequence_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, SequenceEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_sequence_adapter_prepared,
        SequenceEvalError::Coercion,
    )
}

pub fn map_sequence_error_to_ws(e: &SequenceEvalError) -> WorksheetErrorCode {
    match e {
        SequenceEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        SequenceEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        SequenceEvalError::ZeroDimension { .. } => WorksheetErrorCode::Calc,
        SequenceEvalError::InvalidDimension { .. } => WorksheetErrorCode::Value,
        SequenceEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ExcelText, ReferenceLike};

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
    fn eval_sequence_rows_only_defaults_cols_to_one() {
        let args = [CallArgValue::Eval(EvalValue::Number(3.0))];
        let got = eval_sequence_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(1.0)],
                    vec![ArrayCellValue::Number(2.0)],
                    vec![ArrayCellValue::Number(3.0)],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_sequence_parses_full_arity() {
        let args = [
            CallArgValue::Eval(EvalValue::Number(2.0)),
            CallArgValue::Eval(EvalValue::Number(3.0)),
            CallArgValue::Eval(EvalValue::Number(10.0)),
            CallArgValue::Eval(EvalValue::Number(2.0)),
        ];
        let got = eval_sequence_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![
                        ArrayCellValue::Number(10.0),
                        ArrayCellValue::Number(12.0),
                        ArrayCellValue::Number(14.0),
                    ],
                    vec![
                        ArrayCellValue::Number(16.0),
                        ArrayCellValue::Number(18.0),
                        ArrayCellValue::Number(20.0),
                    ],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_sequence_numeric_text_dimension_is_allowed() {
        let args = [CallArgValue::Eval(EvalValue::Text(
            ExcelText::from_utf16_code_units("4".encode_utf16().collect()),
        ))];
        let got = eval_sequence_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(1.0)],
                    vec![ArrayCellValue::Number(2.0)],
                    vec![ArrayCellValue::Number(3.0)],
                    vec![ArrayCellValue::Number(4.0)],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_sequence_rejects_zero_dimension() {
        let args = [CallArgValue::Eval(EvalValue::Number(0.0))];
        let got = eval_sequence_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Err(SequenceEvalError::ZeroDimension { arg_index: 1 })
        );
    }

    #[test]
    fn map_sequence_zero_dimension_to_calc() {
        assert_eq!(
            map_sequence_error_to_ws(&SequenceEvalError::ZeroDimension { arg_index: 1 }),
            WorksheetErrorCode::Calc
        );
    }

    #[test]
    fn eval_sequence_missing_rows_defaults_to_one() {
        let args = [
            CallArgValue::MissingArg,
            CallArgValue::Eval(EvalValue::Number(3.0)),
        ];
        let got = eval_sequence_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(2.0),
                    ArrayCellValue::Number(3.0),
                ]])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_sequence_missing_middle_args_follow_excel_defaults() {
        let args = [
            CallArgValue::Eval(EvalValue::Number(2.0)),
            CallArgValue::MissingArg,
            CallArgValue::Eval(EvalValue::Number(10.0)),
        ];
        let got = eval_sequence_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![ArrayCellValue::Number(10.0)],
                    vec![ArrayCellValue::Number(11.0)],
                ])
                .unwrap()
            ))
        );

        let args = [
            CallArgValue::Eval(EvalValue::Number(2.0)),
            CallArgValue::Eval(EvalValue::Number(3.0)),
            CallArgValue::MissingArg,
            CallArgValue::Eval(EvalValue::Number(2.0)),
        ];
        let got = eval_sequence_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(EvalValue::Array(
                EvalArray::from_rows(vec![
                    vec![
                        ArrayCellValue::Number(1.0),
                        ArrayCellValue::Number(3.0),
                        ArrayCellValue::Number(5.0),
                    ],
                    vec![
                        ArrayCellValue::Number(7.0),
                        ArrayCellValue::Number(9.0),
                        ArrayCellValue::Number(11.0),
                    ],
                ])
                .unwrap()
            ))
        );
    }
}
