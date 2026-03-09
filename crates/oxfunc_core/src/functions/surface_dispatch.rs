use crate::coercion::CoercionError;
use crate::function::ArgPreparationProfile;
use crate::functions::abs::{AbsEvalError, abs_kernel, eval_abs_scalar_value};
use crate::functions::and_fn::{eval_and_surface, map_and_error_to_ws};
use crate::functions::average::{eval_average_surface, map_average_error_to_ws};
use crate::functions::cell::{eval_cell_surface, map_cell_error_to_ws};
use crate::functions::clean_fn::{eval_clean_surface, map_clean_error_to_ws};
use crate::functions::count::{eval_count_surface, map_count_error_to_ws};
use crate::functions::counta::{eval_counta_surface, map_counta_error_to_ws};
use crate::functions::date_fn::{eval_date_surface, map_date_error_to_ws};
use crate::functions::exact_fn::{eval_exact_surface, map_exact_error_to_ws};
use crate::functions::hstack::{eval_hstack_surface, map_hstack_error_to_ws};
use crate::functions::if_fn::{eval_if_surface, map_if_error_to_ws};
use crate::functions::iferror::{eval_iferror_surface, map_iferror_error_to_ws};
use crate::functions::index::{eval_index_surface, map_index_error_to_ws};
use crate::functions::indirect::{eval_indirect_surface, map_indirect_error_to_ws};
use crate::functions::isnumber::{eval_isnumber_surface, map_isnumber_error_to_ws};
use crate::functions::match_fn::{eval_match_surface, map_match_error_to_ws};
use crate::functions::now_fn::{NowProvider, eval_now_surface, map_now_error_to_ws};
use crate::functions::offset::{eval_offset_surface, map_offset_error_to_ws};
use crate::functions::op_add::{eval_op_add_surface, map_op_add_error_to_ws, op_add_kernel};
use crate::functions::pi::eval_pi;
use crate::functions::rand_fn::{RandomProvider, eval_rand_surface, map_rand_error_to_ws};
use crate::functions::round_fn::{eval_round_surface, map_round_error_to_ws, round_kernel};
use crate::functions::sequence::{eval_sequence_surface, map_sequence_error_to_ws};
use crate::functions::sum::{eval_sum_surface, map_sum_error_to_ws};
use crate::functions::textjoin::{eval_textjoin_surface, map_textjoin_error_to_ws};
use crate::functions::today_fn::{TodayProvider, eval_today_surface, map_today_error_to_ws};
use crate::functions::xlookup::{eval_xlookup_surface, map_xlookup_error_to_ws};
use crate::functions::xmatch::XmatchEvalError;
use crate::functions::xmatch_surface::eval_xmatch_surface_value;
use crate::resolver::RefResolutionError;
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalError, EvalValue, Value, WorksheetErrorCode};

pub const FUNC_ID_ABS: &str = "FUNC.ABS";
pub const FUNC_ID_AND: &str = "FUNC.AND";
pub const FUNC_ID_AVERAGE: &str = "FUNC.AVERAGE";
pub const FUNC_ID_CELL: &str = "FUNC.CELL";
pub const FUNC_ID_CLEAN: &str = "FUNC.CLEAN";
pub const FUNC_ID_COUNT: &str = "FUNC.COUNT";
pub const FUNC_ID_COUNTA: &str = "FUNC.COUNTA";
pub const FUNC_ID_DATE: &str = "FUNC.DATE";
pub const FUNC_ID_EXACT: &str = "FUNC.EXACT";
pub const FUNC_ID_HSTACK: &str = "FUNC.HSTACK";
pub const FUNC_ID_IF: &str = "FUNC.IF";
pub const FUNC_ID_IFERROR: &str = "FUNC.IFERROR";
pub const FUNC_ID_INDEX: &str = "FUNC.INDEX";
pub const FUNC_ID_INDIRECT: &str = "FUNC.INDIRECT";
pub const FUNC_ID_ISNUMBER: &str = "FUNC.ISNUMBER";
pub const FUNC_ID_MATCH: &str = "FUNC.MATCH";
pub const FUNC_ID_NOW: &str = "FUNC.NOW";
pub const FUNC_ID_OFFSET: &str = "FUNC.OFFSET";
pub const FUNC_ID_OP_ADD: &str = "FUNC.OP_ADD";
pub const FUNC_ID_PI: &str = "FUNC.PI";
pub const FUNC_ID_RAND: &str = "FUNC.RAND";
pub const FUNC_ID_ROUND: &str = "FUNC.ROUND";
pub const FUNC_ID_SEQUENCE: &str = "FUNC.SEQUENCE";
pub const FUNC_ID_SUM: &str = "FUNC.SUM";
pub const FUNC_ID_TEXTJOIN: &str = "FUNC.TEXTJOIN";
pub const FUNC_ID_TODAY: &str = "FUNC.TODAY";
pub const FUNC_ID_XLOOKUP: &str = "FUNC.XLOOKUP";
pub const FUNC_ID_XMATCH: &str = "FUNC.XMATCH";

