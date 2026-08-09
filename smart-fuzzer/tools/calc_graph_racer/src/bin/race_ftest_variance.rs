//! Offline W109 G3-06 variance-schedule race over the injective F.TEST/FDIST pins.

use oxfunc_core::excel_numeric::research as rx;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

const CW: u16 = rx::CW_PC64_RN;

#[derive(Clone, Copy, Debug)]
enum Op {
    Native,
    X87,
}

fn ext(x: f64) -> rx::Ext80 {
    rx::ext_from_f64(x)
}

fn to_f64(x: &rx::Ext80) -> f64 {
    rx::ext_to_f64(x, CW)
}

fn add(op: Op, a: f64, b: f64) -> f64 {
    match op {
        Op::Native => a + b,
        Op::X87 => to_f64(&rx::ext_add(&ext(a), &ext(b), CW)),
    }
}

fn sub(op: Op, a: f64, b: f64) -> f64 {
    match op {
        Op::Native => a - b,
        Op::X87 => to_f64(&rx::ext_sub(&ext(a), &ext(b), CW)),
    }
}

fn mul(op: Op, a: f64, b: f64) -> f64 {
    match op {
        Op::Native => a * b,
        Op::X87 => to_f64(&rx::ext_mul(&ext(a), &ext(b), CW)),
    }
}

fn div(op: Op, a: f64, b: f64) -> f64 {
    match op {
        Op::Native => a / b,
        Op::X87 => to_f64(&rx::ext_div(&ext(a), &ext(b), CW)),
    }
}

fn ordered(values: &[f64], reverse: bool) -> Vec<f64> {
    let mut out = values.to_vec();
    if reverse {
        out.reverse();
    }
    out
}

fn sum(values: &[f64], op: Op, reverse: bool) -> f64 {
    ordered(values, reverse)
        .into_iter()
        .fold(0.0, |acc, value| add(op, acc, value))
}

fn two_pass(values: &[f64], mean_op: Op, body_op: Op, reverse: bool, correction: bool) -> f64 {
    let mean = div(mean_op, sum(values, mean_op, reverse), values.len() as f64);
    let mut sd = 0.0;
    let mut sd2 = 0.0;
    for value in ordered(values, reverse) {
        let delta = sub(body_op, value, mean);
        sd = add(body_op, sd, delta);
        sd2 = add(body_op, sd2, mul(body_op, delta, delta));
    }
    let numerator = if correction {
        sub(
            body_op,
            sd2,
            div(body_op, mul(body_op, sd, sd), values.len() as f64),
        )
    } else {
        sd2
    };
    div(body_op, numerator, (values.len() - 1) as f64)
}

fn shifted(values: &[f64], op: Op, reverse: bool, anchor_last: bool, correction: bool) -> f64 {
    let anchor = if anchor_last {
        values[values.len() - 1]
    } else {
        values[0]
    };
    let mut sd = 0.0;
    let mut sd2 = 0.0;
    for value in ordered(values, reverse) {
        let delta = sub(op, value, anchor);
        sd = add(op, sd, delta);
        sd2 = add(op, sd2, mul(op, delta, delta));
    }
    let numerator = if correction {
        sub(op, sd2, div(op, mul(op, sd, sd), values.len() as f64))
    } else {
        sd2
    };
    div(op, numerator, (values.len() - 1) as f64)
}

fn one_pass(values: &[f64], op: Op, reverse: bool) -> f64 {
    let mut s = 0.0;
    let mut ss = 0.0;
    for value in ordered(values, reverse) {
        s = add(op, s, value);
        ss = add(op, ss, mul(op, value, value));
    }
    let correction = div(op, mul(op, s, s), values.len() as f64);
    div(op, sub(op, ss, correction), (values.len() - 1) as f64)
}

fn welford(values: &[f64], op: Op, reverse: bool) -> f64 {
    let mut mean = 0.0;
    let mut m2 = 0.0;
    for (index, value) in ordered(values, reverse).into_iter().enumerate() {
        let n = (index + 1) as f64;
        let delta = sub(op, value, mean);
        mean = add(op, mean, div(op, delta, n));
        let delta2 = sub(op, value, mean);
        m2 = add(op, m2, mul(op, delta, delta2));
    }
    div(op, m2, (values.len() - 1) as f64)
}

fn ext_two_pass(values: &[f64], reverse: bool, correction: bool) -> f64 {
    let values = ordered(values, reverse);
    let mut s = ext(0.0);
    for value in &values {
        s = rx::ext_add(&s, &ext(*value), CW);
    }
    let mean = rx::ext_div(&s, &ext(values.len() as f64), CW);
    let mut sd = ext(0.0);
    let mut sd2 = ext(0.0);
    for value in &values {
        let d = rx::ext_sub(&ext(*value), &mean, CW);
        sd = rx::ext_add(&sd, &d, CW);
        sd2 = rx::ext_add(&sd2, &rx::ext_mul(&d, &d, CW), CW);
    }
    let numerator = if correction {
        let corr = rx::ext_div(&rx::ext_mul(&sd, &sd, CW), &ext(values.len() as f64), CW);
        rx::ext_sub(&sd2, &corr, CW)
    } else {
        sd2
    };
    to_f64(&rx::ext_div(
        &numerator,
        &ext((values.len() - 1) as f64),
        CW,
    ))
}

fn ratio(a: f64, b: f64, op: Op) -> f64 {
    if a >= b { div(op, a, b) } else { div(op, b, a) }
}

fn f(bits: u64) -> f64 {
    f64::from_bits(bits)
}

fn bits(value: f64) -> String {
    format!("0x{:016x}", value.to_bits())
}

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut value = *state;
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn generated_values(seed: u64, len: usize, family: usize, side: usize) -> Vec<f64> {
    let mut state = seed ^ ((family as u64) << 48) ^ ((side as u64) << 60) ^ 0x4654_4553_5456_4152;
    let (center, quantum) = match family {
        0 => (((seed % 41) as f64 - 20.0) * 0.25, 2.0_f64.powi(-9)),
        1 => (
            2.0_f64.powi(20) + ((seed % 17) as f64) * 0.125,
            2.0_f64.powi(-28),
        ),
        2 => (
            2.0_f64.powi(40) + ((seed % 13) as f64) * 0.25,
            2.0_f64.powi(-10),
        ),
        3 => (2.0_f64.powi(52), 1.0),
        _ => unreachable!(),
    };
    (0..len)
        .map(|index| {
            let random = splitmix64(&mut state);
            let signed = ((random & 0xfff) as i64 - 2048) as f64;
            let shape = (((index * index + 3 * side + family) % 17) as f64 - 8.0) * 0.25;
            center + (signed + shape) * quantum
        })
        .collect()
}

