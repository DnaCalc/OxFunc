use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, expand_arg_values_only, prepare_arg_values_only, prepare_args_values_only,
};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, ExcelText, WorksheetErrorCode};

const TEXT_META_BASE: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IM_TEXT_BASE",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::None,
};
const NUMBER_META_BASE: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IM_NUMBER_BASE",
    ..TEXT_META_BASE
};

pub const COMPLEX_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.COMPLEX",
    arity: Arity { min: 2, max: 3 },
    ..TEXT_META_BASE
};
pub const IMABS_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IMABS",
    ..NUMBER_META_BASE
};
pub const IMAGINARY_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IMAGINARY",
    ..NUMBER_META_BASE
};
pub const IMARGUMENT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IMARGUMENT",
    ..NUMBER_META_BASE
};
pub const IMCONJUGATE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IMCONJUGATE",
    ..TEXT_META_BASE
};
pub const IMCOS_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IMCOS",
    ..TEXT_META_BASE
};
pub const IMCOSH_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IMCOSH",
    ..TEXT_META_BASE
};
pub const IMCOT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IMCOT",
    ..TEXT_META_BASE
};
pub const IMCSC_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IMCSC",
    ..TEXT_META_BASE
};
pub const IMCSCH_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IMCSCH",
    ..TEXT_META_BASE
};
pub const IMDIV_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IMDIV",
    arity: Arity::exact(2),
    ..TEXT_META_BASE
};
pub const IMEXP_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IMEXP",
    ..TEXT_META_BASE
};
pub const IMLN_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IMLN",
    ..TEXT_META_BASE
};
pub const IMLOG10_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IMLOG10",
    ..TEXT_META_BASE
};
pub const IMLOG2_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IMLOG2",
    ..TEXT_META_BASE
};
pub const IMPOWER_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IMPOWER",
    arity: Arity::exact(2),
    ..TEXT_META_BASE
};
pub const IMPRODUCT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IMPRODUCT",
    arity: Arity { min: 1, max: 255 },
    ..TEXT_META_BASE
};
pub const IMREAL_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IMREAL",
    ..NUMBER_META_BASE
};
pub const IMSEC_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IMSEC",
    ..TEXT_META_BASE
};
pub const IMSECH_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IMSECH",
    ..TEXT_META_BASE
};
pub const IMSIN_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IMSIN",
    ..TEXT_META_BASE
};
pub const IMSINH_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IMSINH",
    ..TEXT_META_BASE
};
pub const IMSQRT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IMSQRT",
    ..TEXT_META_BASE
};
pub const IMSUB_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IMSUB",
    arity: Arity::exact(2),
    ..TEXT_META_BASE
};
pub const IMSUM_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IMSUM",
    arity: Arity { min: 1, max: 255 },
    ..TEXT_META_BASE
};
pub const IMTAN_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.IMTAN",
    ..TEXT_META_BASE
};

