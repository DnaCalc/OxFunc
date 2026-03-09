use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::a1_refs::{A1Reference, format_relative_target, parse_a1_reference};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, run_values_only_prepared,
};
use crate::resolver::{CallerContext, ReferenceResolver};
use crate::value::{CallArgValue, EvalValue, ReferenceKind, ReferenceLike, WorksheetErrorCode};

pub const INDIRECT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.INDIRECT",
    arity: Arity { min: 1, max: 2 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::VolatileContextual,
    host_interaction: HostInteractionClass::WorkbookState,
    thread_safety: ThreadSafetyClass::HostSerialized,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::CallerContext,
    surface_fec_dependency_profile: FecDependencyProfile::CallerContext,
};

#[derive(Debug, Clone, PartialEq)]
pub enum IndirectEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    InvalidReferenceText(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum R1C1Component {
    Same,
    Absolute(usize),
    Relative(i64),
}

fn parse_a1_flag(arg: Option<&PreparedArgValue>) -> Result<bool, IndirectEvalError> {
    match arg {
        None => Ok(true),
        Some(p) => {
            let n = coerce_prepared_to_number(p).map_err(IndirectEvalError::Coercion)?;
            Ok(n != 0.0)
        }
    }
}

fn parse_ref_text(arg: &PreparedArgValue) -> Result<String, IndirectEvalError> {
    match arg {
        PreparedArgValue::Eval(EvalValue::Text(t)) => {
            let s = t.to_string_lossy().trim().to_string();
            if s.is_empty() {
                return Err(IndirectEvalError::InvalidReferenceText(String::new()));
            }
            Ok(s)
        }
        PreparedArgValue::Eval(EvalValue::Error(code)) => Err(IndirectEvalError::Coercion(
            CoercionError::WorksheetError(*code),
        )),
        PreparedArgValue::MissingArg => Err(IndirectEvalError::Coercion(CoercionError::MissingArg)),
        PreparedArgValue::EmptyCell => Err(IndirectEvalError::Coercion(CoercionError::EmptyCell)),
        PreparedArgValue::Eval(other) => {
            let kind = match other {
                EvalValue::Number(_) => "number",
                EvalValue::Logical(_) => "logical",
                EvalValue::Array(_) => "array",
                EvalValue::Reference(_) => "reference_like",
                EvalValue::Lambda(_) => "lambda_value",
                EvalValue::Text(_) | EvalValue::Error(_) => unreachable!(),
            };
            Err(IndirectEvalError::InvalidReferenceText(kind.to_string()))
        }
    }
}

fn split_prefix(target: &str) -> (Option<String>, &str) {
    if let Some(idx) = target.rfind('!') {
        let prefix = target[..idx].trim();
        let rest = target[idx + 1..].trim();
        if prefix.is_empty() {
            (None, rest)
        } else {
            (Some(prefix.to_string()), rest)
        }
    } else {
        (None, target.trim())
    }
}

fn parse_r1c1_component(token: &str, marker: char) -> Option<(R1C1Component, &str)> {
    let rest = token.strip_prefix(marker)?;
    if let Some(rest) = rest.strip_prefix('[') {
        let end = rest.find(']')?;
        let value = rest[..end].parse::<i64>().ok()?;
        return Some((R1C1Component::Relative(value), &rest[end + 1..]));
    }

    let digits_len = rest
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .map(char::len_utf8)
        .sum::<usize>();
    if digits_len == 0 {
        return Some((R1C1Component::Same, rest));
    }

    let value = rest[..digits_len].parse::<usize>().ok()?;
    Some((R1C1Component::Absolute(value), &rest[digits_len..]))
}

fn resolve_r1c1_component(
    component: R1C1Component,
    current: usize,
) -> Option<usize> {
    match component {
        R1C1Component::Same => Some(current),
        R1C1Component::Absolute(value) => Some(value),
        R1C1Component::Relative(offset) => {
            let base = i64::try_from(current).ok()?;
            let resolved = base.checked_add(offset)?;
            usize::try_from(resolved).ok().filter(|value| *value > 0)
        }
    }
}

fn parse_r1c1_point(token: &str, caller: Option<&CallerContext>) -> Option<(usize, usize)> {
    let (row_component, rest) = parse_r1c1_component(token, 'R')?;
    let (col_component, tail) = parse_r1c1_component(rest, 'C')?;
    if !tail.trim().is_empty() {
        return None;
    }

    let current_row = caller.map(|ctx| ctx.row).unwrap_or(1);
    let current_col = caller.map(|ctx| ctx.col).unwrap_or(1);
    if caller.is_none()
        && matches!(
            row_component,
            R1C1Component::Same | R1C1Component::Relative(_)
        )
    {
        return None;
    }
    if caller.is_none()
        && matches!(
            col_component,
            R1C1Component::Same | R1C1Component::Relative(_)
        )
    {
        return None;
    }
    let row = resolve_r1c1_component(row_component, current_row)?;
    let col = resolve_r1c1_component(col_component, current_col)?;
    Some((row, col))
}

fn parse_r1c1_reference(target: &str, caller: Option<&CallerContext>) -> Option<A1Reference> {
    let (prefix, body) = split_prefix(target);
    let parts: Vec<&str> = body.split(':').collect();
    if parts.is_empty() || parts.len() > 2 {
        return None;
    }
    let (start_row, start_col) = parse_r1c1_point(parts[0], caller)?;
    let (end_row, end_col) = if parts.len() == 2 {
        parse_r1c1_point(parts[1], caller)?
    } else {
        (start_row, start_col)
    };

    Some(A1Reference {
        prefix: prefix.or_else(|| caller.and_then(|ctx| ctx.prefix.clone())),
        start_row: start_row.min(end_row),
        start_col: start_col.min(end_col),
        end_row: start_row.max(end_row),
        end_col: start_col.max(end_col),
    })
}

fn reference_like_from_a1(reference: A1Reference) -> Result<EvalValue, IndirectEvalError> {
    let target = format_relative_target(&reference)
        .ok_or_else(|| IndirectEvalError::InvalidReferenceText("unformattable".to_string()))?;
    Ok(EvalValue::Reference(ReferenceLike {
        kind: if reference.width() == 1 && reference.height() == 1 {
            ReferenceKind::A1
        } else {
            ReferenceKind::Area
        },
        target,
    }))
}

pub fn eval_indirect_adapter_prepared(
    args: &[PreparedArgValue],
    caller_context: Option<&CallerContext>,
) -> Result<EvalValue, IndirectEvalError> {
    let argc = args.len();
    if !INDIRECT_META.arity.accepts(argc) {
        return Err(IndirectEvalError::ArityMismatch {
            expected_min: INDIRECT_META.arity.min,
            expected_max: INDIRECT_META.arity.max,
            actual: argc,
        });
    }

    let text = parse_ref_text(&args[0])?;
    let a1_style = parse_a1_flag(args.get(1))?;

    if a1_style {
        let parsed = parse_a1_reference(&text)
            .ok_or_else(|| IndirectEvalError::InvalidReferenceText(text.clone()))?;
        reference_like_from_a1(parsed)
    } else {
        let parsed = parse_r1c1_reference(&text, caller_context)
            .ok_or_else(|| IndirectEvalError::InvalidReferenceText(text.clone()))?;
        reference_like_from_a1(parsed)
    }
}

pub fn eval_indirect_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, IndirectEvalError> {
    let caller_context = resolver.caller_context();
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_indirect_adapter_prepared(prepared, caller_context.as_ref()),
        IndirectEvalError::Coercion,
    )
}

