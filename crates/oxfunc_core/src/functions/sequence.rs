use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{coerce_prepared_to_number, run_values_only_prepared};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{ArrayShape, CalcArray, CalcValue, CoreValue, WorksheetErrorCode};

pub const SEQUENCE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SEQUENCE",
    arity: Arity { min: 1, max: 4 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    lift_broadcast_profile: FunctionMeta::DEFAULT_LIFT_BROADCAST_PROFILE,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
    error_collapse_profile: FunctionMeta::DEFAULT_ERROR_COLLAPSE_PROFILE,
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
    arg: Option<&CalcValue>,
    arg_index: usize,
    default: usize,
) -> Result<usize, SequenceEvalError> {
    match arg {
        None => Ok(default),
        Some(value) if matches!(value.core(), CoreValue::Missing | CoreValue::Empty) => Ok(default),
        Some(other) => parse_dimension(
            coerce_prepared_to_number(other).map_err(SequenceEvalError::Coercion)?,
            arg_index,
        ),
    }
}

fn parse_optional_scalar(arg: Option<&CalcValue>, default: f64) -> Result<f64, SequenceEvalError> {
    match arg {
        None => Ok(default),
        Some(value) if matches!(value.core(), CoreValue::Missing | CoreValue::Empty) => Ok(default),
        Some(other) => coerce_prepared_to_number(other).map_err(SequenceEvalError::Coercion),
    }
}

pub fn eval_sequence_adapter_prepared(args: &[CalcValue]) -> Result<CalcValue, SequenceEvalError> {
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
        cells.push(CalcValue::number(start + (idx as f64) * step));
    }

    Ok(CalcValue::array(
        CalcArray::new(shape, cells).expect("sequence dimensions validated"),
    ))
}

pub fn eval_sequence_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, SequenceEvalError> {
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
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::ExcelText;

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
    fn eval_sequence_rows_only_defaults_cols_to_one() {
        let args = [(CalcValue::number(3.0))];
        let got = eval_sequence_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(1.0)],
                    vec![CalcValue::number(2.0)],
                    vec![CalcValue::number(3.0)],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_sequence_parses_full_arity() {
        let args = [
            (CalcValue::number(2.0)),
            (CalcValue::number(3.0)),
            (CalcValue::number(10.0)),
            (CalcValue::number(2.0)),
        ];
        let got = eval_sequence_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![
                        CalcValue::number(10.0),
                        CalcValue::number(12.0),
                        CalcValue::number(14.0),
                    ],
                    vec![
                        CalcValue::number(16.0),
                        CalcValue::number(18.0),
                        CalcValue::number(20.0),
                    ],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_sequence_numeric_text_dimension_is_allowed() {
        let args = [(CalcValue::text(ExcelText::from_utf16_code_units(
            "4".encode_utf16().collect(),
        )))];
        let got = eval_sequence_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(1.0)],
                    vec![CalcValue::number(2.0)],
                    vec![CalcValue::number(3.0)],
                    vec![CalcValue::number(4.0)],
                ])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_sequence_rejects_zero_dimension() {
        let args = [(CalcValue::number(0.0))];
        let got = eval_sequence_surface(&args, &NoResolver);
        assert_eq!(got, Err(SequenceEvalError::ZeroDimension { arg_index: 1 }));
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
        let args = [CalcValue::missing(), (CalcValue::number(3.0))];
        let got = eval_sequence_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(1.0),
                    CalcValue::number(2.0),
                    CalcValue::number(3.0),
                ]])
                .unwrap()
            ))
        );
    }

    #[test]
    fn eval_sequence_missing_middle_args_follow_excel_defaults() {
        let args = [
            (CalcValue::number(2.0)),
            CalcValue::missing(),
            (CalcValue::number(10.0)),
        ];
        let got = eval_sequence_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(10.0)],
                    vec![CalcValue::number(11.0)],
                ])
                .unwrap()
            ))
        );

        let args = [
            (CalcValue::number(2.0)),
            (CalcValue::number(3.0)),
            CalcValue::missing(),
            (CalcValue::number(2.0)),
        ];
        let got = eval_sequence_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![
                        CalcValue::number(1.0),
                        CalcValue::number(3.0),
                        CalcValue::number(5.0),
                    ],
                    vec![
                        CalcValue::number(7.0),
                        CalcValue::number(9.0),
                        CalcValue::number(11.0),
                    ],
                ])
                .unwrap()
            ))
        );
    }
}