#[derive(Debug, Clone, PartialEq)]
pub enum ComplexFamilyEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ParsedComplex {
    re: f64,
    im: f64,
    suffix: Option<char>,
}
impl ParsedComplex {
    const fn new(re: f64, im: f64, suffix: Option<char>) -> Self {
        Self { re, im, suffix }
    }
    fn add(self, other: Self) -> Self {
        Self::new(self.re + other.re, self.im + other.im, None)
    }
    fn sub(self, other: Self) -> Self {
        Self::new(self.re - other.re, self.im - other.im, None)
    }
    fn mul(self, other: Self) -> Self {
        Self::new(
            self.re * other.re - self.im * other.im,
            self.re * other.im + self.im * other.re,
            None,
        )
    }
    fn div(self, other: Self) -> Result<Self, ComplexFamilyEvalError> {
        let denom = other.re * other.re + other.im * other.im;
        if denom == 0.0 {
            return Err(ComplexFamilyEvalError::Domain(WorksheetErrorCode::Num));
        }
        Ok(Self::new(
            (self.re * other.re + self.im * other.im) / denom,
            (self.im * other.re - self.re * other.im) / denom,
            None,
        ))
    }
    fn conj(self) -> Self {
        Self::new(self.re, -self.im, None)
    }
    fn abs(self) -> f64 {
        self.re.hypot(self.im)
    }
    fn argument(self) -> Result<f64, ComplexFamilyEvalError> {
        if self.re == 0.0 && self.im == 0.0 {
            return Err(ComplexFamilyEvalError::Domain(WorksheetErrorCode::Div0));
        }
        Ok(self.im.atan2(self.re))
    }
    fn exp(self) -> Self {
        let s = self.re.exp();
        Self::new(s * self.im.cos(), s * self.im.sin(), None)
    }
    fn ln(self) -> Result<Self, ComplexFamilyEvalError> {
        let mag = self.abs();
        if mag == 0.0 {
            return Err(ComplexFamilyEvalError::Domain(WorksheetErrorCode::Num));
        }
        Ok(Self::new(mag.ln(), self.im.atan2(self.re), None))
    }
    fn sqrt(self) -> Self {
        let r = self.abs().sqrt();
        let a = self.im.atan2(self.re) / 2.0;
        Self::new(r * a.cos(), r * a.sin(), None)
    }
    fn sin(self) -> Self {
        Self::new(
            self.re.sin() * self.im.cosh(),
            self.re.cos() * self.im.sinh(),
            None,
        )
    }
    fn cos(self) -> Self {
        Self::new(
            self.re.cos() * self.im.cosh(),
            -self.re.sin() * self.im.sinh(),
            None,
        )
    }
    fn tan(self) -> Result<Self, ComplexFamilyEvalError> {
        let denom = (2.0 * self.re).cos() + (2.0 * self.im).cosh();
        if denom == 0.0 {
            return Err(ComplexFamilyEvalError::Domain(WorksheetErrorCode::Num));
        }
        Ok(Self::new(
            (2.0 * self.re).sin() / denom,
            (2.0 * self.im).sinh() / denom,
            None,
        ))
    }
    fn sinh(self) -> Self {
        Self::new(
            self.re.sinh() * self.im.cos(),
            self.re.cosh() * self.im.sin(),
            None,
        )
    }
    fn cosh(self) -> Self {
        Self::new(
            self.re.cosh() * self.im.cos(),
            self.re.sinh() * self.im.sin(),
            None,
        )
    }
    fn cot(self) -> Result<Self, ComplexFamilyEvalError> {
        let denom = (2.0 * self.im).cosh() - (2.0 * self.re).cos();
        if denom == 0.0 {
            return Err(ComplexFamilyEvalError::Domain(WorksheetErrorCode::Num));
        }
        Ok(Self::new(
            (2.0 * self.re).sin() / denom,
            -(2.0 * self.im).sinh() / denom,
            None,
        ))
    }
    fn sec(self) -> Result<Self, ComplexFamilyEvalError> {
        ParsedComplex::new(1.0, 0.0, None).div(self.cos())
    }
    fn csc(self) -> Result<Self, ComplexFamilyEvalError> {
        ParsedComplex::new(1.0, 0.0, None).div(self.sin())
    }
    fn sech(self) -> Result<Self, ComplexFamilyEvalError> {
        ParsedComplex::new(1.0, 0.0, None).div(self.cosh())
    }
    fn csch(self) -> Result<Self, ComplexFamilyEvalError> {
        ParsedComplex::new(1.0, 0.0, None).div(self.sinh())
    }
    fn power(self, exponent: f64) -> Result<Self, ComplexFamilyEvalError> {
        if self.re == 0.0 && self.im == 0.0 {
            if exponent < 0.0 {
                return Err(ComplexFamilyEvalError::Domain(WorksheetErrorCode::Num));
            }
            if exponent == 0.0 {
                return Ok(Self::new(1.0, 0.0, None));
            }
            return Ok(Self::new(0.0, 0.0, None));
        }
        let scale = self.abs().powf(exponent);
        let angle = self.im.atan2(self.re) * exponent;
        Ok(Self::new(scale * angle.cos(), scale * angle.sin(), None))
    }
}

