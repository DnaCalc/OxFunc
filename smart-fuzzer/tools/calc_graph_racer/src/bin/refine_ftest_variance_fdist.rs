//! Offline audit and answer-aware FDIST refinement for W109 G3-06 F.TEST.
//!
//! This utility is deliberately limited to the named discovery artifacts.  It
//! never discovers or opens a heldout file and it never launches Excel.  The
//! refinement bank searches for *external FDIST-equivalence ratios*; equality
//! here does not establish that F.TEST publishes a separately rounded FDIST
//! call internally.

use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const FTEST_BATCH: &str = "batch-ftest-variance-discovery-v2.json";
const FTEST_ANSWERS: &str = "answers-ftest-variance-discovery-v2.json";
const FDIST_BATCH: &str = "batch-fdist-variance-discovery-v2.json";
const FDIST_ANSWERS: &str = "answers-fdist-variance-discovery-v2.json";
const VARS_BATCH: &str = "batch-vars-variance-discovery-v2.json";
const VARS_ANSWERS: &str = "answers-vars-variance-discovery-v2.json";
const SOURCE_META: &str = "meta-ftest-variance-discovery-v2.json";

const REFINEMENT_BATCH: &str = "batch-fdist-variance-refinement-discovery-v3.json";
const REFINEMENT_ANSWERS: &str = "answers-fdist-variance-refinement-discovery-v3.json";
const REFINEMENT_META: &str = "meta-fdist-variance-refinement-discovery-v3.json";
const CDF_BATCH: &str = "batch-fdist-cdf-companion-discovery-v1.json";
const CDF_ANSWERS: &str = "answers-fdist-cdf-companion-discovery-v1.json";
const CDF_META: &str = "meta-fdist-cdf-companion-discovery-v1.json";
const AUDIT_REPORT: &str = "audit-ftest-variance-discovery-v2.json";
const REFINEMENT_SCORE: &str = "score-fdist-variance-refinement-discovery-v3.json";

const LOCAL_RADIUS: i64 = 128;
const GUARD_OFFSETS: [i64; 8] = [-2048, -1024, -512, -256, 256, 512, 1024, 2048];

#[derive(Clone)]
struct Witness {
    args: Value,
    expected_bits: u64,
}

struct AlignedAnswers {
    ordered_ids: Vec<String>,
    by_id: BTreeMap<String, Witness>,
}

struct Audit {
    report: Value,
    no_hits: Vec<NoHit>,
}

#[derive(Clone)]
struct NoHit {
    row_index: usize,
    ftest_id: String,
    family: String,
    target_bits: u64,
    nearest_ratio_bits: u64,
    df_numerator_bits: u64,
    df_denominator_bits: u64,
    nearest_tail_bits: u64,
    nearest_two_sided_bits: u64,
    nearest_output_ulp: u64,
    orientation: String,
}

fn parse_bits(value: &Value, context: &str) -> Result<u64, String> {
    let text = value
        .as_str()
        .ok_or_else(|| format!("{context}: expected a hex-bit string"))?;
    u64::from_str_radix(
        text.strip_prefix("0x")
            .ok_or_else(|| format!("{context}: expected 0x prefix"))?,
        16,
    )
    .map_err(|error| format!("{context}: {error}"))
}

fn bits(value: u64) -> String {
    format!("0x{value:016x}")
}

fn sha256_hex(input: &[u8]) -> String {
    const INITIAL: [u32; 8] = [
        0x6a09_e667,
        0xbb67_ae85,
        0x3c6e_f372,
        0xa54f_f53a,
        0x510e_527f,
        0x9b05_688c,
        0x1f83_d9ab,
        0x5be0_cd19,
    ];
    const K: [u32; 64] = [
        0x428a_2f98,
        0x7137_4491,
        0xb5c0_fbcf,
        0xe9b5_dba5,
        0x3956_c25b,
        0x59f1_11f1,
        0x923f_82a4,
        0xab1c_5ed5,
        0xd807_aa98,
        0x1283_5b01,
        0x2431_85be,
        0x550c_7dc3,
        0x72be_5d74,
        0x80de_b1fe,
        0x9bdc_06a7,
        0xc19b_f174,
        0xe49b_69c1,
        0xefbe_4786,
        0x0fc1_9dc6,
        0x240c_a1cc,
        0x2de9_2c6f,
        0x4a74_84aa,
        0x5cb0_a9dc,
        0x76f9_88da,
        0x983e_5152,
        0xa831_c66d,
        0xb003_27c8,
        0xbf59_7fc7,
        0xc6e0_0bf3,
        0xd5a7_9147,
        0x06ca_6351,
        0x1429_2967,
        0x27b7_0a85,
        0x2e1b_2138,
        0x4d2c_6dfc,
        0x5338_0d13,
        0x650a_7354,
        0x766a_0abb,
        0x81c2_c92e,
        0x9272_2c85,
        0xa2bf_e8a1,
        0xa81a_664b,
        0xc24b_8b70,
        0xc76c_51a3,
        0xd192_e819,
        0xd699_0624,
        0xf40e_3585,
        0x106a_a070,
        0x19a4_c116,
        0x1e37_6c08,
        0x2748_774c,
        0x34b0_bcb5,
        0x391c_0cb3,
        0x4ed8_aa4a,
        0x5b9c_ca4f,
        0x682e_6ff3,
        0x748f_82ee,
        0x78a5_636f,
        0x84c8_7814,
        0x8cc7_0208,
        0x90be_fffa,
        0xa450_6ceb,
        0xbef9_a3f7,
        0xc671_78f2,
    ];
    let bit_length = (input.len() as u64).wrapping_mul(8);
    let mut padded = input.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_length.to_be_bytes());
    let mut state = INITIAL;
    for chunk in padded.chunks_exact(64) {
        let mut w = [0_u32; 64];
        for (index, word) in w.iter_mut().take(16).enumerate() {
            *word = u32::from_be_bytes(chunk[index * 4..index * 4 + 4].try_into().unwrap());
        }
        for index in 16..64 {
            let s0 = w[index - 15].rotate_right(7)
                ^ w[index - 15].rotate_right(18)
                ^ (w[index - 15] >> 3);
            let s1 = w[index - 2].rotate_right(17)
                ^ w[index - 2].rotate_right(19)
                ^ (w[index - 2] >> 10);
            w[index] = w[index - 16]
                .wrapping_add(s0)
                .wrapping_add(w[index - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = state;
        for index in 0..64 {
            let sum1 = h
                .wrapping_add(e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25))
                .wrapping_add((e & f) ^ ((!e) & g))
                .wrapping_add(K[index])
                .wrapping_add(w[index]);
            let sum0 = (a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22))
                .wrapping_add((a & b) ^ (a & c) ^ (b & c));
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(sum1);
            d = c;
            c = b;
            b = a;
            a = sum0.wrapping_add(sum1);
        }
        for (slot, value) in state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *slot = slot.wrapping_add(value);
        }
    }
    state.iter().map(|word| format!("{word:08X}")).collect()
}

fn file_manifest(directory: &Path, names: &[&str]) -> Result<Vec<Value>, String> {
    names
        .iter()
        .map(|name| {
            let path = directory.join(name);
            let bytes = fs::read(&path).map_err(|error| format!("{}: {error}", path.display()))?;
            Ok(json!({
                "path": name,
                "bytes": bytes.len(),
                "sha256": sha256_hex(&bytes)
            }))
        })
        .collect()
}

