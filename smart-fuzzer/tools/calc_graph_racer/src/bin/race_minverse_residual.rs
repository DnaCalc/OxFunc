//! Offline graph race for the remaining W109 G5-01 MINVERSE cells.
//!
//! This intentionally loads only banked black-box worksheet answers. It races
//! LU-elimination spellings and solve accumulation order without touching the
//! live Excel oracle.

use serde::Deserialize;
use std::cmp::Reverse;
use std::fs;
use std::path::{Path, PathBuf};

use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::matrix_family::eval_minverse_surface;
use oxfunc_core::resolver::{
    ReferenceDereferenceRequest, ReferenceResolutionError, ReferenceSystemCapabilities,
    ReferenceSystemProvider,
};
use oxfunc_core::value::{CalcArray, CalcValue, CoreValue};

#[derive(Deserialize)]
struct AnswerFile {
    function: String,
    witnesses: Vec<Witness>,
    #[serde(default)]
    capture_provenance: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct Witness {
    id: String,
    args: Vec<Vec<Vec<String>>>,
    expected_bits: String,
}

#[derive(Deserialize)]
struct BatchFile {
    function: String,
    probes: Vec<ProbeEnvelope>,
}

#[derive(Deserialize)]
struct ProbeEnvelope {
    probe: Probe,
}

#[derive(Deserialize)]
struct Probe {
    id: String,
    args: Vec<Vec<Vec<String>>>,
    result_index: Vec<usize>,
}

#[derive(Clone)]
struct Row {
    id: String,
    matrix: Vec<Vec<f64>>,
    row: usize,
    col: usize,
    expected: u64,
}

struct NoResolver;

impl ReferenceSystemProvider for NoResolver {
    fn capabilities(&self) -> ReferenceSystemCapabilities {
        ReferenceSystemCapabilities::permissive_local()
    }

