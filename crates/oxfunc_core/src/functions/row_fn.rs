use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::a1_refs::parse_a1_reference;
use crate::resolver::ReferenceSystemProvider;
use crate::value::{ArrayShape, CalcArray, CalcValue, ReferenceLike, WorksheetErrorCode};

pub const ROW_META: FunctionMeta = function_spec! {
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
    arg: &CalcValue,
) -> Result<crate::functions::a1_refs::A1Reference, RowEvalError> {
    let reference = arg
        .as_reference()
        .ok_or(RowEvalError::InvalidReferenceArg)?;
    parse_reference(reference)
}

fn parse_reference(
    reference: &ReferenceLike,
) -> Result<crate::functions::a1_refs::A1Reference, RowEvalError> {
    parse_a1_reference(reference.target()).ok_or(RowEvalError::InvalidReferenceArg)
}

fn row_result(start_row: usize, end_row: usize) -> CalcValue {
    if start_row == end_row {
        CalcValue::number(start_row as f64)
    } else {
        let cells = (start_row..=end_row)
            .map(|row| CalcValue::number(row as f64))
            .collect::<Vec<_>>();
        CalcValue::array(
            CalcArray::new(
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
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, RowEvalError> {
    if !ROW_META.arity.accepts(args.len()) {
        return Err(RowEvalError::ArityMismatch {
            expected_min: ROW_META.arity.min,
            expected_max: ROW_META.arity.max,
            actual: args.len(),
        });
    }

    if args.is_empty() || args[0].is_missing() {
        let caller = resolver
            .caller_context()
            .ok_or(RowEvalError::MissingCallerContext)?;
        return Ok(CalcValue::number(caller.row as f64));
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
        ) -> Result<CalcValue, crate::resolver::ReferenceResolutionError> {
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
        assert_eq!(got, Ok(CalcValue::number(7.0)));
    }

    #[test]
    fn eval_row_single_cell_reference_returns_scalar() {
        let got = eval_row_surface(
            &[CalcValue::reference(ReferenceLike::new(
                ReferenceKind::A1,
                "B2".to_string(),
            ))],
            &MockResolver { caller: None },
        );
        assert_eq!(got, Ok(CalcValue::number(2.0)));
    }

    #[test]
    fn eval_row_area_reference_spills_vertically() {
        let got = eval_row_surface(
            &[CalcValue::reference(ReferenceLike::new(
                ReferenceKind::Area,
                "B2:C3".to_string(),
            ))],
            &MockResolver { caller: None },
        )
        .unwrap();
        assert_eq!(
            got,
            CalcValue::array(
                CalcArray::from_rows(vec![
                    vec![CalcValue::number(2.0)],
                    vec![CalcValue::number(3.0)],
                ])
                .unwrap()
            )
        );
    }

    #[test]
    fn eval_row_whole_column_reference_builds_full_height_vector() {
        let got = eval_row_surface(
            &[CalcValue::reference(ReferenceLike::new(
                ReferenceKind::Area,
                "A:A".to_string(),
            ))],
            &MockResolver { caller: None },
        )
        .unwrap();
        let crate::value::CoreValue::Array(array) = got.core else {
            panic!("expected array");
        };
        assert_eq!(
            array.shape(),
            ArrayShape {
                rows: 1_048_576,
                cols: 1
            }
        );
        assert_eq!(array.get(0, 0), Some(&CalcValue::number(1.0)));
        assert_eq!(
            array.get(1_048_575, 0),
            Some(&CalcValue::number(1_048_576.0))
        );
    }
}
