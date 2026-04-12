use oxfunc_core::functions::cashflow_rate_family::eval_xirr_surface;
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
            target: "w87_dummy".to_string(),
        })
    }
}

fn array_arg(values: &[f64]) -> CallArgValue {
    let row = values
        .iter()
        .copied()
        .map(ArrayCellValue::Number)
        .collect::<Vec<_>>();
    CallArgValue::Eval(EvalValue::Array(EvalArray::from_rows(vec![row]).unwrap()))
}

fn worksheet_error_text(code: WorksheetErrorCode) -> &'static str {
    match code {
        WorksheetErrorCode::Null => "#NULL!",
        WorksheetErrorCode::Div0 => "#DIV/0!",
        WorksheetErrorCode::Value => "#VALUE!",
        WorksheetErrorCode::Ref => "#REF!",
        WorksheetErrorCode::Name => "#NAME?",
        WorksheetErrorCode::Num => "#NUM!",
        WorksheetErrorCode::NA => "#N/A",
        WorksheetErrorCode::Busy => "#BUSY!",
        WorksheetErrorCode::GettingData => "#GETTING_DATA",
        WorksheetErrorCode::Spill => "#SPILL!",
        WorksheetErrorCode::Calc => "#CALC!",
        WorksheetErrorCode::Field => "#FIELD!",
        WorksheetErrorCode::Blocked => "#BLOCKED!",
        WorksheetErrorCode::Connect => "#CONNECT!",
    }
}

fn print_num(case_id: &str, guess: f64, value: f64) {
    println!("{case_id},{guess:.10},oxfunc,{value:.15}");
}

fn print_ws_error(case_id: &str, guess: f64, code: WorksheetErrorCode) {
    println!(
        "{case_id},{guess:.10},oxfunc,{}",
        worksheet_error_text(code)
    );
}

fn main() {
    println!("case_id,guess,source,observable");

    let cases = [
        (
            "w087_seed",
            &[-10_000.0, 2_750.0, 4_250.0, 3_250.0, 2_750.0][..],
            &[44_927.0, 45_108.0, 45_292.0, 45_473.0, 45_658.0][..],
            &[0.01, 0.1, 0.5, 1.0][..],
        ),
        (
            "adjacent_a",
            &[-1_000.0, 300.0, 400.0, 500.0][..],
            &[45_000.0, 45_100.0, 45_200.0, 45_365.0][..],
            &[0.01, 0.1, 0.5, 1.0][..],
        ),
        (
            "adjacent_b",
            &[-5_000.0, 1_500.0, 1_500.0, 1_500.0, 1_500.0][..],
            &[45_000.0, 45_090.0, 45_180.0, 45_270.0, 45_360.0][..],
            &[0.01, 0.1, 0.5, 1.0][..],
        ),
        (
            "adjacent_c",
            &[-12_000.0, 1_000.0, 2_000.0, 3_000.0, 7_000.0][..],
            &[45_000.0, 45_150.0, 45_300.0, 45_450.0, 45_600.0][..],
            &[0.01, 0.1, 0.5, 1.0][..],
        ),
        (
            "adjacent_d",
            &[-2_000.0, 2_500.0, 100.0][..],
            &[45_000.0, 45_180.0, 45_365.0][..],
            &[0.01, 0.1, 0.5, 1.0][..],
        ),
        (
            "adjacent_e",
            &[-4_000.0, 500.0, 800.0, 1_200.0, 2_200.0][..],
            &[45_000.0, 45_045.0, 45_120.0, 45_210.0, 45_400.0][..],
            &[0.01, 0.1, 0.5, 1.0][..],
        ),
    ];

    let resolver = DummyResolver;

    for (case_id, values, dates, guesses) in cases {
        let value_arg = array_arg(values);
        let date_arg = array_arg(dates);
        for guess in guesses {
            match eval_xirr_surface(
                &[
                    value_arg.clone(),
                    date_arg.clone(),
                    CallArgValue::Eval(EvalValue::Number(*guess)),
                ],
                &resolver,
            ) {
                Ok(EvalValue::Number(v)) => print_num(case_id, *guess, v),
                Ok(EvalValue::Error(code)) => print_ws_error(case_id, *guess, code),
                Err(err) => match err {
                    oxfunc_core::functions::cashflow_rate_family::CashflowRateEvalError::Domain(
                        code,
                    ) => print_ws_error(case_id, *guess, code),
                    _ => println!("{case_id},{guess:.10},oxfunc,ERR:{err:?}"),
                },
                _ => println!("{case_id},{guess:.10},oxfunc,ERR:unexpected"),
            }
        }
    }
}
