//! Public-dispatch golden observations across MOD remainder and quotient branches.
use oxfunc_core::functions::surface_dispatch::eval_surface_value_call;
use oxfunc_core::resolver::NULL_REFERENCE_SYSTEM_PROVIDER;
use oxfunc_core::value::{CalcValue, CoreValue};

#[test]
fn retained_mod_publication_witnesses_match_public_dispatch() {
    let bank: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/w111_mod_publication.json")).unwrap();
    for row in bank["witnesses"].as_array().unwrap() {
        let args: Vec<_> = row["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|arg| {
                let bits = u64::from_str_radix(arg.as_str().unwrap().trim_start_matches("0x"), 16)
                    .unwrap();
                CalcValue::number(f64::from_bits(bits))
            })
            .collect();
        let actual = match eval_surface_value_call(
            "FUNC.MOD",
            &args,
            &NULL_REFERENCE_SYSTEM_PROVIDER,
            None,
            None,
            None,
            None,
        ) {
            Ok(value) => match value.core() {
                CoreValue::Number(n) => format!("0x{:016x}", n.to_bits()),
                CoreValue::Error(code) => format!("error:{code:?}"),
                other => panic!("unexpected {other:?}"),
            },
            Err(code) => format!("error:{code:?}"),
        };
        assert_eq!(
            actual,
            row["expected_bits"].as_str().unwrap(),
            "{} {:?}",
            row["id"],
            args
        );
    }
}

#[test]
fn explicit_missing_zero_and_left_error_priority_match_both_routes() {
    use oxfunc_core::functions::mod_fn::{eval_mod_surface, map_mod_error_to_ws};
    use oxfunc_core::value::{ExcelText, WorksheetErrorCode as E};
    let text = || CalcValue::text(ExcelText::from_interop_assignment("x"));
    let cases = vec![
        (
            vec![CalcValue::missing(), CalcValue::number(2.25)],
            CalcValue::number(0.0),
        ),
        (
            vec![CalcValue::number(2.25), CalcValue::missing()],
            CalcValue::error(E::Div0),
        ),
        (
            vec![CalcValue::missing(), CalcValue::missing()],
            CalcValue::error(E::Div0),
        ),
        (
            vec![CalcValue::missing(), CalcValue::error(E::Ref)],
            CalcValue::error(E::Ref),
        ),
        (
            vec![text(), CalcValue::missing()],
            CalcValue::error(E::Value),
        ),
        (
            vec![CalcValue::missing(), text()],
            CalcValue::error(E::Value),
        ),
        (
            vec![text(), CalcValue::error(E::Ref)],
            CalcValue::error(E::Value),
        ),
        (
            vec![CalcValue::error(E::Ref), text()],
            CalcValue::error(E::Ref),
        ),
        (
            vec![CalcValue::empty(), CalcValue::number(2.25)],
            CalcValue::number(0.0),
        ),
    ];
    for (args, expected) in cases {
        let direct = eval_mod_surface(&args, &NULL_REFERENCE_SYSTEM_PROVIDER)
            .unwrap_or_else(|e| CalcValue::error(map_mod_error_to_ws(&e)));
        let generated = eval_surface_value_call(
            "FUNC.MOD",
            &args,
            &NULL_REFERENCE_SYSTEM_PROVIDER,
            None,
            None,
            None,
            None,
        )
        .unwrap_or_else(CalcValue::error);
        assert_eq!(direct, expected, "direct {args:?}");
        assert_eq!(generated, expected, "generated {args:?}");
    }
}

#[test]
fn unequal_array_padding_preserves_present_left_error() {
    use oxfunc_core::value::{CalcArray, WorksheetErrorCode as E};
    let column = |values: Vec<CalcValue>| {
        CalcValue::array(
            CalcArray::from_rows(values.into_iter().map(|v| vec![v]).collect()).unwrap(),
        )
    };
    let left = column(vec![
        CalcValue::number(1.0),
        CalcValue::number(2.0),
        CalcValue::error(E::Value),
    ]);
    let right = column(vec![CalcValue::number(3.0), CalcValue::number(4.0)]);
    let actual = eval_surface_value_call(
        "FUNC.MOD",
        &[left, right],
        &NULL_REFERENCE_SYSTEM_PROVIDER,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(
        actual,
        column(vec![
            CalcValue::number(1.0),
            CalcValue::number(2.0),
            CalcValue::error(E::Value)
        ])
    );
}