#[derive(Clone, Debug)]
struct RoutePrediction {
    name: String,
    var_a: f64,
    var_b: f64,
    ratio: f64,
    df_hi: f64,
    df_lo: f64,
}

fn route(
    name: String,
    var_a: f64,
    var_b: f64,
    len_a: usize,
    len_b: usize,
    op: Op,
) -> RoutePrediction {
    let (ratio, df_hi, df_lo) = if var_a >= var_b {
        (
            div(op, var_a, var_b),
            (len_a - 1) as f64,
            (len_b - 1) as f64,
        )
    } else {
        (
            div(op, var_b, var_a),
            (len_b - 1) as f64,
            (len_a - 1) as f64,
        )
    };
    RoutePrediction {
        name: format!("{name} ratio={op:?}"),
        var_a,
        var_b,
        ratio,
        df_hi,
        df_lo,
    }
}

fn predictions(a: &[f64], b: &[f64]) -> Vec<RoutePrediction> {
    let mut variances: Vec<(String, f64, f64)> = Vec::new();
    for mean_op in [Op::Native, Op::X87] {
        for body_op in [Op::Native, Op::X87] {
            for reverse in [false, true] {
                for correction in [false, true] {
                    variances.push((
                        format!(
                            "two-pass mean={mean_op:?} body={body_op:?} rev={reverse} corr={correction}"
                        ),
                        two_pass(a, mean_op, body_op, reverse, correction),
                        two_pass(b, mean_op, body_op, reverse, correction),
                    ));
                }
            }
        }
    }
    for op in [Op::Native, Op::X87] {
        for reverse in [false, true] {
            for anchor_last in [false, true] {
                for correction in [false, true] {
                    variances.push((
                        format!(
                            "shifted op={op:?} rev={reverse} last={anchor_last} corr={correction}"
                        ),
                        shifted(a, op, reverse, anchor_last, correction),
                        shifted(b, op, reverse, anchor_last, correction),
                    ));
                }
            }
            variances.push((
                format!("one-pass op={op:?} rev={reverse}"),
                one_pass(a, op, reverse),
                one_pass(b, op, reverse),
            ));
            variances.push((
                format!("welford op={op:?} rev={reverse}"),
                welford(a, op, reverse),
                welford(b, op, reverse),
            ));
        }
    }
    for reverse in [false, true] {
        for correction in [false, true] {
            variances.push((
                format!("ext-two-pass rev={reverse} corr={correction}"),
                ext_two_pass(a, reverse, correction),
                ext_two_pass(b, reverse, correction),
            ));
        }
    }

    let mut out = Vec::new();
    for (name, var_a, var_b) in variances {
        for op in [Op::Native, Op::X87] {
            out.push(route(name.clone(), var_a, var_b, a.len(), b.len(), op));
        }
    }
    out
}

fn write_pretty(path: &Path, value: &Value) -> Result<(), String> {
    let mut text = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
    text.push('\n');
    fs::write(path, text).map_err(|error| format!("{}: {error}", path.display()))
}

fn parse_hex_bits(text: &str) -> Result<u64, String> {
    u64::from_str_radix(
        text.strip_prefix("0x")
            .ok_or_else(|| format!("expected hexadecimal bits, got {text}"))?,
        16,
    )
    .map_err(|error| error.to_string())
}

fn load_aligned_answers(
    batch_path: &Path,
    answer_path: &Path,
    expected_function: &str,
) -> Result<BTreeMap<String, u64>, String> {
    let batch: Value = serde_json::from_str(
        &fs::read_to_string(batch_path)
            .map_err(|error| format!("{}: {error}", batch_path.display()))?,
    )
    .map_err(|error| error.to_string())?;
    let answers: Value = serde_json::from_str(
        &fs::read_to_string(answer_path)
            .map_err(|error| format!("{}: {error}", answer_path.display()))?,
    )
    .map_err(|error| error.to_string())?;
    if batch["function"].as_str() != Some(expected_function)
        || answers["function"].as_str() != Some(expected_function)
    {
        return Err(format!("function mismatch for {expected_function}"));
    }
    let provenance = &answers["capture_provenance"];
    let environment = &provenance["environment"];
    let cache = &provenance["oracle_cache"];
    if environment["excel_version"].as_str() != Some("16.0")
        || environment["excel_build"].as_str() != Some("20228")
        || environment["excel_bitness"].as_str() != Some("64-bit")
        || environment["workbook_compatibility"].as_str() != Some("2")
        || cache["mode"].as_str() != Some("no_cache")
        || cache["hits"].as_u64() != Some(0)
        || cache["misses"].as_u64() != Some(0)
    {
        return Err(format!(
            "capture provenance mismatch for {expected_function}"
        ));
    }
    let probes = batch["probes"]
        .as_array()
        .ok_or_else(|| "batch probes missing".to_string())?;
    let witnesses = answers["witnesses"]
        .as_array()
        .ok_or_else(|| "answer witnesses missing".to_string())?;
    if probes.len() != witnesses.len() {
        return Err(format!(
            "{expected_function} count mismatch: {} vs {}",
            probes.len(),
            witnesses.len()
        ));
    }
    let mut out = BTreeMap::new();
    for (index, (probe, witness)) in probes.iter().zip(witnesses).enumerate() {
        let probe = &probe["probe"];
        let id = probe["id"]
            .as_str()
            .filter(|id| !id.is_empty())
            .ok_or_else(|| format!("empty probe id at {index}"))?;
        if witness["id"].as_str() != Some(id) || probe["args"] != witness["args"] {
            return Err(format!(
                "ID/argument drift at {expected_function} row {index}"
            ));
        }
        let expected_bits = parse_hex_bits(
            witness["expected_bits"]
                .as_str()
                .ok_or_else(|| format!("nonnumeric {expected_function} result at {index}"))?,
        )?;
        if out.insert(id.to_string(), expected_bits).is_some() {
            return Err(format!("duplicate {expected_function} id {id}"));
        }
    }
    Ok(out)
}

