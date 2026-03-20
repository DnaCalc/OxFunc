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
            target: "w37_dummy".to_string(),
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

fn print_num(case_id: &str, guess: f64, value: f64) {
    println!(
        "{case_id},{guess:.10},oxfunc,{value:.15}",
        guess = guess,
        value = value
    );
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
        WorksheetErrorCode::GettingData => "#GETTING_DATA",
        WorksheetErrorCode::Spill => "#SPILL!",
        WorksheetErrorCode::Calc => "#CALC!",
        WorksheetErrorCode::Field => "#FIELD!",
        WorksheetErrorCode::Blocked => "#BLOCKED!",
        WorksheetErrorCode::Connect => "#CONNECT!",
    }
}

fn print_ws_error(case_id: &str, guess: f64, code: WorksheetErrorCode) {
    println!(
        "{case_id},{guess:.10},oxfunc,{}",
        worksheet_error_text(code),
        guess = guess
    );
}

fn main() {
    println!("case_id,guess,source,observable");

    let values = array_arg(&[15_108_163.384_092_3, -75_382_259.662_842_4]);
    let dates = array_arg(&[36585.0, 36616.0]);
    let cases = [
        ("xirr_large_root_guess_0.0001", 0.0001),
        ("xirr_large_root_guess_0.01", 0.01),
        ("xirr_large_root_guess_0.1", 0.1),
        ("xirr_large_root_guess_1", 1.0),
        ("xirr_large_root_guess_10", 10.0),
        ("xirr_large_root_guess_100", 100.0),
        ("xirr_large_root_guess_1000", 1000.0),
    ];
    let resolver = DummyResolver;

    for (case_id, guess) in cases {
        match eval_xirr_surface(
            &[
                values.clone(),
                dates.clone(),
                CallArgValue::Eval(EvalValue::Number(guess)),
            ],
            &resolver,
        ) {
            Ok(EvalValue::Number(v)) => print_num(case_id, guess, v),
            Ok(EvalValue::Error(code)) => print_ws_error(case_id, guess, code),
            Err(err) => match err {
                oxfunc_core::functions::cashflow_rate_family::CashflowRateEvalError::Domain(code) => {
                    print_ws_error(case_id, guess, code)
                }
                _ => println!("{case_id},{guess:.10},oxfunc,ERR:{err:?}"),
            },
            _ => println!("{case_id},{guess:.10},oxfunc,ERR:unexpected"),
        }
    }
}
