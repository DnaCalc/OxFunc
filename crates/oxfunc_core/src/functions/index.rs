use crate::coercion::{CoercionError, coerce_arg_to_number};
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, ReferenceLike, WorksheetErrorCode};

pub const INDEX_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.INDEX",
    arity: Arity { min: 2, max: 4 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::WorkbookState,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::RefOnly,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

#[derive(Debug, Clone, PartialEq)]
pub enum IndexEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    InvalidIndexNumber(f64),
    InvalidAreaNumber(f64),
    OutOfBounds {
        rows: usize,
        cols: usize,
        row: usize,
        col: usize,
    },
    UnsupportedSource(&'static str),
    ArrayPayloadUnavailable,
}

fn coerce_index_number(
    arg: &CallArgValue,
    resolver: &impl ReferenceResolver,
) -> Result<usize, IndexEvalError> {
    let n = coerce_arg_to_number(arg, resolver).map_err(IndexEvalError::Coercion)?;
    if !n.is_finite() || n < 0.0 || n.fract() != 0.0 {
        return Err(IndexEvalError::InvalidIndexNumber(n));
    }
    Ok(n as usize)
}

fn project_reference(base: &ReferenceLike, row: usize, col: usize) -> EvalValue {
    EvalValue::Reference(ReferenceLike {
        kind: base.kind,
        target: format!("{}#INDEX({row},{col})", base.target),
    })
}

pub fn eval_index_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, IndexEvalError> {
    let argc = args.len();
    if !INDEX_META.arity.accepts(argc) {
        return Err(IndexEvalError::ArityMismatch {
            expected_min: INDEX_META.arity.min,
            expected_max: INDEX_META.arity.max,
            actual: argc,
        });
    }

    let row = coerce_index_number(&args[1], resolver)?;
    let col = if argc >= 3 {
        coerce_index_number(&args[2], resolver)?
    } else {
        1
    };

    if argc >= 4 {
        let area = coerce_arg_to_number(&args[3], resolver).map_err(IndexEvalError::Coercion)?;
        if area != 1.0 {
            return Err(IndexEvalError::InvalidAreaNumber(area));
        }
    }

    match &args[0] {
        CallArgValue::Reference(r) => {
            if row == 0 && col == 0 {
                Ok(EvalValue::Reference(r.clone()))
            } else {
                Ok(project_reference(r, row, col))
            }
        }
        CallArgValue::Eval(EvalValue::Reference(r)) => {
            if row == 0 && col == 0 {
                Ok(EvalValue::Reference(r.clone()))
            } else {
                Ok(project_reference(r, row, col))
            }
        }
        CallArgValue::Eval(EvalValue::Array(shape)) => {
            if row == 0 || col == 0 {
                return Ok(EvalValue::Array(*shape));
            }
            if row > shape.rows || col > shape.cols {
                return Err(IndexEvalError::OutOfBounds {
                    rows: shape.rows,
                    cols: shape.cols,
                    row,
                    col,
                });
            }
            Err(IndexEvalError::ArrayPayloadUnavailable)
        }
        CallArgValue::Eval(_) => Err(IndexEvalError::UnsupportedSource("non_array_non_reference")),
        CallArgValue::MissingArg => Err(IndexEvalError::UnsupportedSource("missing_arg_source")),
        CallArgValue::EmptyCell => Err(IndexEvalError::UnsupportedSource("empty_cell_source")),
    }
}

pub fn map_index_error_to_ws(e: &IndexEvalError) -> WorksheetErrorCode {
    match e {
        IndexEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        IndexEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        IndexEvalError::InvalidIndexNumber(_) => WorksheetErrorCode::Value,
        IndexEvalError::InvalidAreaNumber(_) => WorksheetErrorCode::Ref,
        IndexEvalError::OutOfBounds { .. } => WorksheetErrorCode::Ref,
        IndexEvalError::UnsupportedSource(_) => WorksheetErrorCode::Value,
        IndexEvalError::ArrayPayloadUnavailable => WorksheetErrorCode::Calc,
        IndexEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ArrayShape, ReferenceKind};

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
    fn eval_index_reference_projection_is_shape_only_seed() {
        let args = [
            CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "A1:C3".to_string(),
            }),
            CallArgValue::Eval(EvalValue::Number(2.0)),
            CallArgValue::Eval(EvalValue::Number(1.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "A1:C3#INDEX(2,1)".to_string(),
            }))
        );
    }

    #[test]
    fn eval_index_array_position_is_payload_unavailable() {
        let args = [
            CallArgValue::Eval(EvalValue::Array(ArrayShape { rows: 3, cols: 2 })),
            CallArgValue::Eval(EvalValue::Number(1.0)),
            CallArgValue::Eval(EvalValue::Number(1.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(got, Err(IndexEvalError::ArrayPayloadUnavailable));
    }

    #[test]
    fn eval_index_array_bounds_checked() {
        let args = [
            CallArgValue::Eval(EvalValue::Array(ArrayShape { rows: 2, cols: 2 })),
            CallArgValue::Eval(EvalValue::Number(3.0)),
            CallArgValue::Eval(EvalValue::Number(1.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(
            got,
            Err(IndexEvalError::OutOfBounds {
                rows: 2,
                cols: 2,
                row: 3,
                col: 1,
            })
        );
    }

    #[test]
    fn eval_index_invalid_area_num_rejected() {
        let args = [
            CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target: "A1:C3".to_string(),
            }),
            CallArgValue::Eval(EvalValue::Number(1.0)),
            CallArgValue::Eval(EvalValue::Number(1.0)),
            CallArgValue::Eval(EvalValue::Number(2.0)),
        ];
        let got = eval_index_surface(&args, &NoResolver);
        assert_eq!(got, Err(IndexEvalError::InvalidAreaNumber(2.0)));
    }
}
