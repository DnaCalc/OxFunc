use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{coerce_prepared_to_text, run_values_only_prepared};
use crate::locale_format::{FormatFailure, LocaleFormatContext};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{CalcValue, CoreValue};
use crate::value::{ExcelText, WorksheetErrorCode};

pub const TEXT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.TEXT",
    arity: Arity::exact(2),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::LocaleProfile,
    surface_fec_dependency_profile: FecDependencyProfile::Composite,
};

#[derive(Debug, Clone, PartialEq)]
pub enum TextEvalError {
    ArityMismatch { expected: usize, actual: usize },
    Coercion(CoercionError),
    Format(FormatFailure),
}

fn logical_text(value: bool) -> ExcelText {
    ExcelText::from_utf16_code_units(
        (if value { "TRUE" } else { "FALSE" })
            .encode_utf16()
            .collect(),
    )
}

fn render_scalar_text_value(
    value: &CalcValue,
    format_code_string: &str,
    ctx: &LocaleFormatContext,
) -> Result<CalcValue, TextEvalError> {
    let rendered = match value.core() {
        CoreValue::Number(n) => ctx
            .formatter
            .render_with_code(&ctx.profile, ctx.date_system, *n, format_code_string)
            .map_err(TextEvalError::Format)?,
        CoreValue::Text(text) => {
            let raw = text.to_string_lossy();
            match ctx
                .parser
                .parse_value_text(&ctx.profile, ctx.date_system, &raw)
            {
                Ok(parsed) => ctx
                    .formatter
                    .render_with_code(&ctx.profile, ctx.date_system, parsed, format_code_string)
                    .map_err(TextEvalError::Format)?,
                Err(_) => text.clone(),
            }
        }
        CoreValue::Logical(b) => logical_text(*b),
        CoreValue::Error(code) => return Ok(CalcValue::error(*code)),
        CoreValue::Empty => ctx
            .formatter
            .render_with_code(&ctx.profile, ctx.date_system, 0.0, format_code_string)
            .map_err(TextEvalError::Format)?,
        CoreValue::Missing => {
            return Err(TextEvalError::Coercion(CoercionError::MissingArg));
        }
        CoreValue::Array(_) | CoreValue::Reference(_) => {
            return Err(TextEvalError::Coercion(
                CoercionError::UnsupportedValueKind("text_arg_kind"),
            ));
        }
    };

    Ok(CalcValue::text(rendered))
}

fn text_cell_from_scalar_result(result: CalcValue) -> CalcValue {
    match result.core() {
        CoreValue::Text(text) => CalcValue::text(text.clone()),
        CoreValue::Error(code) => CalcValue::error(*code),
        other => unreachable!("TEXT scalar rendering returned unexpected value: {other:?}"),
    }
}

pub fn eval_text_adapter_prepared(
    args: &[CalcValue],
    ctx: &LocaleFormatContext,
) -> Result<CalcValue, TextEvalError> {
    if !TEXT_META.arity.accepts(args.len()) {
        return Err(TextEvalError::ArityMismatch {
            expected: TEXT_META.arity.min,
            actual: args.len(),
        });
    }

    let format_code = coerce_prepared_to_text(&args[1]).map_err(TextEvalError::Coercion)?;
    let format_code_string = format_code.to_string_lossy();

    match args[0].core() {
        CoreValue::Array(array) => {
            let cells = array
                .iter_row_major()
                .map(|cell| {
                    let prepared = match cell.core() {
                        CoreValue::Number(n) => CalcValue::number(*n),
                        CoreValue::Text(text) => CalcValue::text(text.clone()),
                        CoreValue::Logical(b) => CalcValue::logical(*b),
                        CoreValue::Error(code) => CalcValue::error(*code),
                        CoreValue::Empty | CoreValue::Missing => CalcValue::empty(),
                        CoreValue::Array(_) | CoreValue::Reference(_) => {
                            CalcValue::error(WorksheetErrorCode::Value)
                        }
                    };
                    render_scalar_text_value(&prepared, &format_code_string, ctx)
                        .map(text_cell_from_scalar_result)
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(CalcValue::array(
                crate::value::CalcArray::new(array.shape(), cells)
                    .expect("TEXT array lift preserves the input shape"),
            ))
        }
        _ => render_scalar_text_value(&args[0], &format_code_string, ctx),
    }
}

pub fn eval_text_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    ctx: &LocaleFormatContext,
) -> Result<CalcValue, TextEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_text_adapter_prepared(prepared, ctx),
        TextEvalError::Coercion,
    )
}

