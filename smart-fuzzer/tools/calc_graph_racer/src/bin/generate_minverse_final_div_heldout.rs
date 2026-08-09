//! Generate an oracle-blind W109 MINVERSE arithmetic-site held-out battery.
//!
//! Banked worksheet evidence selects one arithmetic node in the production LU
//! graph: every final back-substitution divide is x87 PC64 followed by a PC53
//! store.  This generator deliberately searches for fresh matrices that
//! distinguish that candidate from plain binary64 and, independently, from
//! x87 double-rounding at each of the other seven arithmetic sites.  Excel
//! answers are never loaded.

use oxfunc_core::excel_numeric::research as rx;
use serde::Deserialize;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const SEED: u64 = 0x5a01_f1a1_d1f0_0809;
const FINAL_TARGET: usize = 256;
const SITE_TARGET: usize = 32;
const CONTROL_TARGET: usize = 96;
const MAX_ATTEMPTS: usize = 1_500_000;
const PUBLICATION_SEED: u64 = 0x5a01_f011_8a87_0809;
const PUBLICATION_SITE_TARGET: usize = 32;
const PUBLICATION_ZERO_TARGET: usize = 64;
const PUBLICATION_CONTROL_TARGET: usize = 96;

#[derive(Deserialize)]
struct AnswerFile {
    witnesses: Vec<BankWitness>,
}

#[derive(Deserialize)]
struct BankWitness {
    args: Vec<Vec<Vec<String>>>,
}

#[derive(Clone)]
struct SelectedRow {
    matrix: Vec<Vec<f64>>,
    row: usize,
    col: usize,
    class: String,
    family: &'static str,
    baseline_bits: u64,
    candidate_bits: u64,
    comparison_mask: u8,
    comparison_bits: u64,
}

#[derive(Clone)]
struct PublicationRow {
    matrix: Vec<Vec<f64>>,
    row: usize,
    col: usize,
    class: String,
    family: &'static str,
    raw_candidate_bits: u64,
    candidate_bits: u64,
    comparison_mask: u8,
    comparison_bits: u64,
}

fn bits(text: &str) -> u64 {
    u64::from_str_radix(text.trim_start_matches("0x"), 16).unwrap()
}

fn hex(value: f64) -> String {
    format!("0x{:016x}", value.to_bits())
}

fn matrix_key(matrix: &[Vec<f64>]) -> Vec<u64> {
    let mut key = Vec::with_capacity(1 + matrix.len() * matrix.len());
    key.push(matrix.len() as u64);
    key.extend(matrix.iter().flatten().map(|value| value.to_bits()));
    key
}

fn load_banked_matrices(root: &Path) -> BTreeSet<Vec<u64>> {
    let mut seen = BTreeSet::new();
    for name in [
        "G5-01-answers-minverse.json",
        "G5-01-answers-minverse-r1.json",
        "G5-01-answers-m4b.json",
    ] {
        let file: AnswerFile = serde_json::from_slice(&fs::read(root.join(name)).unwrap()).unwrap();
        for witness in file.witnesses {
            assert_eq!(witness.args.len(), 1);
            let matrix: Vec<Vec<f64>> = witness.args[0]
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|value| f64::from_bits(bits(value)))
                        .collect()
                })
                .collect();
            seen.insert(matrix_key(&matrix));
        }
    }
    seen
}

fn xorshift(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}

fn random_unit(seed: &mut u64) -> f64 {
    let mantissa = (xorshift(seed) >> 12) & ((1_u64 << 52) - 1);
    let exponent = 1021 + (xorshift(seed) % 5);
    let sign = (xorshift(seed) & 1) << 63;
    f64::from_bits(sign | (exponent << 52) | mantissa)
}

