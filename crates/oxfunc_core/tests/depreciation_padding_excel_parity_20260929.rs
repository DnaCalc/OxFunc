//! Public Excel financial padding observations: earlier errors retain argument order.
use oxfunc_core::functions::surface_dispatch::eval_surface_value_call;
use oxfunc_core::resolver::NULL_REFERENCE_SYSTEM_PROVIDER;
use oxfunc_core::value::{CalcArray, CalcValue, CoreValue, WorksheetErrorCode};

#[test]
fn financial_padding_does_not_overwrite_an_earlier_argument_error() {
    for (name, mut args) in [
        ("FUNC.SLN", vec![1000., 100., 10.]),
        ("FUNC.SYD", vec![1000., 100., 10., 2.]),
        ("FUNC.DB", vec![1000., 100., 10., 2., 6.]),
        ("FUNC.DDB", vec![1000., 100., 10., 2., 2.]),
        ("FUNC.VDB", vec![1000., 100., 10., 1., 2., 2., 1.]),
    ] {
        let mut values: Vec<_> = args.drain(..).map(CalcValue::number).collect();
        if name == "FUNC.VDB" { values[6] = CalcValue::logical(true); }
        values[0] = CalcValue::array(CalcArray::from_rows(vec![
            vec![CalcValue::number(1000.), CalcValue::number(1000.), CalcValue::error(WorksheetErrorCode::Ref)],
            vec![CalcValue::number(1000.), CalcValue::number(1000.), CalcValue::error(WorksheetErrorCode::Ref)],
        ]).unwrap());
        values[1] = CalcValue::array(CalcArray::from_rows(vec![
            vec![CalcValue::number(100.), CalcValue::number(100.)],
            vec![CalcValue::number(100.), CalcValue::number(100.)],
        ]).unwrap());
        let result = eval_surface_value_call(name, &values, &NULL_REFERENCE_SYSTEM_PROVIDER, None, None, None, None).unwrap();
        let CoreValue::Array(array) = result.core() else { panic!("{name}: expected array") };
        assert_eq!(array.get(0, 2), Some(&CalcValue::error(WorksheetErrorCode::Ref)), "{name}");
        assert_eq!(array.get(1, 2), Some(&CalcValue::error(WorksheetErrorCode::Ref)), "{name}");
    }
}