    fn dereference(
        &self,
        request: &ReferenceDereferenceRequest,
    ) -> Result<CalcValue, ReferenceResolutionError> {
        Err(ReferenceResolutionError::UnresolvedReference {
            target: request.reference.target().to_string(),
        })
    }
}

#[derive(Clone, Copy, Debug)]
enum FactorMode {
    Divide,
    Reciprocal,
}

#[derive(Clone, Copy, Debug)]
enum ElimMode {
    FactorMul,
    FactorFma,
    PivotRowRatio,
    ProductDivide,
    ProductDivideFma,
    FractionFree,
}

#[derive(Clone, Copy, Debug)]
enum AccMode {
    StreamAsc,
    StreamDesc,
    SumAsc,
    SumDesc,
}

#[derive(Clone, Copy, Debug)]
enum FinalDiv {
    Divide,
    Reciprocal,
}

#[derive(Clone, Copy, Debug)]
struct Model {
    factor: FactorMode,
    elim: ElimMode,
    forward: AccMode,
    back: AccMode,
    final_div: FinalDiv,
}

fn bits(text: &str) -> u64 {
    u64::from_str_radix(text.trim_start_matches("0x"), 16).unwrap()
}

fn output_index(id: &str) -> (usize, usize) {
    let suffix = id.rsplit('-').next().unwrap();
    let bytes = suffix.as_bytes();
    assert_eq!(bytes.len(), 2, "unexpected witness id {id}");
    let mut row = usize::from(bytes[0] - b'0');
    let mut col = usize::from(bytes[1] - b'0');
    if !id.starts_with("m4-") {
        row -= 1;
        col -= 1;
    }
    (row, col)
}

fn load(path: &Path) -> Vec<Row> {
    let file: AnswerFile = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    assert_eq!(file.function, "MINVERSE");
    file.witnesses
        .into_iter()
        .map(|w| {
            assert_eq!(w.args.len(), 1, "{}", w.id);
            let matrix = w.args[0]
                .iter()
                .map(|row| row.iter().map(|v| f64::from_bits(bits(v))).collect())
                .collect();
            let (row, col) = output_index(&w.id);
            Row {
                id: w.id,
                matrix,
                row,
                col,
                expected: bits(&w.expected_bits),
            }
        })
        .collect()
}

fn load_heldout(batch_path: &Path, answer_path: &Path) -> Vec<Row> {
    let batch: BatchFile = serde_json::from_slice(&fs::read(batch_path).unwrap()).unwrap();
    let answers: AnswerFile = serde_json::from_slice(&fs::read(answer_path).unwrap()).unwrap();
    assert_eq!(batch.function, "MINVERSE");
    assert_eq!(answers.function, "MINVERSE");
    assert_eq!(batch.probes.len(), answers.witnesses.len());

    let provenance = answers
        .capture_provenance
        .as_ref()
        .expect("held-out answers lack capture provenance");
    assert_eq!(provenance["oracle_cache"]["mode"], "no_cache");
    assert_eq!(provenance["environment"]["excel_version"], "16.0");
    assert_eq!(provenance["environment"]["excel_build"], "20228");
    assert_eq!(provenance["environment"]["excel_bitness"], "64-bit");
    assert_eq!(provenance["environment"]["workbook_compatibility"], "2");
    assert_eq!(
        provenance["environment"]["excel_input_plumbing"],
        "cell_value2_matrix"
    );

    batch
        .probes
        .into_iter()
        .zip(answers.witnesses)
        .map(|(envelope, witness)| {
            assert_eq!(envelope.probe.id, witness.id);
            assert_eq!(envelope.probe.args, witness.args, "{}", witness.id);
            assert_eq!(envelope.probe.result_index.len(), 2, "{}", witness.id);
            let (row, col) = output_index(&witness.id);
            assert_eq!(envelope.probe.result_index, [row + 1, col + 1]);
            assert_eq!(witness.args.len(), 1, "{}", witness.id);
            let matrix = witness.args[0]
                .iter()
                .map(|values| {
                    values
                        .iter()
                        .map(|value| f64::from_bits(bits(value)))
                        .collect()
                })
                .collect();
            Row {
                id: witness.id,
                matrix,
                row,
                col,
                expected: bits(&witness.expected_bits),
            }
        })
        .collect()
}

fn factor_value(numerator: f64, pivot: f64, mode: FactorMode) -> f64 {
    match mode {
        FactorMode::Divide => numerator / pivot,
        FactorMode::Reciprocal => numerator * (1.0 / pivot),
    }
}

fn accumulate(
    rhs: f64,
    start: usize,
    end: usize,
    left: impl Fn(usize) -> f64,
    right: impl Fn(usize) -> f64,
    mode: AccMode,
) -> f64 {
    match mode {
        AccMode::StreamAsc => {
            let mut out = rhs;
            for k in start..end {
                out -= left(k) * right(k);
            }
            out
        }
        AccMode::StreamDesc => {
            let mut out = rhs;
            for k in (start..end).rev() {
                out -= left(k) * right(k);
            }
            out
        }
        AccMode::SumAsc => {
            let mut sum = 0.0;
            for k in start..end {
                sum += left(k) * right(k);
            }
            rhs - sum
        }
        AccMode::SumDesc => {
            let mut sum = 0.0;
            for k in (start..end).rev() {
                sum += left(k) * right(k);
            }
            rhs - sum
        }
    }
}

fn inverse(matrix: &[Vec<f64>], model: Model) -> Option<Vec<Vec<f64>>> {
    let n = matrix.len();
    let mut a = matrix.to_vec();
    let mut piv: Vec<usize> = (0..n).collect();

    for k in 0..n {
        let mut pivot_row = k;
        let mut pivot_abs = a[k][k].abs();
        for row in (k + 1)..n {
            let candidate = a[row][k].abs();
            if candidate > pivot_abs {
                pivot_abs = candidate;
                pivot_row = row;
            }
        }
        if pivot_abs < 1e-12 {
            return None;
        }
        if pivot_row != k {
            a.swap(pivot_row, k);
            piv.swap(pivot_row, k);
        }

        let pivot = a[k][k];
        for row in (k + 1)..n {
            let numerator = a[row][k];
            let factor = factor_value(numerator, pivot, model.factor);
            for col in (k + 1)..n {
                a[row][col] = match model.elim {
                    ElimMode::FactorMul => a[row][col] - factor * a[k][col],
                    ElimMode::FactorFma => (-factor).mul_add(a[k][col], a[row][col]),
                    ElimMode::PivotRowRatio => a[row][col] - numerator * (a[k][col] / pivot),
                    ElimMode::ProductDivide => a[row][col] - (numerator * a[k][col]) / pivot,
                    ElimMode::ProductDivideFma => {
                        let correction = (numerator * a[k][col]) / pivot;
                        (-1.0_f64).mul_add(correction, a[row][col])
                    }
                    ElimMode::FractionFree => (a[row][col] * pivot - numerator * a[k][col]) / pivot,
                };
            }
            a[row][k] = factor;
        }
    }

    let mut result = vec![vec![0.0; n]; n];
    for column in 0..n {
        let mut y = vec![0.0; n];
        for i in 0..n {
            let rhs = if piv[i] == column { 1.0 } else { 0.0 };
            y[i] = accumulate(rhs, 0, i, |k| a[i][k], |k| y[k], model.forward);
        }

        let mut x = vec![0.0; n];
        for i in (0..n).rev() {
            let numerator = accumulate(y[i], i + 1, n, |k| a[i][k], |k| x[k], model.back);
            x[i] = match model.final_div {
                FinalDiv::Divide => numerator / a[i][i],
                FinalDiv::Reciprocal => numerator * (1.0 / a[i][i]),
            };
        }
        for i in 0..n {
            result[i][column] = x[i];
        }
    }
    Some(result)
}

fn solve_stored_lu(a: &[Vec<f64>], piv: &[usize]) -> Vec<Vec<f64>> {
    let n = a.len();
    let mut result = vec![vec![0.0; n]; n];
    for column in 0..n {
        let mut y = vec![0.0; n];
        for i in 0..n {
            let mut value = if piv[i] == column { 1.0 } else { 0.0 };
            for k in 0..i {
                value -= a[i][k] * y[k];
            }
            y[i] = value;
        }
        let mut x = vec![0.0; n];
        for i in (0..n).rev() {
            let mut value = y[i];
            for k in (i + 1)..n {
                value -= a[i][k] * x[k];
            }
            x[i] = value / a[i][i];
        }
        for i in 0..n {
            result[i][column] = x[i];
        }
    }
    result
}

/// Conventional left-looking Doolittle spelling. This has the same pivot and
/// solve policy as production, but materializes each column/row from dot
/// products instead of applying right-looking rank-1 updates.
fn inverse_left_looking(matrix: &[Vec<f64>]) -> Option<Vec<Vec<f64>>> {
    let n = matrix.len();
    let mut source = matrix.to_vec();
    let mut lu = vec![vec![0.0; n]; n];
    let mut piv: Vec<usize> = (0..n).collect();

    for k in 0..n {
        for i in k..n {
            let mut value = source[i][k];
            for p in 0..k {
                value -= lu[i][p] * lu[p][k];
            }
            lu[i][k] = value;
        }

        let mut pivot_row = k;
        let mut pivot_abs = lu[k][k].abs();
        for i in (k + 1)..n {
            if lu[i][k].abs() > pivot_abs {
                pivot_abs = lu[i][k].abs();
                pivot_row = i;
            }
        }
        if pivot_abs < 1e-12 {
            return None;
        }
        if pivot_row != k {
            source.swap(pivot_row, k);
            lu.swap(pivot_row, k);
            piv.swap(pivot_row, k);
        }

        for j in (k + 1)..n {
            let mut value = source[k][j];
            for p in 0..k {
                value -= lu[k][p] * lu[p][j];
            }
            lu[k][j] = value;
        }
        for i in (k + 1)..n {
            lu[i][k] /= lu[k][k];
        }
    }
    Some(solve_stored_lu(&lu, &piv))
}

fn inverse_cholesky(matrix: &[Vec<f64>]) -> Option<Vec<Vec<f64>>> {
    let n = matrix.len();
    if (0..n).any(|i| (0..n).any(|j| matrix[i][j].to_bits() != matrix[j][i].to_bits())) {
        return None;
    }
    let mut l = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..=i {
            let mut value = matrix[i][j];
            for k in 0..j {
                value -= l[i][k] * l[j][k];
            }
            if i == j {
                if value <= 0.0 {
                    return None;
                }
                l[i][j] = value.sqrt();
            } else {
                l[i][j] = value / l[j][j];
            }
        }
    }