fn random_integer(seed: &mut u64) -> f64 {
    let value = (xorshift(seed) % 19) as i32 - 9;
    f64::from(value)
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

/// Right-looking Doolittle LU with partial pivoting. Bit order is:
/// factor-div, elimination-mul, elimination-sub, forward-mul, forward-sub,
/// back-mul, back-sub, final-div.
fn inverse_mask(matrix: &[Vec<f64>], mask: u8) -> Option<Vec<Vec<f64>>> {
    let dr = |bit: u8| mask & (1 << bit) != 0;
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
        if !pivot_abs.is_finite() || pivot_abs < 1.0e-12 {
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
            if !x[i].is_finite() {
                return None;
            }
        }
        for i in 0..n {
            result[i][column] = x[i];
        }
    }
    Some(result)
}

fn dense(seed: &mut u64, n: usize) -> Vec<Vec<f64>> {
    let mut matrix = vec![vec![0.0; n]; n];
    for row in &mut matrix {
        for value in row {
            *value = random_unit(seed);
        }
    }
    matrix
}

fn integer(seed: &mut u64, n: usize) -> Vec<Vec<f64>> {
    let mut matrix = vec![vec![0.0; n]; n];
    for row in &mut matrix {
        for value in row {
            *value = random_integer(seed);
        }
    }
    matrix
}

fn triangular(seed: &mut u64, n: usize, upper: bool) -> Vec<Vec<f64>> {
    let mut matrix = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..n {
            if (upper && j >= i) || (!upper && j <= i) {
                matrix[i][j] = random_unit(seed);
            }
        }
        if matrix[i][i].abs() < 0.25 {
            matrix[i][i] = matrix[i][i].copysign(0.25);
        }
    }
    matrix
}

fn near_identity(seed: &mut u64, n: usize) -> Vec<Vec<f64>> {
    let power = 8 + (xorshift(seed) % 34) as i32;
    let epsilon = 2.0_f64.powi(-power);
    let mut matrix = vec![vec![0.0; n]; n];
    for (i, row) in matrix.iter_mut().enumerate() {
        for (j, value) in row.iter_mut().enumerate() {
            let perturbation = random_unit(seed) * epsilon;
            *value = if i == j {
                1.0 + perturbation
            } else {
                perturbation
            };
        }
    }
    matrix
}

fn tridiagonal(seed: &mut u64, n: usize) -> Vec<Vec<f64>> {
    let mut matrix = vec![vec![0.0; n]; n];
    for i in 0..n {
        matrix[i][i] = 1.5 + random_unit(seed).abs();
        if i > 0 {
            matrix[i][i - 1] = random_unit(seed);
        }
        if i + 1 < n {
            matrix[i][i + 1] = random_unit(seed);
        }
    }
    matrix
}

fn scaled_dense(seed: &mut u64, n: usize) -> Vec<Vec<f64>> {
    let mut matrix = dense(seed, n);
    for row in &mut matrix {
        let scale = 2.0_f64.powi((xorshift(seed) % 25) as i32 - 12);
        for value in row {
            *value *= scale;
        }
    }
    matrix
}

fn pivot_stress(seed: &mut u64, n: usize) -> Vec<Vec<f64>> {
    let mut matrix = dense(seed, n);
    matrix[0][0] = f64::from_bits(0x3d80_0000_0000_0000 | (xorshift(seed) & 0x000f_ffff));
    matrix[n - 1][0] = 1.0 + random_unit(seed).abs();
    matrix
}

fn symmetric_dominant(seed: &mut u64, n: usize) -> Vec<Vec<f64>> {
    let mut matrix = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..i {
            let value = random_unit(seed) * 0.25;
            matrix[i][j] = value;
            matrix[j][i] = value;
        }
    }
    for (i, row) in matrix.iter_mut().enumerate() {
        let off_diagonal: f64 = row.iter().map(|value| value.abs()).sum();
        row[i] = off_diagonal + 0.5 + random_unit(seed).abs();
    }
    matrix
}

