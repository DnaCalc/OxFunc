//! Reduced live Excel 20430/CV2 witnesses from W111 SLN discrimination.
use oxfunc_core::functions::depreciation_family::sln_kernel;
use oxfunc_core::functions::surface_dispatch::eval_surface_value_call;
use oxfunc_core::resolver::NULL_REFERENCE_SYSTEM_PROVIDER;
use oxfunc_core::value::{CalcArray, CalcValue, CoreValue, WorksheetErrorCode};

#[test]
fn sln_signs_exact_zero_life_and_publication() {
    assert_eq!(sln_kernel(100.0, 0.0, -1.0), Ok(-100.0));
    assert_eq!(sln_kernel(-3.0, 7.0, 2.0), Ok(-5.0));
    assert_eq!(sln_kernel(1.0, 0.0, 1e-13), Ok(1e13));
    assert_eq!(sln_kernel(1.0, 0.0, 0.0), Err(WorksheetErrorCode::Div0));
    assert_eq!(
        sln_kernel(f64::MAX, -f64::MAX, f64::MAX),
        Err(WorksheetErrorCode::Num)
    );
    for (cost, salvage, life) in [
        (1., 1., -1.),
        (f64::MIN_POSITIVE, 0., 2.),
        (-1e-200, 0., 1e200),
    ] {
        assert_eq!(sln_kernel(cost, salvage, life).unwrap().to_bits(), 0);
    }
    // A subnormal numerator can become a normal result; do not flush it early.
    assert_eq!(
        sln_kernel(
            f64::MIN_POSITIVE,
            f64::from_bits(0x0010000000000001),
            f64::MIN_POSITIVE
        )
        .unwrap()
        .to_bits(),
        (-f64::EPSILON).to_bits()
    );
}

#[test]
#[cfg(target_arch = "x86_64")]
fn sln_extended_division_rounding_witness() {
    assert_eq!(
        sln_kernel(7.0, 3.0, f64::MAX).unwrap().to_bits(),
        f64::MIN_POSITIVE.to_bits()
    );
    assert_eq!(
        sln_kernel(f64::MAX, f64::from_bits(0x7feffffffffffffe), f64::MAX)
            .unwrap()
            .to_bits(),
        0x3ca0000000000000
    );
}

#[test]
fn sln_surface_missing_and_array_cells() {
    let values = CalcArray::from_rows(vec![vec![
        CalcValue::number(10.),
        CalcValue::number(-10.),
        CalcValue::error(WorksheetErrorCode::NA),
    ]])
    .unwrap();
    let result = eval_surface_value_call(
        "FUNC.SLN",
        &[
            CalcValue::array(values),
            CalcValue::missing(),
            CalcValue::number(2.),
        ],
        &NULL_REFERENCE_SYSTEM_PROVIDER,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    let CoreValue::Array(array) = result.core() else {
        panic!("{result:?}");
    };
    assert_eq!(array.get(0, 0), Some(&CalcValue::number(5.)));
    assert_eq!(array.get(0, 1), Some(&CalcValue::number(-5.)));
    assert_eq!(
        array.get(0, 2),
        Some(&CalcValue::error(WorksheetErrorCode::NA))
    );
    let result = eval_surface_value_call(
        "FUNC.SLN",
        &[
            CalcValue::missing(),
            CalcValue::missing(),
            CalcValue::missing(),
        ],
        &NULL_REFERENCE_SYSTEM_PROVIDER,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(result, CalcValue::error(WorksheetErrorCode::Div0));
}
