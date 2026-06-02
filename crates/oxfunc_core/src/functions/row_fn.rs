use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::a1_refs::parse_a1_reference;
use crate::resolver::ReferenceSystemProvider;
use crate::value::{
    ArrayShape, FunctionArg, FunctionArray, FunctionArrayCell, FunctionValue, ReferenceLike,
    WorksheetErrorCode,
};

pub const ROW_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ROW",
    arity: Arity { min: 0, max: 1 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::WorkbookState,
    thread_safety: ThreadSafetyClass::HostSerialized,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::CallerContext,
    surface_fec_dependency_profile: FecDependencyProfile::CallerContext,
};

#[derive(Debug, Clone, PartialEq)]
pub enum RowEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    MissingCallerContext,
    InvalidReferenceArg,
}

fn row_reference_from_arg(
    arg: &FunctionArg,
) -> Result<crate::functions::a1_refs::A1Reference, RowEvalError> {
    let reference = match arg {
        FunctionArg::Reference(r) => r,
        FunctionArg::Eval(FunctionValue::Reference(r)) => r,
        _ => return Err(RowEvalError::InvalidReferenceArg),
    };
    parse_reference(reference)
}

fn parse_reference(
    reference: &ReferenceLike,
) -> Result<crate::functions::a1_refs::A1Reference, RowEvalError> {
    parse_a1_reference(reference.target()).ok_or(RowEvalError::InvalidReferenceArg)
}

fn row_result(start_row: usize, end_row: usize) -> FunctionValue {
    if start_row == end_row {
        FunctionValue::Number(start_row as f64)
    } else {
        let cells = (start_row..=end_row)
            .map(|row| FunctionArrayCell::Number(row as f64))
            .collect::<Vec<_>>();
        FunctionValue::Array(
            FunctionArray::new(
                ArrayShape {
                    rows: end_row - start_row + 1,
                    cols: 1,
                },
                cells,
            )
            .expect("shape preserved"),
        )
    }
}

pub fn eval_row_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, RowEvalError> {
    if !ROW_META.arity.accepts(args.len()) {
        return Err(RowEvalError::ArityMismatch {
            expected_min: ROW_META.arity.min,
            expected_max: ROW_META.arity.max,
            actual: args.len(),
        });
    }

    if args.is_empty() || matches!(args[0], FunctionArg::MissingArg) {
        let caller = resolver
            .caller_context()
            .ok_or(RowEvalError::MissingCallerContext)?;
        return Ok(FunctionValue::Number(caller.row as f64));
    }

    let reference = row_reference_from_arg(&args[0])?;
    Ok(row_result(reference.start_row, reference.end_row))
}

pub fn map_row_error_to_ws(e: &RowEvalError) -> WorksheetErrorCode {
    match e {
        RowEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        RowEvalError::MissingCallerContext => WorksheetErrorCode::Ref,
        RowEvalError::InvalidReferenceArg => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{CallerContext, ReferenceSystemCapabilities};
    use crate::value::{ReferenceKind, ReferenceLike};

    struct MockResolver {
        caller: Option<CallerContext>,
    }

    impl ReferenceSystemProvider for MockResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            ReferenceSystemCapabilities::permissive_local()
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<FunctionValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            Err(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
        }

        fn caller_context(&self) -> Option<CallerContext> {
            self.caller.clone()
        }
    }

    #[test]
    fn eval_row_omitted_uses_caller_row() {
        let got = eval_row_surface(
            &[],
            &MockResolver {
                caller: Some(CallerContext {
                    prefix: Some("Sheet1".to_string()),
                    row: 7,
                    col: 3,
                }),
            },
        );
        assert_eq!(got, Ok(FunctionValue::Number(7.0)));
    }

    #[test]
    fn eval_row_single_cell_reference_returns_scalar() {
        let got = eval_row_surface(
            &[FunctionArg::Reference(ReferenceLike::new(
                ReferenceKind::A1,
                "B2".to_string(),
            ))],
            &MockResolver { caller: None },
        );
        assert_eq!(got, Ok(FunctionValue::Number(2.0)));
    }

    #[test]
    fn eval_row_area_reference_spills_vertically() {
        let got = eval_row_surface(
            &[FunctionArg::Reference(ReferenceLike::new(
                ReferenceKind::Area,
                "B2:C3".to_string(),
            ))],
            &MockResolver { caller: None },
        )
        .unwrap();
        assert_eq!(
            got,
            FunctionValue::Array(
                FunctionArray::from_rows(vec![
                    vec![FunctionArrayCell::Number(2.0)],
                    vec![FunctionArrayCell::Number(3.0)],
                ])
                .unwrap()
            )
        );
    }

    #[test]
    fn eval_row_whole_column_reference_builds_full_height_vector() {
        let got = eval_row_surface(
            &[FunctionArg::Reference(ReferenceLike::new(
                ReferenceKind::Area,
                "A:A".to_string(),
            ))],
            &MockResolver { caller: None },
        )
        .unwrap();
        let FunctionValue::Array(array) = got else {
            panic!("expected array");
        };
        assert_eq!(
            array.shape(),
            ArrayShape {
                rows: 1_048_576,
                cols: 1
            }
        );
        assert_eq!(array.get(0, 0), Some(&FunctionArrayCell::Number(1.0)));
        assert_eq!(
            array.get(1_048_575, 0),
            Some(&FunctionArrayCell::Number(1_048_576.0))
        );
    }
}