fn text_from_string(s: String) -> EvalValue {
    EvalValue::Text(ExcelText::from_utf16_code_units(s.encode_utf16().collect()))
}
fn normalize_zero(n: f64) -> f64 {
    if n == 0.0 { 0.0 } else { n }
}
fn normalize_display_number(n: f64) -> f64 {
    let n = normalize_zero(n);
    if (n - n.round()).abs() < 1.0e-12 {
        n.round()
    } else {
        n
    }
}
fn number_to_excel_text(n: f64) -> String {
    let n = normalize_display_number(n);
    if n == 0.0 {
        return "0".to_string();
    }
    if !n.is_finite() {
        return format!("{}", n).replace('e', "E");
    }

    let sign = if n < 0.0 { "-" } else { "" };
    let scientific = format!("{:.14e}", n.abs());
    let Some((mantissa, exponent_text)) = scientific.split_once('e') else {
        return format!("{}", n).replace('e', "E");
    };
    let Ok(exponent) = exponent_text.parse::<isize>() else {
        return format!("{}", n).replace('e', "E");
    };

    let digits: String = mantissa.chars().filter(|ch| *ch != '.').collect();
    let decimal_pos = exponent + 1;
    let mut out = if decimal_pos <= 0 {
        format!("0.{}{}", "0".repeat((-decimal_pos) as usize), digits)
    } else if decimal_pos as usize >= digits.len() {
        format!(
            "{}{}",
            digits,
            "0".repeat(decimal_pos as usize - digits.len())
        )
    } else {
        let split = decimal_pos as usize;
        format!("{}.{}", &digits[..split], &digits[split..])
    };

    if let Some(dot) = out.find('.') {
        while out.ends_with('0') {
            out.pop();
        }
        if out.len() == dot + 1 {
            out.pop();
        }
    }

    format!("{sign}{out}").replace('e', "E")
}
fn format_complex_text(value: ParsedComplex, suffix: char) -> String {
    let re = normalize_zero(value.re);
    let im = normalize_zero(value.im);
    if im == 0.0 {
        return number_to_excel_text(re);
    }
    let coeff = if im.abs() == 1.0 {
        String::new()
    } else {
        number_to_excel_text(im.abs())
    };
    if re == 0.0 {
        if im < 0.0 {
            format!("-{}{suffix}", coeff)
        } else {
            format!("{}{suffix}", coeff)
        }
    } else {
        let sign = if im < 0.0 { '-' } else { '+' };
        format!("{}{sign}{}{suffix}", number_to_excel_text(re), coeff)
    }
}

fn parse_real_number(text: &str) -> Option<f64> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    let parsed = trimmed.parse::<f64>().ok()?;
    parsed.is_finite().then_some(parsed)
}
fn find_real_imag_split(body: &str) -> Option<usize> {
    let bytes = body.as_bytes();
    let mut out = None;
    for i in 1..bytes.len() {
        let ch = bytes[i] as char;
        let prev = bytes[i - 1] as char;
        if (ch == '+' || ch == '-') && prev != 'e' && prev != 'E' {
            out = Some(i);
        }
    }
    out
}
fn parse_imag_coeff(text: &str) -> Option<f64> {
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed == "+" {
        Some(1.0)
    } else if trimmed == "-" {
        Some(-1.0)
    } else {
        parse_real_number(trimmed)
    }
}
fn parse_complex_text(raw: &str) -> Option<ParsedComplex> {
    let text = raw.trim();
    if text.is_empty() {
        return Some(ParsedComplex::new(0.0, 0.0, None));
    }
    if let Some(last) = text.chars().last() {
        if last == 'i' || last == 'j' {
            let body = &text[..text.len() - last.len_utf8()];
            if let Some(split) = find_real_imag_split(body) {
                return Some(ParsedComplex::new(
                    parse_real_number(&body[..split])?,
                    parse_imag_coeff(&body[split..])?,
                    Some(last),
                ));
            }
            return Some(ParsedComplex::new(0.0, parse_imag_coeff(body)?, Some(last)));
        }
    }
    parse_real_number(text).map(|re| ParsedComplex::new(re, 0.0, None))
}
fn prepared_scalar_to_complex(
    arg: &PreparedArgValue,
) -> Result<ParsedComplex, ComplexFamilyEvalError> {
    match arg {
        PreparedArgValue::Eval(EvalValue::Number(n)) => Ok(ParsedComplex::new(*n, 0.0, None)),
        PreparedArgValue::Eval(EvalValue::Text(t)) => parse_complex_text(&t.to_string_lossy())
            .ok_or(ComplexFamilyEvalError::Domain(WorksheetErrorCode::Num)),
        PreparedArgValue::Eval(EvalValue::Logical(_)) => {
            Err(ComplexFamilyEvalError::Domain(WorksheetErrorCode::Value))
        }
        PreparedArgValue::Eval(EvalValue::Error(code)) => Err(ComplexFamilyEvalError::Coercion(
            CoercionError::WorksheetError(*code),
        )),
        PreparedArgValue::Eval(EvalValue::Array(_))
        | PreparedArgValue::Eval(EvalValue::Reference(_))
        | PreparedArgValue::Eval(EvalValue::Lambda(_)) => {
            Err(ComplexFamilyEvalError::Domain(WorksheetErrorCode::Value))
        }
        PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell => {
            Ok(ParsedComplex::new(0.0, 0.0, None))
        }
    }
}
fn prepared_scalar_to_real_for_complex(
    arg: &PreparedArgValue,
) -> Result<f64, ComplexFamilyEvalError> {
    match arg {
        PreparedArgValue::Eval(EvalValue::Number(n)) => Ok(*n),
        PreparedArgValue::Eval(EvalValue::Text(t)) => parse_real_number(&t.to_string_lossy())
            .ok_or(ComplexFamilyEvalError::Domain(WorksheetErrorCode::Value)),
        PreparedArgValue::Eval(EvalValue::Logical(_)) => {
            Err(ComplexFamilyEvalError::Domain(WorksheetErrorCode::Value))
        }
        PreparedArgValue::Eval(EvalValue::Error(code)) => Err(ComplexFamilyEvalError::Coercion(
            CoercionError::WorksheetError(*code),
        )),
        PreparedArgValue::Eval(EvalValue::Array(_))
        | PreparedArgValue::Eval(EvalValue::Reference(_))
        | PreparedArgValue::Eval(EvalValue::Lambda(_)) => {
            Err(ComplexFamilyEvalError::Domain(WorksheetErrorCode::Value))
        }
        PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell => Ok(0.0),
    }
}
fn prepared_scalar_to_suffix(
    arg: Option<&PreparedArgValue>,
) -> Result<char, ComplexFamilyEvalError> {
    let Some(arg) = arg else {
        return Ok('i');
    };
    match arg {
        PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell => Ok('i'),
        PreparedArgValue::Eval(EvalValue::Text(t)) => match t.to_string_lossy().trim() {
            "" | "i" => Ok('i'),
            "j" => Ok('j'),
            _ => Err(ComplexFamilyEvalError::Domain(WorksheetErrorCode::Value)),
        },
        PreparedArgValue::Eval(EvalValue::Error(code)) => Err(ComplexFamilyEvalError::Coercion(
            CoercionError::WorksheetError(*code),
        )),
        _ => Err(ComplexFamilyEvalError::Domain(WorksheetErrorCode::Value)),
    }
}
fn suffix_from_operands(values: &[ParsedComplex]) -> Result<char, ComplexFamilyEvalError> {
    let mut suffix = None;
    for value in values {
        if let Some(ch) = value.suffix {
            match suffix {
                None => suffix = Some(ch),
                Some(current) if current == ch => {}
                Some(_) => return Err(ComplexFamilyEvalError::Domain(WorksheetErrorCode::Value)),
            }
        }
    }
    Ok(suffix.unwrap_or('i'))
}