fn make_matrix(seed: &mut u64, attempt: usize) -> (&'static str, Vec<Vec<f64>>) {
    let n = 2 + (xorshift(seed) % 5) as usize;
    match attempt % 9 {
        0 => ("dense-binary", dense(seed, n)),
        1 => ("integer", integer(seed, n)),
        2 => ("upper-triangular", triangular(seed, n, true)),
        3 => ("lower-triangular", triangular(seed, n, false)),
        4 => ("near-identity", near_identity(seed, n)),
        5 => ("tridiagonal", tridiagonal(seed, n)),
        6 => ("scaled-dense", scaled_dense(seed, n)),
        7 => ("pivot-stress", pivot_stress(seed, n)),
        _ => ("symmetric-dominant", symmetric_dominant(seed, n)),
    }
}

fn push_selected(
    rows: &mut Vec<SelectedRow>,
    selected_cells: &mut BTreeSet<(Vec<u64>, usize, usize)>,
    matrix: &[Vec<f64>],
    row: usize,
    col: usize,
    class: String,
    family: &'static str,
    baseline: &[Vec<f64>],
    candidate: &[Vec<f64>],
    comparison_mask: u8,
    comparison: &[Vec<f64>],
) -> bool {
    let key = (matrix_key(matrix), row, col);
    if !selected_cells.insert(key) {
        return false;
    }
    rows.push(SelectedRow {
        matrix: matrix.to_vec(),
        row,
        col,
        class,
        family,
        baseline_bits: baseline[row][col].to_bits(),
        candidate_bits: candidate[row][col].to_bits(),
        comparison_mask,
        comparison_bits: comparison[row][col].to_bits(),
    });
    true
}

fn published_bits(value: f64) -> u64 {
    if value == 0.0 { 0 } else { value.to_bits() }
}

fn push_publication(
    rows: &mut Vec<PublicationRow>,
    selected_cells: &mut BTreeSet<(Vec<u64>, usize, usize)>,
    matrix: &[Vec<f64>],
    row: usize,
    col: usize,
    class: String,
    family: &'static str,
    candidate: &[Vec<f64>],
    comparison_mask: u8,
    comparison: &[Vec<f64>],
) -> bool {
    let key = (matrix_key(matrix), row, col);
    if !selected_cells.insert(key) {
        return false;
    }
    rows.push(PublicationRow {
        matrix: matrix.to_vec(),
        row,
        col,
        class,
        family,
        raw_candidate_bits: candidate[row][col].to_bits(),
        candidate_bits: published_bits(candidate[row][col]),
        comparison_mask,
        comparison_bits: published_bits(comparison[row][col]),
    });
    true
}

