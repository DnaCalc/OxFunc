use crate::value::ExcelText;

fn text_from_string(s: String) -> ExcelText {
    ExcelText::from_utf16_code_units(s.encode_utf16().collect())
}

fn push_singleton_or_original(
    out: &mut String,
    mapped: impl Iterator<Item = char>,
    original: char,
) {
    let mut chars = mapped;
    match (chars.next(), chars.next()) {
        (Some(single), None) => out.push(single),
        _ => out.push(original),
    }
}

fn push_excel_upper_char(out: &mut String, ch: char) {
    match ch {
        'ß' => out.push('ß'),
        'ά' => out.push('Α'),
        'έ' => out.push('Ε'),
        'ή' => out.push('Η'),
        'ί' => out.push('Ι'),
        'ό' => out.push('Ο'),
        'ύ' => out.push('Υ'),
        'ώ' => out.push('Ω'),
        'ΐ' => out.push('Ϊ'),
        'ΰ' => out.push('Ϋ'),
        _ => push_singleton_or_original(out, ch.to_uppercase(), ch),
    }
}

fn adjacent_letter(chars: &[char], index: usize, offset: isize) -> bool {
    let adjacent = index as isize + offset;
    adjacent >= 0
        && (adjacent as usize) < chars.len()
        && chars[adjacent as usize].is_alphabetic()
}

fn push_excel_lower_char(out: &mut String, ch: char, is_final_sigma: bool) {
    match ch {
        'İ' => out.push('i'),
        'ẞ' => out.push('ẞ'),
        'Σ' if is_final_sigma => out.push('ς'),
        _ => push_singleton_or_original(out, ch.to_lowercase(), ch),
    }
}

pub fn upper_text(text: &ExcelText) -> ExcelText {
    let mut out = String::new();
    for ch in text.to_string_lossy().chars() {
        push_excel_upper_char(&mut out, ch);
    }
    text_from_string(out)
}

pub fn lower_text(text: &ExcelText) -> ExcelText {
    let chars = text.to_string_lossy().chars().collect::<Vec<_>>();
    let mut out = String::new();
    for (index, ch) in chars.iter().copied().enumerate() {
        let is_final_sigma = ch == 'Σ'
            && adjacent_letter(&chars, index, -1)
            && !adjacent_letter(&chars, index, 1);
        push_excel_lower_char(&mut out, ch, is_final_sigma);
    }
    text_from_string(out)
}