    let mut result = vec![vec![0.0; n]; n];
    for column in 0..n {
        let mut y = vec![0.0; n];
        for i in 0..n {
            let mut value = if i == column { 1.0 } else { 0.0 };
            for k in 0..i {
                value -= l[i][k] * y[k];
            }
            y[i] = value / l[i][i];
        }
        let mut x = vec![0.0; n];
        for i in (0..n).rev() {
            let mut value = y[i];
            for k in (i + 1)..n {
                value -= l[k][i] * x[k];
            }
            x[i] = value / l[i][i];
        }
        for i in 0..n {
            result[i][column] = x[i];
        }
    }
    Some(result)
}

fn op_mul(a: f64, b: f64, double_round: bool) -> f64 {
    if double_round {
        rx::x87_mul(a, b)
    } else {
        a * b
    }
}

fn op_sub(a: f64, b: f64, double_round: bool) -> f64 {
    if double_round {
        let value = rx::ext_sub(&rx::ext_from_f64(a), &rx::ext_from_f64(b), rx::CW_PC64_RN);
        rx::ext_to_f64(&value, rx::CW_PC53_RN)
    } else {
        a - b
    }
}

fn op_div(a: f64, b: f64, double_round: bool) -> f64 {
    if double_round {
        let value = rx::ext_div(&rx::ext_from_f64(a), &rx::ext_from_f64(b), rx::CW_PC64_RN);
        rx::ext_to_f64(&value, rx::CW_PC53_RN)
    } else {
        a / b
    }
}