fn map_ref_resolution_to_ws(e: &RefResolutionError) -> WorksheetErrorCode {
    match e {
        RefResolutionError::CapabilityDenied { .. } => WorksheetErrorCode::Ref,
        RefResolutionError::UnresolvedReference { .. } => WorksheetErrorCode::Ref,
        RefResolutionError::EvalTimeDerefNotAllowed => WorksheetErrorCode::Ref,
        RefResolutionError::ProviderFailure { .. } => WorksheetErrorCode::Value,
    }
}

fn map_coercion_to_ws(e: &CoercionError) -> WorksheetErrorCode {
    match e {
        CoercionError::WorksheetError(code) => *code,
        CoercionError::RefResolution(err) => map_ref_resolution_to_ws(err),
        CoercionError::MissingArg => WorksheetErrorCode::Value,
        CoercionError::EmptyCell => WorksheetErrorCode::Value,
        CoercionError::NonNumericText(_) => WorksheetErrorCode::Value,
        CoercionError::UnsupportedValueKind(_) => WorksheetErrorCode::Value,
    }
}

fn map_abs_error_to_ws(e: &AbsEvalError) -> WorksheetErrorCode {
    match e {
        AbsEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        AbsEvalError::Coercion(err) => map_coercion_to_ws(err),
    }
}

fn map_eval_error_to_ws(e: &EvalError) -> WorksheetErrorCode {
    match e {
        EvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
    }
}

fn map_xmatch_error_to_ws(e: &XmatchEvalError) -> WorksheetErrorCode {
    match e {
        XmatchEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        XmatchEvalError::EmptyLookupArray => WorksheetErrorCode::NA,
        XmatchEvalError::MissingArg => WorksheetErrorCode::Value,
        XmatchEvalError::EmptyCell => WorksheetErrorCode::Value,
        XmatchEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        XmatchEvalError::Coercion(_) => WorksheetErrorCode::Value,
        XmatchEvalError::UnsupportedValueKind(_) => WorksheetErrorCode::Value,
        XmatchEvalError::InvalidMatchMode(_) => WorksheetErrorCode::Value,
        XmatchEvalError::InvalidSearchMode(_) => WorksheetErrorCode::Value,
        XmatchEvalError::UnsupportedMatchModeForSeed(_) => WorksheetErrorCode::NA,
        XmatchEvalError::UnsupportedSearchModeForSeed(_) => WorksheetErrorCode::NA,
        XmatchEvalError::NotAvailable => WorksheetErrorCode::NA,
    }
}

struct FixedNowProvider {
    serial: f64,
}

impl NowProvider for FixedNowProvider {
    fn now_serial(&self) -> f64 {
        self.serial
    }
}

impl TodayProvider for FixedNowProvider {
    fn today_serial(&self) -> f64 {
        self.serial
    }
}

struct FixedRandomProvider {
    value: f64,
}

impl RandomProvider for FixedRandomProvider {
    fn random_unit(&self) -> f64 {
        self.value
    }
}

fn singleton_arg_slice(arg: &CallArgValue) -> Vec<CallArgValue> {
    // Core value model does not yet carry full array payloads in prepared call-args.
    // Keep singleton passthrough until array payload/value expansion is implemented.
    vec![arg.clone()]
}

