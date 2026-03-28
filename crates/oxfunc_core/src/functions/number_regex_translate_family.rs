use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, coerce_prepared_to_text, prepare_args_values_only,
};
use crate::host_info::{
    HostInfoError, HostInfoProvider, TranslateProviderResult, TranslateRequest,
};
use crate::locale_format::LocaleFormatContext;
use crate::resolver::ReferenceResolver;
use crate::value::{
    CallArgValue, EXCEL_TEXT_MAX_UTF16_CODE_UNITS, EvalValue, ExcelText, WorksheetErrorCode,
};

const NUMBER_REGEX_TRANSLATE_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.NUMBER_REGEX_TRANSLATE_BASE",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

pub const NUMBERVALUE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.NUMBERVALUE",
    arity: Arity { min: 1, max: 3 },
    fec_dependency_profile: FecDependencyProfile::LocaleProfile,
    surface_fec_dependency_profile: FecDependencyProfile::LocaleProfile,
    ..NUMBER_REGEX_TRANSLATE_BASE_META
};

pub const REGEXEXTRACT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.REGEXEXTRACT",
    arity: Arity { min: 2, max: 4 },
    ..NUMBER_REGEX_TRANSLATE_BASE_META
};

pub const REGEXREPLACE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.REGEXREPLACE",
    arity: Arity { min: 3, max: 5 },
    ..NUMBER_REGEX_TRANSLATE_BASE_META
};

pub const REGEXTEST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.REGEXTEST",
    arity: Arity { min: 2, max: 3 },
    ..NUMBER_REGEX_TRANSLATE_BASE_META
};