/// Freeze the second, publication-grade gate after the first answer set was
/// explicitly retired into refinement. The surviving graph double-rounds all
/// eight arithmetic sites and canonicalizes a published zero to +0. Selection
/// excludes every banked and first-gate matrix and does not load their answers.
fn generate_publication_v2(work_root: &Path, refinement_rows: &[SelectedRow]) {
    let mut seen_matrices = load_banked_matrices(work_root);
    for row in refinement_rows {
        seen_matrices.insert(matrix_key(&row.matrix));
    }

    let mut selected_cells = BTreeSet::new();
    let mut rows = Vec::new();
    let mut site_counts = [0usize; 8];
    let mut zero_count = 0usize;
    let mut control_count = 0usize;
    let mut family_counts: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut seed = PUBLICATION_SEED;
    let mut attempts = 0usize;

    while attempts < MAX_ATTEMPTS
        && (site_counts
            .iter()
            .any(|count| *count < PUBLICATION_SITE_TARGET)
            || zero_count < PUBLICATION_ZERO_TARGET
            || control_count < PUBLICATION_CONTROL_TARGET)
    {
        let (family, matrix) = make_matrix(&mut seed, attempts + 4);
        attempts += 1;
        if !seen_matrices.insert(matrix_key(&matrix)) {
            continue;
        }
        let Some(candidate) = inverse_mask(&matrix, 0xff) else {
            continue;
        };
        let mut comparisons = Vec::with_capacity(8);
        let mut valid = true;
        for site in 0..8 {
            match inverse_mask(&matrix, 0xff & !(1 << site)) {
                Some(value) => comparisons.push(value),
                None => {
                    valid = false;
                    break;
                }
            }
        }
        if !valid {
            continue;
        }

        let n = matrix.len();
        let start = (xorshift(&mut seed) as usize) % (n * n);
        let mut added_for_matrix = 0usize;

        if zero_count < PUBLICATION_ZERO_TARGET {
            for offset in 0..(n * n) {
                let index = (start + offset) % (n * n);
                let row = index / n;
                let col = index % n;
                if candidate[row][col].to_bits() == 0x8000_0000_0000_0000
                    && push_publication(
                        &mut rows,
                        &mut selected_cells,
                        &matrix,
                        row,
                        col,
                        "published-zero-normalization".to_string(),
                        family,
                        &candidate,
                        0xff,
                        &candidate,
                    )
                {
                    zero_count += 1;
                    added_for_matrix += 1;
                    break;
                }
            }
        }

        for site_offset in 0..8 {
            if added_for_matrix >= 3 {
                break;
            }
            let site = (attempts + site_offset) % 8;
            if site_counts[site] >= PUBLICATION_SITE_TARGET {
                continue;
            }
            for cell_offset in 0..(n * n) {
                let index = (start + cell_offset) % (n * n);
                let row = index / n;
                let col = index % n;
                if published_bits(candidate[row][col])
                    != published_bits(comparisons[site][row][col])
                    && push_publication(
                        &mut rows,
                        &mut selected_cells,
                        &matrix,
                        row,
                        col,
                        format!("missing-site-{site}-disagreement"),
                        family,
                        &candidate,
                        0xff & !(1 << site),
                        &comparisons[site],
                    )
                {
                    site_counts[site] += 1;
                    added_for_matrix += 1;
                    break;
                }
            }
        }

        if control_count < PUBLICATION_CONTROL_TARGET && added_for_matrix == 0 {
            for offset in 0..(n * n) {
                let index = (start + offset) % (n * n);
                let row = index / n;
                let col = index % n;
                let raw = candidate[row][col].to_bits();
                let expected = published_bits(candidate[row][col]);
                if expected != 0
                    && raw != 0x8000_0000_0000_0000
                    && comparisons
                        .iter()
                        .all(|comparison| published_bits(comparison[row][col]) == expected)
                    && push_publication(
                        &mut rows,
                        &mut selected_cells,
                        &matrix,
                        row,
                        col,
                        "all-missing-site-collapse-control".to_string(),
                        family,
                        &candidate,
                        0xff,
                        &candidate,
                    )
                {
                    control_count += 1;
                    added_for_matrix += 1;
                    break;
                }
            }
        }

        if added_for_matrix > 0 {
            *family_counts.entry(family).or_default() += added_for_matrix;
        }
    }

    assert!(
        site_counts
            .iter()
            .all(|count| *count == PUBLICATION_SITE_TARGET),
        "publication site quotas not reached: {site_counts:?}"
    );
    assert_eq!(
        zero_count, PUBLICATION_ZERO_TARGET,
        "publication zero quota not reached"
    );
    assert_eq!(
        control_count, PUBLICATION_CONTROL_TARGET,
        "publication control quota not reached"
    );

    let probes: Vec<_> = rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let matrix: Vec<Vec<String>> = row
                .matrix
                .iter()
                .map(|values| values.iter().map(|value| hex(*value)).collect())
                .collect();
            let mut outputs = BTreeSet::new();
            outputs.insert(format!("0x{:016x}", row.raw_candidate_bits));
            outputs.insert(format!("0x{:016x}", row.candidate_bits));
            outputs.insert(format!("0x{:016x}", row.comparison_bits));
            json!({
                "probe": {
                    "id": format!("minv-p2-{index:05}-{}{}", row.row + 1, row.col + 1),
                    "args": [matrix],
                    "result_index": [row.row + 1, row.col + 1]
                },
                "distinct_outputs": outputs.len(),
                "outputs": outputs
            })
        })
        .collect();
    let batch = json!({
        "function": "MINVERSE",
        "row_id": "G5-01-full-x87-publication-heldout-20260809",
        "probes": probes
    });

    let mut meta = String::from(
        "id,class,family,n,row,col,raw_candidate_bits,published_candidate_bits,comparison_mask,comparison_bits\n",
    );
    for (index, row) in rows.iter().enumerate() {
        meta.push_str(&format!(
            "minv-p2-{index:05}-{}{}, {},{},{},{},{},0x{:016x},0x{:016x},0x{:02x},0x{:016x}\n",
            row.row + 1,
            row.col + 1,
            row.class,
            row.family,
            row.matrix.len(),
            row.row + 1,
            row.col + 1,
            row.raw_candidate_bits,
            row.candidate_bits,
            row.comparison_mask,
            row.comparison_bits,
        ));
    }

    let manifest = json!({
        "schema_version": "oxfunc.w109.minverse_full_x87_publication_generator.v2",
        "generated_date": "2026-08-09",
        "oracle_blind": true,
        "seed_hex": format!("0x{PUBLICATION_SEED:016x}"),
        "attempts": attempts,
        "selection": "bank/v1-disjoint matrices; full-x87 candidate vs each single missing x87 site; signed-zero publication normalization; all-model collapse controls",
        "candidate_mask": "0xff",
        "candidate_publication": "canonicalize every numeric zero result to +0 after the solve",
        "mask_bit_order": [
            "factor-div", "elimination-mul", "elimination-sub", "forward-mul",
            "forward-sub", "back-mul", "back-sub", "final-div"
        ],
        "row_count": rows.len(),
        "missing_site_disagreement_counts": site_counts,
        "published_zero_normalization_count": zero_count,
        "collapse_control_count": control_count,
        "family_row_counts": family_counts,
        "excluded_evidence": [
            "G5-01-answers-minverse.json",
            "G5-01-answers-minverse-r1.json",
            "G5-01-answers-m4b.json",
            "batch-minverse-final-div-heldout-20260809.json"
        ],
        "retired_refinement_note": "The 576-row final-div gate was retired into refinement after it identified full x87 arithmetic and +0 publication normalization. No answer from that set participates in this selection."
    });

    let out_root = work_root.join("G5-01-minverse");
    let batch_path = out_root.join("batch-minverse-full-x87-publication-heldout-20260809.json");
    let meta_path = out_root.join("batch-minverse-full-x87-publication-heldout-20260809-meta.csv");
    let manifest_path =
        out_root.join("batch-minverse-full-x87-publication-heldout-20260809-manifest.json");
    fs::write(&batch_path, serde_json::to_string_pretty(&batch).unwrap())
        .expect("write MINVERSE publication batch");
    fs::write(&meta_path, meta).expect("write MINVERSE publication metadata");
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .expect("write MINVERSE publication manifest");

    println!("wrote MINVERSE publication held-out: {} rows", rows.len());
    println!("publication attempts: {attempts}");
    println!("missing-site disagreements: {site_counts:?}");
    println!("published-zero rows: {zero_count}");
    println!("publication collapse controls: {control_count}");
    println!("publication batch: {}", batch_path.display());
    println!("publication meta: {}", meta_path.display());
    println!("publication manifest: {}", manifest_path.display());
}