pub fn arg_preparation_profile(function_id: &str) -> Option<ArgPreparationProfile> {
    match function_id {
        FUNC_ID_ABS => Some(crate::functions::abs::ABS_META.arg_preparation_profile),
        FUNC_ID_AND => Some(crate::functions::and_fn::AND_META.arg_preparation_profile),
        FUNC_ID_AVERAGE => Some(crate::functions::average::AVERAGE_META.arg_preparation_profile),
        FUNC_ID_CELL => Some(crate::functions::cell::CELL_META.arg_preparation_profile),
        FUNC_ID_CLEAN => Some(crate::functions::clean_fn::CLEAN_META.arg_preparation_profile),
        FUNC_ID_COUNT => Some(crate::functions::count::COUNT_META.arg_preparation_profile),
        FUNC_ID_COUNTA => Some(crate::functions::counta::COUNTA_META.arg_preparation_profile),
        FUNC_ID_DATE => Some(crate::functions::date_fn::DATE_META.arg_preparation_profile),
        FUNC_ID_EXACT => Some(crate::functions::exact_fn::EXACT_META.arg_preparation_profile),
        FUNC_ID_HSTACK => Some(crate::functions::hstack::HSTACK_META.arg_preparation_profile),
        FUNC_ID_IF => Some(crate::functions::if_fn::IF_META.arg_preparation_profile),
        FUNC_ID_IFERROR => Some(crate::functions::iferror::IFERROR_META.arg_preparation_profile),
        FUNC_ID_INDEX => Some(crate::functions::index::INDEX_META.arg_preparation_profile),
        FUNC_ID_INDIRECT => Some(crate::functions::indirect::INDIRECT_META.arg_preparation_profile),
        FUNC_ID_ISNUMBER => Some(crate::functions::isnumber::ISNUMBER_META.arg_preparation_profile),
        FUNC_ID_MATCH => Some(crate::functions::match_fn::MATCH_META.arg_preparation_profile),
        FUNC_ID_NOW => Some(crate::functions::now_fn::NOW_META.arg_preparation_profile),
        FUNC_ID_OFFSET => Some(crate::functions::offset::OFFSET_META.arg_preparation_profile),
        FUNC_ID_OP_ADD => Some(crate::functions::op_add::OP_ADD_META.arg_preparation_profile),
        FUNC_ID_PI => Some(crate::functions::pi::PI_META.arg_preparation_profile),
        FUNC_ID_RAND => Some(crate::functions::rand_fn::RAND_META.arg_preparation_profile),
        FUNC_ID_ROUND => Some(crate::functions::round_fn::ROUND_META.arg_preparation_profile),
        FUNC_ID_SEQUENCE => Some(crate::functions::sequence::SEQUENCE_META.arg_preparation_profile),
        FUNC_ID_SUM => Some(crate::functions::sum::SUM_META.arg_preparation_profile),
        FUNC_ID_TEXTJOIN => Some(crate::functions::textjoin::TEXTJOIN_META.arg_preparation_profile),
        FUNC_ID_TODAY => Some(crate::functions::today_fn::TODAY_META.arg_preparation_profile),
        FUNC_ID_XLOOKUP => Some(crate::functions::xlookup::XLOOKUP_META.arg_preparation_profile),
        FUNC_ID_XMATCH => Some(crate::functions::xmatch::XMATCH_META.arg_preparation_profile),
        _ => None,
    }
}

