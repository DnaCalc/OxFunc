use crate::coercion::{CoercionError, coerce_calc_scalar_to_number};
use crate::functions::adapters::{
    prepare_arg_values_only, sparse_reference_values_for_aggregate_arg,
};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{CalcValue, CoreValue, WorksheetErrorCode};

pub(crate) const GCD_LCM_LIMIT: f64 = 9_007_199_254_740_992.0;

fn collect_scalar(
    value: &CalcValue,
    array_cell: bool,
    numbers: &mut Vec<f64>,
) -> Result<(), CoercionError> {
    match value.core() {
        CoreValue::Empty => {
            if array_cell {
                numbers.push(0.0);
            }
        }
        CoreValue::Logical(_) => {
            return Err(CoercionError::UnsupportedValueKind("gcd_lcm_logical"));
        }
        _ => numbers.push(coerce_calc_scalar_to_number(value)?),
    }
    Ok(())
}

/// Preserve argument groups while coercing every argument before numeric-domain
/// validation. LCM's observable rounded reduction order is handled by its kernel.
pub(crate) fn collect_integer_groups(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<Vec<Vec<i64>>, CoercionError> {
    let mut groups = Vec::with_capacity(args.len());
    for (index, arg) in args.iter().enumerate() {
        if matches!(arg.core(), CoreValue::Missing) {
            if index == 0 {
                return Err(CoercionError::WorksheetError(WorksheetErrorCode::NA));
            }
            continue;
        }
        let mut numbers = Vec::new();
        if let Some(mut values) = sparse_reference_values_for_aggregate_arg(arg, resolver)? {
            let cells = values.declared_cell_count();
            let array_cell = cells > 1;
            // Provider enumeration order is not a substitute for worksheet order.
            values
                .defined_cells
                .sort_by_key(|cell| (cell.row, cell.col));
            for cell in &values.defined_cells {
                collect_scalar(&cell.value, array_cell, &mut numbers)?;
            }
            if array_cell && values.defined_cells.len() < cells {
                // One implicit blank suffices for the reducer's zero behavior;
                // no dense allocation is needed for a large sparse reference.
                numbers.push(0.0);
            }
        } else {
            let prepared = prepare_arg_values_only(arg, resolver)?;
            if let CoreValue::Array(array) = prepared.core() {
                for cell in array.iter_row_major() {
                    collect_scalar(cell, true, &mut numbers)?;
                }
            } else {
                collect_scalar(&prepared, false, &mut numbers)?;
            }
        }
        groups.push(numbers);
    }
    if groups.iter().all(Vec::is_empty) {
        return Err(CoercionError::WorksheetError(WorksheetErrorCode::Value));
    }
    if groups
        .iter()
        .flatten()
        .any(|n| !n.is_finite() || *n < 0.0 || *n > GCD_LCM_LIMIT)
    {
        return Err(CoercionError::WorksheetError(WorksheetErrorCode::Num));
    }
    Ok(groups
        .into_iter()
        .map(|group| group.into_iter().map(|n| n.trunc() as i64).collect())
        .collect())
}

pub fn gcd_int(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a.abs()
}

pub fn lcm_int(a: i64, b: i64) -> Result<i64, WorksheetErrorCode> {
    if a == 0 || b == 0 {
        Ok(0)
    } else {
        // Each admitted operand is at most 2^53, so the exact product fits
        // u128. Publish its binary64 rounding before checking the next step.
        // In particular, 2^53 + 1 rounds to the admitted boundary 2^53.
        let product = (a / gcd_int(a, b)) as u128 * b as u128;
        let rounded = product as f64;
        if rounded > GCD_LCM_LIMIT {
            Err(WorksheetErrorCode::Num)
        } else {
            Ok(rounded as i64)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::functions::lcm_fn::eval_lcm_surface;
    use crate::resolver::{
        ReferenceEnumerationRequest, ReferenceResolutionError, ResolvedReferenceCell,
        ResolvedReferenceExtent, ResolvedReferenceValues,
    };
    use crate::value::{ReferenceKind, ReferenceLike};

    struct Sparse(ResolvedReferenceValues);
    impl ReferenceSystemProvider for Sparse {
        fn enumerate_values(
            &self,
            _: &ReferenceEnumerationRequest,
        ) -> Result<Option<ResolvedReferenceValues>, ReferenceResolutionError> {
            Ok(Some(self.0.clone()))
        }
    }

    #[test]
    fn sparse_reference_extent_and_order_preserve_reduction_semantics() {
        let reference =
            CalcValue::reference(ReferenceLike::new(ReferenceKind::Area, "A1:C1"));
        let sparse = |cols, cells| {
            Sparse(ResolvedReferenceValues::new(
                ResolvedReferenceExtent::new(1, cols),
                cells,
                None,
            ))
        };
        let blank = sparse(1, vec![]);
        assert_eq!(
            collect_integer_groups(&[reference.clone()], &blank),
            Err(CoercionError::WorksheetError(WorksheetErrorCode::Value))
        );
        let blank_and_six = sparse(
            2,
            vec![ResolvedReferenceCell::new(0, 1, CalcValue::number(6.0))],
        );
        assert_eq!(
            eval_lcm_surface(&[reference.clone()], &blank_and_six).unwrap(),
            CalcValue::number(0.0)
        );
        let unordered = sparse(
            3,
            vec![
                ResolvedReferenceCell::new(0, 2, CalcValue::number(3.0)),
                ResolvedReferenceCell::new(0, 0, CalcValue::number(3_002_399_751_580_331.0)),
                ResolvedReferenceCell::new(0, 1, CalcValue::number(3.0)),
            ],
        );
        assert_eq!(
            eval_lcm_surface(&[reference.clone()], &unordered).unwrap(),
            CalcValue::number(GCD_LCM_LIMIT)
        );
        let errors = sparse(
            2,
            vec![
                ResolvedReferenceCell::new(0, 1, CalcValue::error(WorksheetErrorCode::NA)),
                ResolvedReferenceCell::new(0, 0, CalcValue::error(WorksheetErrorCode::Div0)),
            ],
        );
        assert_eq!(
            collect_integer_groups(&[reference], &errors),
            Err(CoercionError::WorksheetError(WorksheetErrorCode::Div0))
        );
    }

    #[test]
    fn gcd_and_lcm_basic_lanes() {
        assert_eq!(gcd_int(24, 36), 12);
        assert_eq!(gcd_int(0, 5), 5);
        assert_eq!(lcm_int(6, 8), Ok(24));
        assert_eq!(lcm_int(0, 5), Ok(0));
        assert_eq!(lcm_int(3, 3_002_399_751_580_331), Ok(9_007_199_254_740_992));
        assert_eq!(
            lcm_int(9_007_199_254_740_991, 9_007_199_254_740_990),
            Err(WorksheetErrorCode::Num)
        );
    }
}