fn expand_complex_aggregate_args(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<Vec<PreparedArgValue>, ComplexFamilyEvalError> {
    let mut values = Vec::new();
    for arg in args {
        values.extend(
            expand_arg_values_only(arg, resolver).map_err(ComplexFamilyEvalError::Coercion)?,
        );
    }
    Ok(values)
}

fn unary_text(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    meta: &FunctionMeta,
    kernel: impl FnOnce(ParsedComplex) -> Result<ParsedComplex, ComplexFamilyEvalError>,
) -> Result<EvalValue, ComplexFamilyEvalError> {
    if !meta.arity.accepts(args.len()) {
        return Err(ComplexFamilyEvalError::ArityMismatch {
            expected_min: meta.arity.min,
            expected_max: meta.arity.max,
            actual: args.len(),
        });
    }
    let prepared =
        prepare_arg_values_only(&args[0], resolver).map_err(ComplexFamilyEvalError::Coercion)?;
    let value = prepared_scalar_to_complex(&prepared)?;
    let suffix = suffix_from_operands(&[value])?;
    Ok(text_from_string(format_complex_text(
        kernel(value)?,
        suffix,
    )))
}
fn unary_number(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    meta: &FunctionMeta,
    kernel: impl FnOnce(ParsedComplex) -> Result<f64, ComplexFamilyEvalError>,
) -> Result<EvalValue, ComplexFamilyEvalError> {
    if !meta.arity.accepts(args.len()) {
        return Err(ComplexFamilyEvalError::ArityMismatch {
            expected_min: meta.arity.min,
            expected_max: meta.arity.max,
            actual: args.len(),
        });
    }
    let prepared =
        prepare_arg_values_only(&args[0], resolver).map_err(ComplexFamilyEvalError::Coercion)?;
    Ok(EvalValue::Number(normalize_zero(kernel(
        prepared_scalar_to_complex(&prepared)?,
    )?)))
}

pub fn eval_complex_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, ComplexFamilyEvalError> {
    if !COMPLEX_META.arity.accepts(args.len()) {
        return Err(ComplexFamilyEvalError::ArityMismatch {
            expected_min: COMPLEX_META.arity.min,
            expected_max: COMPLEX_META.arity.max,
            actual: args.len(),
        });
    }
    let prepared =
        prepare_args_values_only(args, resolver).map_err(ComplexFamilyEvalError::Coercion)?;
    Ok(text_from_string(format_complex_text(
        ParsedComplex::new(
            prepared_scalar_to_real_for_complex(&prepared[0])?,
            prepared_scalar_to_real_for_complex(&prepared[1])?,
            None,
        ),
        prepared_scalar_to_suffix(prepared.get(2))?,
    )))
}
pub fn eval_imabs_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, ComplexFamilyEvalError> {
    unary_number(args, resolver, &IMABS_META, |z| Ok(z.abs()))
}
pub fn eval_imaginary_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, ComplexFamilyEvalError> {
    unary_number(args, resolver, &IMAGINARY_META, |z| Ok(z.im))
}
pub fn eval_imargument_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, ComplexFamilyEvalError> {
    unary_number(args, resolver, &IMARGUMENT_META, |z| z.argument())
}
pub fn eval_imreal_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, ComplexFamilyEvalError> {
    unary_number(args, resolver, &IMREAL_META, |z| Ok(z.re))
}
pub fn eval_imconjugate_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, ComplexFamilyEvalError> {
    unary_text(args, resolver, &IMCONJUGATE_META, |z| Ok(z.conj()))
}
pub fn eval_imcos_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, ComplexFamilyEvalError> {
    unary_text(args, resolver, &IMCOS_META, |z| Ok(z.cos()))
}
pub fn eval_imcosh_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, ComplexFamilyEvalError> {
    unary_text(args, resolver, &IMCOSH_META, |z| Ok(z.cosh()))
}
pub fn eval_imcot_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, ComplexFamilyEvalError> {
    unary_text(args, resolver, &IMCOT_META, |z| z.cot())
}
pub fn eval_imcsc_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, ComplexFamilyEvalError> {
    unary_text(args, resolver, &IMCSC_META, |z| z.csc())
}
pub fn eval_imcsch_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, ComplexFamilyEvalError> {
    unary_text(args, resolver, &IMCSCH_META, |z| z.csch())
}
pub fn eval_imexp_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, ComplexFamilyEvalError> {
    unary_text(args, resolver, &IMEXP_META, |z| Ok(z.exp()))
}
pub fn eval_imln_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, ComplexFamilyEvalError> {
    unary_text(args, resolver, &IMLN_META, |z| z.ln())
}
pub fn eval_imlog10_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, ComplexFamilyEvalError> {
    unary_text(args, resolver, &IMLOG10_META, |z| {
        let ln = z.ln()?;
        Ok(ParsedComplex::new(
            ln.re / 10.0_f64.ln(),
            ln.im / 10.0_f64.ln(),
            None,
        ))
    })
}
pub fn eval_imlog2_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, ComplexFamilyEvalError> {
    unary_text(args, resolver, &IMLOG2_META, |z| {
        let ln = z.ln()?;
        Ok(ParsedComplex::new(
            ln.re / 2.0_f64.ln(),
            ln.im / 2.0_f64.ln(),
            None,
        ))
    })
}
pub fn eval_imsec_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, ComplexFamilyEvalError> {
    unary_text(args, resolver, &IMSEC_META, |z| z.sec())
}
pub fn eval_imsech_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, ComplexFamilyEvalError> {
    unary_text(args, resolver, &IMSECH_META, |z| z.sech())
}
pub fn eval_imsin_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, ComplexFamilyEvalError> {
    unary_text(args, resolver, &IMSIN_META, |z| Ok(z.sin()))
}
pub fn eval_imsinh_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, ComplexFamilyEvalError> {
    unary_text(args, resolver, &IMSINH_META, |z| Ok(z.sinh()))
}
pub fn eval_imsqrt_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, ComplexFamilyEvalError> {
    unary_text(args, resolver, &IMSQRT_META, |z| Ok(z.sqrt()))
}
pub fn eval_imtan_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, ComplexFamilyEvalError> {
    unary_text(args, resolver, &IMTAN_META, |z| z.tan())
}

pub fn eval_imdiv_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, ComplexFamilyEvalError> {
    if !IMDIV_META.arity.accepts(args.len()) {
        return Err(ComplexFamilyEvalError::ArityMismatch {
            expected_min: IMDIV_META.arity.min,
            expected_max: IMDIV_META.arity.max,
            actual: args.len(),
        });
    }
    let prepared =
        prepare_args_values_only(args, resolver).map_err(ComplexFamilyEvalError::Coercion)?;
    let lhs = prepared_scalar_to_complex(&prepared[0])?;
    let rhs = prepared_scalar_to_complex(&prepared[1])?;
    Ok(text_from_string(format_complex_text(
        lhs.div(rhs)?,
        suffix_from_operands(&[lhs, rhs])?,
    )))
}
pub fn eval_impower_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, ComplexFamilyEvalError> {
    if !IMPOWER_META.arity.accepts(args.len()) {
        return Err(ComplexFamilyEvalError::ArityMismatch {
            expected_min: IMPOWER_META.arity.min,
            expected_max: IMPOWER_META.arity.max,
            actual: args.len(),
        });
    }
    let prepared =
        prepare_args_values_only(args, resolver).map_err(ComplexFamilyEvalError::Coercion)?;
    let base = prepared_scalar_to_complex(&prepared[0])?;
    Ok(text_from_string(format_complex_text(
        base.power(prepared_scalar_to_real_for_complex(&prepared[1])?)?,
        suffix_from_operands(&[base])?,
    )))
}
pub fn eval_imsub_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, ComplexFamilyEvalError> {
    if !IMSUB_META.arity.accepts(args.len()) {
        return Err(ComplexFamilyEvalError::ArityMismatch {
            expected_min: IMSUB_META.arity.min,
            expected_max: IMSUB_META.arity.max,
            actual: args.len(),
        });
    }
    let prepared =
        prepare_args_values_only(args, resolver).map_err(ComplexFamilyEvalError::Coercion)?;
    let lhs = prepared_scalar_to_complex(&prepared[0])?;
    let rhs = prepared_scalar_to_complex(&prepared[1])?;
    Ok(text_from_string(format_complex_text(
        lhs.sub(rhs),
        suffix_from_operands(&[lhs, rhs])?,
    )))
}
pub fn eval_imsum_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, ComplexFamilyEvalError> {
    if !IMSUM_META.arity.accepts(args.len()) {
        return Err(ComplexFamilyEvalError::ArityMismatch {
            expected_min: IMSUM_META.arity.min,
            expected_max: IMSUM_META.arity.max,
            actual: args.len(),
        });
    }
    let prepared = expand_complex_aggregate_args(args, resolver)?;
    let values = prepared
        .iter()
        .map(prepared_scalar_to_complex)
        .collect::<Result<Vec<_>, _>>()?;
    let sum = values
        .iter()
        .copied()
        .fold(ParsedComplex::new(0.0, 0.0, None), |acc, value| {
            acc.add(value)
        });
    Ok(text_from_string(format_complex_text(
        sum,
        suffix_from_operands(&values)?,
    )))
}
pub fn eval_improduct_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, ComplexFamilyEvalError> {
    if !IMPRODUCT_META.arity.accepts(args.len()) {
        return Err(ComplexFamilyEvalError::ArityMismatch {
            expected_min: IMPRODUCT_META.arity.min,
            expected_max: IMPRODUCT_META.arity.max,
            actual: args.len(),
        });
    }
    let prepared = expand_complex_aggregate_args(args, resolver)?;
    let values = prepared
        .iter()
        .map(prepared_scalar_to_complex)
        .collect::<Result<Vec<_>, _>>()?;
    let product = values
        .iter()
        .copied()
        .fold(ParsedComplex::new(1.0, 0.0, None), |acc, value| {
            acc.mul(value)
        });
    Ok(text_from_string(format_complex_text(
        product,
        suffix_from_operands(&values)?,
    )))
}

pub fn map_complex_family_error_to_ws(error: &ComplexFamilyEvalError) -> WorksheetErrorCode {
    match error {
        ComplexFamilyEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        ComplexFamilyEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        ComplexFamilyEvalError::Coercion(_) => WorksheetErrorCode::Value,
        ComplexFamilyEvalError::Domain(code) => *code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NoResolver;
    impl ReferenceResolver for NoResolver {
        fn capabilities(&self) -> crate::resolver::ResolverCapabilities {
            crate::resolver::ResolverCapabilities::permissive_local()
        }
        fn resolve_reference(
            &self,
            reference: &crate::value::ReferenceLike,
        ) -> Result<EvalValue, crate::resolver::RefResolutionError> {
            Err(crate::resolver::RefResolutionError::UnresolvedReference {
                target: reference.target.clone(),
            })
        }
    }

    fn txt(s: &str) -> CallArgValue {
        CallArgValue::Eval(text_from_string(s.to_string()))
    }
    fn text_array_row(values: &[&str]) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Array(
            crate::value::EvalArray::from_rows(vec![
                values
                    .iter()
                    .map(|value| {
                        crate::value::ArrayCellValue::Text(ExcelText::from_interop_assignment(
                            value,
                        ))
                    })
                    .collect(),
            ])
            .unwrap(),
        ))
    }
    fn num(n: f64) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Number(n))
    }
    fn bool_arg(b: bool) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Logical(b))
    }
    fn text_result(value: EvalValue) -> String {
        match value {
            EvalValue::Text(t) => t.to_string_lossy(),
            other => panic!("expected text result, got {other:?}"),
        }
    }
    fn parsed_result(value: EvalValue) -> ParsedComplex {
        parse_complex_text(&text_result(value)).unwrap()
    }
    fn assert_close(value: EvalValue, re: f64, im: f64, suffix: char) {
        let got = parsed_result(value);
        assert_eq!(got.suffix.unwrap_or('i'), suffix);
        assert!((got.re - re).abs() < 1.0e-10, "re {} vs {}", got.re, re);
        assert!((got.im - im).abs() < 1.0e-10, "im {} vs {}", got.im, im);
    }

    #[test]
    fn complex_and_parser_cover_common_excel_lanes() {
        assert_eq!(
            text_result(eval_complex_surface(&[num(3.0), num(4.0)], &NoResolver).unwrap()),
            "3+4i"
        );
        assert_eq!(
            text_result(eval_complex_surface(&[num(0.0), num(1.0)], &NoResolver).unwrap()),
            "i"
        );
        assert_eq!(
            text_result(
                eval_complex_surface(&[num(3.0), num(4.0), txt("j")], &NoResolver).unwrap()
            ),
            "3+4j"
        );
        assert_eq!(
            eval_complex_surface(&[bool_arg(true), num(1.0)], &NoResolver),
            Err(ComplexFamilyEvalError::Domain(WorksheetErrorCode::Value))
        );
        assert_eq!(
            parse_complex_text("3+j"),
            Some(ParsedComplex::new(3.0, 1.0, Some('j')))
        );
        assert_eq!(
            parse_complex_text("-i"),
            Some(ParsedComplex::new(0.0, -1.0, Some('i')))
        );
        assert_eq!(parse_complex_text("1+2J"), None);
    }

    #[test]
    fn arithmetic_extractors_and_suffix_propagation_match_probes() {
        assert_eq!(
            text_result(eval_imsum_surface(&[txt("3+4i"), txt("1-2i")], &NoResolver).unwrap()),
            "4+2i"
        );
        assert_eq!(
            text_result(eval_imsub_surface(&[txt("3+4i"), txt("1-2i")], &NoResolver).unwrap()),
            "2+6i"
        );
        assert_eq!(
            text_result(eval_improduct_surface(&[txt("3+4i"), txt("1-2i")], &NoResolver).unwrap()),
            "11-2i"
        );
        assert_eq!(
            text_result(eval_imdiv_surface(&[txt("3+4i"), txt("1-2i")], &NoResolver).unwrap()),
            "-1+2i"
        );
        assert_eq!(
            text_result(eval_imsum_surface(&[txt("3"), txt("1+2j")], &NoResolver).unwrap()),
            "4+2j"
        );
        assert_eq!(
            text_result(eval_imconjugate_surface(&[txt("3+4j")], &NoResolver).unwrap()),
            "3-4j"
        );
        assert_eq!(
            eval_imsum_surface(&[txt("i"), txt("j")], &NoResolver),
            Err(ComplexFamilyEvalError::Domain(WorksheetErrorCode::Value))
        );
        assert_eq!(
            eval_imabs_surface(&[txt("3+4i")], &NoResolver),
            Ok(EvalValue::Number(5.0))
        );
        assert_eq!(
            eval_imreal_surface(&[txt("3+4i")], &NoResolver),
            Ok(EvalValue::Number(3.0))
        );
        assert_eq!(
            eval_imaginary_surface(&[txt("3+4i")], &NoResolver),
            Ok(EvalValue::Number(4.0))
        );
    }

    #[test]
    fn aggregate_complex_functions_flatten_array_literals() {
        assert_eq!(
            text_result(
                eval_improduct_surface(
                    &[text_array_row(&["3+4i", "3+4i"]), txt("1-2i")],
                    &NoResolver,
                )
                .unwrap()
            ),
            "41+38i"
        );
        assert_eq!(
            text_result(
                eval_improduct_surface(
                    &[txt("3+4i"), text_array_row(&["1-2i", "1-2i"])],
                    &NoResolver,
                )
                .unwrap()
            ),
            "7-24i"
        );
        assert_eq!(
            text_result(
                eval_imsum_surface(
                    &[text_array_row(&["3+4i", "3+4i"]), txt("1-2i")],
                    &NoResolver,
                )
                .unwrap()
            ),
            "7+6i"
        );
        assert_eq!(
            text_result(
                eval_imsum_surface(
                    &[txt("3+4i"), text_array_row(&["1-2i", "1-2i"])],
                    &NoResolver,
                )
                .unwrap()
            ),
            "5"
        );
    }

    #[test]
    fn tan_and_cot_use_excel_stable_identities() {
        assert_eq!(
            text_result(eval_imtan_surface(&[txt("3+4i")], &NoResolver).unwrap()),
            "-0.000187346204629478+0.999355987381473i"
        );
        assert_eq!(
            text_result(eval_imcot_surface(&[txt("3+4i")], &NoResolver).unwrap()),
            "-0.000187587737983659-1.00064439247156i"
        );
    }

    #[test]
    fn transcendental_lanes_match_seed_rows() {
        assert_eq!(
            text_result(eval_imconjugate_surface(&[txt("3+4i")], &NoResolver).unwrap()),
            "3-4i"
        );
        assert_close(
            eval_imsqrt_surface(&[txt("3+4i")], &NoResolver).unwrap(),
            2.0,
            1.0,
            'i',
        );
        assert_close(
            eval_imexp_surface(&[txt("3+4i")], &NoResolver).unwrap(),
            -13.128783081462158,
            -15.200784463067954,
            'i',
        );
        assert_close(
            eval_imln_surface(&[txt("3+4i")], &NoResolver).unwrap(),
            1.6094379124341003,
            0.9272952180016122,
            'i',
        );
        assert_close(
            eval_imlog10_surface(&[txt("3+4i")], &NoResolver).unwrap(),
            0.6989700043360187,
            0.4027191962733731,
            'i',
        );
        assert_close(
            eval_imlog2_surface(&[txt("3+4i")], &NoResolver).unwrap(),
            2.321928094887362,
            1.3378042124509761,
            'i',
        );
        assert_close(
            eval_imsin_surface(&[txt("3+4i")], &NoResolver).unwrap(),
            3.853738037919377,
            -27.016813258003932,
            'i',
        );
        assert_close(
            eval_imcos_surface(&[txt("3+4i")], &NoResolver).unwrap(),
            -27.034945603074224,
            -3.851153334811777,
            'i',
        );
        assert_close(
            eval_imtan_surface(&[txt("3+4i")], &NoResolver).unwrap(),
            -0.00018734620462949035,
            0.9993559873814731,
            'i',
        );
        assert_close(
            eval_imsinh_surface(&[txt("3+4i")], &NoResolver).unwrap(),
            -6.5481200409110025,
            -7.61923172032141,
            'i',
        );
        assert_close(
            eval_imcosh_surface(&[txt("3+4i")], &NoResolver).unwrap(),
            -6.580663040551157,
            -7.581552742746545,
            'i',
        );
        assert_close(
            eval_imsec_surface(&[txt("3+4i")], &NoResolver).unwrap(),
            -0.03625349691586887,
            0.005164344607753178,
            'i',
        );
        assert_close(
            eval_imsech_surface(&[txt("3+4i")], &NoResolver).unwrap(),
            -0.06529402785794704,
            0.07522496030277322,
            'i',
        );
        assert_close(
            eval_imcsc_surface(&[txt("3+4i")], &NoResolver).unwrap(),
            0.005174473184019398,
            0.03627588962862602,
            'i',
        );
        assert_close(
            eval_imcsch_surface(&[txt("3+4i")], &NoResolver).unwrap(),
            -0.0648774713706355,
            0.0754898329158637,
            'i',
        );
        assert_close(
            eval_imcot_surface(&[txt("3+4i")], &NoResolver).unwrap(),
            -0.0001875877379836592,
            -1.0006443924715591,
            'i',
        );
    }

    #[test]
    fn power_and_domain_lanes_match_excel_observations() {
        assert_eq!(
            text_result(eval_impower_surface(&[txt("3+4i"), num(2.0)], &NoResolver).unwrap()),
            "-7+24i"
        );
        assert_close(
            eval_impower_surface(&[txt("3+4i"), num(0.5)], &NoResolver).unwrap(),
            2.0,
            1.0,
            'i',
        );
        assert_eq!(
            text_result(eval_impower_surface(&[txt("3+4i"), num(0.0)], &NoResolver).unwrap()),
            "1"
        );
        assert_eq!(
            eval_imargument_surface(&[num(0.0)], &NoResolver),
            Err(ComplexFamilyEvalError::Domain(WorksheetErrorCode::Div0))
        );
        assert_eq!(
            eval_imdiv_surface(&[txt("3+4i"), num(0.0)], &NoResolver),
            Err(ComplexFamilyEvalError::Domain(WorksheetErrorCode::Num))
        );
        assert_eq!(
            eval_imln_surface(&[num(0.0)], &NoResolver),
            Err(ComplexFamilyEvalError::Domain(WorksheetErrorCode::Num))
        );
        assert_eq!(
            eval_imsum_surface(&[txt("3+4i"), bool_arg(true)], &NoResolver),
            Err(ComplexFamilyEvalError::Domain(WorksheetErrorCode::Value))
        );
    }
}