pub fn map_text_error_to_ws(e: &TextEvalError) -> WorksheetErrorCode {
    match e {
        TextEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        TextEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        TextEvalError::Coercion(_) | TextEvalError::Format(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locale_format::test_current_excel_host_context;
    use crate::resolver::ReferenceSystemCapabilities;

    struct NoResolver;
    impl ReferenceSystemProvider for NoResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            ReferenceSystemCapabilities::permissive_local()
        }
        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<CalcValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            Err(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
        }
    }

    #[test]
    fn text_current_host_seed_rows() {
        let ctx = test_current_excel_host_context();
        let got = eval_text_surface(
            &[
                (CalcValue::number(0.125)),
                (CalcValue::text(ExcelText::from_utf16_code_units(
                    "0%".encode_utf16().collect(),
                ))),
            ],
            &NoResolver,
            &ctx,
        );
        assert_eq!(
            got,
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "13%".encode_utf16().collect()
            )))
        );

        let got_text = eval_text_surface(
            &[
                (CalcValue::text(ExcelText::from_utf16_code_units(
                    "x".encode_utf16().collect(),
                ))),
                (CalcValue::text(ExcelText::from_utf16_code_units(
                    "0".encode_utf16().collect(),
                ))),
            ],
            &NoResolver,
            &ctx,
        );
        assert_eq!(
            got_text,
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "x".encode_utf16().collect()
            )))
        );

        let got_logical = eval_text_surface(
            &[
                (CalcValue::logical(true)),
                (CalcValue::text(ExcelText::from_utf16_code_units(
                    "0".encode_utf16().collect(),
                ))),
            ],
            &NoResolver,
            &ctx,
        );
        assert_eq!(
            got_logical,
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "TRUE".encode_utf16().collect()
            )))
        );
    }

    #[test]
    fn text_formats_supported_calendar_codes_and_lifts_arrays() {
        let ctx = test_current_excel_host_context();

        let got_month = eval_text_surface(
            &[
                (CalcValue::number(45474.0)),
                (CalcValue::text(ExcelText::from_utf16_code_units(
                    "MMMM".encode_utf16().collect(),
                ))),
            ],
            &NoResolver,
            &ctx,
        );
        assert_eq!(
            got_month,
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "July".encode_utf16().collect(),
            )))
        );

        let got_dd = eval_text_surface(
            &[
                (CalcValue::number(15.0)),
                (CalcValue::text(ExcelText::from_utf16_code_units(
                    "00".encode_utf16().collect(),
                ))),
            ],
            &NoResolver,
            &ctx,
        );
        assert_eq!(
            got_dd,
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "15".encode_utf16().collect(),
            )))
        );

        let got_headers = eval_text_surface(
            &[
                (CalcValue::array(
                    crate::value::CalcArray::from_rows(vec![vec![
                        crate::value::CalcValue::number(45298.0),
                        crate::value::CalcValue::number(45299.0),
                        crate::value::CalcValue::number(45300.0),
                    ]])
                    .unwrap(),
                )),
                (CalcValue::text(ExcelText::from_utf16_code_units(
                    "DDD".encode_utf16().collect(),
                ))),
            ],
            &NoResolver,
            &ctx,
        );
        assert_eq!(
            got_headers,
            Ok(CalcValue::array(
                crate::value::CalcArray::from_rows(vec![vec![
                    crate::value::CalcValue::text(ExcelText::from_utf16_code_units(
                        "Sun".encode_utf16().collect(),
                    )),
                    crate::value::CalcValue::text(ExcelText::from_utf16_code_units(
                        "Mon".encode_utf16().collect(),
                    )),
                    crate::value::CalcValue::text(ExcelText::from_utf16_code_units(
                        "Tue".encode_utf16().collect(),
                    )),
                ]])
                .unwrap()
            ))
        );
    }

    #[test]
    fn text_conditional_section_supports_in_range_and_out_of_range_dates() {
        let ctx = test_current_excel_host_context();
        let in_range = eval_text_surface(
            &[
                (CalcValue::number(45366.0)),
                (CalcValue::text(ExcelText::from_utf16_code_units(
                    "[<45352] ;[>45382] ;dd".encode_utf16().collect(),
                ))),
            ],
            &NoResolver,
            &ctx,
        );
        assert_eq!(
            in_range,
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                "15".encode_utf16().collect(),
            )))
        );

        let out_of_range = eval_text_surface(
            &[
                (CalcValue::number(45350.0)),
                (CalcValue::text(ExcelText::from_utf16_code_units(
                    "[<45352] ;[>45382] ;dd".encode_utf16().collect(),
                ))),
            ],
            &NoResolver,
            &ctx,
        );
        assert_eq!(
            out_of_range,
            Ok(CalcValue::text(ExcelText::from_utf16_code_units(
                " ".encode_utf16().collect(),
            )))
        );
    }
}
