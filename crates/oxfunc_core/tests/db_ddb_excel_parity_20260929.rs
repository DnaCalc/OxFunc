//! Exact public Excel Value2 observations, build 20430/CV2, W111 discovery.
use oxfunc_core::functions::depreciation_family::{db_kernel, ddb_kernel, vdb_kernel};
use oxfunc_core::functions::surface_dispatch::eval_surface_value_call;
use oxfunc_core::resolver::NULL_REFERENCE_SYSTEM_PROVIDER;
use oxfunc_core::value::{CalcArray, CalcValue, CoreValue, WorksheetErrorCode};

#[test]
fn db_period_month_and_book_value_graph() {
    for (cost, salvage, life, period, month, bits) in [
        (1000., 100., 0.5, 0.5, 1., 0x4054a00000000000),
        (1000., 100., 0.5, 1.99, 1.5, 0x408a050ccccccccc),
        (1000., 100., 0.5, 1., 6., 0x406f3f3333333334),
        (1000., 100., 10., 11., 1., 0x40374721db621c07),
        (1., 2000., 10., 10., 12., 0xc09098da01837956),
        (0., 2000., 10., 2., 12., 0),
    ] {
        assert_eq!(
            db_kernel(cost, salvage, life, period, month)
                .unwrap()
                .to_bits(),
            bits
        );
    }
    assert_eq!(
        db_kernel(1000., 100., 10., 11., 12.),
        Err(WorksheetErrorCode::Num)
    );
    assert_eq!(
        db_kernel(1000., 100., 10., 1., 0.5),
        Err(WorksheetErrorCode::Num)
    );
}

#[test]
fn ddb_fractional_period_and_integer_power_publication() {
    for (cost, salvage, life, period, factor, bits) in [
        (1000., 100., 10., 0.5, 2., 0x4069000000000000),
        (1000., 100., 10., 10., 2., 0x403ad7f29abcaf4f),
        (1000., 100., 10., 10., 0.5, 0x403f8331440a866c),
        (1000., 100., 10., 10., 1., 0x40435efb7556c434),
        (1., 0., 10., 10., 2., 0x3f9b7cdfd9d7bdc2),
    ] {
        assert_eq!(
            ddb_kernel(cost, salvage, life, period, factor)
                .unwrap()
                .to_bits(),
            bits
        );
    }
    assert_eq!(ddb_kernel(1000., 2000., 10., 1., 2.), Ok(0.));
    assert_eq!(
        ddb_kernel(1000., 100., 10., 11., 2.),
        Err(WorksheetErrorCode::Num)
    );
}