pub fn eval_surface_value_call(
    function_id: &str,
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    now_serial: Option<f64>,
    random_value: Option<f64>,
) -> Result<EvalValue, WorksheetErrorCode> {
    match function_id {
        FUNC_ID_ABS => eval_abs_scalar_value(args, resolver).map_err(|e| map_abs_error_to_ws(&e)),
        FUNC_ID_AND => eval_and_surface(args, resolver).map_err(|e| map_and_error_to_ws(&e)),
        FUNC_ID_AVERAGE => {
            eval_average_surface(args, resolver).map_err(|e| map_average_error_to_ws(&e))
        }
        FUNC_ID_CELL => eval_cell_surface(args, resolver).map_err(|e| map_cell_error_to_ws(&e)),
        FUNC_ID_CLEAN => eval_clean_surface(args, resolver).map_err(|e| map_clean_error_to_ws(&e)),
        FUNC_ID_COUNT => eval_count_surface(args, resolver).map_err(|e| map_count_error_to_ws(&e)),
        FUNC_ID_COUNTA => {
            eval_counta_surface(args, resolver).map_err(|e| map_counta_error_to_ws(&e))
        }
        FUNC_ID_DATE => eval_date_surface(args, resolver).map_err(|e| map_date_error_to_ws(&e)),
        FUNC_ID_EXACT => eval_exact_surface(args, resolver).map_err(|e| map_exact_error_to_ws(&e)),
        FUNC_ID_HSTACK => {
            eval_hstack_surface(args, resolver).map_err(|e| map_hstack_error_to_ws(&e))
        }
        FUNC_ID_SUM => eval_sum_surface(args, resolver).map_err(|e| map_sum_error_to_ws(&e)),
        FUNC_ID_IF => eval_if_surface(args, resolver).map_err(|e| map_if_error_to_ws(&e)),
        FUNC_ID_IFERROR => {
            eval_iferror_surface(args, resolver).map_err(|e| map_iferror_error_to_ws(&e))
        }
        FUNC_ID_INDEX => eval_index_surface(args, resolver).map_err(|e| map_index_error_to_ws(&e)),
        FUNC_ID_MATCH => {
            if args.len() < 2 {
                return Err(WorksheetErrorCode::Value);
            }
            let lookup_array = singleton_arg_slice(&args[1]);
            let match_type = args.get(2);
            eval_match_surface(&args[0], &lookup_array, match_type, resolver)
                .map_err(|e| map_match_error_to_ws(&e))
        }
        FUNC_ID_ISNUMBER => {
            eval_isnumber_surface(args, resolver).map_err(|e| map_isnumber_error_to_ws(&e))
        }
        FUNC_ID_NOW => {
            let serial = now_serial.ok_or(WorksheetErrorCode::Value)?;
            let provider = FixedNowProvider { serial };
            eval_now_surface(args, &provider).map_err(|e| map_now_error_to_ws(&e))
        }
        FUNC_ID_OFFSET => {
            eval_offset_surface(args, resolver).map_err(|e| map_offset_error_to_ws(&e))
        }
        FUNC_ID_XLOOKUP => {
            if args.len() < 3 {
                return Err(WorksheetErrorCode::Value);
            }
            let lookup_array = singleton_arg_slice(&args[1]);
            let return_array = singleton_arg_slice(&args[2]);
            eval_xlookup_surface(
                &args[0],
                &lookup_array,
                &return_array,
                args.get(3),
                args.get(4),
                args.get(5),
                resolver,
            )
            .map_err(|e| map_xlookup_error_to_ws(&e))
        }
        FUNC_ID_INDIRECT => {
            eval_indirect_surface(args, resolver).map_err(|e| map_indirect_error_to_ws(&e))
        }
        FUNC_ID_RAND => {
            let value = random_value.ok_or(WorksheetErrorCode::Value)?;
            let provider = FixedRandomProvider { value };
            eval_rand_surface(args, &provider).map_err(|e| map_rand_error_to_ws(&e))
        }
        FUNC_ID_ROUND => {
            eval_round_surface(args, resolver).map_err(|e| map_round_error_to_ws(&e))
        }
        FUNC_ID_SEQUENCE => {
            eval_sequence_surface(args, resolver).map_err(|e| map_sequence_error_to_ws(&e))
        }
        FUNC_ID_OP_ADD => eval_op_add_surface(args, resolver).map_err(|e| map_op_add_error_to_ws(&e)),
        FUNC_ID_TEXTJOIN => {
            eval_textjoin_surface(args, resolver).map_err(|e| map_textjoin_error_to_ws(&e))
        }
        FUNC_ID_TODAY => {
            let serial = now_serial.ok_or(WorksheetErrorCode::Value)?;
            let provider = FixedNowProvider { serial };
            eval_today_surface(args, &provider).map_err(|e| map_today_error_to_ws(&e))
        }
        FUNC_ID_XMATCH => {
            if args.len() < 2 {
                return Err(WorksheetErrorCode::Value);
            }
            let lookup_array = singleton_arg_slice(&args[1]);
            eval_xmatch_surface_value(&args[0], &lookup_array, args.get(2), args.get(3), resolver)
                .map_err(|e| map_xmatch_error_to_ws(&e))
        }
        FUNC_ID_PI => {
            if !args.is_empty() {
                return Err(WorksheetErrorCode::Value);
            }
            let pi_args: Vec<Value> = Vec::new();
            match eval_pi(&pi_args) {
                Ok(Value::Number(n)) => Ok(EvalValue::Number(n)),
                Ok(Value::Error(_)) => Err(WorksheetErrorCode::Value),
                Err(e) => Err(map_eval_error_to_ws(&e)),
            }
        }
        _ => Err(WorksheetErrorCode::Value),
    }
}

