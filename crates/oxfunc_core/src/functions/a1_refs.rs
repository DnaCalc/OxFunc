#[derive(Debug, Clone, PartialEq, Eq)]
pub struct A1Reference {
    pub prefix: Option<String>,
    pub start_row: usize,
    pub start_col: usize,
    pub end_row: usize,
    pub end_col: usize,
}

impl A1Reference {
    pub fn width(&self) -> usize {
        self.end_col - self.start_col + 1
    }

    pub fn height(&self) -> usize {
        self.end_row - self.start_row + 1
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

fn parse_cell_token(token: &str) -> Option<(usize, usize)> {
    let trimmed = token.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut letters = String::new();
    let mut digits = String::new();
    for ch in trimmed.chars() {
        if ch == '$' {
            continue;
        }
        if ch.is_ascii_alphabetic() && digits.is_empty() {
            letters.push(ch.to_ascii_uppercase());
            continue;
        }
        if ch.is_ascii_digit() {
            digits.push(ch);
            continue;
        }
        return None;
    }

    if letters.is_empty() || digits.is_empty() {
        return None;
    }

    let mut col = 0usize;
    for ch in letters.chars() {
        let value = usize::from((ch as u8) - b'A' + 1);
        col = col.checked_mul(26)?.checked_add(value)?;
    }

    let row = digits.parse::<usize>().ok()?;
    if row == 0 || col == 0 {
        return None;
    }

    Some((row, col))
}

fn column_label(mut col: usize) -> Option<String> {
    if col == 0 {
        return None;
    }

    let mut chars = Vec::new();
    while col > 0 {
        let rem = (col - 1) % 26;
        chars.push(char::from(b'A' + u8::try_from(rem).ok()?));
        col = (col - 1) / 26;
    }
    chars.reverse();
    Some(chars.into_iter().collect())
}

pub fn parse_a1_reference(target: &str) -> Option<A1Reference> {
    let (prefix, body) = split_prefix(target);
    if body.is_empty() {
        return None;
    }

    let parts: Vec<&str> = body.split(':').collect();
    if parts.len() > 2 {
        return None;
    }

    let (start_row, start_col) = parse_cell_token(parts[0])?;
    let (end_row, end_col) = if parts.len() == 2 {
        parse_cell_token(parts[1])?
    } else {
        (start_row, start_col)
    };

    Some(A1Reference {
        prefix,
        start_row: start_row.min(end_row),
        start_col: start_col.min(end_col),
        end_row: start_row.max(end_row),
        end_col: start_col.max(end_col),
    })
}

pub fn format_relative_target(reference: &A1Reference) -> Option<String> {
    let start = format!(
        "{}{}",
        column_label(reference.start_col)?,
        reference.start_row
    );
    let end = format!("{}{}", column_label(reference.end_col)?, reference.end_row);

    let body = if start == end {
        start
    } else {
        format!("{start}:{end}")
    };

    Some(match &reference.prefix {
        Some(prefix) => format!("{prefix}!{body}"),
        None => body,
    })
}

pub fn format_absolute_address(reference: &A1Reference) -> Option<String> {
    let start = format!(
        "${}${}",
        column_label(reference.start_col)?,
        reference.start_row
    );
    let end = format!(
        "${}${}",
        column_label(reference.end_col)?,
        reference.end_row
    );

    let body = if start == end {
        start
    } else {
        format!("{start}:{end}")
    };

    Some(match &reference.prefix {
        Some(prefix) => format!("{prefix}!{body}"),
        None => body,
    })
}

pub fn offset_reference(
    base: &A1Reference,
    row_offset: i64,
    col_offset: i64,
    height: Option<usize>,
    width: Option<usize>,
) -> Option<A1Reference> {
    let top = i64::try_from(base.start_row).ok()?.checked_add(row_offset)?;
    let left = i64::try_from(base.start_col).ok()?.checked_add(col_offset)?;
    if top <= 0 || left <= 0 {
        return None;
    }

    let h = height.unwrap_or_else(|| base.height());
    let w = width.unwrap_or_else(|| base.width());
    if h == 0 || w == 0 {
        return None;
    }

    let start_row = usize::try_from(top).ok()?;
    let start_col = usize::try_from(left).ok()?;
    let end_row = start_row.checked_add(h.checked_sub(1)?)?;
    let end_col = start_col.checked_add(w.checked_sub(1)?)?;

    Some(A1Reference {
        prefix: base.prefix.clone(),
        start_row,
        start_col,
        end_row,
        end_col,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_cell_reference() {
        let got = parse_a1_reference("Sheet1!$B$3").unwrap();
        assert_eq!(
            got,
            A1Reference {
                prefix: Some("Sheet1".to_string()),
                start_row: 3,
                start_col: 2,
                end_row: 3,
                end_col: 2,
            }
        );
    }

    #[test]
    fn parse_area_reference_normalizes_bounds() {
        let got = parse_a1_reference("C4:A2").unwrap();
        assert_eq!(got.start_row, 2);
        assert_eq!(got.start_col, 1);
        assert_eq!(got.end_row, 4);
        assert_eq!(got.end_col, 3);
    }

    #[test]
    fn format_absolute_and_relative_reference() {
        let reference = A1Reference {
            prefix: Some("Sheet1".to_string()),
            start_row: 2,
            start_col: 1,
            end_row: 3,
            end_col: 2,
        };
        assert_eq!(
            format_relative_target(&reference),
            Some("Sheet1!A2:B3".to_string())
        );
        assert_eq!(
            format_absolute_address(&reference),
            Some("Sheet1!$A$2:$B$3".to_string())
        );
    }

    #[test]
    fn offset_reference_preserves_shape() {
        let base = parse_a1_reference("A1:B2").unwrap();
        let got = offset_reference(&base, 2, 1, None, None).unwrap();
        assert_eq!(format_relative_target(&got), Some("B3:C4".to_string()));
    }
}
