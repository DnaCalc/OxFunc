use oxfunc_core::functions::bond_core_family::{pricemat_kernel, yieldmat_kernel};
use oxfunc_core::functions::cashflow_rate_family::{eval_xirr_surface, xnpv_kernel};
use oxfunc_core::functions::coupon_family::{coupdaybs_kernel, coupdays_kernel, coupdaysnc_kernel};
use oxfunc_core::functions::financial_time_value_family::{PaymentTiming, pmt, rate};
use oxfunc_core::functions::odd_bond_family::{
    oddfyield_kernel, oddlprice_kernel, oddlyield_kernel,
};
use oxfunc_core::locale_format::{WorkbookDateSystem, excel_serial_from_ymd};
use oxfunc_core::resolver::{RefResolutionError, ReferenceResolver, ResolverCapabilities};
use oxfunc_core::value::{ArrayCellValue, CallArgValue, EvalArray, EvalValue, WorksheetErrorCode};

struct DummyResolver;

impl ReferenceResolver for DummyResolver {
    fn capabilities(&self) -> ResolverCapabilities {
        ResolverCapabilities::permissive_local()
    }

    fn resolve_reference(
        &self,
        _reference: &oxfunc_core::value::ReferenceLike,
    ) -> Result<EvalValue, RefResolutionError> {
        Err(RefResolutionError::UnresolvedReference {
            target: "w29_dummy".to_string(),
        })
    }
}

fn serial(y: i64, m: i64, d: i64) -> f64 {
    excel_serial_from_ymd(WorkbookDateSystem::System1900, y, m, d).unwrap()
}

fn array_arg(values: &[f64]) -> CallArgValue {
    let row = values
        .iter()
        .copied()
        .map(ArrayCellValue::Number)
        .collect::<Vec<_>>();
    CallArgValue::Eval(EvalValue::Array(EvalArray::from_rows(vec![row]).unwrap()))
}

fn print_num(case_id: &str, source: &str, value: f64) {
    println!("{case_id},{source},{:.15}", value);
}

fn print_ws_error(case_id: &str, source: &str, code: WorksheetErrorCode) {
    println!("{case_id},{source},#{code:?}");
}

