use crate::coercion::{CoercionError, coerce_eval_to_number};
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::resolver::{
    ReferenceSystemError, ReferenceSystemProvider, ReferenceTransformKind,
    ReferenceTransformRequest, resolve_eval_value,
};
use crate::value::{CalcValue, CoreValue, ReferenceLike, WorksheetErrorCode};

pub const OFFSET_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.OFFSET",
    arity: Arity { min: 3, max: 5 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::VolatileContextual,
    host_interaction: HostInteractionClass::WorkbookState,
    thread_safety: ThreadSafetyClass::HostSerialized,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::CallerContext,
    surface_fec_dependency_profile: FecDependencyProfile::CallerContext,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
};

#[derive(Debug, Clone, PartialEq)]
pub enum OffsetEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    RefArgRequired,
    ReferenceSystem(ReferenceSystemError),
    Coercion(CoercionError),
    InvalidDimension,
}

fn parse_offset_number(
    arg: &CalcValue,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<i64, OffsetEvalError> {
    let number = match arg.core() {
        CoreValue::Reference(r) => resolve_eval_value(resolver, r)
            .map_err(CoercionError::RefResolution)
            .and_then(|v| coerce_eval_to_number(&v, resolver)),
        CoreValue::Missing => Err(CoercionError::MissingArg),
        CoreValue::Empty => Err(CoercionError::EmptyCell),
        _ => coerce_eval_to_number(arg, resolver),
    }
    .map_err(OffsetEvalError::Coercion)?;
    Ok(number.trunc() as i64)
}

fn parse_optional_positive_dimension(
    arg: Option<&CalcValue>,
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<Option<usize>, OffsetEvalError> {
    let Some(arg) = arg else {
        return Ok(None);
    };

    let value = parse_offset_number(arg, resolver)?;
    if value <= 0 {
        return Err(OffsetEvalError::InvalidDimension);
    }
    usize::try_from(value)
        .map(Some)
        .map_err(|_| OffsetEvalError::InvalidDimension)
}

fn parse_reference_arg(arg: &CalcValue) -> Result<ReferenceLike, OffsetEvalError> {
    arg.as_reference()
        .cloned()
        .ok_or(OffsetEvalError::RefArgRequired)
}

pub fn eval_offset_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, OffsetEvalError> {
    let argc = args.len();
    if !OFFSET_META.arity.accepts(argc) {
        return Err(OffsetEvalError::ArityMismatch {
            expected_min: OFFSET_META.arity.min,
            expected_max: OFFSET_META.arity.max,
            actual: argc,
        });
    }

    let base = parse_reference_arg(&args[0])?;
    let row_offset = parse_offset_number(&args[1], resolver)?;
    let col_offset = parse_offset_number(&args[2], resolver)?;
    let height = parse_optional_positive_dimension(args.get(3), resolver)?;
    let width = parse_optional_positive_dimension(args.get(4), resolver)?;
    resolver
        .transform_reference(&ReferenceTransformRequest {
            reference: base,
            transform: ReferenceTransformKind::Offset {
                row_offset,
                col_offset,
                height,
                width,
            },
        })
        .map(CalcValue::reference)
        .map_err(OffsetEvalError::ReferenceSystem)
}

pub fn map_offset_error_to_ws(e: &OffsetEvalError) -> WorksheetErrorCode {
    match e {
        OffsetEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        OffsetEvalError::RefArgRequired => WorksheetErrorCode::Value,
        OffsetEvalError::ReferenceSystem(_) => WorksheetErrorCode::Ref,
        OffsetEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        OffsetEvalError::Coercion(_) => WorksheetErrorCode::Value,
        OffsetEvalError::InvalidDimension => WorksheetErrorCode::Ref,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::ReferenceKind;

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

        fn transform_reference(
            &self,
            request: &ReferenceTransformRequest,
        ) -> Result<ReferenceLike, ReferenceSystemError> {
            let ReferenceTransformKind::Offset {
                row_offset,
                col_offset,
                height,
                width,
            } = request.transform
            else {
                return Err(ReferenceSystemError::Unsupported {
                    operation: crate::resolver::ReferenceSystemOperation::Transform,
                });
            };
            match (
                request.reference.target(),
                row_offset,
                col_offset,
                height,
                width,
            ) {
                ("A1", 1, 2, None, None) => Ok(ReferenceLike::new(ReferenceKind::A1, "C2")),
                ("A1:B2", 0, 1, Some(1), Some(3)) => {
                    Ok(ReferenceLike::new(ReferenceKind::Area, "B1:D1"))
                }
                ("B2:C3", 1, 1, None, None) => Ok(ReferenceLike::new(ReferenceKind::Area, "C3:D4")),
                ("Sheet1!B2", 1, 2, None, None) => {
                    Ok(ReferenceLike::new(ReferenceKind::A1, "Sheet1!D3"))
                }
                _ => Err(ReferenceSystemError::Unsupported {
                    operation: crate::resolver::ReferenceSystemOperation::Transform,
                }),
            }
        }
    }

    #[test]
    fn eval_offset_shifts_single_cell_reference() {
        let got = eval_offset_surface(
            &[
                CalcValue::reference(ReferenceLike::new(ReferenceKind::A1, "A1".to_string())),
                (CalcValue::number(1.0)),
                (CalcValue::number(2.0)),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::reference(ReferenceLike::new(
                ReferenceKind::A1,
                "C2".to_string()
            )))
        );
    }

    #[test]
    fn eval_offset_resizes_reference_area() {
        let got = eval_offset_surface(
            &[
                CalcValue::reference(ReferenceLike::new(ReferenceKind::Area, "A1:B2".to_string())),
                (CalcValue::number(0.0)),
                (CalcValue::number(1.0)),
                (CalcValue::number(1.0)),
                (CalcValue::number(3.0)),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::reference(ReferenceLike::new(
                ReferenceKind::Area,
                "B1:D1".to_string()
            )))
        );
    }

    #[test]
    fn eval_offset_defaults_height_and_width_to_base_shape() {
        let got = eval_offset_surface(
            &[
                CalcValue::reference(ReferenceLike::new(ReferenceKind::Area, "B2:C3".to_string())),
                (CalcValue::number(1.0)),
                (CalcValue::number(1.0)),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::reference(ReferenceLike::new(
                ReferenceKind::Area,
                "C3:D4".to_string()
            )))
        );
    }

    #[test]
    fn eval_offset_preserves_sheet_prefix() {
        let got = eval_offset_surface(
            &[
                CalcValue::reference(ReferenceLike::new(
                    ReferenceKind::A1,
                    "Sheet1!B2".to_string(),
                )),
                (CalcValue::number(1.0)),
                (CalcValue::number(2.0)),
            ],
            &NoResolver,
        );
        assert_eq!(
            got,
            Ok(CalcValue::reference(ReferenceLike::new(
                ReferenceKind::A1,
                "Sheet1!D3".to_string()
            )))
        );
    }
}