pub fn eval_surface_q_unary_number(
    function_id: &str,
    value: f64,
) -> Result<f64, WorksheetErrorCode> {
    match function_id {
        FUNC_ID_ABS => Ok(abs_kernel(value)),
        _ => Err(WorksheetErrorCode::Value),
    }
}

pub fn eval_surface_q_binary_number(
    function_id: &str,
    lhs: f64,
    rhs: f64,
) -> Result<f64, WorksheetErrorCode> {
    match function_id {
        FUNC_ID_OP_ADD => Ok(op_add_kernel(lhs, rhs)),
        FUNC_ID_ROUND => Ok(round_kernel(lhs, rhs.trunc() as i32)),
        _ => Err(WorksheetErrorCode::Value),
    }
}

pub fn eval_surface_q_nullary_number(function_id: &str) -> Result<f64, WorksheetErrorCode> {
    match function_id {
        FUNC_ID_PI => match eval_pi(&[]) {
            Ok(Value::Number(n)) => Ok(n),
            Ok(_) => Err(WorksheetErrorCode::Value),
            Err(e) => Err(map_eval_error_to_ws(&e)),
        },
        _ => Err(WorksheetErrorCode::Value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ExcelText, ReferenceLike};

    struct NoReferenceResolver;

    impl ReferenceResolver for NoReferenceResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            ResolverCapabilities::permissive_local()
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            Err(RefResolutionError::UnresolvedReference {
                target: reference.target.clone(),
            })
        }
    }

    #[test]
    fn eval_surface_value_call_abs_accepts_text_numeric() {
        let arg = CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
            " -2 ".encode_utf16().collect(),
        )));
        let got = eval_surface_value_call(
            FUNC_ID_ABS,
            &[arg],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
        );
        assert_eq!(got, Ok(EvalValue::Number(2.0)));
    }

    #[test]
    fn eval_surface_value_call_rejects_unknown_id() {
        let arg = CallArgValue::Eval(EvalValue::Number(1.0));
        let got = eval_surface_value_call(
            "FUNC.UNKNOWN",
            &[arg],
            &NoReferenceResolver,
            Some(46000.0),
            Some(0.5),
        );
        assert_eq!(got, Err(WorksheetErrorCode::Value));
    }

    #[test]
    fn eval_surface_q_unary_number_abs_calls_kernel() {
        let got = eval_surface_q_unary_number(FUNC_ID_ABS, -3.0);
        assert_eq!(got, Ok(3.0));
    }

    #[test]
    fn eval_surface_q_binary_number_add_calls_kernel() {
        let got = eval_surface_q_binary_number(FUNC_ID_OP_ADD, 1.5, 2.0);
        assert_eq!(got, Ok(3.5));
    }

    #[test]
    fn eval_surface_q_binary_number_round_calls_kernel() {
        let got = eval_surface_q_binary_number(FUNC_ID_ROUND, 12.34, 1.0);
        assert_eq!(got, Ok(12.3));
    }

    #[test]
    fn eval_surface_q_nullary_number_pi_returns_constant() {
        let got = eval_surface_q_nullary_number(FUNC_ID_PI);
        assert_eq!(got, Ok(std::f64::consts::PI));
    }
}