fn score_discovery(directory: &Path) -> Result<(), String> {
    let ftest_batch: Value = serde_json::from_str(
        &fs::read_to_string(directory.join("batch-ftest-variance-discovery-v2.json"))
            .map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let ftest = load_aligned_answers(
        &directory.join("batch-ftest-variance-discovery-v2.json"),
        &directory.join("answers-ftest-variance-discovery-v2.json"),
        "F.TEST",
    )?;
    let fdist = load_aligned_answers(
        &directory.join("batch-fdist-variance-discovery-v2.json"),
        &directory.join("answers-fdist-variance-discovery-v2.json"),
        "FDIST",
    )?;
    let vars = load_aligned_answers(
        &directory.join("batch-vars-variance-discovery-v2.json"),
        &directory.join("answers-vars-variance-discovery-v2.json"),
        "VAR.S",
    )?;
    let meta: Value = serde_json::from_str(
        &fs::read_to_string(directory.join("meta-ftest-variance-discovery-v2.json"))
            .map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let rows = meta["rows"]
        .as_array()
        .ok_or_else(|| "metadata rows missing".to_string())?;
    if rows.len() != 48 || ftest.len() != 48 || vars.len() != 96 {
        return Err("unexpected discovery counts".to_string());
    }

    let mut scores: BTreeMap<String, usize> = BTreeMap::new();
    let mut accepted_group_histogram: BTreeMap<usize, usize> = BTreeMap::new();
    let mut public_vars_in_grammar = 0_usize;
    let mut public_vars_selected = 0_usize;
    for (index, row) in rows.iter().enumerate() {
        let ftest_id = row["ftest_id"]
            .as_str()
            .ok_or_else(|| format!("missing F.TEST id at {index}"))?;
        let ftest_bits = *ftest
            .get(ftest_id)
            .ok_or_else(|| format!("missing F.TEST answer {ftest_id}"))?;
        let groups = row["candidate_groups"]
            .as_array()
            .ok_or_else(|| format!("missing groups at {index}"))?;
        let mut accepted_keys = BTreeSet::new();
        for group in groups {
            let fdist_id = group["fdist_id"]
                .as_str()
                .ok_or_else(|| "missing FDIST id".to_string())?;
            let tail_bits = *fdist
                .get(fdist_id)
                .ok_or_else(|| format!("missing FDIST answer {fdist_id}"))?;
            let tail = f64::from_bits(tail_bits);
            let two_tail_bits = if tail <= 0.5 {
                (tail * 2.0).to_bits()
            } else {
                ((1.0 - tail) * 2.0).to_bits()
            };
            if two_tail_bits == ftest_bits {
                let key = (
                    parse_hex_bits(group["ratio_bits"].as_str().unwrap())?,
                    parse_hex_bits(group["df_hi_bits"].as_str().unwrap())?,
                    parse_hex_bits(group["df_lo_bits"].as_str().unwrap())?,
                );
                accepted_keys.insert(key);
                for model in group["models"].as_array().unwrap() {
                    *scores
                        .entry(model.as_str().unwrap().to_string())
                        .or_default() += 1;
                }
            }
        }
        *accepted_group_histogram
            .entry(accepted_keys.len())
            .or_default() += 1;

        let var_a = f64::from_bits(
            *vars
                .get(&format!("vars-{index:03}-a"))
                .ok_or_else(|| format!("missing VAR.S a row {index}"))?,
        );
        let var_b = f64::from_bits(
            *vars
                .get(&format!("vars-{index:03}-b"))
                .ok_or_else(|| format!("missing VAR.S b row {index}"))?,
        );
        let sample_args = &ftest_batch["probes"][index]["probe"]["args"];
        let len_a = sample_args[0]
            .as_array()
            .ok_or_else(|| format!("missing F.TEST sample a at {index}"))?
            .len();
        let len_b = sample_args[1]
            .as_array()
            .ok_or_else(|| format!("missing F.TEST sample b at {index}"))?
            .len();
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
        let public_group_exists = groups.iter().any(|group| {
            let key = (
                parse_hex_bits(group["ratio_bits"].as_str().unwrap()).unwrap(),
                parse_hex_bits(group["df_hi_bits"].as_str().unwrap()).unwrap(),
                parse_hex_bits(group["df_lo_bits"].as_str().unwrap()).unwrap(),
            );
            key == public_key
        });
        if public_group_exists {
            public_vars_in_grammar += 1;
            if accepted_keys.contains(&public_key) {
                public_vars_selected += 1;
            }
        }
        if accepted_keys.is_empty() {
            let forward = groups
                .iter()
                .find(|group| {
                    group["models"].as_array().unwrap().iter().any(|model| {
                        model.as_str()
                            == Some(
                                "two-pass mean=Native body=Native rev=false corr=false ratio=Native",
                            )
                    })
                })
                .ok_or_else(|| format!("forward model missing at {index}"))?;
            println!(
                "NO-HIT {ftest_id} family={} ftest=0x{ftest_bits:016x} forward={} public=0x{:016x}",
                row["family"].as_str().unwrap(),
                forward["ratio_bits"].as_str().unwrap(),
                public_key.0
            );
        }
    }

    let mut ranked: Vec<(usize, String)> = scores
        .into_iter()
        .map(|(name, exact)| (exact, name))
        .collect();
    ranked.sort_by(|left, right| right.0.cmp(&left.0).then(left.1.cmp(&right.1)));
    println!("accepted-group histogram: {accepted_group_histogram:?}");
    println!(
        "published VAR.S ratio is in frozen grammar on {public_vars_in_grammar}/48 rows and selected on {public_vars_selected}/48"
    );
    for (exact, name) in ranked.into_iter().take(40) {
        println!("{exact}/48 {name}");
    }
    Ok(())
}

fn symmetric_unit_sample(len: usize, scale: f64) -> Vec<f64> {
    assert!(len >= 3 && len % 2 == 1);
    let half = (len - 1) / 2;
    let mut values = vec![-scale; half];
    values.push(0.0);
    values.extend(std::iter::repeat_n(scale, half));
    values
}

fn binomial(n: u32, k: u32) -> u128 {
    let k = k.min(n - k);
    (1..=k).fold(1_u128, |value, index| {
        value * u128::from(n - k + index) / u128::from(index)
    })
}

/// Exact regularized-beta value for integer shape parameters and rational x.
///
/// The equal-variance wrapper battery uses even degrees of freedom <= 10, so
/// this binomial-CDF identity stays far below `u128` and both final integers
/// are exactly representable as binary64 before the one rounded division.
fn integer_beta_rational(x_num: u32, x_den: u32, a: u32, b: u32) -> (u128, u128) {
    let n = a + b - 1;
    let numerator = (a..=n).fold(0_u128, |value, index| {
        value
            + binomial(n, index)
                * u128::from(x_num).pow(index)
                * u128::from(x_den - x_num).pow(n - index)
    });
    (numerator, u128::from(x_den).pow(n))
}

fn exact_equal_variance_two_tail(df1: u32, df2: u32) -> f64 {
    assert!(df1.is_multiple_of(2) && df2.is_multiple_of(2));
    let (tail_num, denominator) = integer_beta_rational(df2, df1 + df2, df2 / 2, df1 / 2);
    let smaller = tail_num.min(denominator - tail_num);
    (2 * smaller) as f64 / denominator as f64
}

fn generate_wrapper_heldout(directory: &Path) -> Result<(), String> {
    fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    let mut cases: Vec<(usize, usize, f64, f64, String)> = Vec::new();
    for (len_a, len_b) in [
        (3, 5),
        (3, 7),
        (3, 9),
        (3, 11),
        (5, 7),
        (5, 9),
        (5, 11),
        (7, 9),
        (7, 11),
        (9, 11),
    ] {
        cases.push((len_a, len_b, 1.0, 1.0, "equal-forward".to_string()));
        cases.push((len_b, len_a, 1.0, 1.0, "equal-reverse".to_string()));
    }
    cases.extend([
        (3, 11, 2.0, 1.0, "ratio-four-a".to_string()),
        (11, 3, 1.0, 2.0, "ratio-four-b".to_string()),
        (5, 9, 2.0, 1.0, "ratio-four-a".to_string()),
        (9, 5, 1.0, 2.0, "ratio-four-b".to_string()),
    ]);
    if cases.len() != 24 {
        return Err("wrapper heldout case count drift".to_string());
    }

    let discovery: Value = serde_json::from_str(
        &fs::read_to_string(directory.join("batch-ftest-variance-discovery-v2.json"))
            .map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let discovery_args: BTreeSet<String> = discovery["probes"]
        .as_array()
        .ok_or_else(|| "discovery probes missing".to_string())?
        .iter()
        .map(|probe| serde_json::to_string(&probe["probe"]["args"]).unwrap())
        .collect();

    let mut ftest_probes = Vec::new();
    let mut fdist_probes = Vec::new();
    let mut meta_rows = Vec::new();
    let mut heldout_args = BTreeSet::new();
    for (index, (len_a, len_b, scale_a, scale_b, family)) in cases.into_iter().enumerate() {
        let a = symmetric_unit_sample(len_a, scale_a);
        let b = symmetric_unit_sample(len_b, scale_b);
        let args = json!([
            a.iter().map(|value| bits(*value)).collect::<Vec<_>>(),
            b.iter().map(|value| bits(*value)).collect::<Vec<_>>()
        ]);
        let key = serde_json::to_string(&args).unwrap();
        if discovery_args.contains(&key) || !heldout_args.insert(key) {
            return Err(format!("wrapper heldout overlap at row {index}"));
        }
        let var_a = scale_a * scale_a;
        let var_b = scale_b * scale_b;
        let (ratio, df_hi, df_lo) = if var_a >= var_b {
            (var_a / var_b, (len_a - 1) as f64, (len_b - 1) as f64)
        } else {
            (var_b / var_a, (len_b - 1) as f64, (len_a - 1) as f64)
        };
        let ftest_id = format!("ft-wrapper-ho-{index:03}-{family}");
        let fdist_id = format!("fd-wrapper-ho-{index:03}-{family}");
        ftest_probes.push(json!({
            "probe": {"id": ftest_id, "args": args},
            "distinct_outputs": 2,
            "outputs": []
        }));
        fdist_probes.push(json!({
            "probe": {
                "id": fdist_id,
                "args": [bits(ratio), bits(df_hi), bits(df_lo)]
            },
            "distinct_outputs": 0,
            "outputs": []
        }));
        meta_rows.push(json!({
            "ftest_id": ftest_id,
            "fdist_id": fdist_id,
            "family": family,
            "ratio_bits": bits(ratio),
            "df_hi_bits": bits(df_hi),
            "df_lo_bits": bits(df_lo)
        }));
    }

    write_pretty(
        &directory.join("batch-ftest-wrapper-heldout-v1.json"),
        &json!({
            "function": "F.TEST",
            "row_id": "G3-06-ftest-two-tail-wrapper-heldout-v1-20260809",
            "probes": ftest_probes
        }),
    )?;
    write_pretty(
        &directory.join("batch-fdist-wrapper-heldout-v1.json"),
        &json!({
            "function": "FDIST",
            "row_id": "G3-06-ftest-two-tail-wrapper-fdist-heldout-v1-20260809",
            "probes": fdist_probes
        }),
    )?;
    write_pretty(
        &directory.join("meta-ftest-wrapper-heldout-v1.json"),
        &json!({
            "freeze_id": "w109-g3-06-ftest-two-tail-wrapper-heldout-v1-20260809",
            "selection": "answer-blind exact-variance wrapper discriminator",
            "frozen_candidate": "tail <= 0.5 ? 2*tail : 2*(1-tail)",
            "declared_controls": ["min(1,2*tail)", "tail<=0.5 ? 2*tail : 2-2*tail"],
            "rows": meta_rows
        }),
    )?;
    println!("generated 24 F.TEST and 24 FDIST wrapper-heldout probes");
    Ok(())
}

fn score_wrapper_heldout(directory: &Path) -> Result<(), String> {
    let ftest = load_aligned_answers(
        &directory.join("batch-ftest-wrapper-heldout-v1.json"),
        &directory.join("answers-ftest-wrapper-heldout-v1.json"),
        "F.TEST",
    )?;
    let fdist = load_aligned_answers(
        &directory.join("batch-fdist-wrapper-heldout-v1.json"),
        &directory.join("answers-fdist-wrapper-heldout-v1.json"),
        "FDIST",
    )?;
    let meta: Value = serde_json::from_str(
        &fs::read_to_string(directory.join("meta-ftest-wrapper-heldout-v1.json"))
            .map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let rows = meta["rows"]
        .as_array()
        .ok_or_else(|| "wrapper metadata rows missing".to_string())?;
    let mut current = 0_usize;
    let mut frozen = 0_usize;
    let mut subtract_after = 0_usize;
    let mut complement_rows = 0_usize;
    let mut exact_rational = 0_usize;
    let mut exact_rational_rows = 0_usize;
    for (index, row) in rows.iter().enumerate() {
        let target = *ftest
            .get(row["ftest_id"].as_str().unwrap())
            .ok_or_else(|| "missing wrapper F.TEST answer".to_string())?;
        let tail = f64::from_bits(
            *fdist
                .get(row["fdist_id"].as_str().unwrap())
                .ok_or_else(|| "missing wrapper FDIST answer".to_string())?,
        );
        let current_bits = (tail * 2.0).min(1.0).to_bits();
        let frozen_bits = if tail <= 0.5 {
            (tail * 2.0).to_bits()
        } else {
            complement_rows += 1;
            ((1.0 - tail) * 2.0).to_bits()
        };
        let subtract_after_bits = if tail <= 0.5 {
            (tail * 2.0).to_bits()
        } else {
            (2.0 - 2.0 * tail).to_bits()
        };
        current += usize::from(current_bits == target);
        frozen += usize::from(frozen_bits == target);
        subtract_after += usize::from(subtract_after_bits == target);
        if row["family"]
            .as_str()
            .is_some_and(|family| family.starts_with("equal-"))
        {
            let df1 = f64::from_bits(parse_hex_bits(row["df_hi_bits"].as_str().unwrap())?) as u32;
            let df2 = f64::from_bits(parse_hex_bits(row["df_lo_bits"].as_str().unwrap())?) as u32;
            let mathematical_bits = exact_equal_variance_two_tail(df1, df2).to_bits();
            exact_rational_rows += 1;
            exact_rational += usize::from(mathematical_bits == target);
            println!(
                "equal row={index:02} df=({df1},{df2}) target=0x{target:016x} external=0x{frozen_bits:016x} rational=0x{mathematical_bits:016x} delta_external={} delta_rational={}",
                (target as i128) - (frozen_bits as i128),
                (target as i128) - (mathematical_bits as i128),
            );
        }
    }
    println!(
        "wrapper heldout rows={} complement_rows={complement_rows}",
        rows.len()
    );
    println!("current min(1,2*tail): {current}/{}", rows.len());
    println!("frozen 2*(1-tail): {frozen}/{}", rows.len());
    println!("control 2-2*tail: {subtract_after}/{}", rows.len());
    println!("exact rational equal-variance tail: {exact_rational}/{exact_rational_rows}");
    Ok(())
}

fn generate_cdf_companion(directory: &Path) -> Result<(), String> {
    fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    let sources = [
        ("discovery", "batch-fdist-variance-discovery-v2.json"),
        ("retired_wrapper", "batch-fdist-wrapper-heldout-v1.json"),
    ];
    let mut probes = Vec::new();
    let mut rows = Vec::new();
    let mut seen_ids = BTreeSet::new();
    let mut ids_by_args: BTreeMap<String, String> = BTreeMap::new();
    let mut source_rows = 0_usize;
    for (source, file_name) in sources {
        let batch: Value = serde_json::from_str(
            &fs::read_to_string(directory.join(file_name)).map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
        if batch["function"].as_str() != Some("FDIST") {
            return Err(format!("unexpected source function in {file_name}"));
        }
        for (index, item) in batch["probes"]
            .as_array()
            .ok_or_else(|| format!("missing probes in {file_name}"))?
            .iter()
            .enumerate()
        {
            let source_probe = &item["probe"];
            let source_id = source_probe["id"]
                .as_str()
                .ok_or_else(|| format!("missing source id in {file_name} row {index}"))?;
            let source_args = source_probe["args"]
                .as_array()
                .ok_or_else(|| format!("missing source args in {file_name} row {index}"))?;
            if source_args.len() != 3 {
                return Err(format!("source arity drift in {file_name} row {index}"));
            }
            let args = json!([
                source_args[0].clone(),
                source_args[1].clone(),
                source_args[2].clone(),
                true
            ]);
            let args_key = serde_json::to_string(&args).unwrap();
            let id = if let Some(id) = ids_by_args.get(&args_key) {
                id.clone()
            } else {
                let id = format!("fcdf-{source}-{index:03}");
                if !seen_ids.insert(id.clone()) {
                    return Err(format!("duplicate companion id {id}"));
                }
                probes.push(json!({
                    "probe": {"id": id, "args": args},
                    "distinct_outputs": 0,
                    "outputs": []
                }));
                ids_by_args.insert(args_key, id.clone());
                id
            };
            rows.push(json!({
                "cdf_id": id,
                "source": source,
                "source_id": source_id,
                "source_file": file_name
            }));
            source_rows += 1;
        }
    }
    if source_rows != 374 || rows.len() != source_rows {
        return Err(format!(
            "companion source-count drift: source_rows={source_rows} mappings={}",
            rows.len()
        ));
    }
    write_pretty(
        &directory.join("batch-fdist-cdf-companion-discovery-v1.json"),
        &json!({
            "function": "F.DIST",
            "row_id": "G3-06-ftest-public-cdf-companion-discovery-v1-20260809",
            "probes": probes
        }),
    )?;
    write_pretty(
        &directory.join("meta-fdist-cdf-companion-discovery-v1.json"),
        &json!({
            "freeze_id": "w109-g3-06-ftest-public-cdf-companion-discovery-v1-20260809",
            "selection": "answer-blind transform of every previously captured FDIST ratio into F.DIST cumulative=true",
            "purpose": "separate direct-CDF publication from 1-right-tail complement staging",
            "rows": rows
        }),
    )?;
    println!(
        "generated {} unique F.DIST cumulative companions for {source_rows} source rows",
        probes.len()
    );
    Ok(())
}

fn load_cdf_source_map(directory: &Path) -> Result<BTreeMap<String, String>, String> {
    let meta: Value = serde_json::from_str(
        &fs::read_to_string(directory.join("meta-fdist-cdf-companion-discovery-v1.json"))
            .map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let mut out = BTreeMap::new();
    for row in meta["rows"]
        .as_array()
        .ok_or_else(|| "CDF companion metadata rows missing".to_string())?
    {
        let source_id = row["source_id"]
            .as_str()
            .ok_or_else(|| "CDF companion source id missing".to_string())?;
        let cdf_id = row["cdf_id"]
            .as_str()
            .ok_or_else(|| "CDF companion id missing".to_string())?;
        if out
            .insert(source_id.to_string(), cdf_id.to_string())
            .is_some()
        {
            return Err(format!("duplicate CDF source mapping {source_id}"));
        }
    }
    if out.len() != 374 {
        return Err(format!("CDF source mapping count drift: {}", out.len()));
    }
    Ok(out)
}

fn score_cdf_companion(directory: &Path) -> Result<(), String> {
    let cdf = load_aligned_answers(
        &directory.join("batch-fdist-cdf-companion-discovery-v1.json"),
        &directory.join("answers-fdist-cdf-companion-discovery-v1.json"),
        "F.DIST",
    )?;
    let cdf_by_source = load_cdf_source_map(directory)?;

    let ftest = load_aligned_answers(
        &directory.join("batch-ftest-variance-discovery-v2.json"),
        &directory.join("answers-ftest-variance-discovery-v2.json"),
        "F.TEST",
    )?;
    let rt = load_aligned_answers(
        &directory.join("batch-fdist-variance-discovery-v2.json"),
        &directory.join("answers-fdist-variance-discovery-v2.json"),
        "FDIST",
    )?;
    let discovery_meta: Value = serde_json::from_str(
        &fs::read_to_string(directory.join("meta-ftest-variance-discovery-v2.json"))
            .map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let mut scores: BTreeMap<String, usize> = BTreeMap::new();
    let mut histogram: BTreeMap<usize, usize> = BTreeMap::new();
    for (index, row) in discovery_meta["rows"]
        .as_array()
        .ok_or_else(|| "discovery metadata rows missing".to_string())?
        .iter()
        .enumerate()
    {
        let target = *ftest
            .get(row["ftest_id"].as_str().unwrap())
            .ok_or_else(|| format!("missing F.TEST row {index}"))?;
        let mut accepted = BTreeSet::new();
        for group in row["candidate_groups"].as_array().unwrap() {
            let rt_id = group["fdist_id"].as_str().unwrap();
            let rt_value = f64::from_bits(*rt.get(rt_id).unwrap());
            let cdf_id = cdf_by_source
                .get(rt_id)
                .ok_or_else(|| format!("missing CDF mapping for {rt_id}"))?;
            let cdf_value = f64::from_bits(*cdf.get(cdf_id).unwrap());
            if (2.0 * rt_value.min(cdf_value)).to_bits() == target {
                let key = (
                    parse_hex_bits(group["ratio_bits"].as_str().unwrap())?,
                    parse_hex_bits(group["df_hi_bits"].as_str().unwrap())?,
                    parse_hex_bits(group["df_lo_bits"].as_str().unwrap())?,
                );
                accepted.insert(key);
                for model in group["models"].as_array().unwrap() {
                    *scores
                        .entry(model.as_str().unwrap().to_string())
                        .or_default() += 1;
                }
            }
        }
        *histogram.entry(accepted.len()).or_default() += 1;
        if accepted.is_empty() {
            println!(
                "CDF NO-HIT row={index:02} id={}",
                row["ftest_id"].as_str().unwrap()
            );
        }
    }
    let mut ranked: Vec<(usize, String)> = scores
        .into_iter()
        .map(|(name, exact)| (exact, name))
        .collect();
    ranked.sort_by(|left, right| right.0.cmp(&left.0).then(left.1.cmp(&right.1)));
    println!("CDF discovery accepted-group histogram: {histogram:?}");
    for (exact, name) in ranked.into_iter().take(16) {
        println!("CDF {exact}/48 {name}");
    }

    let wrapper_ftest = load_aligned_answers(
        &directory.join("batch-ftest-wrapper-heldout-v1.json"),
        &directory.join("answers-ftest-wrapper-heldout-v1.json"),
        "F.TEST",
    )?;
    let wrapper_rt = load_aligned_answers(
        &directory.join("batch-fdist-wrapper-heldout-v1.json"),
        &directory.join("answers-fdist-wrapper-heldout-v1.json"),
        "FDIST",
    )?;
    let wrapper_meta: Value = serde_json::from_str(
        &fs::read_to_string(directory.join("meta-ftest-wrapper-heldout-v1.json"))
            .map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let mut wrapper_exact = 0_usize;
    for (index, row) in wrapper_meta["rows"].as_array().unwrap().iter().enumerate() {
        let target = *wrapper_ftest
            .get(row["ftest_id"].as_str().unwrap())
            .unwrap();
        let rt_id = row["fdist_id"].as_str().unwrap();
        let rt_value = f64::from_bits(*wrapper_rt.get(rt_id).unwrap());
        let cdf_id = cdf_by_source.get(rt_id).unwrap();
        let cdf_value = f64::from_bits(*cdf.get(cdf_id).unwrap());
        let predicted = (2.0 * rt_value.min(cdf_value)).to_bits();
        wrapper_exact += usize::from(predicted == target);
        if predicted != target {
            println!(
                "CDF WRAPPER MISS row={index:02} target=0x{target:016x} rt={} cdf={} predicted=0x{predicted:016x}",
                bits(rt_value),
                bits(cdf_value)
            );
        }
    }
    println!("CDF direct two-sided wrapper: {wrapper_exact}/24");
    Ok(())
}

fn generate_discovery(directory: &Path) -> Result<(), String> {
    fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    let family_names = [
        "moderate",
        "translate_2p20",
        "translate_2p40",
        "translate_2p52",
    ];
    let mut selected: Vec<(usize, u64, Vec<f64>, Vec<f64>, Vec<RoutePrediction>)> = Vec::new();
    let mut seen_args = BTreeSet::new();

    for family in 0..family_names.len() {
        for seed in 0_u64..100_000 {
            if selected.iter().filter(|row| row.0 == family).count() >= 12 {
                break;
            }
            let len_a = 3 + (seed as usize % 10);
            let len_b = 3 + ((seed.rotate_left(17) as usize + 5) % 10);
            let a = generated_values(seed, len_a, family, 0);
            let b = generated_values(seed ^ 0xa5a5_5a5a_d3c1_b2e7, len_b, family, 1);
            let candidates = predictions(&a, &b);
            let groups: BTreeSet<(u64, u64, u64)> = candidates
                .iter()
                .filter(|candidate| {
                    candidate.var_a.is_finite()
                        && candidate.var_b.is_finite()
                        && candidate.var_a > 0.0
                        && candidate.var_b > 0.0
                        && candidate.ratio.is_finite()
                        && (1.0..=64.0).contains(&candidate.ratio)
                })
                .map(|candidate| {
                    (
                        candidate.ratio.to_bits(),
                        candidate.df_hi.to_bits(),
                        candidate.df_lo.to_bits(),
                    )
                })
                .collect();
            if groups.len() < 4
                || candidates
                    .iter()
                    .any(|candidate| !candidate.ratio.is_finite())
            {
                continue;
            }
            let key = format!(
                "{}|{}",
                a.iter()
                    .map(|value| bits(*value))
                    .collect::<Vec<_>>()
                    .join(","),
                b.iter()
                    .map(|value| bits(*value))
                    .collect::<Vec<_>>()
                    .join(",")
            );
            if seen_args.insert(key) {
                selected.push((family, seed, a, b, candidates));
            }
        }
    }
    if selected.len() != 48 {
        return Err(format!("selected {} rows, expected 48", selected.len()));
    }

    let mut ftest_probes = Vec::new();
    let mut fdist_probes = Vec::new();
    let mut vars_probes = Vec::new();
    let mut meta_rows = Vec::new();
    for (index, (family, seed, a, b, candidates)) in selected.iter().enumerate() {
        let id = format!("ft-var-v2-{index:03}-{}", family_names[*family]);
        let args_a: Vec<String> = a.iter().map(|value| bits(*value)).collect();
        let args_b: Vec<String> = b.iter().map(|value| bits(*value)).collect();
        let mut grouped: BTreeMap<(u64, u64, u64), Vec<String>> = BTreeMap::new();
        for candidate in candidates {
            if candidate.var_a > 0.0
                && candidate.var_b > 0.0
                && candidate.ratio.is_finite()
                && (1.0..=64.0).contains(&candidate.ratio)
            {
                grouped
                    .entry((
                        candidate.ratio.to_bits(),
                        candidate.df_hi.to_bits(),
                        candidate.df_lo.to_bits(),
                    ))
                    .or_default()
                    .push(candidate.name.clone());
            }
        }
        ftest_probes.push(json!({
            "probe": {"id": id, "args": [args_a, args_b]},
            "distinct_outputs": grouped.len(),
            "outputs": []
        }));
        vars_probes.push(json!({
            "probe": {"id": format!("vars-{index:03}-a"), "args": [a.iter().map(|value| bits(*value)).collect::<Vec<_>>() ]},
            "distinct_outputs": 0,
            "outputs": []
        }));
        vars_probes.push(json!({
            "probe": {"id": format!("vars-{index:03}-b"), "args": [b.iter().map(|value| bits(*value)).collect::<Vec<_>>() ]},
            "distinct_outputs": 0,
            "outputs": []
        }));

        let mut group_meta = Vec::new();
        for (candidate_index, ((ratio_bits, df_hi_bits, df_lo_bits), models)) in
            grouped.into_iter().enumerate()
        {
            let candidate_id = format!("fd-var-v2-{index:03}-{candidate_index:03}");
            fdist_probes.push(json!({
                "probe": {
                    "id": candidate_id,
                    "args": [
                        format!("0x{ratio_bits:016x}"),
                        format!("0x{df_hi_bits:016x}"),
                        format!("0x{df_lo_bits:016x}")
                    ]
                },
                "distinct_outputs": 0,
                "outputs": []
            }));
            group_meta.push(json!({
                "fdist_id": candidate_id,
                "ratio_bits": format!("0x{ratio_bits:016x}"),
                "df_hi_bits": format!("0x{df_hi_bits:016x}"),
                "df_lo_bits": format!("0x{df_lo_bits:016x}"),
                "models": models
            }));
        }
        meta_rows.push(json!({
            "ftest_id": id,
            "family": family_names[*family],
            "seed": seed,
            "candidate_groups": group_meta
        }));
    }

    write_pretty(
        &directory.join("batch-ftest-variance-discovery-v2.json"),
        &json!({
            "function": "F.TEST",
            "row_id": "G3-06-ftest-variance-discovery-v2-20260809",
            "probes": ftest_probes
        }),
    )?;
    write_pretty(
        &directory.join("batch-fdist-variance-discovery-v2.json"),
        &json!({
            "function": "FDIST",
            "row_id": "G3-06-ftest-variance-fdist-discovery-v2-20260809",
            "probes": fdist_probes
        }),
    )?;
    write_pretty(
        &directory.join("batch-vars-variance-discovery-v2.json"),
        &json!({
            "function": "VAR.S",
            "row_id": "G3-06-ftest-variance-vars-discovery-v2-20260809",
            "probes": vars_probes
        }),
    )?;
    write_pretty(
        &directory.join("meta-ftest-variance-discovery-v2.json"),
        &json!({
            "freeze_id": "w109-g3-06-ftest-variance-discovery-v2-20260809",
            "selection": "answer-blind deterministic variance-schedule disagreements",
            "families": family_names,
            "ftest_count": 48,
            "vars_count": 96,
            "fdist_count": meta_rows.iter().map(|row| row["candidate_groups"].as_array().unwrap().len()).sum::<usize>(),
            "rows": meta_rows
        }),
    )?;
    println!(
        "generated 48 F.TEST, {} FDIST, and 96 VAR.S discovery probes in {}",
        meta_rows
            .iter()
            .map(|row| row["candidate_groups"].as_array().unwrap().len())
            .sum::<usize>(),
        directory.display()
    );
    Ok(())
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--generate") {
        let directory = args
            .get(2)
            .ok_or_else(|| "usage: race_ftest_variance --generate <directory>".to_string())?;
        return generate_discovery(Path::new(directory));
    }
    if args.get(1).map(String::as_str) == Some("--score") {
        let directory = args
            .get(2)
            .ok_or_else(|| "usage: race_ftest_variance --score <directory>".to_string())?;
        return score_discovery(Path::new(directory));
    }
    if args.get(1).map(String::as_str) == Some("--generate-wrapper-heldout") {
        let directory = args.get(2).ok_or_else(|| {
            "usage: race_ftest_variance --generate-wrapper-heldout <directory>".to_string()
        })?;
        return generate_wrapper_heldout(Path::new(directory));
    }
    if args.get(1).map(String::as_str) == Some("--score-wrapper-heldout") {
        let directory = args.get(2).ok_or_else(|| {
            "usage: race_ftest_variance --score-wrapper-heldout <directory>".to_string()
        })?;
        return score_wrapper_heldout(Path::new(directory));
    }
    if args.get(1).map(String::as_str) == Some("--generate-cdf-companion") {
        let directory = args.get(2).ok_or_else(|| {
            "usage: race_ftest_variance --generate-cdf-companion <directory>".to_string()
        })?;
        return generate_cdf_companion(Path::new(directory));
    }
    if args.get(1).map(String::as_str) == Some("--score-cdf-companion") {
        let directory = args.get(2).ok_or_else(|| {
            "usage: race_ftest_variance --score-cdf-companion <directory>".to_string()
        })?;
        return score_cdf_companion(Path::new(directory));
    }
    let rows: Vec<(Vec<f64>, Vec<f64>, Vec<u64>)> = vec![
        (
            vec![
                f(0x4018000000000000),
                f(0x401c000000000000),
                f(0x4022000000000000),
                f(0x402e000000000000),
                f(0x4035000000000000),
            ],
            vec![
                f(0x4034000000000000),
                f(0x403c000000000000),
                f(0x403f000000000000),
                f(0x4043000000000000),
                f(0x4044000000000000),
            ],
            vec![0x3ffa0cdd442e4fc3, 0x3ffa0cdd442e4fc4],
        ),
        (
            vec![
                f(0x4008cccccccccccd),
                f(0x4010cccccccccccd),
                f(0x401799999999999a),
                f(0x4004cccccccccccd),
                f(0x4020cccccccccccd),
                f(0x401c666666666666),
            ],
            vec![
                f(0x3ff8000000000000),
                f(0x4023cccccccccccd),
                f(0x401199999999999a),
                f(0x401a666666666666),
                f(0x400a666666666666),
                f(0x4016000000000000),
                f(0x400199999999999a),
            ],
            vec![0x3ff90bbcff177cef],
        ),
        (
            vec![
                f(0x4059000000000000),
                f(0x4059400000000000),
                f(0x4059800000000000),
            ],
            vec![
                f(0x4049000000000000),
                f(0x404e000000000000),
                f(0x4051800000000000),
                f(0x4054000000000000),
            ],
            vec![0x4064d55555555555, 0x4064d55555555556],
        ),
    ];

    let mut candidates: Vec<(String, Vec<f64>)> = Vec::new();
    for mean_op in [Op::Native, Op::X87] {
        for body_op in [Op::Native, Op::X87] {
            for reverse in [false, true] {
                for correction in [false, true] {
                    candidates.push((
                        format!("two-pass mean={mean_op:?} body={body_op:?} rev={reverse} corr={correction}"),
                        rows.iter().flat_map(|(a,b,_)| [two_pass(a,mean_op,body_op,reverse,correction), two_pass(b,mean_op,body_op,reverse,correction)]).collect(),
                    ));
                }
            }
        }
    }
    for op in [Op::Native, Op::X87] {
        for reverse in [false, true] {
            for anchor_last in [false, true] {
                for correction in [false, true] {
                    candidates.push((
                        format!(
                            "shifted op={op:?} rev={reverse} last={anchor_last} corr={correction}"
                        ),
                        rows.iter()
                            .flat_map(|(a, b, _)| {
                                [
                                    shifted(a, op, reverse, anchor_last, correction),
                                    shifted(b, op, reverse, anchor_last, correction),
                                ]
                            })
                            .collect(),
                    ));
                }
            }
            candidates.push((
                format!("one-pass op={op:?} rev={reverse}"),
                rows.iter()
                    .flat_map(|(a, b, _)| [one_pass(a, op, reverse), one_pass(b, op, reverse)])
                    .collect(),
            ));
            candidates.push((
                format!("welford op={op:?} rev={reverse}"),
                rows.iter()
                    .flat_map(|(a, b, _)| [welford(a, op, reverse), welford(b, op, reverse)])
                    .collect(),
            ));
        }
    }
    for reverse in [false, true] {
        for correction in [false, true] {
            candidates.push((
                format!("ext-two-pass rev={reverse} corr={correction}"),
                rows.iter()
                    .flat_map(|(a, b, _)| {
                        [
                            ext_two_pass(a, reverse, correction),
                            ext_two_pass(b, reverse, correction),
                        ]
                    })
                    .collect(),
            ));
        }
    }

    let mut scored = Vec::new();
    for (name, vars) in candidates {
        for ratio_op in [Op::Native, Op::X87] {
            let bits: Vec<u64> = (0..3)
                .map(|i| ratio(vars[2 * i], vars[2 * i + 1], ratio_op).to_bits())
                .collect();
            let exact = bits
                .iter()
                .zip(&rows)
                .filter(|(got, (_, _, want))| want.contains(got))
                .count();
            scored.push((exact, format!("{name} ratio={ratio_op:?}"), bits));
        }
    }
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    for (exact, name, bits) in scored.into_iter().take(40) {
        println!(
            "{exact}/3 {name}: {:016x} {:016x} {:016x}",
            bits[0], bits[1], bits[2]
        );
    }
    Ok(())
}