fn load_json(path: &Path) -> Result<Value, String> {
    serde_json::from_str(
        &fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?,
    )
    .map_err(|error| format!("{}: {error}", path.display()))
}

fn load_aligned(
    batch_path: &Path,
    answers_path: &Path,
    expected_function: &str,
) -> Result<AlignedAnswers, String> {
    let batch = load_json(batch_path)?;
    let answers = load_json(answers_path)?;
    if batch["function"].as_str() != Some(expected_function)
        || answers["function"].as_str() != Some(expected_function)
    {
        return Err(format!(
            "{} / {}: expected function {expected_function}",
            batch_path.display(),
            answers_path.display()
        ));
    }
    let probes = batch["probes"]
        .as_array()
        .ok_or_else(|| format!("{}: probes missing", batch_path.display()))?;
    let witnesses = answers["witnesses"]
        .as_array()
        .ok_or_else(|| format!("{}: witnesses missing", answers_path.display()))?;
    if probes.len() != witnesses.len() {
        return Err(format!(
            "{} / {}: count mismatch {} != {}",
            batch_path.display(),
            answers_path.display(),
            probes.len(),
            witnesses.len()
        ));
    }
    let mut ordered_ids = Vec::with_capacity(probes.len());
    let mut by_id = BTreeMap::new();
    for (index, (probe_row, witness)) in probes.iter().zip(witnesses).enumerate() {
        let probe = &probe_row["probe"];
        let id = probe["id"]
            .as_str()
            .ok_or_else(|| format!("{}: probe {index} id missing", batch_path.display()))?;
        if witness["id"].as_str() != Some(id) {
            return Err(format!(
                "{} / {}: id mismatch at {index}",
                batch_path.display(),
                answers_path.display()
            ));
        }
        if witness["args"] != probe["args"] {
            return Err(format!(
                "{} / {}: argument mismatch for {id}",
                batch_path.display(),
                answers_path.display()
            ));
        }
        let expected_bits = parse_bits(&witness["expected_bits"], &format!("{id} expected_bits"))?;
        ordered_ids.push(id.to_string());
        if by_id
            .insert(
                id.to_string(),
                Witness {
                    args: probe["args"].clone(),
                    expected_bits,
                },
            )
            .is_some()
        {
            return Err(format!("duplicate id {id}"));
        }
    }
    Ok(AlignedAnswers { ordered_ids, by_id })
}

fn ulp_distance(left: u64, right: u64) -> u64 {
    left.abs_diff(right)
}

fn two_sided_subtract_first(tail: f64) -> f64 {
    if tail <= 0.5 {
        tail * 2.0
    } else {
        (1.0 - tail) * 2.0
    }
}

fn two_sided_subtract_after(tail: f64) -> f64 {
    if tail <= 0.5 {
        tail * 2.0
    } else {
        2.0 - 2.0 * tail
    }
}

fn orientation(
    df_numerator_bits: u64,
    df_denominator_bits: u64,
    len_a: usize,
    len_b: usize,
) -> Result<&'static str, String> {
    let a_df = ((len_a - 1) as f64).to_bits();
    let b_df = ((len_b - 1) as f64).to_bits();
    let ab = df_numerator_bits == a_df && df_denominator_bits == b_df;
    let ba = df_numerator_bits == b_df && df_denominator_bits == a_df;
    match (ab, ba) {
        (true, false) => Ok("A_over_B"),
        (false, true) => Ok("B_over_A"),
        (true, true) => Ok("equal_df_ambiguous"),
        (false, false) => Err(format!(
            "df mismatch: candidate {}/{} is neither A/B {}/{} nor B/A {}/{}",
            bits(df_numerator_bits),
            bits(df_denominator_bits),
            bits(a_df),
            bits(b_df),
            bits(b_df),
            bits(a_df)
        )),
    }
}

// Public-domain Numerical Recipes-style incomplete-beta implementation.  It
// is only a mathematical seed; live FDIST observations decide exactness.
fn ln_gamma(z: f64) -> f64 {
    const COEFFICIENTS: [f64; 9] = [
        0.999_999_999_999_809_9,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];
    if z < 0.5 {
        return std::f64::consts::PI.ln()
            - (std::f64::consts::PI * z).sin().ln()
            - ln_gamma(1.0 - z);
    }
    let z = z - 1.0;
    let mut x = COEFFICIENTS[0];
    for (index, coefficient) in COEFFICIENTS.iter().enumerate().skip(1) {
        x += coefficient / (z + index as f64);
    }
    let t = z + 7.5;
    0.5 * (2.0 * std::f64::consts::PI).ln() + (z + 0.5) * t.ln() - t + x.ln()
}

fn beta_fraction(a: f64, b: f64, x: f64) -> Result<f64, String> {
    const MAX_ITERATIONS: usize = 512;
    const EPSILON: f64 = 2.0e-16;
    const FLOOR: f64 = 1.0e-300;
    let qab = a + b;
    let qap = a + 1.0;
    let qam = a - 1.0;
    let mut c = 1.0;
    let mut d = 1.0 - qab * x / qap;
    if d.abs() < FLOOR {
        d = FLOOR;
    }
    d = 1.0 / d;
    let mut h = d;
    for iteration in 1..=MAX_ITERATIONS {
        let m = iteration as f64;
        let m2 = 2.0 * m;
        let mut aa = m * (b - m) * x / ((qam + m2) * (a + m2));
        d = 1.0 + aa * d;
        if d.abs() < FLOOR {
            d = FLOOR;
        }
        c = 1.0 + aa / c;
        if c.abs() < FLOOR {
            c = FLOOR;
        }
        d = 1.0 / d;
        h *= d * c;
        aa = -(a + m) * (qab + m) * x / ((a + m2) * (qap + m2));
        d = 1.0 + aa * d;
        if d.abs() < FLOOR {
            d = FLOOR;
        }
        c = 1.0 + aa / c;
        if c.abs() < FLOOR {
            c = FLOOR;
        }
        d = 1.0 / d;
        let delta = d * c;
        h *= delta;
        if (delta - 1.0).abs() <= EPSILON {
            return Ok(h);
        }
    }
    Err(format!(
        "incomplete-beta continued fraction did not converge for a={a} b={b} x={x}"
    ))
}

fn regularized_beta(x: f64, a: f64, b: f64) -> Result<f64, String> {
    if x <= 0.0 {
        return Ok(0.0);
    }
    if x >= 1.0 {
        return Ok(1.0);
    }
    let front = (ln_gamma(a + b) - ln_gamma(a) - ln_gamma(b) + a * x.ln() + b * (-x).ln_1p()).exp();
    if x < (a + 1.0) / (a + b + 2.0) {
        Ok(front * beta_fraction(a, b, x)? / a)
    } else {
        Ok(1.0 - front * beta_fraction(b, a, 1.0 - x)? / b)
    }
}

fn mathematical_f_tail(ratio: f64, df_numerator: f64, df_denominator: f64) -> Result<f64, String> {
    let denominator = df_denominator + df_numerator * ratio;
    regularized_beta(
        df_denominator / denominator,
        df_denominator * 0.5,
        df_numerator * 0.5,
    )
}