#[test]
fn depreciation_surface_missing_and_lifted_errors() {
    for fn_id in ["FUNC.DB", "FUNC.DDB"] {
        let cost = CalcArray::from_rows(vec![vec![
            CalcValue::missing(),
            CalcValue::error(WorksheetErrorCode::NA),
        ]])
        .unwrap();
        let result = eval_surface_value_call(
            fn_id,
            &[
                CalcValue::array(cost),
                CalcValue::missing(),
                CalcValue::number(10.),
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
            panic!("{result:?}")
        };
        assert_eq!(array.get(0, 0), Some(&CalcValue::number(0.)));
        assert_eq!(
            array.get(0, 1),
            Some(&CalcValue::error(WorksheetErrorCode::NA))
        );
    }
}

#[test]
fn explicit_optional_missing_is_zero_and_vdb_switch_is_logical() {
    for fn_id in ["FUNC.DB", "FUNC.DDB"] {
        let result = eval_surface_value_call(
            fn_id,
            &[
                CalcValue::number(1000.),
                CalcValue::number(100.),
                CalcValue::number(10.),
                CalcValue::number(2.),
                CalcValue::missing(),
            ],
            &NULL_REFERENCE_SYSTEM_PROVIDER,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        assert_eq!(result, CalcValue::error(WorksheetErrorCode::Num));
    }
    let call = |factor, flag| {
        eval_surface_value_call(
            "FUNC.VDB",
            &[
                CalcValue::number(1000.),
                CalcValue::number(100.),
                CalcValue::number(10.),
                CalcValue::number(1.),
                CalcValue::number(2.),
                factor,
                flag,
            ],
            &NULL_REFERENCE_SYSTEM_PROVIDER,
            None,
            None,
            None,
            None,
        )
        .unwrap_or_else(CalcValue::error)
    };
    assert_eq!(
        call(CalcValue::missing(), CalcValue::logical(false)),
        CalcValue::number(90.)
    );
    let text = |s: &str| {
        CalcValue::text(oxfunc_core::value::ExcelText::from_utf16_code_units(
            s.encode_utf16().collect(),
        ))
    };
    assert_eq!(
        call(CalcValue::number(2.), text("TRUE")),
        CalcValue::number(160.)
    );
    assert_eq!(
        call(CalcValue::number(2.), text("2")),
        CalcValue::error(WorksheetErrorCode::Value)
    );
}

#[test]
fn db_rounding_substrates_and_ddb_overflow_precedence() {
    // A ratio overflow produces DB rate 1, then the regular book recurrence.
    assert_eq!(
        db_kernel(1e-300, 1e300, 10., 1., 12.).unwrap().to_bits(),
        1e-300f64.to_bits()
    );
    assert_eq!(db_kernel(1e-300, 1e300, 10., 3., 1.).unwrap().to_bits(), 0);
    assert_eq!(
        ddb_kernel(1e100, 1e266, 10., 1., 1e240),
        Err(WorksheetErrorCode::Num)
    );
    assert_eq!(ddb_kernel(1e100, 1e266, 10., 2., 1e240), Ok(0.));
    assert_eq!(
        ddb_kernel(
            904591238.3917238,
            817304558.7741821,
            25.598408815758944,
            1.0000000000000002,
            25.59840881575894
        )
        .unwrap()
        .to_bits(),
        4725630119395744496
    );
}

#[test]
fn db_host_count_conversion_and_power_overflow_branches() {
    assert_eq!(
        db_kernel(1000., 0., 2147483648., 0.5, 6.),
        Err(WorksheetErrorCode::Num)
    );
    assert_eq!(db_kernel(1000., 0., 4294967296., 0.5, 6.), Ok(250.));
    assert_eq!(db_kernel(1000., 0., 4294967296., 1e20, 6.), Ok(500.));
    assert_eq!(db_kernel(1., 1e300, 0.5, 0.5, 6.), Ok(0.5));
    assert_eq!(
        db_kernel(1., 1e300, 0.4, 0.5, 6.),
        Err(WorksheetErrorCode::Num)
    );
    assert_eq!(
        db_kernel(1., 1e300, 2.3283064365386963e-10, 0.5, 6.),
        Err(WorksheetErrorCode::Num)
    );
    // A subnormal rate must not be rescued by a subsequent large cost product.
    assert_eq!(
        ddb_kernel(1e300, 0., 2., 1., 4.450147717014402e-308)
            .unwrap()
            .to_bits(),
        0
    );
}

#[test]
fn vdb_no_switch_rounds_partial_products_separately() {
    for (start, end, bits) in [
        (0.25, 0.375, 0x4054d55555555556),
        (0.5, 0.625, 0x4054d55555555554),
    ] {
        assert_eq!(
            vdb_kernel(1000., 100., 0.75, start, end, 0.5, true)
                .unwrap()
                .to_bits(),
            bits
        );
    }
    assert_eq!(vdb_kernel(1000., 100., 1., 0.5, 1., 2., true), Ok(450.));
    assert_eq!(vdb_kernel(1000., 100., 0.75, 0., 0.5, 0., true), Ok(0.));
}

#[test]
fn db_extended_book_subtraction_and_excluded_power_exponent() {
    assert_eq!(
        db_kernel(
            1.663307456757216e-274,
            2.2928650922546898e-54,
            48.38050540006145,
            32.435174011596814,
            1.7384173064343713
        )
        .unwrap()
        .to_bits(),
        0xa518ca7d821c514f
    );
    assert_eq!(
        db_kernel(1., 1.25, 2.3283064370807974e-10, 0.234, 7.123),
        Err(WorksheetErrorCode::Num)
    );
}

#[test]
fn vdb_no_switch_forms_final_fraction_from_unused_tail() {
    let a = [
        0x6748c1b4142f9949,
        0x6745a00e7c9beecb,
        0x3fefc80a0be6ce60,
        0x3f9a45a5eb707908,
        0x3f9a45a5eb70790b,
        0x3ffea746c249ae38,
    ]
    .map(f64::from_bits);
    assert_eq!(
        vdb_kernel(a[0], a[1], a[2], a[3], a[4], a[5], true)
            .unwrap()
            .to_bits(),
        16404361642697031680
    );
}

#[test]
fn vdb_adjacent_partial_products_use_extended_stores() {
    let a = [
        0x25a47d7ffa4b822c,
        0x25a18652dbe780f0,
        0x4033392a2f6f4875,
        0x3fc1d3ba730acb96,
        0x3fc1d3ba730acb98,
        0x4008768467f00c94,
    ]
    .map(f64::from_bits);
    assert_eq!(
        vdb_kernel(a[0], a[1], a[2], a[3], a[4], a[5], true)
            .unwrap()
            .to_bits(),
        0x2218000000000000
    );
}

#[test]
fn vdb_final_subtraction_preserves_observed_subnormal() {
    let a = [
        4.3557019575870734e-44,
        1.8333906020999901e-44,
        17.80270473530464,
        17.735922290315518,
        17.761093171442802,
        6.523203775810138e-263,
    ];
    assert_eq!(
        vdb_kernel(a[0], a[1], a[2], a[3], a[4], a[5], true).unwrap().to_bits(),
        0x0002e383ddabdbd8
    );
}

#[test]
fn ddb_declining_product_uses_extended_precision() {
    let a = [
        0x3bdfbb6613f33669,
        0x3bd42f17f051f175,
        0x403088db8b909b65,
        0x3ff0000000000000,
        0x401118ab93c791ca,
    ]
    .map(f64::from_bits);
    assert_eq!(
        ddb_kernel(a[0], a[1], a[2], a[3], a[4]).unwrap().to_bits(),
        4305555259651307774
    );
}

#[test]
fn db_internal_nan_rate_differs_from_power_underflow_and_overflow() {
    assert_eq!(
        db_kernel(1., 1e-300, 1e-306, 0.375, 9.).unwrap().to_bits(),
        0
    );
    assert_eq!(db_kernel(1., 1e-300, 1e-304, 0.375, 9.), Ok(0.75));
    assert_eq!(
        db_kernel(1., 1e300, 1e-306, 0.375, 9.).unwrap().to_bits(),
        0
    );
    assert_eq!(
        db_kernel(1., 1e300, 1e-304, 0.375, 9.),
        Err(WorksheetErrorCode::Num)
    );
}

#[test]
fn db_first_and_final_product_store_discriminators() {
    for (raw, expected) in [
        (
            [
                0x6b7af23d73312313,
                0x6bb5e31b33355bf5,
                0x4000000000000000,
                0x3ff0000000000000,
                0x3ff0000000000000,
            ],
            0xeb5765f8c5bb4eaa,
        ),
        (
            [
                0x15cabe23de1fbe2e,
                0x161bf55a19136a01,
                0x4000000000000000,
                0x3ff0000000000000,
                0x4014000000000000,
            ],
            0x95daa751d3ff35de,
        ),
        (
            [
                0x0fa7a758a292601f,
                0x0fc5e8c3b2bdfaf4,
                0x3ff0000000000000,
                0x4000000000000000,
                0x4010000000000000,
            ],
            0x8fc4477565686ee8,
        ),
    ] {
        let a = raw.map(f64::from_bits);
        assert_eq!(
            db_kernel(a[0], a[1], a[2], a[3], a[4]).unwrap().to_bits(),
            expected
        );
    }
}

#[test]
fn db_depreciation_is_published_before_book_update() {
    let a = [
        0x0027e4c8056507a8,
        0x0adbe4b8951ad783,
        0x40654b3fd2fc61bc,
        0x4064bfcfb019b07d,
        0x4001ac80b239e566,
    ]
    .map(f64::from_bits);
    assert_eq!(
        db_kernel(a[0], a[1], a[2], a[3], a[4]).unwrap().to_bits(),
        9972450844786598181
    );
    let cost = f64::MIN_POSITIVE;
    assert_eq!(
        db_kernel(cost, cost * 1.5f64.powi(5), 5., 4., 1.)
            .unwrap()
            .to_bits(),
        0
    );
}

#[test]
fn depreciation_internal_power_count_and_accumulator_stores() {
    // DB's rounded rate distinguishes the internal 32-bit dispatch from POWER.
    let a = [
        0x3ff0000000000000,
        0x3feffffffffffe00,
        0x3dc4d15513c7bbbd,
        0x3fd8000000000000,
        0x4018000000000000,
    ]
    .map(f64::from_bits);
    assert_eq!(
        db_kernel(a[0], a[1], a[2], a[3], a[4]).unwrap().to_bits(),
        0x3f40624dd2f1a9fc
    );
    // The first DDB witness pins the integer accumulator's x87 store; the
    // second pins the exp/log route above the unsigned-count sentinel.
    for (raw, expected) in [
        (
            [
                0x3ff0000000000000,
                0x0000000000000000,
                0x41dfffffffc00000,
                0x41dfffffff800000,
                0x4024000000000000,
            ],
            0x3d4dc0d813a69ce6,
        ),
        (
            [
                0x3ff0000000000000,
                0x0000000000000000,
                0x4340000000000000,
                0x4340000000000000,
                0x4024000000000000,
            ],
            0x3bedc0d822bd9ba4,
        ),
    ] {
        let a = raw.map(f64::from_bits);
        assert_eq!(
            ddb_kernel(a[0], a[1], a[2], a[3], a[4]).unwrap().to_bits(),
            expected
        );
    }
}

#[test]
fn db_final_quotient_and_rate_product_are_published_before_recovery() {
    for raw in [
        [
            0x00162f3296c3e592,
            0x00362f3296c3e592,
            0x3ff0000000000000,
            0x4000000000000000,
            0x4024000000000000,
        ],
        [
            0x005a0b8142dea4af,
            0x00677787ba692cf0,
            0x4014000000000000,
            0x4018000000000000,
            0x4010000000000000,
        ],
    ] {
        let a = raw.map(f64::from_bits);
        assert_eq!(
            db_kernel(a[0], a[1], a[2], a[3], a[4]).unwrap().to_bits(),
            0
        );
    }
}

#[test]
fn ddb_large_fractional_period_exposes_base_subtraction_store() {
    let a = [
        0x2650bc19031d0098,
        0x0000000000000000,
        0x41c4bc793de17e43,
        0x41c4bc793de17e43,
        0x4024000000000000,
    ]
    .map(f64::from_bits);
    assert_eq!(
        ddb_kernel(a[0], a[1], a[2], a[3], a[4]).unwrap().to_bits(),
        0x23c80305a78d7075
    );
}
