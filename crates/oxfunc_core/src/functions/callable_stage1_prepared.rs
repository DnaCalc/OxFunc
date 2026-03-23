use std::collections::{BTreeMap, BTreeSet};

use crate::coercion::CoercionError;
use crate::functions::adapters::{PreparedArgValue, coerce_prepared_to_number};
use crate::value::{
    ArrayCellValue, CallableArityShape, CallableCaptureMode, EvalValue, LambdaValue,
    WorksheetErrorCode,
};

#[cfg(test)]
use crate::value::EvalArray;

#[derive(Debug, Clone, PartialEq)]
pub enum Stage1Expr {
    Prepared(PreparedArgValue),
    Name(String),
    Add(Box<Stage1Expr>, Box<Stage1Expr>),
    Mul(Box<Stage1Expr>, Box<Stage1Expr>),
    Sum(Box<Stage1Expr>),
    Let {
        bindings: Vec<(String, Stage1Expr)>,
        body: Box<Stage1Expr>,
    },
    Lambda {
        params: Vec<String>,
        body: Box<Stage1Expr>,
    },
    Invoke {
        callee: Box<Stage1Expr>,
        args: Vec<Stage1Expr>,
    },
    IsOmitted(Box<Stage1Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stage1Value {
    Prepared(PreparedArgValue),
    Lambda(Stage1LambdaClosure),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stage1LambdaClosure {
    pub meta: LambdaValue,
    pub params: Vec<String>,
    pub body: Box<Stage1Expr>,
    pub captures: BTreeMap<String, Stage1Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stage1FormationError {
    DuplicateLetName(String),
    DuplicateLambdaParam(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stage1EvalError {
    Formation(Stage1FormationError),
    UnknownName(String),
    NonCallable,
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    UnsupportedValueKind(&'static str),
    Coercion(CoercionError),
}

fn validate_expr(expr: &Stage1Expr) -> Result<(), Stage1FormationError> {
    match expr {
        Stage1Expr::Prepared(_) | Stage1Expr::Name(_) => Ok(()),
        Stage1Expr::Add(a, b) | Stage1Expr::Mul(a, b) => {
            validate_expr(a)?;
            validate_expr(b)
        }
        Stage1Expr::Sum(inner) | Stage1Expr::IsOmitted(inner) => validate_expr(inner),
        Stage1Expr::Let { bindings, body } => {
            let mut seen = BTreeSet::new();
            for (name, binding_expr) in bindings {
                if !seen.insert(name.clone()) {
                    return Err(Stage1FormationError::DuplicateLetName(name.clone()));
                }
                validate_expr(binding_expr)?;
            }
            validate_expr(body)
        }
        Stage1Expr::Lambda { params, body } => {
            let mut seen = BTreeSet::new();
            for param in params {
                if !seen.insert(param.clone()) {
                    return Err(Stage1FormationError::DuplicateLambdaParam(param.clone()));
                }
            }
            validate_expr(body)
        }
        Stage1Expr::Invoke { callee, args } => {
            validate_expr(callee)?;
            for arg in args {
                validate_expr(arg)?;
            }
            Ok(())
        }
    }
}

fn prepared_to_number(value: &Stage1Value) -> Result<f64, Stage1EvalError> {
    match value {
        Stage1Value::Prepared(prepared) => {
            coerce_prepared_to_number(prepared).map_err(Stage1EvalError::Coercion)
        }
        Stage1Value::Lambda(_) => Err(Stage1EvalError::UnsupportedValueKind("lambda_value")),
    }
}

fn sum_prepared(prepared: &PreparedArgValue) -> Result<f64, Stage1EvalError> {
    match prepared {
        PreparedArgValue::Eval(EvalValue::Number(n)) => Ok(*n),
        PreparedArgValue::Eval(EvalValue::Array(array)) => {
            let mut total = 0.0;
            for cell in array.iter_row_major() {
                match cell {
                    ArrayCellValue::Number(n) => total += *n,
                    ArrayCellValue::EmptyCell => {}
                    ArrayCellValue::Error(code) => {
                        return Err(Stage1EvalError::Coercion(CoercionError::WorksheetError(
                            *code,
                        )));
                    }
                    _ => {
                        return Err(Stage1EvalError::UnsupportedValueKind(
                            "sum_non_numeric_cell",
                        ));
                    }
                }
            }
            Ok(total)
        }
        PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell => Ok(0.0),
        PreparedArgValue::Eval(EvalValue::Error(code)) => Err(Stage1EvalError::Coercion(
            CoercionError::WorksheetError(*code),
        )),
        PreparedArgValue::Eval(EvalValue::Text(_))
        | PreparedArgValue::Eval(EvalValue::Logical(_))
        | PreparedArgValue::Eval(EvalValue::Reference(_))
        | PreparedArgValue::Eval(EvalValue::Lambda(_)) => Err(
            Stage1EvalError::UnsupportedValueKind("sum_unsupported_value"),
        ),
    }
}

fn publish_value(value: &Stage1Value) -> Result<EvalValue, Stage1EvalError> {
    match value {
        Stage1Value::Prepared(PreparedArgValue::Eval(v)) => Ok(v.clone()),
        Stage1Value::Prepared(PreparedArgValue::MissingArg)
        | Stage1Value::Prepared(PreparedArgValue::EmptyCell) => Ok(EvalValue::Text(
            crate::value::ExcelText::from_utf16_code_units(Vec::new()),
        )),
        Stage1Value::Lambda(_) => Ok(EvalValue::Error(WorksheetErrorCode::Calc)),
    }
}

fn omitted_truth(inner: &Stage1Expr, env: &BTreeMap<String, Stage1Value>) -> bool {
    match inner {
        Stage1Expr::Name(name) => matches!(
            env.get(name),
            Some(Stage1Value::Prepared(PreparedArgValue::MissingArg))
        ),
        _ => false,
    }
}

fn invoke_closure(
    closure: &Stage1LambdaClosure,
    args: &[Stage1Value],
) -> Result<Stage1Value, Stage1EvalError> {
    let argc = args.len();
    if !closure.meta.arity_shape.accepts(argc) {
        return Err(Stage1EvalError::ArityMismatch {
            expected_min: closure.meta.arity_shape.min,
            expected_max: closure.meta.arity_shape.max,
            actual: argc,
        });
    }

    let mut env = closure.captures.clone();
    for (param, arg) in closure.params.iter().zip(args.iter()) {
        env.insert(param.clone(), arg.clone());
    }
    eval_expr(&closure.body, &env)
}

pub fn eval_expr(
    expr: &Stage1Expr,
    env: &BTreeMap<String, Stage1Value>,
) -> Result<Stage1Value, Stage1EvalError> {
    match expr {
        Stage1Expr::Prepared(value) => Ok(Stage1Value::Prepared(value.clone())),
        Stage1Expr::Name(name) => env
            .get(name)
            .cloned()
            .ok_or_else(|| Stage1EvalError::UnknownName(name.clone())),
        Stage1Expr::Add(left, right) => {
            let l = prepared_to_number(&eval_expr(left, env)?)?;
            let r = prepared_to_number(&eval_expr(right, env)?)?;
            Ok(Stage1Value::Prepared(PreparedArgValue::Eval(
                EvalValue::Number(l + r),
            )))
        }
        Stage1Expr::Mul(left, right) => {
            let l = prepared_to_number(&eval_expr(left, env)?)?;
            let r = prepared_to_number(&eval_expr(right, env)?)?;
            Ok(Stage1Value::Prepared(PreparedArgValue::Eval(
                EvalValue::Number(l * r),
            )))
        }
        Stage1Expr::Sum(inner) => {
            let value = eval_expr(inner, env)?;
            let total = match value {
                Stage1Value::Prepared(ref prepared) => sum_prepared(prepared)?,
                Stage1Value::Lambda(_) => {
                    return Err(Stage1EvalError::UnsupportedValueKind("sum_lambda"));
                }
            };
            Ok(Stage1Value::Prepared(PreparedArgValue::Eval(
                EvalValue::Number(total),
            )))
        }
        Stage1Expr::Let { bindings, body } => {
            let mut next_env = env.clone();
            for (name, binding_expr) in bindings {
                let value = eval_expr(binding_expr, &next_env)?;
                next_env.insert(name.clone(), value);
            }
            eval_expr(body, &next_env)
        }
        Stage1Expr::Lambda { params, body } => {
            let meta = LambdaValue::helper_lambda(
                format!("stage1.lambda.{}", params.len()),
                CallableArityShape::exact(params.len()),
                if env.is_empty() {
                    CallableCaptureMode::NoCapture
                } else {
                    CallableCaptureMode::LexicalCapture
                },
                "stage1.direct_lambda",
            );
            Ok(Stage1Value::Lambda(Stage1LambdaClosure {
                meta,
                params: params.clone(),
                body: body.clone(),
                captures: env.clone(),
            }))
        }
        Stage1Expr::Invoke { callee, args } => {
            let callee_value = eval_expr(callee, env)?;
            let arg_values = args
                .iter()
                .map(|arg| eval_expr(arg, env))
                .collect::<Result<Vec<_>, _>>()?;
            match callee_value {
                Stage1Value::Lambda(closure) => invoke_closure(&closure, &arg_values),
                Stage1Value::Prepared(_) => Err(Stage1EvalError::NonCallable),
            }
        }
        Stage1Expr::IsOmitted(inner) => Ok(Stage1Value::Prepared(PreparedArgValue::Eval(
            EvalValue::Logical(omitted_truth(inner, env)),
        ))),
    }
}

pub fn evaluate_stage1_worksheet(expr: &Stage1Expr) -> Result<EvalValue, Stage1EvalError> {
    validate_expr(expr).map_err(Stage1EvalError::Formation)?;
    publish_value(&eval_expr(expr, &BTreeMap::new())?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::ExcelText;

    fn num(n: f64) -> Stage1Expr {
        Stage1Expr::Prepared(PreparedArgValue::Eval(EvalValue::Number(n)))
    }

    fn row_array(nums: &[f64]) -> Stage1Expr {
        let cells = nums
            .iter()
            .copied()
            .map(ArrayCellValue::Number)
            .collect::<Vec<_>>();
        let array = EvalArray::from_rows(vec![cells]).expect("row array");
        Stage1Expr::Prepared(PreparedArgValue::Eval(EvalValue::Array(array)))
    }

    #[test]
    fn let_basic_seed_matches_excel() {
        let expr = Stage1Expr::Let {
            bindings: vec![("x".to_string(), num(2.0))],
            body: Box::new(Stage1Expr::Add(
                Box::new(Stage1Expr::Name("x".to_string())),
                Box::new(num(3.0)),
            )),
        };
        assert_eq!(evaluate_stage1_worksheet(&expr), Ok(EvalValue::Number(5.0)));
    }

    #[test]
    fn let_two_names_seed_matches_excel() {
        let expr = Stage1Expr::Let {
            bindings: vec![("a".to_string(), num(1.0)), ("b".to_string(), num(2.0))],
            body: Box::new(Stage1Expr::Add(
                Box::new(Stage1Expr::Name("a".to_string())),
                Box::new(Stage1Expr::Name("b".to_string())),
            )),
        };
        assert_eq!(evaluate_stage1_worksheet(&expr), Ok(EvalValue::Number(3.0)));
    }

    #[test]
    fn let_array_sum_seed_matches_excel() {
        let expr = Stage1Expr::Let {
            bindings: vec![("x".to_string(), row_array(&[1.0, 2.0]))],
            body: Box::new(Stage1Expr::Sum(Box::new(Stage1Expr::Name("x".to_string())))),
        };
        assert_eq!(evaluate_stage1_worksheet(&expr), Ok(EvalValue::Number(3.0)));
    }

    #[test]
    fn let_duplicate_name_is_rejected() {
        let expr = Stage1Expr::Let {
            bindings: vec![("x".to_string(), num(2.0)), ("x".to_string(), num(5.0))],
            body: Box::new(Stage1Expr::Add(
                Box::new(Stage1Expr::Name("x".to_string())),
                Box::new(num(1.0)),
            )),
        };
        assert_eq!(
            evaluate_stage1_worksheet(&expr),
            Err(Stage1EvalError::Formation(
                Stage1FormationError::DuplicateLetName("x".to_string())
            ))
        );
    }

    #[test]
    fn immediate_lambda_invocation_seed_matches_excel() {
        let expr = Stage1Expr::Invoke {
            callee: Box::new(Stage1Expr::Lambda {
                params: vec!["x".to_string()],
                body: Box::new(Stage1Expr::Add(
                    Box::new(Stage1Expr::Name("x".to_string())),
                    Box::new(num(1.0)),
                )),
            }),
            args: vec![num(5.0)],
        };
        assert_eq!(evaluate_stage1_worksheet(&expr), Ok(EvalValue::Number(6.0)));
    }

    #[test]
    fn bare_lambda_publishes_calc() {
        let expr = Stage1Expr::Lambda {
            params: vec!["x".to_string()],
            body: Box::new(Stage1Expr::Add(
                Box::new(Stage1Expr::Name("x".to_string())),
                Box::new(num(1.0)),
            )),
        };
        assert_eq!(
            evaluate_stage1_worksheet(&expr),
            Ok(EvalValue::Error(WorksheetErrorCode::Calc))
        );
    }

    #[test]
    fn lambda_arity_mismatch_seed_matches_excel() {
        let expr = Stage1Expr::Invoke {
            callee: Box::new(Stage1Expr::Lambda {
                params: vec!["x".to_string(), "y".to_string()],
                body: Box::new(Stage1Expr::Add(
                    Box::new(Stage1Expr::Name("x".to_string())),
                    Box::new(Stage1Expr::Name("y".to_string())),
                )),
            }),
            args: vec![num(1.0)],
        };
        assert_eq!(
            evaluate_stage1_worksheet(&expr),
            Err(Stage1EvalError::ArityMismatch {
                expected_min: 2,
                expected_max: 2,
                actual: 1,
            })
        );
    }

    #[test]
    fn lambda_duplicate_params_are_rejected() {
        let expr = Stage1Expr::Invoke {
            callee: Box::new(Stage1Expr::Lambda {
                params: vec!["x".to_string(), "x".to_string()],
                body: Box::new(Stage1Expr::Add(
                    Box::new(Stage1Expr::Name("x".to_string())),
                    Box::new(num(1.0)),
                )),
            }),
            args: vec![num(2.0)],
        };
        assert_eq!(
            evaluate_stage1_worksheet(&expr),
            Err(Stage1EvalError::Formation(
                Stage1FormationError::DuplicateLambdaParam("x".to_string())
            ))
        );
    }

    #[test]
    fn let_lambda_capture_seed_matches_excel() {
        let expr = Stage1Expr::Let {
            bindings: vec![("x".to_string(), num(2.0))],
            body: Box::new(Stage1Expr::Invoke {
                callee: Box::new(Stage1Expr::Lambda {
                    params: vec!["y".to_string()],
                    body: Box::new(Stage1Expr::Add(
                        Box::new(Stage1Expr::Name("x".to_string())),
                        Box::new(Stage1Expr::Name("y".to_string())),
                    )),
                }),
                args: vec![num(3.0)],
            }),
        };
        assert_eq!(evaluate_stage1_worksheet(&expr), Ok(EvalValue::Number(5.0)));
    }

    #[test]
    fn lambda_let_inner_seed_matches_excel() {
        let expr = Stage1Expr::Invoke {
            callee: Box::new(Stage1Expr::Lambda {
                params: vec!["x".to_string()],
                body: Box::new(Stage1Expr::Let {
                    bindings: vec![("y".to_string(), num(2.0))],
                    body: Box::new(Stage1Expr::Add(
                        Box::new(Stage1Expr::Name("x".to_string())),
                        Box::new(Stage1Expr::Name("y".to_string())),
                    )),
                }),
            }),
            args: vec![num(3.0)],
        };
        assert_eq!(evaluate_stage1_worksheet(&expr), Ok(EvalValue::Number(5.0)));
    }

    #[test]
    fn isomitted_present_seed_matches_excel() {
        let expr = Stage1Expr::Invoke {
            callee: Box::new(Stage1Expr::Lambda {
                params: vec!["a".to_string()],
                body: Box::new(Stage1Expr::IsOmitted(Box::new(Stage1Expr::Name(
                    "a".to_string(),
                )))),
            }),
            args: vec![num(3.0)],
        };
        assert_eq!(
            evaluate_stage1_worksheet(&expr),
            Ok(EvalValue::Logical(false))
        );
    }

    #[test]
    fn isomitted_explicit_missing_placeholder_returns_true() {
        let expr = Stage1Expr::Invoke {
            callee: Box::new(Stage1Expr::Lambda {
                params: vec!["x".to_string(), "y".to_string()],
                body: Box::new(Stage1Expr::IsOmitted(Box::new(Stage1Expr::Name(
                    "y".to_string(),
                )))),
            }),
            args: vec![num(1.0), Stage1Expr::Prepared(PreparedArgValue::MissingArg)],
        };
        assert_eq!(
            evaluate_stage1_worksheet(&expr),
            Ok(EvalValue::Logical(true))
        );
    }

    #[test]
    fn isomitted_top_level_literal_is_false() {
        let expr = Stage1Expr::IsOmitted(Box::new(num(1.0)));
        assert_eq!(
            evaluate_stage1_worksheet(&expr),
            Ok(EvalValue::Logical(false))
        );
    }

    #[test]
    fn direct_under_application_does_not_expose_omission_channel() {
        let expr = Stage1Expr::Invoke {
            callee: Box::new(Stage1Expr::Lambda {
                params: vec!["a".to_string()],
                body: Box::new(Stage1Expr::IsOmitted(Box::new(Stage1Expr::Name(
                    "a".to_string(),
                )))),
            }),
            args: vec![],
        };
        assert_eq!(
            evaluate_stage1_worksheet(&expr),
            Err(Stage1EvalError::ArityMismatch {
                expected_min: 1,
                expected_max: 1,
                actual: 0,
            })
        );
    }

    #[test]
    fn lambda_capture_metadata_marks_lexical_capture() {
        let expr = Stage1Expr::Let {
            bindings: vec![("x".to_string(), num(2.0))],
            body: Box::new(Stage1Expr::Lambda {
                params: vec!["y".to_string()],
                body: Box::new(Stage1Expr::Add(
                    Box::new(Stage1Expr::Name("x".to_string())),
                    Box::new(Stage1Expr::Name("y".to_string())),
                )),
            }),
        };
        let value = eval_expr(&expr, &BTreeMap::new()).expect("lambda value");
        let closure = match value {
            Stage1Value::Lambda(closure) => closure,
            other => panic!("expected lambda, got {other:?}"),
        };
        assert_eq!(
            closure.meta.capture_mode,
            CallableCaptureMode::LexicalCapture
        );
        assert_eq!(
            publish_value(&Stage1Value::Lambda(closure)),
            Ok(EvalValue::Error(WorksheetErrorCode::Calc))
        );
    }

    #[test]
    fn publish_missing_arg_normalizes_to_empty_text() {
        let value = publish_value(&Stage1Value::Prepared(PreparedArgValue::MissingArg))
            .expect("published value");
        assert_eq!(
            value,
            EvalValue::Text(ExcelText::from_utf16_code_units(Vec::new()))
        );
    }
}