fn mathematical_inverse(
    target_tail: f64,
    df_numerator: f64,
    df_denominator: f64,
) -> Result<f64, String> {
    let (mut lo, mut hi) = (1.0_f64, 64.0_f64);
    let tail_lo = mathematical_f_tail(lo, df_numerator, df_denominator)?;
    let tail_hi = mathematical_f_tail(hi, df_numerator, df_denominator)?;
    if !(tail_hi <= target_tail && target_tail <= tail_lo) {
        return Err(format!(
            "target tail {target_tail} is not bracketed on [1,64]: [{tail_hi},{tail_lo}] for dfs {df_numerator}/{df_denominator}"
        ));
    }
    while lo.to_bits() + 1 < hi.to_bits() {
        let mid_bits = lo.to_bits() + (hi.to_bits() - lo.to_bits()) / 2;
        let mid = f64::from_bits(mid_bits);
        if mathematical_f_tail(mid, df_numerator, df_denominator)? > target_tail {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let lo_error = (mathematical_f_tail(lo, df_numerator, df_denominator)? - target_tail).abs();
    let hi_error = (mathematical_f_tail(hi, df_numerator, df_denominator)? - target_tail).abs();
    Ok(if lo_error <= hi_error { lo } else { hi })
}

fn audit(directory: &Path) -> Result<Audit, String> {
    let ftest_batch_value = load_json(&directory.join(FTEST_BATCH))?;
    let ftest = load_aligned(
        &directory.join(FTEST_BATCH),
        &directory.join(FTEST_ANSWERS),
        "F.TEST",
    )?;
    let fdist = load_aligned(
        &directory.join(FDIST_BATCH),
        &directory.join(FDIST_ANSWERS),
        "FDIST",
    )?;
    let vars = load_aligned(
        &directory.join(VARS_BATCH),
        &directory.join(VARS_ANSWERS),
        "VAR.S",
    )?;
    let meta = load_json(&directory.join(SOURCE_META))?;
    let rows = meta["rows"]
        .as_array()
        .ok_or_else(|| "source metadata rows missing".to_string())?;
    if rows.len() != 48 || ftest.by_id.len() != 48 || vars.by_id.len() != 96 {
        return Err(format!(
            "unexpected source counts rows={} F.TEST={} VAR.S={}",
            rows.len(),
            ftest.by_id.len(),
            vars.by_id.len()
        ));
    }
    let metadata_fdist_count = meta["fdist_count"]
        .as_u64()
        .ok_or_else(|| "source metadata fdist_count missing".to_string())?
        as usize;
    let group_sum: usize = rows
        .iter()
        .map(|row| row["candidate_groups"].as_array().map_or(0, Vec::len))
        .sum();
    if metadata_fdist_count != group_sum || group_sum != fdist.by_id.len() {
        return Err(format!(
            "FDIST count mismatch metadata={metadata_fdist_count} groups={group_sum} answers={}",
            fdist.by_id.len()
        ));
    }

    let ftest_probes = ftest_batch_value["probes"]
        .as_array()
        .ok_or_else(|| "F.TEST probes missing".to_string())?;
    let mut model_scores: BTreeMap<String, usize> = BTreeMap::new();
    let mut accepted_histogram: BTreeMap<usize, usize> = BTreeMap::new();
    let mut seen_fdist_ids = BTreeSet::new();
    let mut no_hits = Vec::new();
    let mut row_reports = Vec::new();
    let mut low_side_hits = 0_usize;
    let mut high_side_hits = 0_usize;
    let mut subtract_after_hits = 0_usize;
    let mut legacy_twice_hits = 0_usize;
    let mut public_vars_exact = 0_usize;
    let mut forward_native_exact = 0_usize;
    let mut forward_x87_exact = 0_usize;
    let mut ratio_one_rows = 0_usize;
    let mut ratio_one_exact_rows = 0_usize;
    let mut inverse_calibration_max_ulp = 0_u64;

    for (row_index, row) in rows.iter().enumerate() {
        let ftest_id = row["ftest_id"]
            .as_str()
            .ok_or_else(|| format!("row {row_index}: ftest_id missing"))?;
        if ftest.ordered_ids.get(row_index).map(String::as_str) != Some(ftest_id) {
            return Err(format!("row {row_index}: metadata/F.TEST order mismatch"));
        }
        let target_bits = ftest
            .by_id
            .get(ftest_id)
            .ok_or_else(|| format!("row {row_index}: missing F.TEST answer"))?
            .expected_bits;
        let sample_args = &ftest_probes[row_index]["probe"]["args"];
        let len_a = sample_args[0]
            .as_array()
            .ok_or_else(|| format!("row {row_index}: sample A missing"))?
            .len();
        let len_b = sample_args[1]
            .as_array()
            .ok_or_else(|| format!("row {row_index}: sample B missing"))?
            .len();
        let groups = row["candidate_groups"]
            .as_array()
            .ok_or_else(|| format!("row {row_index}: candidate_groups missing"))?;
        let mut accepted_keys = BTreeSet::new();
        let mut accepted_orientations = BTreeSet::new();
        let mut accepted_sides = BTreeSet::new();
        let mut nearest: Option<(u64, &Value, u64, u64)> = None;
        let mut row_legacy_twice = false;
        let mut row_subtract_after = false;
        let mut row_has_ratio_one = false;
        let mut row_ratio_one_exact = false;
        let mut row_forward_native = false;
        let mut row_forward_x87 = false;
        let mut forward_native_keys = BTreeSet::new();
        let mut forward_x87_keys = BTreeSet::new();

        for (group_index, group) in groups.iter().enumerate() {
            let fdist_id = group["fdist_id"]
                .as_str()
                .ok_or_else(|| format!("row {row_index} group {group_index}: fdist_id missing"))?;
            if !seen_fdist_ids.insert(fdist_id.to_string()) {
                return Err(format!("duplicate metadata FDIST id {fdist_id}"));
            }
            let ratio_bits = parse_bits(
                &group["ratio_bits"],
                &format!("row {row_index} group {group_index} ratio"),
            )?;
            let df_numerator_bits = parse_bits(
                &group["df_hi_bits"],
                &format!("row {row_index} group {group_index} numerator df"),
            )?;
            let df_denominator_bits = parse_bits(
                &group["df_lo_bits"],
                &format!("row {row_index} group {group_index} denominator df"),
            )?;
            let group_orientation =
                orientation(df_numerator_bits, df_denominator_bits, len_a, len_b)?;
            let witness = fdist
                .by_id
                .get(fdist_id)
                .ok_or_else(|| format!("row {row_index}: missing FDIST answer {fdist_id}"))?;
            let captured_args = witness.args.as_array().ok_or_else(|| {
                format!("row {row_index}: FDIST args are not an array for {fdist_id}")
            })?;
            if captured_args.len() != 3
                || parse_bits(&captured_args[0], &format!("{fdist_id} ratio"))? != ratio_bits
                || parse_bits(&captured_args[1], &format!("{fdist_id} numerator df"))?
                    != df_numerator_bits
                || parse_bits(&captured_args[2], &format!("{fdist_id} denominator df"))?
                    != df_denominator_bits
            {
                return Err(format!(
                    "row {row_index}: accepted-key/df mismatch between metadata and captured FDIST {fdist_id}"
                ));
            }
            let tail = f64::from_bits(witness.expected_bits);
            let two_sided_bits = two_sided_subtract_first(tail).to_bits();
            let distance = ulp_distance(two_sided_bits, target_bits);
            if nearest.as_ref().is_none_or(|current| distance < current.0) {
                nearest = Some((distance, group, witness.expected_bits, two_sided_bits));
            }
            if (tail * 2.0).to_bits() == target_bits {
                row_legacy_twice = true;
            }
            if two_sided_subtract_after(tail).to_bits() == target_bits {
                row_subtract_after = true;
            }
            let models = group["models"]
                .as_array()
                .ok_or_else(|| format!("row {row_index} group {group_index}: models missing"))?;
            let is_forward_native = models.iter().any(|model| {
                model.as_str()
                    == Some("two-pass mean=Native body=Native rev=false corr=false ratio=Native")
            });
            let is_forward_x87 = models.iter().any(|model| {
                model.as_str()
                    == Some("two-pass mean=Native body=Native rev=false corr=false ratio=X87")
            });
            let candidate_key = (ratio_bits, df_numerator_bits, df_denominator_bits);
            if is_forward_native {
                forward_native_keys.insert(candidate_key);
            }
            if is_forward_x87 {
                forward_x87_keys.insert(candidate_key);
            }
            if ratio_bits == 1.0_f64.to_bits() {
                row_has_ratio_one = true;
            }
            if two_sided_bits == target_bits {
                accepted_keys.insert(candidate_key);
                accepted_orientations.insert(group_orientation.to_string());
                accepted_sides.insert(if tail <= 0.5 { "low" } else { "high" });
                if ratio_bits == 1.0_f64.to_bits() {
                    row_ratio_one_exact = true;
                }
                if is_forward_native {
                    row_forward_native = true;
                }
                if is_forward_x87 {
                    row_forward_x87 = true;
                }
                for model in models {
                    let model = model.as_str().ok_or_else(|| {
                        format!("row {row_index} group {group_index}: non-string model")
                    })?;
                    *model_scores.entry(model.to_string()).or_default() += 1;
                }
                let ideal_tail = if tail <= 0.5 {
                    f64::from_bits(target_bits) * 0.5
                } else {
                    1.0 - f64::from_bits(target_bits) * 0.5
                };
                let seed = mathematical_inverse(
                    ideal_tail,
                    f64::from_bits(df_numerator_bits),
                    f64::from_bits(df_denominator_bits),
                )?;
                inverse_calibration_max_ulp =
                    inverse_calibration_max_ulp.max(ulp_distance(seed.to_bits(), ratio_bits));
            }
        }
        if accepted_sides.contains("low") {
            low_side_hits += 1;
        }
        if accepted_sides.contains("high") {
            high_side_hits += 1;
        }
        if row_subtract_after {
            subtract_after_hits += 1;
        }
        if row_legacy_twice {
            legacy_twice_hits += 1;
        }
        if row_has_ratio_one {
            ratio_one_rows += 1;
        }
        if row_ratio_one_exact {
            ratio_one_exact_rows += 1;
        }
        if row_forward_native {
            forward_native_exact += 1;
        }
        if row_forward_x87 {
            forward_x87_exact += 1;
        }
        if forward_native_keys.len() != 1 || forward_x87_keys.len() != 1 {
            return Err(format!(
                "row {row_index}: expected one forward Native and one forward X87 stored-ratio key, found {}/{}",
                forward_native_keys.len(),
                forward_x87_keys.len()
            ));
        }
        let forward_native_key = *forward_native_keys.iter().next().unwrap();
        let forward_x87_key = *forward_x87_keys.iter().next().unwrap();
        if forward_native_key.1 != forward_x87_key.1 || forward_native_key.2 != forward_x87_key.2 {
            return Err(format!(
                "row {row_index}: forward Native/X87 ratios select different df orientations"
            ));
        }

        let var_a = f64::from_bits(
            vars.by_id
                .get(&format!("vars-{row_index:03}-a"))
                .ok_or_else(|| format!("row {row_index}: public VAR.S A missing"))?
                .expected_bits,
        );
        let var_b = f64::from_bits(
            vars.by_id
                .get(&format!("vars-{row_index:03}-b"))
                .ok_or_else(|| format!("row {row_index}: public VAR.S B missing"))?
                .expected_bits,
        );
        let public_key = if var_a >= var_b {
            (
                (var_a / var_b).to_bits(),
                ((len_a - 1) as f64).to_bits(),
                ((len_b - 1) as f64).to_bits(),
            )
        } else {
            (
                (var_b / var_a).to_bits(),
                ((len_b - 1) as f64).to_bits(),
                ((len_a - 1) as f64).to_bits(),
            )
        };
        let public_vars_row_exact = accepted_keys.contains(&public_key);
        if public_vars_row_exact {
            public_vars_exact += 1;
        }
        *accepted_histogram.entry(accepted_keys.len()).or_default() += 1;

        let (nearest_distance, nearest_group, nearest_tail_bits, nearest_two_sided_bits) =
            nearest.ok_or_else(|| format!("row {row_index}: no candidate groups"))?;
        let nearest_ratio_bits = parse_bits(
            &nearest_group["ratio_bits"],
            &format!("row {row_index} nearest ratio"),
        )?;
        let nearest_df_numerator_bits = parse_bits(
            &nearest_group["df_hi_bits"],
            &format!("row {row_index} nearest numerator df"),
        )?;
        let nearest_df_denominator_bits = parse_bits(
            &nearest_group["df_lo_bits"],
            &format!("row {row_index} nearest denominator df"),
        )?;
        let nearest_orientation = orientation(
            nearest_df_numerator_bits,
            nearest_df_denominator_bits,
            len_a,
            len_b,
        )?;
        if accepted_keys.is_empty() {
            if nearest_df_numerator_bits != forward_native_key.1
                || nearest_df_denominator_bits != forward_native_key.2
            {
                return Err(format!(
                    "row {row_index}: nearest no-hit group uses a different orientation from both forward stored ratios"
                ));
            }
            no_hits.push(NoHit {
                row_index,
                ftest_id: ftest_id.to_string(),
                family: row["family"].as_str().unwrap_or("unknown").to_string(),
                target_bits,
                nearest_ratio_bits,
                df_numerator_bits: nearest_df_numerator_bits,
                df_denominator_bits: nearest_df_denominator_bits,
                nearest_tail_bits,
                nearest_two_sided_bits,
                nearest_output_ulp: nearest_distance,
                orientation: nearest_orientation.to_string(),
            });
        }
        row_reports.push(json!({
            "row_index": row_index,
            "ftest_id": ftest_id,
            "family": row["family"],
            "target_bits": bits(target_bits),
            "accepted_group_count": accepted_keys.len(),
            "accepted_keys": accepted_keys.into_iter().map(|(ratio, numerator, denominator)| json!({
                "ratio_bits": bits(ratio),
                "df_numerator_bits": bits(numerator),
                "df_denominator_bits": bits(denominator)
            })).collect::<Vec<_>>(),
            "accepted_orientations": accepted_orientations,
            "accepted_sides": accepted_sides,
            "legacy_twice_hit": row_legacy_twice,
            "subtract_after_hit": row_subtract_after,
            "forward_native_stored_ratio_hit": row_forward_native,
            "forward_x87_stored_ratio_hit": row_forward_x87,
            "forward_native_stored_key": {
                "ratio_bits": bits(forward_native_key.0),
                "df_numerator_bits": bits(forward_native_key.1),
                "df_denominator_bits": bits(forward_native_key.2)
            },
            "forward_x87_stored_key": {
                "ratio_bits": bits(forward_x87_key.0),
                "df_numerator_bits": bits(forward_x87_key.1),
                "df_denominator_bits": bits(forward_x87_key.2)
            },
            "public_vars_ratio_bits": bits(public_key.0),
            "public_vars_df_numerator_bits": bits(public_key.1),
            "public_vars_df_denominator_bits": bits(public_key.2),
            "public_vars_ratio_hit": public_vars_row_exact,
            "ratio_one_candidate": row_has_ratio_one,
            "ratio_one_hit": row_ratio_one_exact,
            "nearest": {
                "ratio_bits": bits(nearest_ratio_bits),
                "df_numerator_bits": bits(nearest_df_numerator_bits),
                "df_denominator_bits": bits(nearest_df_denominator_bits),
                "orientation": nearest_orientation,
                "tail_bits": bits(nearest_tail_bits),
                "two_sided_bits": bits(nearest_two_sided_bits),
                "output_ulp": nearest_distance
            }
        }));
    }
    if seen_fdist_ids.len() != fdist.by_id.len() {
        return Err(format!(
            "metadata references {} of {} captured FDIST ids",
            seen_fdist_ids.len(),
            fdist.by_id.len()
        ));
    }
    let histogram_json: BTreeMap<String, usize> = accepted_histogram
        .into_iter()
        .map(|(key, value)| (key.to_string(), value))
        .collect();
    let mut ranked_models: Vec<(usize, String)> = model_scores
        .into_iter()
        .map(|(name, score)| (score, name))
        .collect();
    ranked_models.sort_by(|left, right| right.0.cmp(&left.0).then(left.1.cmp(&right.1)));
    let report = json!({
        "audit_id": "w109-g3-06-ftest-variance-discovery-v2-offline-audit-20260809",
        "scope": "named discovery artifacts only; no heldout or Excel access",
        "counts": {
            "ftest": ftest.by_id.len(),
            "fdist": fdist.by_id.len(),
            "vars": vars.by_id.len(),
            "rows": rows.len()
        },
        "two_sided_rule": "tail <= 0.5 ? 2*tail : 2*(1-tail)",
        "accepted_group_histogram": histogram_json,
        "rows_with_any_exact_group": rows.len() - no_hits.len(),
        "no_hit_rows": no_hits.len(),
        "low_side_exact_rows": low_side_hits,
        "high_side_exact_rows": high_side_hits,
        "legacy_unconditional_twice_rows": legacy_twice_hits,
        "subtract_after_rows": subtract_after_hits,
        "forward_native_stored_ratio_exact": forward_native_exact,
        "forward_x87_stored_ratio_exact": forward_x87_exact,
        "public_vars_stored_ratio_exact": public_vars_exact,
        "ratio_one_boundary": {
            "candidate_rows": ratio_one_rows,
            "exact_rows": ratio_one_exact_rows,
            "note": "reported separately because F=1 can take a distinct internal-tail boundary route"
        },
        "mathematical_inverse_calibration_max_ulp": inverse_calibration_max_ulp,
        "ranked_models": ranked_models.into_iter().map(|(score, name)| json!({"score": score, "model": name})).collect::<Vec<_>>(),
        "rows_detail": row_reports,
        "interpretation_guard": "An exact external FDIST-equivalence ratio is not proof that F.TEST internally calls separately published FDIST."
    });
    Ok(Audit { report, no_hits })
}

fn add_offset(bits: u64, offset: i64) -> Result<u64, String> {
    if offset < 0 {
        bits.checked_sub(offset.unsigned_abs())
            .ok_or_else(|| format!("ratio-bit underflow at {offset}"))
    } else {
        bits.checked_add(offset as u64)
            .ok_or_else(|| format!("ratio-bit overflow at {offset}"))
    }
}

fn write_pretty(path: &Path, value: &Value) -> Result<(), String> {
    let mut text = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
    text.push('\n');
    fs::write(path, text).map_err(|error| format!("{}: {error}", path.display()))
}

fn freeze(directory: &Path) -> Result<(), String> {
    let audit = audit(directory)?;
    if directory.join(REFINEMENT_ANSWERS).exists() {
        return Err(format!(
            "refusing to rewrite a frozen refinement after answers exist: {}",
            directory.join(REFINEMENT_ANSWERS).display()
        ));
    }
    let mut probes = Vec::new();
    let mut meta_rows = Vec::new();
    for no_hit in &audit.no_hits {
        let target = f64::from_bits(no_hit.target_bits);
        let ideal_tail = target * 0.5;
        let df_numerator = f64::from_bits(no_hit.df_numerator_bits);
        let df_denominator = f64::from_bits(no_hit.df_denominator_bits);
        let seed = mathematical_inverse(ideal_tail, df_numerator, df_denominator)?;
        let seed_bits = seed.to_bits();
        let mut offsets: BTreeSet<i64> = (-LOCAL_RADIUS..=LOCAL_RADIUS).collect();
        offsets.extend(GUARD_OFFSETS);
        let mut row_probe_ids = Vec::new();
        let mut offset_meta = Vec::new();
        for offset in offsets {
            let ratio_bits = add_offset(seed_bits, offset)?;
            let ratio = f64::from_bits(ratio_bits);
            if !ratio.is_finite() || !(1.0..=64.0).contains(&ratio) {
                return Err(format!(
                    "row {}: refinement ratio {} outside admitted range",
                    no_hit.row_index,
                    bits(ratio_bits)
                ));
            }
            let probe_id = format!(
                "fd-var-ref-v3-{:03}-{:04}",
                no_hit.row_index,
                row_probe_ids.len()
            );
            probes.push(json!({
                "probe": {
                    "id": probe_id,
                    "args": [
                        bits(ratio_bits),
                        bits(no_hit.df_numerator_bits),
                        bits(no_hit.df_denominator_bits)
                    ]
                },
                "distinct_outputs": 0,
                "outputs": []
            }));
            row_probe_ids.push(probe_id);
            offset_meta.push(json!({"offset": offset, "ratio_bits": bits(ratio_bits)}));
        }
        meta_rows.push(json!({
            "row_index": no_hit.row_index,
            "ftest_id": no_hit.ftest_id,
            "family": no_hit.family,
            "target_bits": bits(no_hit.target_bits),
            "target_low_tail_bits": bits(ideal_tail.to_bits()),
            "orientation": no_hit.orientation,
            "df_numerator_bits": bits(no_hit.df_numerator_bits),
            "df_denominator_bits": bits(no_hit.df_denominator_bits),
            "nearest_source_ratio_bits": bits(no_hit.nearest_ratio_bits),
            "nearest_source_tail_bits": bits(no_hit.nearest_tail_bits),
            "nearest_source_two_sided_bits": bits(no_hit.nearest_two_sided_bits),
            "nearest_source_output_ulp": no_hit.nearest_output_ulp,
            "mathematical_seed_bits": bits(seed_bits),
            "probe_ids": row_probe_ids,
            "offsets": offset_meta
        }));
    }
    if audit.no_hits.len() != 15 {
        return Err(format!(
            "corrected two-sided audit found {} no-hit rows, expected 15",
            audit.no_hits.len()
        ));
    }
    let batch = json!({
        "function": "FDIST",
        "row_id": "G3-06-ftest-variance-refinement-discovery-v3-20260809",
        "probes": probes
    });
    let mut batch_text = serde_json::to_string_pretty(&batch).map_err(|error| error.to_string())?;
    batch_text.push('\n');
    let batch_sha256 = sha256_hex(batch_text.as_bytes());
    let source_manifest = file_manifest(
        directory,
        &[
            FTEST_BATCH,
            FTEST_ANSWERS,
            FDIST_BATCH,
            FDIST_ANSWERS,
            VARS_BATCH,
            VARS_ANSWERS,
            SOURCE_META,
        ],
    )?;
    let refinement_meta = json!({
        "freeze_id": "w109-g3-06-ftest-variance-refinement-discovery-v3-20260809",
        "scope": "answer-aware discovery refinement over the 15 corrected no-hit rows; no heldout",
        "source_manifest": source_manifest,
        "two_sided_rule": "tail <= 0.5 ? 2*tail : 2*(1-tail)",
        "seed": "independent public incomplete-beta inverse with per-group numerator/denominator dfs",
        "window": {"local_radius_ulp": LOCAL_RADIUS, "guard_offsets_ulp": GUARD_OFFSETS},
        "row_count": meta_rows.len(),
        "probe_count": batch["probes"].as_array().unwrap().len(),
        "batch_path": REFINEMENT_BATCH,
        "batch_sha256": batch_sha256,
        "future_answer_path": REFINEMENT_ANSWERS,
        "future_answer_present_at_freeze": false,
        "capture_contract": {
            "excel_version": "16.0",
            "excel_build": "20228",
            "excel_bitness": "64-bit",
            "workbook_compatibility": "2",
            "excel_input_plumbing": "cell_value2_bulk",
            "oracle_cache": "no_cache",
            "required_pre_excel_process_count": 0,
            "required_post_excel_process_count": 0
        },
        "generation_assertions": {
            "corrected_no_hit_rows": 15,
            "all_seed_dfs_match_both_forward_stored_ratio_orientations": true,
            "known_hit_inverse_calibration_max_ulp": audit.report["mathematical_inverse_calibration_max_ulp"],
            "unique_probe_ids": true
        },
        "rows": meta_rows,
        "interpretation_guard": "Recovered ratios are external FDIST-equivalence witnesses, not automatically F.TEST internal variance ratios."
    });
    write_pretty(&directory.join(AUDIT_REPORT), &audit.report)?;
    write_pretty(&directory.join(REFINEMENT_BATCH), &batch)?;
    write_pretty(&directory.join(REFINEMENT_META), &refinement_meta)?;
    println!(
        "frozen {} probes across {} corrected no-hit rows",
        batch["probes"].as_array().unwrap().len(),
        audit.no_hits.len()
    );
    Ok(())
}

fn score_cdf(directory: &Path) -> Result<(), String> {
    let ftest = load_aligned(
        &directory.join(FTEST_BATCH),
        &directory.join(FTEST_ANSWERS),
        "F.TEST",
    )?;
    let right_tail = load_aligned(
        &directory.join(FDIST_BATCH),
        &directory.join(FDIST_ANSWERS),
        "FDIST",
    )?;
    let cdf = load_aligned(
        &directory.join(CDF_BATCH),
        &directory.join(CDF_ANSWERS),
        "F.DIST",
    )?;
    let source_meta = load_json(&directory.join(SOURCE_META))?;
    let cdf_meta = load_json(&directory.join(CDF_META))?;
    let mapping_rows = cdf_meta["rows"]
        .as_array()
        .ok_or_else(|| "CDF mapping rows missing".to_string())?;
    if cdf.by_id.len() != 369 {
        return Err(format!(
            "CDF companion expected 369 unique probes, found {}",
            cdf.by_id.len()
        ));
    }
    let mut mapping = BTreeMap::new();
    for (index, mapping_row) in mapping_rows.iter().enumerate() {
        if mapping_row["source"].as_str() != Some("discovery") {
            continue;
        }
        if mapping_row["source_file"].as_str() != Some(FDIST_BATCH) {
            return Err(format!(
                "CDF mapping {index}: unexpected discovery source file"
            ));
        }
        let source_id = mapping_row["source_id"]
            .as_str()
            .ok_or_else(|| format!("CDF mapping {index}: source id missing"))?;
        let cdf_id = mapping_row["cdf_id"]
            .as_str()
            .ok_or_else(|| format!("CDF mapping {index}: CDF id missing"))?;
        if mapping
            .insert(source_id.to_string(), cdf_id.to_string())
            .is_some()
        {
            return Err(format!("duplicate CDF mapping for {source_id}"));
        }
    }
    if mapping.len() != right_tail.by_id.len() {
        return Err(format!(
            "CDF discovery mapping count {} != RT count {}",
            mapping.len(),
            right_tail.by_id.len()
        ));
    }
    let rows = source_meta["rows"]
        .as_array()
        .ok_or_else(|| "source rows missing".to_string())?;
    let mut histogram: BTreeMap<usize, usize> = BTreeMap::new();
    let mut forward_native = 0_usize;
    let mut forward_x87 = 0_usize;
    let mut exact_rows = 0_usize;
    for (row_index, row) in rows.iter().enumerate() {
        let ftest_id = row["ftest_id"]
            .as_str()
            .ok_or_else(|| format!("row {row_index}: ftest id missing"))?;
        let target_bits = ftest
            .by_id
            .get(ftest_id)
            .ok_or_else(|| format!("row {row_index}: F.TEST answer missing"))?
            .expected_bits;
        let groups = row["candidate_groups"]
            .as_array()
            .ok_or_else(|| format!("row {row_index}: groups missing"))?;
        let mut accepted = BTreeSet::new();
        let mut row_forward_native = false;
        let mut row_forward_x87 = false;
        for group in groups {
            let rt_id = group["fdist_id"]
                .as_str()
                .ok_or_else(|| format!("row {row_index}: FDIST id missing"))?;
            let cdf_id = mapping
                .get(rt_id)
                .map(String::as_str)
                .ok_or_else(|| format!("row {row_index}: CDF mapping missing for {rt_id}"))?;
            let rt_witness = right_tail
                .by_id
                .get(rt_id)
                .ok_or_else(|| format!("row {row_index}: RT answer missing for {rt_id}"))?;
            let cdf_witness = cdf
                .by_id
                .get(cdf_id)
                .ok_or_else(|| format!("row {row_index}: CDF answer missing for {cdf_id}"))?;
            let cdf_args = cdf_witness
                .args
                .as_array()
                .ok_or_else(|| format!("row {row_index}: CDF args missing for {cdf_id}"))?;
            let rt_args = rt_witness
                .args
                .as_array()
                .ok_or_else(|| format!("row {row_index}: RT args missing for {rt_id}"))?;
            if cdf_args.len() != 4
                || cdf_args[..3] != rt_args[..]
                || cdf_args[3].as_bool() != Some(true)
            {
                return Err(format!(
                    "row {row_index}: CDF/RT argument mismatch for {cdf_id}/{rt_id}"
                ));
            }
            let rt = f64::from_bits(rt_witness.expected_bits);
            let direct_cdf = f64::from_bits(cdf_witness.expected_bits);
            let output_bits = (2.0 * rt.min(direct_cdf)).to_bits();
            if output_bits == target_bits {
                let key = (
                    parse_bits(&group["ratio_bits"], "ratio")?,
                    parse_bits(&group["df_hi_bits"], "numerator df")?,
                    parse_bits(&group["df_lo_bits"], "denominator df")?,
                );
                accepted.insert(key);
                let models = group["models"]
                    .as_array()
                    .ok_or_else(|| format!("row {row_index}: models missing"))?;
                row_forward_native |= models.iter().any(|model| {
                    model.as_str()
                        == Some(
                            "two-pass mean=Native body=Native rev=false corr=false ratio=Native",
                        )
                });
                row_forward_x87 |= models.iter().any(|model| {
                    model.as_str()
                        == Some("two-pass mean=Native body=Native rev=false corr=false ratio=X87")
                });
            }
        }
        if !accepted.is_empty() {
            exact_rows += 1;
        }
        if row_forward_native {
            forward_native += 1;
        }
        if row_forward_x87 {
            forward_x87 += 1;
        }
        *histogram.entry(accepted.len()).or_default() += 1;
        println!(
            "row={row_index:02} id={ftest_id} accepted={} forward_native={} forward_x87={}",
            accepted.len(),
            row_forward_native,
            row_forward_x87
        );
    }
    println!("direct-CDF accepted histogram {histogram:?}");
    println!("direct-CDF exact rows {exact_rows}/{}", rows.len());
    println!(
        "direct-CDF forward Native/X87 {forward_native}/{forward_x87} of {}",
        rows.len()
    );
    Ok(())
}

fn score_refinement(directory: &Path) -> Result<(), String> {
    let original_ftest = load_aligned(
        &directory.join(FTEST_BATCH),
        &directory.join(FTEST_ANSWERS),
        "F.TEST",
    )?;
    let answers = load_aligned(
        &directory.join(REFINEMENT_BATCH),
        &directory.join(REFINEMENT_ANSWERS),
        "FDIST",
    )?;
    let meta = load_json(&directory.join(REFINEMENT_META))?;
    let answer_document = load_json(&directory.join(REFINEMENT_ANSWERS))?;
    let source_meta = load_json(&directory.join(SOURCE_META))?;
    let audit_report = load_json(&directory.join(AUDIT_REPORT))?;
    let actual_batch_bytes = fs::read(directory.join(REFINEMENT_BATCH))
        .map_err(|error| format!("{}: {error}", directory.join(REFINEMENT_BATCH).display()))?;
    let actual_batch_sha = sha256_hex(&actual_batch_bytes);
    if meta["batch_sha256"].as_str() != Some(actual_batch_sha.as_str()) {
        return Err(format!(
            "refinement batch SHA mismatch metadata={:?} actual={actual_batch_sha}",
            meta["batch_sha256"]
        ));
    }
    let source_manifest = meta["source_manifest"]
        .as_array()
        .ok_or_else(|| "refinement source_manifest missing".to_string())?;
    for entry in source_manifest {
        let name = entry["path"]
            .as_str()
            .ok_or_else(|| "source manifest path missing".to_string())?;
        let source_bytes = fs::read(directory.join(name))
            .map_err(|error| format!("{}: {error}", directory.join(name).display()))?;
        if entry["bytes"].as_u64() != Some(source_bytes.len() as u64)
            || entry["sha256"].as_str() != Some(sha256_hex(&source_bytes).as_str())
        {
            return Err(format!("source manifest mismatch for {name}"));
        }
    }
    let provenance = &answer_document["capture_provenance"];
    let environment = &provenance["environment"];
    let cache = &provenance["oracle_cache"];
    if environment["excel_version"].as_str() != Some("16.0")
        || environment["excel_build"].as_str() != Some("20228")
        || environment["excel_bitness"].as_str() != Some("64-bit")
        || environment["workbook_compatibility"].as_str() != Some("2")
        || environment["excel_input_plumbing"].as_str() != Some("cell_value2_bulk")
        || cache["mode"].as_str() != Some("no_cache")
        || cache["hits"].as_u64() != Some(0)
        || cache["misses"].as_u64() != Some(0)
    {
        return Err(
            "refinement capture provenance does not satisfy the frozen contract".to_string(),
        );
    }
    let rows = meta["rows"]
        .as_array()
        .ok_or_else(|| "refinement metadata rows missing".to_string())?;
    let expected_count = meta["probe_count"]
        .as_u64()
        .ok_or_else(|| "refinement probe_count missing".to_string())?
        as usize;
    if expected_count != answers.by_id.len() {
        return Err(format!(
            "refinement count mismatch metadata={expected_count} answers={}",
            answers.by_id.len()
        ));
    }
    let original_rows = source_meta["rows"]
        .as_array()
        .ok_or_else(|| "original metadata rows missing".to_string())?;
    let audit_rows = audit_report["rows_detail"]
        .as_array()
        .ok_or_else(|| "audit row detail missing".to_string())?;
    let mut exact_rows = 0_usize;
    let mut skipped_rows = 0_usize;
    let mut unbracketed_rows = 0_usize;
    let mut row_reports = Vec::new();
    for row in rows {
        let row_index = row["row_index"]
            .as_u64()
            .ok_or_else(|| "refinement row_index missing".to_string())?
            as usize;
        let ftest_id = row["ftest_id"]
            .as_str()
            .ok_or_else(|| "refinement ftest_id missing".to_string())?;
        let target_bits = original_ftest
            .by_id
            .get(ftest_id)
            .ok_or_else(|| format!("missing original F.TEST {ftest_id}"))?
            .expected_bits;
        let probe_ids = row["probe_ids"]
            .as_array()
            .ok_or_else(|| format!("{ftest_id}: probe_ids missing"))?;
        let offsets = row["offsets"]
            .as_array()
            .ok_or_else(|| format!("{ftest_id}: offsets missing"))?;
        if offsets.len() != probe_ids.len() {
            return Err(format!("{ftest_id}: probe/offset count mismatch"));
        }
        let df_numerator_bits = parse_bits(&row["df_numerator_bits"], "numerator df")?;
        let df_denominator_bits = parse_bits(&row["df_denominator_bits"], "denominator df")?;
        let mut entries = Vec::new();
        let mut exact = Vec::new();
        let mut nearest: Option<(u64, String, u64, u64, u64, i64)> = None;
        for (id, offset_meta) in probe_ids.iter().zip(offsets) {
            let id = id
                .as_str()
                .ok_or_else(|| format!("{ftest_id}: non-string probe id"))?;
            let offset = offset_meta["offset"]
                .as_i64()
                .ok_or_else(|| format!("{ftest_id}: offset missing for {id}"))?;
            let ratio_bits = parse_bits(&offset_meta["ratio_bits"], "offset ratio")?;
            let witness = answers
                .by_id
                .get(id)
                .ok_or_else(|| format!("missing refinement answer {id}"))?;
            let witness_args = witness
                .args
                .as_array()
                .ok_or_else(|| format!("{ftest_id}: captured args missing for {id}"))?;
            if witness_args.len() != 3
                || parse_bits(&witness_args[0], "captured ratio")? != ratio_bits
                || parse_bits(&witness_args[1], "captured numerator df")? != df_numerator_bits
                || parse_bits(&witness_args[2], "captured denominator df")? != df_denominator_bits
            {
                return Err(format!("{ftest_id}: captured key/df mismatch for {id}"));
            }
            let tail = f64::from_bits(witness.expected_bits);
            if tail > 0.5 {
                return Err(format!(
                    "{ftest_id}: refinement unexpectedly crossed to the high-tail branch"
                ));
            }
            let output_bits = two_sided_subtract_first(tail).to_bits();
            let distance = ulp_distance(output_bits, target_bits);
            if distance == 0 {
                exact.push((id.to_string(), ratio_bits, witness.expected_bits, offset));
            }
            if nearest.as_ref().is_none_or(|current| distance < current.0) {
                nearest = Some((
                    distance,
                    id.to_string(),
                    ratio_bits,
                    witness.expected_bits,
                    output_bits,
                    offset,
                ));
            }
            entries.push((
                ratio_bits,
                output_bits,
                witness.expected_bits,
                id.to_string(),
                offset,
            ));
        }
        entries.sort_by_key(|entry| entry.0);
        let monotonic_violation_count = entries
            .windows(2)
            .filter(|pair| pair[0].1 < pair[1].1)
            .count();
        let local: Vec<_> = entries
            .iter()
            .filter(|entry| entry.4.abs() <= LOCAL_RADIUS)
            .collect();
        if local.len() != (2 * LOCAL_RADIUS + 1) as usize
            || local.windows(2).any(|pair| pair[0].0 + 1 != pair[1].0)
        {
            return Err(format!(
                "{ftest_id}: local refinement is not a contiguous {}-bit window",
                2 * LOCAL_RADIUS + 1
            ));
        }
        let local_max = local.iter().map(|entry| entry.1).max().unwrap();
        let local_min = local.iter().map(|entry| entry.1).min().unwrap();
        let bracketed = local_min <= target_bits && target_bits <= local_max;
        let crossing = local.windows(2).find(|pair| {
            (pair[0].1 > target_bits && pair[1].1 < target_bits)
                || (pair[0].1 < target_bits && pair[1].1 > target_bits)
        });
        let classification = if !exact.is_empty() {
            exact_rows += 1;
            "external_fdist_equivalence_found"
        } else if let Some(pair) = crossing {
            if pair[0].0 + 1 != pair[1].0 {
                return Err(format!("{ftest_id}: non-contiguous skip crossing"));
            }
            skipped_rows += 1;
            "external_fdist_local_output_skip"
        } else {
            unbracketed_rows += 1;
            "not_resolved_by_frozen_window"
        };
        let nearest = nearest.unwrap();
        let original_groups = original_rows[row_index]["candidate_groups"]
            .as_array()
            .ok_or_else(|| format!("{ftest_id}: original groups missing"))?;
        let original_keys: BTreeSet<(u64, u64, u64)> = original_groups
            .iter()
            .map(|group| {
                Ok((
                    parse_bits(&group["ratio_bits"], "original ratio")?,
                    parse_bits(&group["df_hi_bits"], "original numerator df")?,
                    parse_bits(&group["df_lo_bits"], "original denominator df")?,
                ))
            })
            .collect::<Result<_, String>>()?;
        let audit_row = &audit_rows[row_index];
        if audit_row["ftest_id"].as_str() != Some(ftest_id) {
            return Err(format!("{ftest_id}: audit row order mismatch"));
        }
        let forward_native_ratio = parse_bits(
            &audit_row["forward_native_stored_key"]["ratio_bits"],
            "forward Native ratio",
        )?;
        let forward_x87_ratio = parse_bits(
            &audit_row["forward_x87_stored_key"]["ratio_bits"],
            "forward X87 ratio",
        )?;
        let public_vars_ratio =
            parse_bits(&audit_row["public_vars_ratio_bits"], "public VAR.S ratio")?;
        let signed_delta = |value: u64, base: u64| -> Result<i64, String> {
            let delta = value as i128 - base as i128;
            i64::try_from(delta).map_err(|_| "ratio-bit delta overflow".to_string())
        };
        let exact_json = exact
            .iter()
            .map(|(id, ratio, tail, offset)| {
                Ok(json!({
                    "probe_id": id,
                    "ratio_bits": bits(*ratio),
                    "tail_bits": bits(*tail),
                    "seed_offset_ulp": offset,
                    "delta_from_forward_native_ulp": signed_delta(*ratio, forward_native_ratio)?,
                    "delta_from_forward_x87_ulp": signed_delta(*ratio, forward_x87_ratio)?,
                    "delta_from_public_vars_ulp": signed_delta(*ratio, public_vars_ratio)?,
                    "matches_original_candidate_key": original_keys.contains(&(*ratio, df_numerator_bits, df_denominator_bits))
                }))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let crossing_json = crossing.map(|pair| {
            let high_output = pair[0].1.max(pair[1].1);
            let low_output = pair[0].1.min(pair[1].1);
            json!({
                "lower_ratio_bits": bits(pair[0].0),
                "lower_ratio_output_bits": bits(pair[0].1),
                "upper_ratio_bits": bits(pair[1].0),
                "upper_ratio_output_bits": bits(pair[1].1),
                "consecutive_input_bits": pair[0].0 + 1 == pair[1].0,
                "skipped_output_bit_count": high_output - low_output - 1
            })
        });
        println!(
            "{} class={} exact={} nearest={} id={} output={}",
            ftest_id,
            classification,
            exact.len(),
            nearest.0,
            nearest.1,
            bits(nearest.4)
        );
        row_reports.push(json!({
            "row_index": row_index,
            "ftest_id": ftest_id,
            "family": row["family"],
            "target_bits": bits(target_bits),
            "classification": classification,
            "orientation": row["orientation"],
            "df_numerator_bits": bits(df_numerator_bits),
            "df_denominator_bits": bits(df_denominator_bits),
            "local_window": {
                "input_count": local.len(),
                "radius_ulp": LOCAL_RADIUS,
                "monotonic_violation_count": monotonic_violation_count,
                "bracketed": bracketed,
                "max_output_bits": bits(local_max),
                "min_output_bits": bits(local_min)
            },
            "crossing": crossing_json,
            "exact_witnesses": exact_json,
            "nearest": {
                "output_ulp": nearest.0,
                "probe_id": nearest.1,
                "ratio_bits": bits(nearest.2),
                "tail_bits": bits(nearest.3),
                "output_bits": bits(nearest.4),
                "seed_offset_ulp": nearest.5
            }
        }));
    }
    let score = json!({
        "score_id": "w109-g3-06-ftest-variance-refinement-discovery-v3-score-20260809",
        "scope": "discovery only; no heldout claim",
        "batch_sha256": actual_batch_sha,
        "answer_sha256": sha256_hex(&fs::read(directory.join(REFINEMENT_ANSWERS)).map_err(|error| error.to_string())?),
        "capture_provenance": provenance,
        "row_count": rows.len(),
        "external_fdist_equivalence_rows": exact_rows,
        "external_fdist_local_output_skip_rows": skipped_rows,
        "not_resolved_rows": unbracketed_rows,
        "rows": row_reports,
        "interpretation": {
            "equivalence_found": "The live external FDIST graph can publish the F.TEST bits at a nearby ratio, but that ratio is not automatically F.TEST's internal statistic.",
            "output_skip": "Within the frozen contiguous window, two consecutive ratio inputs bracket the target while live external FDIST skips its bit pattern; the miss cannot be attributed solely to variance/statistic construction."
        }
    });
    write_pretty(&directory.join(REFINEMENT_SCORE), &score)?;
    println!(
        "refinement classification equivalence={exact_rows} output_skip={skipped_rows} unresolved={unbracketed_rows} total={}",
        rows.len()
    );
    Ok(())
}

fn print_audit(directory: &Path) -> Result<(), String> {
    let audit = audit(directory)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&audit.report).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let command = args.next().ok_or_else(|| {
        "usage: refine_ftest_variance_fdist <audit|freeze|score-cdf|score-refinement> <directory>"
            .to_string()
    })?;
    let directory = PathBuf::from(
        args.next()
            .ok_or_else(|| "missing discovery directory".to_string())?,
    );
    if args.next().is_some() {
        return Err("too many arguments".to_string());
    }
    match command.as_str() {
        "audit" => print_audit(&directory),
        "freeze" => freeze(&directory),
        "score-cdf" => score_cdf(&directory),
        "score-refinement" => score_refinement(&directory),
        _ => Err(format!("unknown command {command}")),
    }
}