/// Baseline right-looking LU with an independent plain-vs-x87-double-round
/// choice at each arithmetic site. Bit order: factor-div, elimination-mul,
/// elimination-sub, forward-mul, forward-sub, back-mul, back-sub, final-div.
fn inverse_precision_mask(matrix: &[Vec<f64>], mask: u8) -> Option<Vec<Vec<f64>>> {
    let dr = |bit: u8| mask & (1 << bit) != 0;
    let n = matrix.len();
    let mut a = matrix.to_vec();
    let mut piv: Vec<usize> = (0..n).collect();
    for k in 0..n {
        let mut pivot_row = k;
        let mut pivot_abs = a[k][k].abs();
        for row in (k + 1)..n {
            if a[row][k].abs() > pivot_abs {
                pivot_abs = a[row][k].abs();
                pivot_row = row;
            }
        }
        if pivot_abs < 1e-12 {
            return None;
        }
        if pivot_row != k {
            a.swap(pivot_row, k);
            piv.swap(pivot_row, k);
        }
        let pivot = a[k][k];
        for row in (k + 1)..n {
            let factor = op_div(a[row][k], pivot, dr(0));
            a[row][k] = factor;
            for col in (k + 1)..n {
                let product = op_mul(factor, a[k][col], dr(1));
                a[row][col] = op_sub(a[row][col], product, dr(2));
            }
        }
    }

    let mut result = vec![vec![0.0; n]; n];
    for column in 0..n {
        let mut y = vec![0.0; n];
        for i in 0..n {
            let mut value = if piv[i] == column { 1.0 } else { 0.0 };
            for k in 0..i {
                let product = op_mul(a[i][k], y[k], dr(3));
                value = op_sub(value, product, dr(4));
            }
            y[i] = value;
        }
        let mut x = vec![0.0; n];
        for i in (0..n).rev() {
            let mut value = y[i];
            for k in (i + 1)..n {
                let product = op_mul(a[i][k], x[k], dr(5));
                value = op_sub(value, product, dr(6));
            }
            x[i] = op_div(value, a[i][i], dr(7));
        }
        for i in 0..n {
            result[i][column] = x[i];
        }
    }
    Some(result)
}

