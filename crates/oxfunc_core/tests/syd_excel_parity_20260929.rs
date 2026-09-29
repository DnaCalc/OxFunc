//! Public Excel build 20430/CV2 exact observations, W111 SYD discovery.
use oxfunc_core::functions::depreciation_family::syd_kernel;
use oxfunc_core::functions::surface_dispatch::eval_surface_value_call;
use oxfunc_core::resolver::NULL_REFERENCE_SYSTEM_PROVIDER;
use oxfunc_core::value::{CalcArray, CalcValue, WorksheetErrorCode};

#[test]
fn syd_accepts_negative_cost_but_rejects_negative_salvage() {
    assert_eq!(syd_kernel(-100., 0., 10., 1.).unwrap().to_bits(), 0xc0322e8ba2e8ba2f);
    assert_eq!(syd_kernel(100., -20., 10., 1.), Err(WorksheetErrorCode::Num));
    assert_eq!(syd_kernel(100., 0., 10., 0.), Err(WorksheetErrorCode::Num));
}

#[test]
fn syd_publishes_internal_stages_before_the_quotient() {
    assert_eq!(syd_kernel(100., 0., f64::MIN_POSITIVE, f64::MIN_POSITIVE), Err(WorksheetErrorCode::Div0));
    assert_eq!(syd_kernel(100., 0., 1e155, 5e154), Err(WorksheetErrorCode::Div0));
    assert_eq!(syd_kernel(-f64::MAX, f64::MAX, 1., 1.), Err(WorksheetErrorCode::Num));
    assert_eq!(syd_kernel(-f64::MAX, f64::MAX, 1e155, 1e155), Err(WorksheetErrorCode::Div0));
    assert_eq!(syd_kernel(100., 0., 1e154, 1e154), Ok(0.));
    assert_eq!(syd_kernel(1e-300, 0., 1e-300, 1e-300), Ok(2.));
}

#[test]
fn syd_addition_and_subtraction_use_extended_stores() {
    assert_eq!(
        syd_kernel(1., f64::from_bits(0x3c90000000000001), 1., 1.).unwrap(),
        1.
    );
    assert_eq!(
        syd_kernel(1., 0., f64::from_bits(0x3ca0000000000001), f64::from_bits(0x3c90000000000001))
            .unwrap().to_bits(),
        0x434ffffffffffffe
    );
}

#[test]
fn syd_multiply_and_divide_require_extended_stores() {
    for (args, expected) in [
        ([0x3e289ba1b6da0a72, 0x3e2633265543da78, 0x3fd78050f2c28988, 0x3fbd6e5a6557cf9f], 0x3e1806afc487fbb4),
        ([0x45ed99352f82c439, 0x45e2edda0775b48c, 0x3fbcbd8acd5bd3ae, 0x3fb178938a12195b], 0x46164d12da5ea758),
        ([0x39576175f0ee3d26, 0x3920832bf9202f48, 0x4058870d62695183, 0x403bb490f8d161f6], 0x38f4095fa56e4c00),
    ] {
        let a = args.map(f64::from_bits);
        assert_eq!(syd_kernel(a[0], a[1], a[2], a[3]).unwrap().to_bits(), expected);
    }
}

#[test]
fn syd_missing_and_array_values_follow_scalar_coercion() {
    let result = eval_surface_value_call(
        "FUNC.SYD",
        &[
            CalcValue::array(CalcArray::from_rows(vec![vec![
                CalcValue::missing(), CalcValue::number(-100.),
                CalcValue::error(WorksheetErrorCode::Ref),
            ]]).unwrap()),
            CalcValue::number(0.), CalcValue::number(10.), CalcValue::number(1.),
        ],
        &NULL_REFERENCE_SYSTEM_PROVIDER, None, None, None, None,
    ).unwrap();
    assert_eq!(result, CalcValue::array(CalcArray::from_rows(vec![vec![
        CalcValue::number(0.), CalcValue::number(f64::from_bits(0xc0322e8ba2e8ba2f)),
        CalcValue::error(WorksheetErrorCode::Ref),
    ]]).unwrap()));
}