pub const TRANSLATE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.TRANSLATE",
    arity: Arity { min: 1, max: 3 },
    host_interaction: HostInteractionClass::ExternalProvider,
    thread_safety: ThreadSafetyClass::HostSerialized,
    fec_dependency_profile: FecDependencyProfile::ExternalProvider,
    surface_fec_dependency_profile: FecDependencyProfile::ExternalProvider,
    ..NUMBER_REGEX_TRANSLATE_BASE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum NumberRegexTranslateEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
    LocaleContextMissing,
    HostInfoProviderMissing(&'static str),
    HostInfo(HostInfoError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RegexAtom {
    Literal(char),
    Any,
    Digit,
    NotDigit,
    Word,
    NotWord,
    Space,
    NotSpace,
    Class(RegexClass),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RegexClass {
    negated: bool,
    items: Vec<RegexClassItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RegexClassItem {
    Single(char),
    Range(char, char),
    Digit,
    Word,
    Space,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RegexQuantifier {
    One,
    ZeroOrMore,
    OneOrMore,
    ZeroOrOne,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RegexToken {
    atom: RegexAtom,
    quantifier: RegexQuantifier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RegexClassPiece {
    Literal(char),
    Digit,
    Word,
    Space,
}

fn text_from_string_checked(s: String) -> Result<ExcelText, WorksheetErrorCode> {
    let units: Vec<u16> = s.encode_utf16().collect();
    if units.len() > EXCEL_TEXT_MAX_UTF16_CODE_UNITS {
        return Err(WorksheetErrorCode::Value);
    }
    Ok(ExcelText::from_utf16_code_units(units))
}

fn value_error() -> NumberRegexTranslateEvalError {
    NumberRegexTranslateEvalError::Domain(WorksheetErrorCode::Value)
}

fn guard_arity(meta: &FunctionMeta, argc: usize) -> Result<(), NumberRegexTranslateEvalError> {
    if meta.arity.accepts(argc) {
        Ok(())
    } else {
        Err(NumberRegexTranslateEvalError::ArityMismatch {
            expected_min: meta.arity.min,
            expected_max: meta.arity.max,
            actual: argc,
        })
    }
}

fn prepare_and_guard(
    meta: &FunctionMeta,
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<Vec<PreparedArgValue>, NumberRegexTranslateEvalError> {
    guard_arity(meta, args.len())?;
    prepare_args_values_only(args, resolver).map_err(NumberRegexTranslateEvalError::Coercion)
}

fn required_text_arg(
    prepared: &[PreparedArgValue],
    index: usize,
) -> Result<ExcelText, NumberRegexTranslateEvalError> {
    coerce_prepared_to_text(&prepared[index]).map_err(NumberRegexTranslateEvalError::Coercion)
}

fn optional_text_arg(
    prepared: Option<&PreparedArgValue>,
) -> Result<Option<ExcelText>, NumberRegexTranslateEvalError> {
    match prepared {
        None | Some(PreparedArgValue::MissingArg) | Some(PreparedArgValue::EmptyCell) => Ok(None),
        Some(arg) => Ok(Some(
            coerce_prepared_to_text(arg).map_err(NumberRegexTranslateEvalError::Coercion)?,
        )),
    }
}

fn optional_number_arg(
    prepared: Option<&PreparedArgValue>,
) -> Result<Option<f64>, NumberRegexTranslateEvalError> {
    match prepared {
        None | Some(PreparedArgValue::MissingArg) | Some(PreparedArgValue::EmptyCell) => Ok(None),
        Some(arg) => Ok(Some(
            coerce_prepared_to_number(arg).map_err(NumberRegexTranslateEvalError::Coercion)?,
        )),
    }
}

fn parse_optional_separator(
    prepared: Option<&PreparedArgValue>,
    default: Option<char>,
) -> Result<Option<char>, NumberRegexTranslateEvalError> {
    let Some(prepared) = prepared else {
        return Ok(default);
    };
    if matches!(
        prepared,
        PreparedArgValue::MissingArg | PreparedArgValue::EmptyCell
    ) {
        return Ok(default);
    }
    let text =
        coerce_prepared_to_text(prepared).map_err(NumberRegexTranslateEvalError::Coercion)?;
    let raw = text.to_string_lossy();
    let mut chars = raw.chars();
    match (chars.next(), chars.next()) {
        (None, _) => Ok(None),
        (Some(ch), None) => Ok(Some(ch)),
        _ => Err(value_error()),
    }
}

fn parse_locale_default_separator(
    locale_ctx: &LocaleFormatContext,
    decimal_separator: bool,
) -> Result<Option<char>, NumberRegexTranslateEvalError> {
    let raw = if decimal_separator {
        locale_ctx.profile.decimal_separator
    } else {
        locale_ctx.profile.thousands_separator
    };
    let mut chars = raw.chars();
    match (chars.next(), chars.next()) {
        (None, _) => Ok(None),
        (Some(ch), None) => Ok(Some(ch)),
        _ => Err(value_error()),
    }
}

fn normalize_language_tag(text: &ExcelText) -> String {
    text.to_string_lossy().trim().to_ascii_lowercase()
}

pub fn numbervalue_kernel(
    text: &ExcelText,
    decimal_separator: Option<char>,
    group_separator: Option<char>,
) -> Result<f64, WorksheetErrorCode> {
    if decimal_separator.is_some() && decimal_separator == group_separator {
        return Err(WorksheetErrorCode::Value);
    }

    let mut raw: String = text
        .to_string_lossy()
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect();
    if raw.is_empty() {
        return Err(WorksheetErrorCode::Value);
    }

    let mut percent_count = 0u32;
    while raw.ends_with('%') {
        raw.pop();
        percent_count += 1;
    }
    if raw.is_empty() {
        return Err(WorksheetErrorCode::Value);
    }

    let mut chars = raw.chars().peekable();
    let mut sign = 1.0;
    if let Some(ch) = chars.peek().copied() {
        if ch == '+' || ch == '-' {
            sign = if ch == '-' { -1.0 } else { 1.0 };
            chars.next();
        }
    }

    let mut normalized = String::new();
    let mut seen_digit = false;
    let mut seen_decimal = false;

    for ch in chars {
        if ch.is_ascii_digit() {
            normalized.push(ch);
            seen_digit = true;
            continue;
        }
        if decimal_separator == Some(ch) {
            if seen_decimal {
                return Err(WorksheetErrorCode::Value);
            }
            normalized.push('.');
            seen_decimal = true;
            continue;
        }
        if group_separator == Some(ch) {
            if seen_decimal {
                return Err(WorksheetErrorCode::Value);
            }
            continue;
        }
        return Err(WorksheetErrorCode::Value);
    }

    if !seen_digit || normalized == "." {
        return Err(WorksheetErrorCode::Value);
    }

    let mut value = normalized
        .parse::<f64>()
        .map_err(|_| WorksheetErrorCode::Value)?;
    value *= sign;
    for _ in 0..percent_count {
        value /= 100.0;
    }
    Ok(value)
}

fn parse_case_sensitive(
    prepared: Option<&PreparedArgValue>,
) -> Result<bool, NumberRegexTranslateEvalError> {
    let Some(value) = optional_number_arg(prepared)? else {
        return Ok(true);
    };
    if !value.is_finite() {
        return Err(value_error());
    }
    match value.trunc() as i64 {
        0 => Ok(true),
        1 => Ok(false),
        _ => Err(value_error()),
    }
}

fn parse_return_mode(
    prepared: Option<&PreparedArgValue>,
) -> Result<i64, NumberRegexTranslateEvalError> {
    let Some(value) = optional_number_arg(prepared)? else {
        return Ok(0);
    };
    if !value.is_finite() {
        return Err(value_error());
    }
    Ok(value.trunc() as i64)
}

fn parse_occurrence(
    prepared: Option<&PreparedArgValue>,
) -> Result<i64, NumberRegexTranslateEvalError> {
    let Some(value) = optional_number_arg(prepared)? else {
        return Ok(0);
    };
    if !value.is_finite() {
        return Err(value_error());
    }
    Ok(value.trunc() as i64)
}

fn is_word_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn chars_equal(lhs: char, rhs: char, case_sensitive: bool) -> bool {
    if case_sensitive {
        lhs == rhs
    } else if lhs.is_ascii() && rhs.is_ascii() {
        lhs.eq_ignore_ascii_case(&rhs)
    } else {
        lhs == rhs
    }
}

fn range_contains(ch: char, start: char, end: char, case_sensitive: bool) -> bool {
    if case_sensitive {
        ch >= start && ch <= end
    } else if ch.is_ascii() && start.is_ascii() && end.is_ascii() {
        let folded = ch.to_ascii_lowercase();
        let lo = start.to_ascii_lowercase();
        let hi = end.to_ascii_lowercase();
        folded >= lo && folded <= hi
    } else {
        ch >= start && ch <= end
    }
}

fn class_item_matches(item: &RegexClassItem, ch: char, case_sensitive: bool) -> bool {
    match item {
        RegexClassItem::Single(expected) => chars_equal(ch, *expected, case_sensitive),
        RegexClassItem::Range(start, end) => range_contains(ch, *start, *end, case_sensitive),
        RegexClassItem::Digit => ch.is_ascii_digit(),
        RegexClassItem::Word => is_word_char(ch),
        RegexClassItem::Space => ch.is_whitespace(),
    }
}

fn atom_matches(atom: &RegexAtom, ch: char, case_sensitive: bool) -> bool {
    match atom {
        RegexAtom::Literal(expected) => chars_equal(ch, *expected, case_sensitive),
        RegexAtom::Any => true,
        RegexAtom::Digit => ch.is_ascii_digit(),
        RegexAtom::NotDigit => !ch.is_ascii_digit(),
        RegexAtom::Word => is_word_char(ch),
        RegexAtom::NotWord => !is_word_char(ch),
        RegexAtom::Space => ch.is_whitespace(),
        RegexAtom::NotSpace => !ch.is_whitespace(),
        RegexAtom::Class(class) => {
            let found = class
                .items
                .iter()
                .any(|item| class_item_matches(item, ch, case_sensitive));
            if class.negated { !found } else { found }
        }
    }
}

fn match_atom_once(
    token: &RegexToken,
    chars: &[char],
    position: usize,
    case_sensitive: bool,
) -> Option<usize> {
    let ch = *chars.get(position)?;
    if atom_matches(&token.atom, ch, case_sensitive) {
        Some(position + 1)
    } else {
        None
    }
}

fn match_tokens_from(
    tokens: &[RegexToken],
    token_index: usize,
    chars: &[char],
    position: usize,
    case_sensitive: bool,
) -> Option<usize> {
    if token_index >= tokens.len() {
        return Some(position);
    }

    let token = &tokens[token_index];
    match token.quantifier {
        RegexQuantifier::One => {
            match_atom_once(token, chars, position, case_sensitive).and_then(|next| {
                match_tokens_from(tokens, token_index + 1, chars, next, case_sensitive)
            })
        }
        RegexQuantifier::ZeroOrOne => {
            if let Some(next) =
                match_atom_once(token, chars, position, case_sensitive).and_then(|next| {
                    match_tokens_from(tokens, token_index + 1, chars, next, case_sensitive)
                })
            {
                return Some(next);
            }
            match_tokens_from(tokens, token_index + 1, chars, position, case_sensitive)
        }
        RegexQuantifier::ZeroOrMore | RegexQuantifier::OneOrMore => {
            let minimum = if matches!(token.quantifier, RegexQuantifier::OneOrMore) {
                1usize
            } else {
                0usize
            };
            let mut positions = vec![position];
            let mut cursor = position;
            while let Some(next) = match_atom_once(token, chars, cursor, case_sensitive) {
                positions.push(next);
                cursor = next;
            }
            if positions.len().saturating_sub(1) < minimum {
                return None;
            }
            for count in (minimum..positions.len()).rev() {
                if let Some(end) = match_tokens_from(
                    tokens,
                    token_index + 1,
                    chars,
                    positions[count],
                    case_sensitive,
                ) {
                    return Some(end);
                }
            }
            None
        }
    }
}

fn find_first_match_from(
    chars: &[char],
    tokens: &[RegexToken],
    start_index: usize,
    case_sensitive: bool,
) -> Option<(usize, usize)> {
    for start in start_index..=chars.len() {
        if let Some(end) = match_tokens_from(tokens, 0, chars, start, case_sensitive) {
            if end >= start {
                return Some((start, end));
            }
        }
    }
    None
}

fn find_all_matches(
    chars: &[char],
    tokens: &[RegexToken],
    case_sensitive: bool,
) -> Vec<(usize, usize)> {
    let mut matches = Vec::new();
    let mut cursor = 0usize;
    while cursor <= chars.len() {
        let Some((start, end)) = find_first_match_from(chars, tokens, cursor, case_sensitive)
        else {
            break;
        };
        matches.push((start, end));
        cursor = end.max(start + 1);
    }
    matches
}

fn parse_escape_atom(chars: &[char], index: &mut usize) -> Result<RegexAtom, WorksheetErrorCode> {
    let Some(ch) = chars.get(*index).copied() else {
        return Err(WorksheetErrorCode::Value);
    };
    *index += 1;
    Ok(match ch {
        'd' => RegexAtom::Digit,
        'D' => RegexAtom::NotDigit,
        'w' => RegexAtom::Word,
        'W' => RegexAtom::NotWord,
        's' => RegexAtom::Space,
        'S' => RegexAtom::NotSpace,
        other => RegexAtom::Literal(other),
    })
}

fn parse_class_piece(
    chars: &[char],
    index: &mut usize,
) -> Result<RegexClassPiece, WorksheetErrorCode> {
    let Some(ch) = chars.get(*index).copied() else {
        return Err(WorksheetErrorCode::Value);
    };
    if ch == '\\' {
        *index += 1;
        let Some(escaped) = chars.get(*index).copied() else {
            return Err(WorksheetErrorCode::Value);
        };
        *index += 1;
        return match escaped {
            'd' => Ok(RegexClassPiece::Digit),
            'w' => Ok(RegexClassPiece::Word),
            's' => Ok(RegexClassPiece::Space),
            'D' | 'W' | 'S' => Err(WorksheetErrorCode::Value),
            other => Ok(RegexClassPiece::Literal(other)),
        };
    }
    *index += 1;
    Ok(RegexClassPiece::Literal(ch))
}

fn parse_char_class(chars: &[char], index: &mut usize) -> Result<RegexAtom, WorksheetErrorCode> {
    let mut negated = false;
    if matches!(chars.get(*index), Some('^')) {
        negated = true;
        *index += 1;
    }

    let mut pieces = Vec::new();
    while let Some(ch) = chars.get(*index).copied() {
        if ch == ']' {
            break;
        }
        pieces.push(parse_class_piece(chars, index)?);
    }

    if !matches!(chars.get(*index), Some(']')) || pieces.is_empty() {
        return Err(WorksheetErrorCode::Value);
    }
    *index += 1;

    let mut items = Vec::new();
    let mut i = 0usize;
    while i < pieces.len() {
        if i + 2 < pieces.len() {
            if let (
                RegexClassPiece::Literal(start),
                RegexClassPiece::Literal('-'),
                RegexClassPiece::Literal(end),
            ) = (&pieces[i], &pieces[i + 1], &pieces[i + 2])
            {
                items.push(RegexClassItem::Range(*start, *end));
                i += 3;
                continue;
            }
        }
        match &pieces[i] {
            RegexClassPiece::Literal(ch) => items.push(RegexClassItem::Single(*ch)),
            RegexClassPiece::Digit => items.push(RegexClassItem::Digit),
            RegexClassPiece::Word => items.push(RegexClassItem::Word),
            RegexClassPiece::Space => items.push(RegexClassItem::Space),
        }
        i += 1;
    }

    Ok(RegexAtom::Class(RegexClass { negated, items }))
}

fn parse_regex_pattern(pattern: &str) -> Result<Vec<RegexToken>, WorksheetErrorCode> {
    let chars: Vec<char> = pattern.chars().collect();
    if chars.is_empty() {
        return Err(WorksheetErrorCode::Value);
    }

    let mut tokens = Vec::new();
    let mut index = 0usize;
    while let Some(ch) = chars.get(index).copied() {
        let atom = match ch {
            '.' => {
                index += 1;
                RegexAtom::Any
            }
            '\\' => {
                index += 1;
                parse_escape_atom(&chars, &mut index)?
            }
            '[' => {
                index += 1;
                parse_char_class(&chars, &mut index)?
            }
            '*' | '+' | '?' | '(' | ')' | '{' | '}' | '|' | '^' | '$' => {
                return Err(WorksheetErrorCode::Value);
            }
            other => {
                index += 1;
                RegexAtom::Literal(other)
            }
        };

        let quantifier = match chars.get(index).copied() {
            Some('*') => {
                index += 1;
                RegexQuantifier::ZeroOrMore
            }
            Some('+') => {
                index += 1;
                RegexQuantifier::OneOrMore
            }
            Some('?') => {
                index += 1;
                RegexQuantifier::ZeroOrOne
            }
            _ => RegexQuantifier::One,
        };

        tokens.push(RegexToken { atom, quantifier });
    }

    if tokens.is_empty() {
        return Err(WorksheetErrorCode::Value);
    }
    Ok(tokens)
}

pub fn regexextract_kernel(
    text: &ExcelText,
    pattern: &ExcelText,
    return_mode: i64,
    case_sensitive: bool,
) -> Result<ExcelText, WorksheetErrorCode> {
    if return_mode != 0 {
        return Err(WorksheetErrorCode::Value);
    }
    let haystack: Vec<char> = text.to_string_lossy().chars().collect();
    let tokens = parse_regex_pattern(&pattern.to_string_lossy())?;
    let Some((start, end)) = find_first_match_from(&haystack, &tokens, 0, case_sensitive) else {
        return Err(WorksheetErrorCode::NA);
    };
    text_from_string_checked(haystack[start..end].iter().collect())
}

pub fn regextest_kernel(
    text: &ExcelText,
    pattern: &ExcelText,
    case_sensitive: bool,
) -> Result<bool, WorksheetErrorCode> {
    let haystack: Vec<char> = text.to_string_lossy().chars().collect();
    let tokens = parse_regex_pattern(&pattern.to_string_lossy())?;
    Ok(find_first_match_from(&haystack, &tokens, 0, case_sensitive).is_some())
}

pub fn regexreplace_kernel(
    text: &ExcelText,
    pattern: &ExcelText,
    replacement: &ExcelText,
    occurrence: i64,
    case_sensitive: bool,
) -> Result<ExcelText, WorksheetErrorCode> {
    if occurrence < 0 {
        return Err(WorksheetErrorCode::Value);
    }
    let haystack: Vec<char> = text.to_string_lossy().chars().collect();
    let replacement_string = replacement.to_string_lossy();
    let tokens = parse_regex_pattern(&pattern.to_string_lossy())?;
    let matches = find_all_matches(&haystack, &tokens, case_sensitive);
    if matches.is_empty() {
        return Ok(text.clone());
    }

    let selected: Vec<(usize, usize)> = if occurrence == 0 {
        matches
    } else {
        let target = occurrence as usize;
        match matches.get(target.saturating_sub(1)).copied() {
            Some(item) => vec![item],
            None => return Ok(text.clone()),
        }
    };

    let mut out = String::new();
    let mut cursor = 0usize;
    for (start, end) in selected {
        out.extend(haystack[cursor..start].iter().copied());
        out.push_str(&replacement_string);
        cursor = end;
    }
    out.extend(haystack[cursor..].iter().copied());
    text_from_string_checked(out)
}

fn normalize_phrase(input: &str) -> String {
    input.trim().to_lowercase()
}

fn normalize_lang_code(text: &ExcelText) -> Result<String, WorksheetErrorCode> {
    let code = text.to_string_lossy().trim().to_lowercase();
    if matches!(code.as_str(), "en" | "es" | "fr" | "de") {
        Ok(code)
    } else {
        Err(WorksheetErrorCode::Value)
    }
}

fn phrasebook() -> &'static [(&'static str, &'static str, &'static str, &'static str)] {
    &[
        ("en", "es", "hello, world!", "hola mundo!"),
        ("es", "en", "hola mundo!", "hello, world!"),
        ("en", "fr", "good morning", "bonjour"),
        ("fr", "en", "bonjour", "good morning"),
        ("en", "de", "good night", "gute nacht"),
        ("de", "en", "gute nacht", "good night"),
        ("en", "es", "thank you", "gracias"),
        ("es", "en", "gracias", "thank you"),
    ]
}

pub fn translate_kernel(
    text: &ExcelText,
    source_language: Option<&ExcelText>,
    target_language: Option<&ExcelText>,
) -> Result<ExcelText, WorksheetErrorCode> {
    let target = match target_language {
        Some(text) => normalize_lang_code(text)?,
        None => return Err(WorksheetErrorCode::Value),
    };
    let source = source_language.map(normalize_lang_code).transpose()?;
    if source.as_deref() == Some(target.as_str()) {
        return Ok(text.clone());
    }

    let phrase = normalize_phrase(&text.to_string_lossy());
    let matches: Vec<_> = phrasebook()
        .iter()
        .filter(|(src, dst, src_phrase, _)| {
            dst == &target.as_str()
                && src_phrase == &phrase
                && source.as_deref().is_none_or(|wanted| wanted == *src)
        })
        .collect();

    if matches.len() != 1 {
        return Err(WorksheetErrorCode::Value);
    }

    text_from_string_checked(matches[0].3.to_string())
}

pub fn eval_numbervalue_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    locale_ctx: Option<&LocaleFormatContext>,
) -> Result<EvalValue, NumberRegexTranslateEvalError> {
    let prepared = prepare_and_guard(&NUMBERVALUE_META, args, resolver)?;
    let text = required_text_arg(&prepared, 0)?;
    let decimal_separator = if matches!(
        prepared.get(1),
        None | Some(PreparedArgValue::MissingArg) | Some(PreparedArgValue::EmptyCell)
    ) {
        let ctx = locale_ctx.ok_or(NumberRegexTranslateEvalError::LocaleContextMissing)?;
        parse_locale_default_separator(ctx, true)?
    } else {
        parse_optional_separator(prepared.get(1), None)?
    };
    let group_separator = if matches!(
        prepared.get(2),
        None | Some(PreparedArgValue::MissingArg) | Some(PreparedArgValue::EmptyCell)
    ) {
        let ctx = locale_ctx.ok_or(NumberRegexTranslateEvalError::LocaleContextMissing)?;
        parse_locale_default_separator(ctx, false)?
    } else {
        parse_optional_separator(prepared.get(2), None)?
    };
    Ok(EvalValue::Number(
        numbervalue_kernel(&text, decimal_separator, group_separator)
            .map_err(NumberRegexTranslateEvalError::Domain)?,
    ))
}

pub fn eval_regexextract_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, NumberRegexTranslateEvalError> {
    let prepared = prepare_and_guard(&REGEXEXTRACT_META, args, resolver)?;
    let text = required_text_arg(&prepared, 0)?;
    let pattern = required_text_arg(&prepared, 1)?;
    let return_mode = parse_return_mode(prepared.get(2))?;
    let case_sensitive = parse_case_sensitive(prepared.get(3))?;
    Ok(EvalValue::Text(
        regexextract_kernel(&text, &pattern, return_mode, case_sensitive)
            .map_err(NumberRegexTranslateEvalError::Domain)?,
    ))
}

pub fn eval_regexreplace_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, NumberRegexTranslateEvalError> {
    let prepared = prepare_and_guard(&REGEXREPLACE_META, args, resolver)?;
    let text = required_text_arg(&prepared, 0)?;
    let pattern = required_text_arg(&prepared, 1)?;
    let replacement = required_text_arg(&prepared, 2)?;
    let occurrence = parse_occurrence(prepared.get(3))?;
    let case_sensitive = parse_case_sensitive(prepared.get(4))?;
    Ok(EvalValue::Text(
        regexreplace_kernel(&text, &pattern, &replacement, occurrence, case_sensitive)
            .map_err(NumberRegexTranslateEvalError::Domain)?,
    ))
}

pub fn eval_regextest_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, NumberRegexTranslateEvalError> {
    let prepared = prepare_and_guard(&REGEXTEST_META, args, resolver)?;
    let text = required_text_arg(&prepared, 0)?;
    let pattern = required_text_arg(&prepared, 1)?;
    let case_sensitive = parse_case_sensitive(prepared.get(2))?;
    Ok(EvalValue::Logical(
        regextest_kernel(&text, &pattern, case_sensitive)
            .map_err(NumberRegexTranslateEvalError::Domain)?,
    ))
}

pub fn eval_translate_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    host_info: Option<&dyn HostInfoProvider>,
) -> Result<EvalValue, NumberRegexTranslateEvalError> {
    let prepared = prepare_and_guard(&TRANSLATE_META, args, resolver)?;
    let text = required_text_arg(&prepared, 0)?;
    let source = optional_text_arg(prepared.get(1))?;
    let target = optional_text_arg(prepared.get(2))?;
    if source
        .as_ref()
        .zip(target.as_ref())
        .is_some_and(|(src, dst)| {
            !normalize_language_tag(src).is_empty()
                && normalize_language_tag(src) == normalize_language_tag(dst)
        })
    {
        return Ok(EvalValue::Text(text));
    }
    let provider = host_info.ok_or(NumberRegexTranslateEvalError::HostInfoProviderMissing(
        "translate_provider",
    ))?;
    let request = TranslateRequest {
        text,
        source_language: source,
        target_language: target,
    };
    match provider
        .query_translate(&request)
        .map_err(NumberRegexTranslateEvalError::HostInfo)?
    {
        TranslateProviderResult::Text(text) => Ok(EvalValue::Text(text)),
        TranslateProviderResult::Busy => Ok(EvalValue::Error(WorksheetErrorCode::Busy)),
        TranslateProviderResult::CapabilityDenied => {
            Ok(EvalValue::Error(WorksheetErrorCode::Blocked))
        }
        TranslateProviderResult::ProviderError(code) => Ok(EvalValue::Error(code)),
    }
}

pub fn map_number_regex_translate_error_to_ws(
    error: &NumberRegexTranslateEvalError,
) -> WorksheetErrorCode {
    match error {
        NumberRegexTranslateEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        NumberRegexTranslateEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        NumberRegexTranslateEvalError::Coercion(_) => WorksheetErrorCode::Value,
        NumberRegexTranslateEvalError::Domain(code) => *code,
        NumberRegexTranslateEvalError::LocaleContextMissing => WorksheetErrorCode::Value,
        NumberRegexTranslateEvalError::HostInfoProviderMissing(_) => WorksheetErrorCode::Value,
        NumberRegexTranslateEvalError::HostInfo(HostInfoError::ProviderFailure { .. }) => {
            WorksheetErrorCode::Value
        }
        NumberRegexTranslateEvalError::HostInfo(
            HostInfoError::UnsupportedTranslateQuery
            | HostInfoError::UnsupportedWidthConversionProfileQuery(_)
            | HostInfoError::UnsupportedImageQuery
            | HostInfoError::UnsupportedCellInfoQuery(_)
            | HostInfoError::UnsupportedInfoQuery(_)
            | HostInfoError::UnsupportedFormulaTextQuery
            | HostInfoError::UnsupportedSheetIndexQuery
            | HostInfoError::UnsupportedSheetCountQuery
            | HostInfoError::UnsupportedAggregateReferenceContextQuery,
        ) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host_info::{HostInfoProvider, TranslateProviderResult, TranslateRequest};
    use crate::locale_format::{current_excel_host_context, en_us_context};
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{EvalValue, ReferenceKind, ReferenceLike};

    fn txt(input: &str) -> ExcelText {
        ExcelText::from_interop_assignment(input)
    }

    struct DummyResolver;

    impl ReferenceResolver for DummyResolver {
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

    struct MockTranslateProvider {
        result: TranslateProviderResult,
    }

    impl HostInfoProvider for MockTranslateProvider {
        fn query_translate(
            &self,
            request: &TranslateRequest,
        ) -> Result<TranslateProviderResult, HostInfoError> {
            assert_eq!(request.text.to_string_lossy(), "hello");
            Ok(self.result.clone())
        }
    }

    #[test]
    fn numbervalue_parses_default_separators_and_percent() {
        let got = numbervalue_kernel(&txt(" 1,234.5%% "), Some('.'), Some(','));
        assert_eq!(got, Ok(0.12345));
    }

    #[test]
    fn numbervalue_parses_custom_separators() {
        let got = numbervalue_kernel(&txt("1.234,56"), Some(','), Some('.'));
        assert_eq!(got, Ok(1234.56));
    }

    #[test]
    fn numbervalue_rejects_duplicate_decimal_separator() {
        let got = numbervalue_kernel(&txt("1.2.3"), Some('.'), Some(','));
        assert_eq!(got, Err(WorksheetErrorCode::Value));
    }

    #[test]
    fn regex_parser_supports_literals_classes_and_quantifiers() {
        let tokens = parse_regex_pattern("[A-C]+\\d?").unwrap();
        assert_eq!(tokens.len(), 2);
    }

    #[test]
    fn regexextract_returns_first_match() {
        let got = regexextract_kernel(&txt("abc-123 def"), &txt("[a-z]+-\\d+"), 0, true);
        assert_eq!(got, Ok(txt("abc-123")));
    }

    #[test]
    fn regexextract_returns_na_when_no_match() {
        let got = regexextract_kernel(&txt("abc"), &txt("\\d+"), 0, true);
        assert_eq!(got, Err(WorksheetErrorCode::NA));
    }

    #[test]
    fn regextest_respects_case_insensitive_flag() {
        let got = regextest_kernel(&txt("Alpha"), &txt("alpha"), false);
        assert_eq!(got, Ok(true));
    }

    #[test]
    fn regexreplace_replaces_all_or_selected_occurrence() {
        let all = regexreplace_kernel(&txt("a1 b2 c3"), &txt("\\d"), &txt("#"), 0, true);
        assert_eq!(all, Ok(txt("a# b# c#")));

        let second = regexreplace_kernel(&txt("a1 b2 c3"), &txt("\\d"), &txt("#"), 2, true);
        assert_eq!(second, Ok(txt("a1 b# c3")));
    }

    #[test]
    fn regexreplace_rejects_unsupported_pattern_features() {
        let got = regexreplace_kernel(&txt("abc"), &txt("(a)"), &txt("x"), 0, true);
        assert_eq!(got, Err(WorksheetErrorCode::Value));
    }

    #[test]
    fn translate_same_language_passthrough_is_local() {
        let resolver = DummyResolver;
        let got = eval_translate_surface(
            &[
                CallArgValue::Eval(EvalValue::Text(txt("plain text"))),
                CallArgValue::Eval(EvalValue::Text(txt("es"))),
                CallArgValue::Eval(EvalValue::Text(txt("es"))),
            ],
            &resolver,
            None,
        );
        assert_eq!(got, Ok(EvalValue::Text(txt("plain text"))));
    }

    #[test]
    fn numbervalue_omitted_defaults_use_locale_context() {
        let resolver = DummyResolver;
        let current_host = current_excel_host_context();
        let en_us = en_us_context();
        let host_default = eval_numbervalue_surface(
            &[CallArgValue::Eval(EvalValue::Text(txt("1,234.5%")))],
            &resolver,
            Some(&current_host),
        );
        assert_eq!(
            host_default,
            Err(NumberRegexTranslateEvalError::Domain(
                WorksheetErrorCode::Value
            ))
        );
        let en_us_default = eval_numbervalue_surface(
            &[CallArgValue::Eval(EvalValue::Text(txt("1,234.5%")))],
            &resolver,
            Some(&en_us),
        );
        assert_eq!(en_us_default, Ok(EvalValue::Number(12.345)));
    }

    #[test]
    fn numbervalue_omitted_defaults_require_locale_context() {
        let resolver = DummyResolver;
        let got = eval_numbervalue_surface(
            &[CallArgValue::Eval(EvalValue::Text(txt("1,234.5%")))],
            &resolver,
            None,
        );
        assert_eq!(
            got,
            Err(NumberRegexTranslateEvalError::LocaleContextMissing)
        );
    }

    #[test]
    fn translate_uses_provider_for_cross_language_lane() {
        let resolver = DummyResolver;
        let got = eval_translate_surface(
            &[
                CallArgValue::Eval(EvalValue::Text(txt("hello"))),
                CallArgValue::Eval(EvalValue::Text(txt("en"))),
                CallArgValue::Eval(EvalValue::Text(txt("es"))),
            ],
            &resolver,
            Some(&MockTranslateProvider {
                result: TranslateProviderResult::Busy,
            }),
        );
        assert_eq!(got, Ok(EvalValue::Error(WorksheetErrorCode::Busy)));
    }

    #[test]
    fn numbervalue_surface_uses_optional_separator_args() {
        let resolver = DummyResolver;
        let got = eval_numbervalue_surface(
            &[
                CallArgValue::Eval(EvalValue::Text(txt("1.234,5"))),
                CallArgValue::Eval(EvalValue::Text(txt(","))),
                CallArgValue::Eval(EvalValue::Text(txt("."))),
            ],
            &resolver,
            None,
        );
        assert_eq!(got, Ok(EvalValue::Number(1234.5)));
    }

    #[test]
    fn regexextract_surface_returns_text() {
        let resolver = DummyResolver;
        let got = eval_regexextract_surface(
            &[
                CallArgValue::Eval(EvalValue::Text(txt("Code-42"))),
                CallArgValue::Eval(EvalValue::Text(txt("[A-Z][a-z]+-\\d+"))),
            ],
            &resolver,
        );
        assert_eq!(got, Ok(EvalValue::Text(txt("Code-42"))));
    }

    #[test]
    fn regextest_surface_returns_logical() {
        let resolver = DummyResolver;
        let got = eval_regextest_surface(
            &[
                CallArgValue::Eval(EvalValue::Text(txt("abc123"))),
                CallArgValue::Eval(EvalValue::Text(txt("\\d+"))),
            ],
            &resolver,
        );
        assert_eq!(got, Ok(EvalValue::Logical(true)));
    }

    #[test]
    fn translate_surface_maps_capability_denial() {
        let resolver = DummyResolver;
        let got = eval_translate_surface(
            &[
                CallArgValue::Eval(EvalValue::Text(txt("hello"))),
                CallArgValue::Eval(EvalValue::Text(txt("en"))),
                CallArgValue::Eval(EvalValue::Text(txt("es"))),
            ],
            &resolver,
            Some(&MockTranslateProvider {
                result: TranslateProviderResult::CapabilityDenied,
            }),
        );
        assert_eq!(got, Ok(EvalValue::Error(WorksheetErrorCode::Blocked)));
    }

    #[test]
    fn dummy_resolver_reference_lane_stays_unresolved() {
        let resolver = DummyResolver;
        let got = resolver.resolve_reference(&ReferenceLike {
            kind: ReferenceKind::A1,
            target: "A1".to_string(),
        });
        assert!(matches!(
            got,
            Err(RefResolutionError::UnresolvedReference { .. })
        ));
    }
}