fn main() {
    println!("case_id,source,observable");

    let coupon_settlement = serial(2012, 1, 1);
    let coupon_maturity = serial(2016, 2, 29);
    let coupdays = coupdays_kernel(coupon_settlement, coupon_maturity, 2.0, Some(1.0)).unwrap();
    let coupdaybs = coupdaybs_kernel(coupon_settlement, coupon_maturity, 2.0, Some(1.0)).unwrap();
    let coupdaysnc = coupdaysnc_kernel(coupon_settlement, coupon_maturity, 2.0, Some(1.0)).unwrap();
    print_num("coupon_leap_coupdays", "oxfunc", coupdays);
    print_num("coupon_leap_coupdaybs", "oxfunc", coupdaybs);
    print_num("coupon_leap_coupdaysnc", "oxfunc", coupdaysnc);
    print_num(
        "coupon_leap_diff",
        "oxfunc",
        coupdays - coupdaybs - coupdaysnc,
    );

    let xnpv_values_1 = [206_101_714.849_377, -156_650_972.542_65];
    let xnpv_dates = [serial(2000, 2, 29) as i64, serial(2000, 3, 31) as i64];
    match xnpv_kernel(-0.960_452_195, &xnpv_values_1, &xnpv_dates) {
        Ok(v) => print_num("xnpv_negative_rate_case1", "oxfunc", v),
        Err(code) => print_ws_error("xnpv_negative_rate_case1", "oxfunc", code),
    }
    match xnpv_kernel(-0.960_452_189, &xnpv_values_1, &xnpv_dates) {
        Ok(v) => print_num("xnpv_negative_rate_case2", "oxfunc", v),
        Err(code) => print_ws_error("xnpv_negative_rate_case2", "oxfunc", code),
    }

    let resolver = DummyResolver;
    let xirr_values_1 = array_arg(&xnpv_values_1);
    let xirr_dates_1 = array_arg(&[serial(2000, 2, 29), serial(2000, 3, 31)]);
    match eval_xirr_surface(
        &[
            xirr_values_1.clone(),
            xirr_dates_1.clone(),
            CallArgValue::Eval(EvalValue::Number(-0.1)),
        ],
        &resolver,
    ) {
        Ok(EvalValue::Number(v)) => print_num("xirr_negative_rate_case1_guess_neg", "oxfunc", v),
        Ok(EvalValue::Error(code)) => {
            print_ws_error("xirr_negative_rate_case1_guess_neg", "oxfunc", code)
        }
        Err(err) => println!("xirr_negative_rate_case1_guess_neg,oxfunc,ERR:{err:?}"),
        _ => println!("xirr_negative_rate_case1_guess_neg,oxfunc,ERR:unexpected"),
    }

    let xirr_values_2 = array_arg(&[15_108_163.384_092_3, -75_382_259.662_842_4]);
    let xirr_dates_2 = array_arg(&[serial(2000, 2, 29), serial(2000, 3, 31)]);
    match eval_xirr_surface(
        &[
            xirr_values_2.clone(),
            xirr_dates_2.clone(),
            CallArgValue::Eval(EvalValue::Number(-0.1)),
        ],
        &resolver,
    ) {
        Ok(EvalValue::Number(v)) => print_num("xirr_negative_rate_case2_guess_neg", "oxfunc", v),
        Ok(EvalValue::Error(code)) => {
            print_ws_error("xirr_negative_rate_case2_guess_neg", "oxfunc", code)
        }
        Err(err) => println!("xirr_negative_rate_case2_guess_neg,oxfunc,ERR:{err:?}"),
        _ => println!("xirr_negative_rate_case2_guess_neg,oxfunc,ERR:unexpected"),
    }
    match eval_xirr_surface(
        &[
            xirr_values_2,
            xirr_dates_2,
            CallArgValue::Eval(EvalValue::Number(0.1)),
        ],
        &resolver,
    ) {
        Ok(EvalValue::Number(v)) => print_num("xirr_negative_rate_case2_guess_pos", "oxfunc", v),
        Ok(EvalValue::Error(code)) => {
            print_ws_error("xirr_negative_rate_case2_guess_pos", "oxfunc", code)
        }
        Err(err) => println!("xirr_negative_rate_case2_guess_pos,oxfunc,ERR:{err:?}"),
        _ => println!("xirr_negative_rate_case2_guess_pos,oxfunc,ERR:unexpected"),
    }

    let rate_payment = pmt(0.01, 48.0, 8000.0, 0.0, PaymentTiming::EndOfPeriod).unwrap();
    match rate(
        48.0,
        rate_payment,
        8000.0,
        0.0,
        PaymentTiming::EndOfPeriod,
        Some(0.1),
    ) {
        Ok(v) => print_num("rate_seed_sample", "oxfunc", v),
        Err(err) => println!("rate_seed_sample,oxfunc,ERR:{err:?}"),
    }

    let pricemat = pricemat_kernel(
        serial(2024, 6, 15),
        serial(2025, 12, 31),
        serial(2024, 1, 1),
        0.0525,
        0.061,
        Some(1.0),
    )
    .unwrap();
    print_num("pricemat_basis1_seed", "oxfunc", pricemat);

    let yieldmat = yieldmat_kernel(
        serial(2024, 6, 15),
        serial(2025, 12, 31),
        serial(2024, 1, 1),
        0.0525,
        pricemat,
        Some(1.0),
    )
    .unwrap();
    print_num("yieldmat_basis1_seed", "oxfunc", yieldmat);

    let oddlprice = oddlprice_kernel(
        serial(2008, 2, 7),
        serial(2008, 6, 15),
        serial(2007, 10, 15),
        0.0375,
        0.0405,
        100.0,
        2.0,
        Some(0.0),
    )
    .unwrap();
    print_num("oddlprice_seed", "oxfunc", oddlprice);

    let oddlyield = oddlyield_kernel(
        serial(2008, 2, 7),
        serial(2008, 6, 15),
        serial(2007, 10, 15),
        0.0375,
        oddlprice,
        100.0,
        2.0,
        Some(0.0),
    )
    .unwrap();
    print_num("oddlyield_seed", "oxfunc", oddlyield);

    let oddfyield = oddfyield_kernel(
        serial(2008, 11, 11),
        serial(2021, 3, 1),
        serial(2008, 10, 15),
        serial(2009, 3, 1),
        0.0785,
        113.597_717_474_079,
        100.0,
        2.0,
        Some(1.0),
    )
    .unwrap();
    print_num("oddfyield_seed", "oxfunc", oddfyield);
}