pub fn proper_text(text: &ExcelText) -> ExcelText {
    let chars = text.to_string_lossy().chars().collect::<Vec<_>>();
    let mut out = String::new();
    let mut start_of_word = true;
    for (index, ch) in chars.iter().copied().enumerate() {
        if ch.is_alphabetic() {
            if start_of_word {
                push_excel_upper_char(&mut out, ch);
            } else {
                let is_final_sigma = ch == 'Σ'
                    && adjacent_letter(&chars, index, -1)
                    && !adjacent_letter(&chars, index, 1);
                push_excel_lower_char(&mut out, ch, is_final_sigma);
            }
            start_of_word = false;
        } else {
            out.push(ch);
            start_of_word = true;
        }
    }
    text_from_string(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Provisional boundary matrix:
    // these rows characterize the current shared OxFunc casing layer in lanes where we do not
    // yet have enough Excel evidence to promote them from local-boundary witnesses to stronger
    // Excel-alignment claims.
    #[test]
    fn excel_casing_provisional_boundary_matrix_matches_current_local_behavior() {
        let cases = [
            (
                "UPPER ß",
                upper_text(&ExcelText::from_interop_assignment("ß")),
                ExcelText::from_interop_assignment("ß"),
            ),
            (
                "LOWER ẞ",
                lower_text(&ExcelText::from_interop_assignment("ẞ")),
                ExcelText::from_interop_assignment("ẞ"),
            ),
            (
                "PROPER ß",
                proper_text(&ExcelText::from_interop_assignment("ß")),
                ExcelText::from_interop_assignment("ß"),
            ),
            (
                "UPPER ı",
                upper_text(&ExcelText::from_interop_assignment("ı")),
                ExcelText::from_interop_assignment("I"),
            ),
            (
                "LOWER Iİıi",
                lower_text(&ExcelText::from_interop_assignment("Iİıi")),
                ExcelText::from_interop_assignment("iiıi"),
            ),
            (
                "UPPER i+combining-dot",
                upper_text(&ExcelText::from_utf16_code_units(vec![105, 775])),
                ExcelText::from_utf16_code_units(vec![73, 775]),
            ),
            (
                "LOWER I+combining-dot",
                lower_text(&ExcelText::from_utf16_code_units(vec![73, 775])),
                ExcelText::from_utf16_code_units(vec![105, 775]),
            ),
            (
                "UPPER decomposed-ός",
                upper_text(&ExcelText::from_utf16_code_units(vec![959, 769, 962])),
                ExcelText::from_utf16_code_units(vec![927, 769, 931]),
            ),
            (
                "LOWER Σ",
                lower_text(&ExcelText::from_interop_assignment("Σ")),
                ExcelText::from_interop_assignment("σ"),
            ),
            (
                "LOWER ΟΣΟΣ",
                lower_text(&ExcelText::from_interop_assignment("ΟΣΟΣ")),
                ExcelText::from_interop_assignment("οσος"),
            ),
            (
                "PROPER hello-newline-world",
                proper_text(&ExcelText::from_interop_assignment("hello\nworld")),
                ExcelText::from_interop_assignment("Hello\nWorld"),
            ),
            (
                "PROPER abc_ß",
                proper_text(&ExcelText::from_interop_assignment("abc_ß")),
                ExcelText::from_interop_assignment("Abc_ß"),
            ),
            (
                "UPPER ﬃ",
                upper_text(&ExcelText::from_interop_assignment("ﬃ")),
                ExcelText::from_interop_assignment("ﬃ"),
            ),
            (
                "LOWER ﬃ",
                lower_text(&ExcelText::from_interop_assignment("ﬃ")),
                ExcelText::from_interop_assignment("ﬃ"),
            ),
            (
                "PROPER ﬃ",
                proper_text(&ExcelText::from_interop_assignment("ﬃ")),
                ExcelText::from_interop_assignment("ﬃ"),
            ),
        ];

        for (name, got, expected) in cases {
            assert_eq!(got, expected, "{name}");
        }
    }

    #[test]
    fn excel_casing_observed_matrix_matches_local_helper_behavior() {
        let cases = [
            (
                upper_text(&ExcelText::from_interop_assignment("straße")),
                ExcelText::from_interop_assignment("STRAßE"),
            ),
            (
                lower_text(&ExcelText::from_interop_assignment("STRAẞE")),
                ExcelText::from_interop_assignment("straẞe"),
            ),
            (
                proper_text(&ExcelText::from_interop_assignment("straße")),
                ExcelText::from_interop_assignment("Straße"),
            ),
            (
                upper_text(&ExcelText::from_interop_assignment("weiß")),
                ExcelText::from_interop_assignment("WEIß"),
            ),
            (
                upper_text(&ExcelText::from_interop_assignment("İstanbul")),
                ExcelText::from_interop_assignment("İSTANBUL"),
            ),
            (
                lower_text(&ExcelText::from_interop_assignment("İSTANBUL")),
                ExcelText::from_interop_assignment("istanbul"),
            ),
            (
                upper_text(&ExcelText::from_interop_assignment("istanbul")),
                ExcelText::from_interop_assignment("ISTANBUL"),
            ),
            (
                lower_text(&ExcelText::from_interop_assignment("I")),
                ExcelText::from_interop_assignment("i"),
            ),
            (
                lower_text(&ExcelText::from_interop_assignment("İ")),
                ExcelText::from_interop_assignment("i"),
            ),
            (
                upper_text(&ExcelText::from_interop_assignment("κόσμος")),
                ExcelText::from_interop_assignment("ΚΟΣΜΟΣ"),
            ),
            (
                lower_text(&ExcelText::from_interop_assignment("ΟΣ")),
                ExcelText::from_interop_assignment("ος"),
            ),
            (
                upper_text(&ExcelText::from_interop_assignment("café")),
                ExcelText::from_interop_assignment("CAFÉ"),
            ),
        ];

        for (got, expected) in cases {
            assert_eq!(got, expected);
        }
    }
}