fn score(rows: &[Row], model: Model) -> (usize, Vec<String>) {
    let mut exact = 0;
    let mut misses = Vec::new();
    for row in rows {
        let Some(inv) = inverse(&row.matrix, model) else {
            misses.push(format!("{}:model-error", row.id));
            continue;
        };
        let got = inv[row.row][row.col].to_bits();
        if got == row.expected {
            exact += 1;
        } else {
            misses.push(format!(
                "{}: got={got:016x} want={:016x}",
                row.id, row.expected
            ));
        }
    }
    (exact, misses)
}

fn score_algorithm(
    rows: &[Row],
    algorithm: fn(&[Vec<f64>]) -> Option<Vec<Vec<f64>>>,
) -> (usize, Vec<String>) {
    let mut exact = 0;
    let mut misses = Vec::new();
    for row in rows {
        let Some(inv) = algorithm(&row.matrix) else {
            continue;
        };
        let got = inv[row.row][row.col].to_bits();
        if got == row.expected {
            exact += 1;
        } else {
            misses.push(format!(
                "{}: got={got:016x} want={:016x}",
                row.id, row.expected
            ));
        }
    }
    (exact, misses)
}

fn score_mask(rows: &[Row], mask: u8) -> usize {
    rows.iter()
        .filter(|row| {
            inverse_precision_mask(&row.matrix, mask)
                .is_some_and(|inv| published_bits(inv[row.row][row.col]) == row.expected)
        })
        .count()
}

fn published_bits(value: f64) -> u64 {
    if value == 0.0 { 0 } else { value.to_bits() }
}

fn mask_misses(rows: &[Row], mask: u8) -> Vec<String> {
    rows.iter()
        .filter_map(|row| {
            let got = inverse_precision_mask(&row.matrix, mask)
                .map(|inverse| published_bits(inverse[row.row][row.col]));
            if got == Some(row.expected) {
                None
            } else {
                Some(format!(
                    "{}: got={} want={:016x}",
                    row.id,
                    got.map_or_else(
                        || "model-error".to_string(),
                        |value| format!("{value:016x}")
                    ),
                    row.expected
                ))
            }
        })
        .collect()
}

fn production_inverse(matrix: &[Vec<f64>]) -> Option<Vec<Vec<f64>>> {
    let input = CalcArray::from_rows(
        matrix
            .iter()
            .map(|row| row.iter().copied().map(CalcValue::number).collect())
            .collect(),
    )?;
    let result = eval_minverse_surface(&[CalcValue::array(input)], &NoResolver).ok()?;
    let CoreValue::Array(array) = result.core() else {
        return None;
    };
    let shape = array.shape();
    let mut output = vec![vec![0.0; shape.cols]; shape.rows];
    for (row, values) in output.iter_mut().enumerate() {
        for (col, value) in values.iter_mut().enumerate() {
            let Some(CoreValue::Number(number)) = array.get(row, col).map(CalcValue::core) else {
                return None;
            };
            *value = *number;
        }
    }
    Some(output)
}

