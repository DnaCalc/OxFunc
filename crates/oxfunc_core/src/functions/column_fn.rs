use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::a1_refs::parse_a1_reference;
use crate::resolver::ReferenceSystemProvider;
use crate::value::{
    ArrayShape, FunctionArg, FunctionArray, FunctionArrayCell, FunctionValue, WorksheetErrorCode,
};

pub const COLUMN_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.COLUMN",
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
pub enum ColumnEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    MissingCallerContext,
    InvalidReferenceArg,
}

fn column_reference_from_arg(
    arg: &FunctionArg,
) -> Result<crate::functions::a1_refs::A1Reference, ColumnEvalError> {
    let reference = match arg {
        FunctionArg::Reference(r) => r,
        FunctionArg::Eval(FunctionValue::Reference(r)) => r,
        _ => return Err(ColumnEvalError::InvalidReferenceArg),
    };
    parse_a1_reference(reference.target()).ok_or(ColumnEvalError::InvalidReferenceArg)
}

fn column_result(start_col: usize, end_col: usize) -> FunctionValue {
    if start_col == end_col {
        FunctionValue::Number(start_col as f64)
    } else {
        let cells = (start_col..=end_col)
            .map(|col| FunctionArrayCell::Number(col as f64))
            .collect::<Vec<_>>();
        FunctionValue::Array(
            FunctionArray::new(
                ArrayShape {
                    rows: 1,
                    cols: end_col - start_col + 1,
                },
                cells,
            )
            .expect("shape preserved"),
        )
    }
}

pub fn eval_column_surface(
    args: &[FunctionArg],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<FunctionValue, ColumnEvalError> {
    if !COLUMN_META.arity.accepts(args.len()) {
        return Err(ColumnEvalError::ArityMismatch {
            expected_min: COLUMN_META.arity.min,
            expected_max: COLUMN_META.arity.max,
            actual: args.len(),
        });
    }

    if args.is_empty() || matches!(args[0], FunctionArg::MissingArg) {
        let caller = resolver
            .caller_context()
            .ok_or(ColumnEvalError::MissingCallerContext)?;
        return Ok(FunctionValue::Number(caller.col as f64));
    }

    let reference = column_reference_from_arg(&args[0])?;
    Ok(column_result(reference.start_col, reference.end_col))
}

pub fn map_column_error_to_ws(e: &ColumnEvalError) -> WorksheetErrorCode {
    match e {
        ColumnEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        ColumnEvalError::MissingCallerContext => WorksheetErrorCode::Ref,
        ColumnEvalError::InvalidReferenceArg => WorksheetErrorCode::Value,
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
    fn eval_column_omitted_uses_caller_column() {
        let got = eval_column_surface(
            &[],
            &MockResolver {
                caller: Some(CallerContext {
                    prefix: Some("Sheet1".to_string()),
                    row: 7,
                    col: 3,
                }),
            },
        );
        assert_eq!(got, Ok(FunctionValue::Number(3.0)));
    }

    #[test]
    fn eval_column_single_cell_reference_returns_scalar() {
        let got = eval_column_surface(
            &[FunctionArg::Reference(ReferenceLike::new(
                ReferenceKind::A1,
                "B2".to_string(),
            ))],
            &MockResolver { caller: None },
        );
        assert_eq!(got, Ok(FunctionValue::Number(2.0)));
    }

    #[test]
    fn eval_column_area_reference_spills_horizontally() {
        let got = eval_column_surface(
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
                FunctionArray::from_rows(vec![vec![
                    FunctionArrayCell::Number(2.0),
                    FunctionArrayCell::Number(3.0),
                ]])
                .unwrap()
            )
        );
    }

    #[test]
    fn eval_column_whole_row_reference_builds_full_width_vector() {
        let got = eval_column_surface(
            &[FunctionArg::Reference(ReferenceLike::new(
                ReferenceKind::Area,
                "1:1".to_string(),
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
                rows: 1,
                cols: 16_384
            }
        );
        assert_eq!(array.get(0, 0), Some(&FunctionArrayCell::Number(1.0)));
        assert_eq!(
            array.get(0, 16_383),
            Some(&FunctionArrayCell::Number(16_384.0))
        );
    }
}