fn main() {
    let work_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../work/w109");
    let mut seen_matrices = load_banked_matrices(&work_root);
    let mut selected_cells = BTreeSet::new();
    let mut rows = Vec::new();
    let mut final_count = 0usize;
    let mut site_counts = [0usize; 7];
    let mut control_count = 0usize;
    let mut family_counts: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut seed = SEED;
    let mut attempts = 0usize;

    while attempts < MAX_ATTEMPTS
        && (final_count < FINAL_TARGET
            || site_counts.iter().any(|count| *count < SITE_TARGET)
            || control_count < CONTROL_TARGET)
    {
        let (family, matrix) = make_matrix(&mut seed, attempts);
        attempts += 1;
        let matrix_key_value = matrix_key(&matrix);
        if !seen_matrices.insert(matrix_key_value) {
            continue;
        }
        let Some(baseline) = inverse_mask(&matrix, 0) else {
            continue;
        };
        let Some(candidate) = inverse_mask(&matrix, 0x80) else {
            continue;
        };
        let mut comparisons = Vec::with_capacity(7);
        let mut valid = true;
        for site in 0..7 {
            match inverse_mask(&matrix, 0x80 | (1 << site)) {
                Some(value) => comparisons.push(value),
                None => {
                    valid = false;
                    break;
                }
            }
        }
        if !valid {
            continue;
        }

        let n = matrix.len();
        let start = (xorshift(&mut seed) as usize) % (n * n);
        let mut added_for_matrix = 0usize;

        if final_count < FINAL_TARGET {
            for offset in 0..(n * n) {
                let index = (start + offset) % (n * n);
                let row = index / n;
                let col = index % n;
                if baseline[row][col].to_bits() != candidate[row][col].to_bits()
                    && push_selected(
                        &mut rows,
                        &mut selected_cells,
                        &matrix,
                        row,
                        col,
                        "final-div-disagreement".to_string(),
                        family,
                        &baseline,
                        &candidate,
                        0,
                        &baseline,
                    )
                {
                    final_count += 1;
                    added_for_matrix += 1;
                    break;
                }
            }
        }

        for site_offset in 0..7 {
            if added_for_matrix >= 3 {
                break;
            }
            let site = (attempts + site_offset) % 7;
            if site_counts[site] >= SITE_TARGET {
                continue;
            }
            for cell_offset in 0..(n * n) {
                let index = (start + cell_offset) % (n * n);
                let row = index / n;
                let col = index % n;
                if candidate[row][col].to_bits() != comparisons[site][row][col].to_bits()
                    && push_selected(
                        &mut rows,
                        &mut selected_cells,
                        &matrix,
                        row,
                        col,
                        format!("site-{site}-disagreement"),
                        family,
                        &baseline,
                        &candidate,
                        0x80 | (1 << site),
                        &comparisons[site],
                    )
                {
                    site_counts[site] += 1;
                    added_for_matrix += 1;
                    break;
                }
            }
        }

        if control_count < CONTROL_TARGET && added_for_matrix == 0 {
            for offset in 0..(n * n) {
                let index = (start + offset) % (n * n);
                let row = index / n;
                let col = index % n;
                let candidate_bits = candidate[row][col].to_bits();
                if baseline[row][col].to_bits() == candidate_bits
                    && comparisons
                        .iter()
                        .all(|comparison| comparison[row][col].to_bits() == candidate_bits)
                    && push_selected(
                        &mut rows,
                        &mut selected_cells,
                        &matrix,
                        row,
                        col,
                        "all-mask-collapse-control".to_string(),
                        family,
                        &baseline,
                        &candidate,
                        0x80,
                        &candidate,
                    )
                {
                    control_count += 1;
                    added_for_matrix += 1;
                    break;
                }
            }
        }

        if added_for_matrix > 0 {
            *family_counts.entry(family).or_default() += added_for_matrix;
        }
    }

    assert_eq!(final_count, FINAL_TARGET, "final-div quota not reached");
    assert!(
        site_counts.iter().all(|count| *count == SITE_TARGET),
        "site quotas not reached: {site_counts:?}"
    );
    assert_eq!(control_count, CONTROL_TARGET, "control quota not reached");

    let probes: Vec<_> = rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let matrix: Vec<Vec<String>> = row
                .matrix
                .iter()
                .map(|values| values.iter().map(|value| hex(*value)).collect())
                .collect();
            let mut outputs = BTreeSet::new();
            outputs.insert(format!("0x{:016x}", row.baseline_bits));
            outputs.insert(format!("0x{:016x}", row.candidate_bits));
            outputs.insert(format!("0x{:016x}", row.comparison_bits));
            json!({
                "probe": {
                    "id": format!("minv-ho-{index:05}-{}{}", row.row + 1, row.col + 1),
                    "args": [matrix],
                    "result_index": [row.row + 1, row.col + 1]
                },
                "distinct_outputs": outputs.len(),
                "outputs": outputs
            })
        })
        .collect();
    let batch = json!({
        "function": "MINVERSE",
        "row_id": "G5-01-final-div-heldout-20260809",
        "probes": probes
    });

    let mut meta = String::from(
        "id,class,family,n,row,col,baseline_bits,candidate_mask80_bits,comparison_mask,comparison_bits\n",
    );
    for (index, row) in rows.iter().enumerate() {
        meta.push_str(&format!(
            "minv-ho-{index:05}-{}{}, {},{},{},{},{},0x{:016x},0x{:016x},0x{:02x},0x{:016x}\n",
            row.row + 1,
            row.col + 1,
            row.class,
            row.family,
            row.matrix.len(),
            row.row + 1,
            row.col + 1,
            row.baseline_bits,
            row.candidate_bits,
            row.comparison_mask,
            row.comparison_bits,
        ));
    }

    let manifest = json!({
        "schema_version": "oxfunc.w109.minverse_final_div_generator.v1",
        "generated_date": "2026-08-09",
        "oracle_blind": true,
        "seed_hex": format!("0x{SEED:016x}"),
        "attempts": attempts,
        "selection": "bank-disjoint structured/random matrices; candidate-vs-plain final-div disagreements; candidate-vs-each-other-site x87 disagreements; all-mask collapse controls",
        "baseline_mask": "0x00",
        "candidate_mask": "0x80",
        "mask_bit_order": [
            "factor-div", "elimination-mul", "elimination-sub", "forward-mul",
            "forward-sub", "back-mul", "back-sub", "final-div"
        ],
        "row_count": rows.len(),
        "final_div_disagreement_count": final_count,
        "other_site_disagreement_counts": site_counts,
        "collapse_control_count": control_count,
        "family_row_counts": family_counts,
        "bank_exclusions": [
            "G5-01-answers-minverse.json",
            "G5-01-answers-minverse-r1.json",
            "G5-01-answers-m4b.json"
        ]
    });

    let out_root = work_root.join("G5-01-minverse");
    fs::create_dir_all(&out_root).expect("create MINVERSE work directory");
    let batch_path = out_root.join("batch-minverse-final-div-heldout-20260809.json");
    let meta_path = out_root.join("batch-minverse-final-div-heldout-20260809-meta.csv");
    let manifest_path = out_root.join("batch-minverse-final-div-heldout-20260809-manifest.json");
    fs::write(&batch_path, serde_json::to_string_pretty(&batch).unwrap())
        .expect("write MINVERSE batch");
    fs::write(&meta_path, meta).expect("write MINVERSE metadata");
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .expect("write MINVERSE manifest");

    println!("wrote MINVERSE held-out: {} rows", rows.len());
    println!("attempts: {attempts}");
    println!("final-div disagreements: {final_count}");
    println!("other-site disagreements: {site_counts:?}");
    println!("collapse controls: {control_count}");
    println!("batch: {}", batch_path.display());
    println!("meta: {}", meta_path.display());
    println!("manifest: {}", manifest_path.display());

    generate_publication_v2(&work_root, &rows);
}