pub fn map_indirect_error_to_ws(e: &IndirectEvalError) -> WorksheetErrorCode {
    match e {
        IndirectEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        IndirectEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        IndirectEvalError::InvalidReferenceText(_) => WorksheetErrorCode::Ref,
        IndirectEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::ExcelText;

    struct MockResolver {
        caller: Option<CallerContext>,
    }

    impl ReferenceResolver for MockResolver {
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

        fn caller_context(&self) -> Option<CallerContext> {
            self.caller.clone()
        }
    }

    fn text_arg(s: &str) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
            s.encode_utf16().collect(),
        )))
    }

    #[test]
    fn eval_indirect_a1_text_returns_reference_like() {
        let got = eval_indirect_surface(
            &[text_arg("Sheet1!A1")],
            &MockResolver { caller: None },
        );
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::A1,
                target: "Sheet1!A1".to_string(),
            }))
        );
    }

    #[test]
    fn eval_indirect_absolute_r1c1_is_supported() {
        let got = eval_indirect_surface(
            &[
                text_arg("R1C2"),
                CallArgValue::Eval(EvalValue::Number(0.0)),
            ],
            &MockResolver {
                caller: Some(CallerContext {
                    prefix: Some("Sheet1".to_string()),
                    row: 10,
                    col: 10,
                }),
            },
        );
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::A1,
                target: "Sheet1!B1".to_string(),
            }))
        );
    }

    #[test]
    fn eval_indirect_relative_r1c1_uses_caller_context() {
        let got = eval_indirect_surface(
            &[
                text_arg("R[1]C[-1]"),
                CallArgValue::Eval(EvalValue::Number(0.0)),
            ],
            &MockResolver {
                caller: Some(CallerContext {
                    prefix: Some("Sheet1".to_string()),
                    row: 3,
                    col: 3,
                }),
            },
        );
        assert_eq!(
            got,
            Ok(EvalValue::Reference(ReferenceLike {
                kind: ReferenceKind::A1,
                target: "Sheet1!B4".to_string(),
            }))
        );
    }

    #[test]
    fn eval_indirect_r1c1_requires_context_for_relative_refs() {
        let got = eval_indirect_surface(
            &[
                text_arg("R[1]C[1]"),
                CallArgValue::Eval(EvalValue::Number(0.0)),
            ],
            &MockResolver { caller: None },
        );
        assert_eq!(
            got,
            Err(IndirectEvalError::InvalidReferenceText("R[1]C[1]".to_string()))
        );
    }

    #[test]
    fn eval_indirect_rejects_non_text_reference_expression() {
        let got = eval_indirect_surface(
            &[CallArgValue::Eval(EvalValue::Number(1.0))],
            &MockResolver { caller: None },
        );
        assert_eq!(
            got,
            Err(IndirectEvalError::InvalidReferenceText(
                "number".to_string()
            ))
        );
    }
}