fn score_production(rows: &[Row]) -> usize {
    rows.iter()
        .filter(|row| {
            production_inverse(&row.matrix)
                .is_some_and(|inverse| inverse[row.row][row.col].to_bits() == row.expected)
        })
        .count()
}

fn main() {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../work/w109");
    let mut rows3 = load(&base.join("G5-01-answers-minverse.json"));
    rows3.extend(load(&base.join("G5-01-answers-minverse-r1.json")));
    let rows4 = load(&base.join("G5-01-answers-m4b.json"));

    let factors = [FactorMode::Divide, FactorMode::Reciprocal];
    let eliminations = [
        ElimMode::FactorMul,
        ElimMode::FactorFma,
        ElimMode::PivotRowRatio,
        ElimMode::ProductDivide,
        ElimMode::ProductDivideFma,
        ElimMode::FractionFree,
    ];
    let accumulations = [
        AccMode::StreamAsc,
        AccMode::StreamDesc,
        AccMode::SumAsc,
        AccMode::SumDesc,
    ];
    let final_divisions = [FinalDiv::Divide, FinalDiv::Reciprocal];

    let mut scored = Vec::new();
    for factor in factors {
        for elim in eliminations {
            for forward in accumulations {
                for back in accumulations {
                    for final_div in final_divisions {
                        let model = Model {
                            factor,
                            elim,
                            forward,
                            back,
                            final_div,
                        };
                        let (s3, _) = score(&rows3, model);
                        let (s4, _) = score(&rows4, model);
                        scored.push((s3 + s4, s3, s4, model));
                    }
                }
            }
        }
    }
    scored.sort_by_key(|(total, s3, s4, _)| (Reverse(*total), Reverse(*s4), Reverse(*s3)));

    println!("bank sizes: 3x3={} 4x4={}", rows3.len(), rows4.len());
    println!(
        "production bank replay: {}/{}",
        score_production(&rows3) + score_production(&rows4),
        rows3.len() + rows4.len()
    );
    for (rank, (total, s3, s4, model)) in scored.iter().take(24).enumerate() {
        println!(
            "{rank:02}: total={total}/{} s3={s3}/{} s4={s4}/{} {model:?}",
            rows3.len() + rows4.len(),
            rows3.len(),
            rows4.len()
        );
    }

    println!("best score per elimination spelling:");
    for elim in eliminations {
        let (total, s3, s4, model) = scored
            .iter()
            .find(|(_, _, _, model)| {
                std::mem::discriminant(&model.elim) == std::mem::discriminant(&elim)
            })
            .unwrap();
        println!("  {elim:?}: total={total} s3={s3} s4={s4} via {model:?}");
    }

    let (left3, left_misses) = score_algorithm(&rows3, inverse_left_looking);
    let (left4, _) = score_algorithm(&rows4, inverse_left_looking);
    let symmetric3: Vec<_> = rows3
        .iter()
        .filter(|row| {
            let n = row.matrix.len();
            (0..n).all(|i| (0..n).all(|j| row.matrix[i][j].to_bits() == row.matrix[j][i].to_bits()))
        })
        .cloned()
        .collect();
    let (chol3, chol_misses) = score_algorithm(&symmetric3, inverse_cholesky);
    println!(
        "left-looking Doolittle: s3={left3}/{} s4={left4}/{} misses3={}",
        rows3.len(),
        rows4.len(),
        left_misses.len()
    );
    println!(
        "Cholesky on symmetric rows: exact={chol3}/{} misses={}",
        symmetric3.len(),
        chol_misses.len()
    );

    let mut masks: Vec<_> = (0_u16..=255)
        .map(|mask| {
            let mask = mask as u8;
            let s3 = score_mask(&rows3, mask);
            let s4 = score_mask(&rows4, mask);
            (s3 + s4, s3, s4, mask)
        })
        .collect();
    masks.sort_by_key(|(total, s3, s4, mask)| (Reverse(*total), Reverse(*s4), Reverse(*s3), *mask));
    println!("top partial x87-double-round masks:");
    for (total, s3, s4, mask) in masks.iter().take(16) {
        println!("  mask={mask:08b} total={total}/607 s3={s3}/159 s4={s4}/448");
    }

    let heldout_batch = base.join("G5-01-minverse/batch-minverse-final-div-heldout-20260809.json");
    let heldout_answers =
        base.join("G5-01-minverse/answers-minverse-final-div-heldout-20260809.json");
    if heldout_answers.exists() {
        let heldout = load_heldout(&heldout_batch, &heldout_answers);
        println!(
            "production refinement replay: {}/{}",
            score_production(&heldout),
            heldout.len()
        );
        let mut heldout_masks: Vec<_> = (0_u16..=255)
            .map(|mask| {
                let mask = mask as u8;
                (score_mask(&heldout, mask), mask)
            })
            .collect();
        heldout_masks.sort_by_key(|(score, mask)| (Reverse(*score), *mask));
        println!("held-out size: {}", heldout.len());
        println!(
            "held-out controls: plain={}/{} final-div-dr={}/{} full-dr={}/{}",
            score_mask(&heldout, 0),
            heldout.len(),
            score_mask(&heldout, 0x80),
            heldout.len(),
            score_mask(&heldout, 0xff),
            heldout.len()
        );
        println!("top held-out precision masks:");
        for (score, mask) in heldout_masks.iter().take(16) {
            println!("  mask={mask:08b} exact={score}/{}", heldout.len());
        }
        println!("full-x87 held-out misses:");
        for miss in mask_misses(&heldout, 0xff) {
            println!("  {miss}");
        }

        let bank_total = rows3.len() + rows4.len();
        let mut combined_masks: Vec<_> = (0_u16..=255)
            .map(|mask| {
                let mask = mask as u8;
                (
                    score_mask(&rows3, mask)
                        + score_mask(&rows4, mask)
                        + score_mask(&heldout, mask),
                    mask,
                )
            })
            .collect();
        combined_masks.sort_by_key(|(score, mask)| (Reverse(*score), *mask));
        println!("top combined bank + held-out masks:");
        for (score, mask) in combined_masks.iter().take(16) {
            println!(
                "  mask={mask:08b} exact={score}/{}",
                bank_total + heldout.len()
            );
        }
    }

    let publication_batch =
        base.join("G5-01-minverse/batch-minverse-full-x87-publication-heldout-20260809.json");
    let publication_answers =
        base.join("G5-01-minverse/answers-minverse-full-x87-publication-heldout-20260809.json");
    if publication_answers.exists() {
        let publication = load_heldout(&publication_batch, &publication_answers);
        println!(
            "production publication replay: {}/{}",
            score_production(&publication),
            publication.len()
        );
        let mut publication_masks: Vec<_> = (0_u16..=255)
            .map(|mask| {
                let mask = mask as u8;
                (score_mask(&publication, mask), mask)
            })
            .collect();
        publication_masks.sort_by_key(|(score, mask)| (Reverse(*score), *mask));
        println!("publication held-out size: {}", publication.len());
        println!(
            "publication controls: plain={}/{} final-div-dr={}/{} full-dr-plus-zero-normalization={}/{}",
            score_mask(&publication, 0),
            publication.len(),
            score_mask(&publication, 0x80),
            publication.len(),
            score_mask(&publication, 0xff),
            publication.len()
        );
        println!("top publication precision masks:");
        for (score, mask) in publication_masks.iter().take(16) {
            println!("  mask={mask:08b} exact={score}/{}", publication.len());
        }
        println!("full-x87 publication misses:");
        for miss in mask_misses(&publication, 0xff) {
            println!("  {miss}");
        }
    }

    let best = scored[0].3;
    let (_, misses) = score(&rows3, best);
    println!("best 3x3 misses ({}):", misses.len());
    for miss in misses {
        println!("  {miss}");
    }
}
