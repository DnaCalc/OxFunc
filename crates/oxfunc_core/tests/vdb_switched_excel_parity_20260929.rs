//! Reduced public Excel Value2 witnesses from retained W111 failed/fresh cohorts.
//! Numeric inputs are original IEEE hex; equality includes nonzero subnormals.
use oxfunc_core::functions::surface_dispatch::eval_surface_value_call;
use oxfunc_core::resolver::NULL_REFERENCE_SYSTEM_PROVIDER;
use oxfunc_core::value::{CalcValue, CoreValue, WorksheetErrorCode};

fn check(args: &[u64], expected: Result<u64, WorksheetErrorCode>) {
    let values: Vec<_> = args
        .iter()
        .map(|bits| CalcValue::number(f64::from_bits(*bits)))
        .collect();
    let got = eval_surface_value_call(
        "FUNC.VDB",
        &values,
        &NULL_REFERENCE_SYSTEM_PROVIDER,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    match (got.core(), expected) {
        (CoreValue::Number(value), Ok(bits)) => assert_eq!(value.to_bits(), bits, "args={args:x?}"),
        (CoreValue::Error(error), Err(want)) => assert_eq!(*error, want, "args={args:x?}"),
        pair => panic!("wrong result kind {pair:?}, args={args:x?}"),
    }
}

#[test]
fn switched_vdb_fractional_cursor() {
    // w111vdb-general-independent-03255
    check(
        &[
            0x37ed6424f6a0cf14,
            0x3815bce117a42154,
            0x3fef45ede58066c6,
            0x3fd56f481e27ec1c,
            0x3feae91f71f3e6ab,
            0x4000000000000000,
            0x0000000000000000,
        ],
        Ok(0xb8024839e169ef5c),
    );
    // w111vdb-general-independent-04211
    check(
        &[
            0x4a1199fae464d8ef,
            0x4a1199fae464d8ee,
            0x3fe4ed5cecb52239,
            0x3fbed55d0079ad30,
            0x3fceb660f2210928,
            0x0000000000000000,
            0x0000000000000000,
        ],
        Ok(0x46a7638891ffafe2),
    );
    // w111vdb-general-independent-04849
    check(
        &[
            0x4937ad196b14dad2,
            0x4933fcb6ed2a5d5e,
            0x4041540fae6125ce,
            0x3fe8290874c98230,
            0x4022cfceb12002a8,
            0x0000000000000000,
            0x0000000000000000,
        ],
        Ok(0x48ed77772288692e),
    );
    // w111vdb-general-independent-05129
    check(
        &[
            0x30507accf46c7fa5,
            0x30507accf46c7fa4,
            0x403223af1474b00f,
            0x3fdffe0b58e433bc,
            0x402f3832997baf68,
            0x40210a47ff6a7425,
            0x0000000000000000,
        ],
        Ok(0x0000000000000000),
    );
    // w111vdb-general-independent-15991
    check(
        &[
            0x41c6a4abe3b9442a,
            0x41c5f7f9d51118c3,
            0x40379e7792105073,
            0x3fed8673e0373101,
            0x3ffec339f01b9881,
            0x3fd0000000000000,
            0x0000000000000000,
        ],
        Ok(0x415e60de336f4b4a),
    );
}

#[test]
fn switched_vdb_below_salvage() {
    // w111vdb-cursor-independent-09685
    check(
        &[
            0x3b3279d807a14e37,
            0x3b4fb7e6e9aa9f5b,
            0x4048cf3b7f3a4eed,
            0x4045f8a92a4035dd,
            0x4048c6a8db4d5a6d,
            0x0000000000000000,
            0x0000000000000000,
        ],
        Ok(0x37ce1ea55cdb9aa3),
    );
    // w111vdb-cursor-independent-09899
    check(
        &[
            0x362839702f5ec8db,
            0x363f788b54a90925,
            0x4049a1eef243509d,
            0x40426dea9f901b59,
            0x40434696906e04c1,
            0x0000000000000000,
            0x0000000000000000,
        ],
        Ok(0x32a18ac0477e27d8),
    );
    // w111vdb-cursor-independent-09954
    check(
        &[
            0x3afc74d6e67507da,
            0x3b1bde61bc60c367,
            0x4022ae0ea5e22607,
            0x3fd6a2f34a48d964,
            0x401c41b3ef7bccb2,
            0x4000000000000000,
            0x0000000000000000,
        ],
        Ok(0xbb14c12c02c38171),
    );
    // w111vdb-cursor-independent-09959
    check(
        &[
            0x3d892a343c22bf6e,
            0x3da7f820739c6e3f,
            0x4029526227df5474,
            0x4021336336538e3f,
            0x4021da2d7a69a01e,
            0x0000000000000000,
            0x0000000000000000,
        ],
        Ok(0x3a0e28d44ba17614),
    );
    // w111vdb-cursor-independent-10385
    check(
        &[
            0x48d6c325f46597cf,
            0x48eeeee15b5b54c1,
            0x404735b2a3f64f49,
            0x3ff350c705499c93,
            0x403e7b8b4a58a26c,
            0x0000000000000000,
            0x0000000000000000,
        ],
        Ok(0x4594b86517481cf9),
    );
    // w111vdb-cursor-independent-10718
    check(
        &[
            0x3a6b3cfed21cf476,
            0x3a88a4bbde6dbb4f,
            0x404c1f8bba3d2c96,
            0x4048d85688e7d2f2,
            0x404a46c1e94d6cdb,
            0x0000000000000000,
            0x0000000000000000,
        ],
        Ok(0x36fadd8ba169dd60),
    );
}

#[test]
fn switched_vdb_publication_and_clipping() {
    // w111vdb-publication-independent-08039
    check(
        &[
            0x001b00b3c87c2144,
            0x0020fac6b213cb21,
            0x5613a18b0163bf88,
            0x0000000000000000,
            0x408c200000000000,
            0x402590a52fb5b809,
            0x0000000000000000,
        ],
        Ok(0x8006f4d99bab74fe),
    );
    // w111vdb-publication-independent-08134
    check(
        &[
            0x00bb9705d6aa16e7,
            0x00bb9705d6aa16e6,
            0x403e049552c3e7e1,
            0x3fe9c085c9e9d59c,
            0x4030fa455f21923a,
            0x40163a25be2f6d65,
            0x0000000000000000,
        ],
        Ok(0x0000000000000400),
    );
    // w111vdb-publication-independent-08136
    check(
        &[
            0x00df155e0307a238,
            0x0000000000000000,
            0x407430734313c2e2,
            0x40532bbd6b23d98c,
            0x407430734313c2e2,
            0x401486af7910d3b6,
            0x0000000000000000,
        ],
        Ok(0x00c2356f8ee55aea),
    );
    // w111vdb-publication-independent-08140
    check(
        &[
            0x00eab8f6acfa977f,
            0x0000000000000000,
            0x40f21f6a65d73429,
            0x4081356f0128278d,
            0x4086db5f85ccb3a6,
            0x401f5cd2c3f51f67,
            0x0000000000000000,
        ],
        Ok(0x008e6481ed405ea3),
    );
    // w111vdb-publication-independent-08278
    check(
        &[
            0x012f6c3f495f93cc,
            0x012f6c3f495f93cb,
            0x405be86a1a2b9eb9,
            0x0000000000000000,
            0x3ff18cdf9648dbc4,
            0x4011d5665dec0256,
            0x0000000000000000,
        ],
        Ok(0x0000000000020000),
    );
    // w111vdb-publication-independent-08282
    check(
        &[
            0x007efc36eeb4043f,
            0x007efc36eeb4043e,
            0x404f7544f411d469,
            0x0000000000000000,
            0x404f7544f411d469,
            0x402575f6070c68af,
            0x0000000000000000,
        ],
        Ok(0x0000000000000040),
    );
    // w111vdb-publication-independent-08382
    check(
        &[
            0x0065712c8842e127,
            0x0065712c8842e126,
            0x4035864698fab518,
            0x0000000000000000,
            0x4013eb0b091ca752,
            0x400cb8851866ef89,
            0x0000000000000000,
        ],
        Ok(0x0000000000000020),
    );
    // w111vdb-publication-independent-08448
    check(
        &[
            0x001af144dd21206e,
            0x001bcc5df08ef736,
            0x7cceef9b94f783e4,
            0x0000000000000000,
            0x408c200000000000,
            0x012ba6aa80fd5065,
            0x0000000000000000,
        ],
        Ok(0x8000db19136dd6c8),
    );
    // w111vdb-publication-independent-12769
    check(
        &[
            0x00f370027b8c891e,
            0x00f370027b8c891d,
            0x407cc67148ef84d6,
            0x0000000000000000,
            0x407b3b9c8fd9a649,
            0x4020a7ef605047eb,
            0x0000000000000000,
        ],
        Ok(0x0000000000004000),
    );
    // w111vdb-publication-independent-12848
    check(
        &[
            0x002ded34e2129a73,
            0x002e30e740cf52c5,
            0x4074b05ba21d5691,
            0x0000000000000000,
            0x406df902815c44b3,
            0x0000000000000000,
            0x0000000000000000,
        ],
        Ok(0x80008764bd7970a4),
    );
    // w111vdb-publication-independent-12952
    check(
        &[
            0x008b7ae0197db747,
            0x008b7ae0197db746,
            0x40797dfa808c00e4,
            0x0000000000000000,
            0x4070b6f56e0858f5,
            0x4008241dba679bb0,
            0x0000000000000000,
        ],
        Ok(0x0000000000000080),
    );
    // w111vdb-publication-independent-12956
    check(
        &[
            0x004a758ad516ab07,
            0x004b6aba320406be,
            0x7c785da870457170,
            0x0000000000000000,
            0x408c200000000000,
            0x0000000000000000,
            0x0000000000000000,
        ],
        Ok(0x8007a97ae76addb8),
    );
    // w111vdb-publication-independent-12957
    check(
        &[
            0x018b3a4ff60c1689,
            0x018b3a4ff60c1688,
            0x40ce6723fefcd53b,
            0x0000000000000000,
            0x40888a68ae364256,
            0x3fff6487ebe519be,
            0x0000000000000000,
        ],
        Ok(0x0000000000800000),
    );
    // w111vdb-publication-independent-13121
    check(
        &[
            0x7d163882a7a27e76,
            0x7fefffffffffffff,
            0x3ff16e863b64af6b,
            0x3fc47b7ebc0bf9ee,
            0x3fd44fe9cc953c8e,
            0x4006621b9a6f48ff,
            0x0000000000000000,
        ],
        Err(WorksheetErrorCode::Num),
    );
}

#[test]
fn switched_vdb_inclusive_switch_threshold() {
    // w111vdb-switch-boundary-0000
    check(
        &[
            0x0030000000000000,
            0x0000000000000000,
            0x4000000000000000,
            0x0000000000000000,
            0x3ff0000000000000,
            0x3feeffffffffffff,
            0x0000000000000000,
        ],
        Ok(0x0020000000000000),
    );
    // w111vdb-switch-boundary-0001
    check(
        &[
            0x0030000000000000,
            0x0000000000000000,
            0x4000000000000000,
            0x3fd0000000000000,
            0x3ff4000000000000,
            0x3feeffffffffffff,
            0x0000000000000000,
        ],
        Ok(0x0020000000000000),
    );
    // w111vdb-switch-boundary-0002
    check(
        &[
            0x0030000000000000,
            0x0000000000000000,
            0x4000000000000000,
            0x3ff0000000000000,
            0x4000000000000000,
            0x3feeffffffffffff,
            0x0000000000000000,
        ],
        Ok(0x0020000000000000),
    );
    // w111vdb-switch-boundary-0003
    check(
        &[
            0x0030000000000000,
            0x0000000000000000,
            0x4000000000000000,
            0x0000000000000000,
            0x3ff0000000000000,
            0x3fef000000000000,
            0x0000000000000000,
        ],
        Ok(0x0020000000000000),
    );
    // w111vdb-switch-boundary-0004
    check(
        &[
            0x0030000000000000,
            0x0000000000000000,
            0x4000000000000000,
            0x3fd0000000000000,
            0x3ff4000000000000,
            0x3fef000000000000,
            0x0000000000000000,
        ],
        Ok(0x0020000000000000),
    );
    // w111vdb-switch-boundary-0005
    check(
        &[
            0x0030000000000000,
            0x0000000000000000,
            0x4000000000000000,
            0x3ff0000000000000,
            0x4000000000000000,
            0x3fef000000000000,
            0x0000000000000000,
        ],
        Ok(0x0020000000000000),
    );
    // w111vdb-switch-boundary-0006
    check(
        &[
            0x0030000000000000,
            0x0000000000000000,
            0x4000000000000000,
            0x0000000000000000,
            0x3ff0000000000000,
            0x3fef000000000001,
            0x0000000000000000,
        ],
        Ok(0x001f000000000001),
    );
    // w111vdb-switch-boundary-0007
    check(
        &[
            0x0030000000000000,
            0x0000000000000000,
            0x4000000000000000,
            0x3fd0000000000000,
            0x3ff4000000000000,
            0x3fef000000000001,
            0x0000000000000000,
        ],
        Ok(0x0022492492492492),
    );
    // w111vdb-switch-boundary-0008
    check(
        &[
            0x0030000000000000,
            0x0000000000000000,
            0x4000000000000000,
            0x3ff0000000000000,
            0x4000000000000000,
            0x3fef000000000001,
            0x0000000000000000,
        ],
        Ok(0x0020800000000000),
    );
    // w111vdb-switch-boundary-0009
    check(
        &[
            0x0030000000000000,
            0x0000000000000000,
            0x4000000000000000,
            0x0000000000000000,
            0x3ff0000000000000,
            0x3feeffffffffe000,
            0x0000000000000000,
        ],
        Ok(0x0020000000000000),
    );
    // w111vdb-switch-boundary-0010
    check(
        &[
            0x0030000000000000,
            0x0000000000000000,
            0x4000000000000000,
            0x3fd0000000000000,
            0x3ff4000000000000,
            0x3feeffffffffe000,
            0x0000000000000000,
        ],
        Ok(0x0020000000000000),
    );
    // w111vdb-switch-boundary-0011
    check(
        &[
            0x0030000000000000,
            0x0000000000000000,
            0x4000000000000000,
            0x3ff0000000000000,
            0x4000000000000000,
            0x3feeffffffffe000,
            0x0000000000000000,
        ],
        Ok(0x0020000000000000),
    );
    // w111vdb-switch-boundary-0012
    check(
        &[
            0x0030000000000000,
            0x0000000000000000,
            0x4000000000000000,
            0x0000000000000000,
            0x3ff0000000000000,
            0x3fef000000002000,
            0x0000000000000000,
        ],
        Ok(0x001f000000002000),
    );
    // w111vdb-switch-boundary-0013
    check(
        &[
            0x0030000000000000,
            0x0000000000000000,
            0x4000000000000000,
            0x3fd0000000000000,
            0x3ff4000000000000,
            0x3fef000000002000,
            0x0000000000000000,
        ],
        Ok(0x0022492492492492),
    );
    // w111vdb-switch-boundary-0014
    check(
        &[
            0x0030000000000000,
            0x0000000000000000,
            0x4000000000000000,
            0x3ff0000000000000,
            0x4000000000000000,
            0x3fef000000002000,
            0x0000000000000000,
        ],
        Ok(0x00207ffffffff000),
    );
    // w111vdb-switch-boundary-0224
    check(
        &[
            0x0170000000000000,
            0x0000000000000000,
            0x4010000000000000,
            0x0000000000000000,
            0x3ff0000000000000,
            0x3fefffffffffffff,
            0x0000000000000000,
        ],
        Ok(0x014fffffffffffff),
    );
    // w111vdb-switch-boundary-0225
    check(
        &[
            0x0170000000000000,
            0x0000000000000000,
            0x4010000000000000,
            0x0000000000000000,
            0x3ff0000000000000,
            0x3ff0000000000000,
            0x0000000000000000,
        ],
        Ok(0x0150000000000000),
    );
    // w111vdb-switch-boundary-0226
    check(
        &[
            0x0170000000000000,
            0x0000000000000000,
            0x4010000000000000,
            0x0000000000000000,
            0x3ff0000000000000,
            0x3ff0000000000001,
            0x0000000000000000,
        ],
        Ok(0x0150000000000001),
    );
    // w111vdb-switch-boundary-0227
    check(
        &[
            0x0170000000000000,
            0x0000000000000000,
            0x4010000000000000,
            0x0000000000000000,
            0x3ff0000000000000,
            0x3fefffffff800000,
            0x0000000000000000,
        ],
        Ok(0x014fffffff800000),
    );
}
