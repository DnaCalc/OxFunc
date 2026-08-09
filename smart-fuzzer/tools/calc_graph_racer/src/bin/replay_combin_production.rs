//! Offline replay of captured COMBIN witnesses through the production kernel.

use oxfunc_core::functions::combin::combin_kernel;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

fn parse_bits(text: &str) -> Result<u64, String> {
    let hex = text
        .strip_prefix("0x")
        .ok_or_else(|| format!("bit string lacks 0x prefix: {text}"))?;
    if hex.len() != 16 {
        return Err(format!("bit string is not 16 hex digits: {text}"));
    }
    u64::from_str_radix(hex, 16).map_err(|error| format!("invalid bit string {text}: {error}"))
}

fn replay(path: &Path) -> Result<(usize, usize), String> {
    let text = fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let document: Value = serde_json::from_str(&text).map_err(|error| error.to_string())?;
    if document["function"].as_str() != Some("COMBIN") {
        return Err(format!("{} is not a COMBIN witness set", path.display()));
    }
    let witnesses = document["witnesses"]
        .as_array()
        .ok_or_else(|| format!("{} has no witnesses array", path.display()))?;
    if witnesses.is_empty() {
        return Err(format!("{} has an empty witnesses array", path.display()));
    }
    let mut exact = 0_usize;
    let mut histogram: BTreeMap<i64, usize> = BTreeMap::new();
    let mut ids = BTreeSet::new();
    let mut first_mismatches = Vec::new();
    for witness in witnesses {
        let witness = witness
            .as_object()
            .ok_or_else(|| format!("{} contains a non-object witness", path.display()))?;
        let id = witness
            .get("id")
            .and_then(Value::as_str)
            .filter(|id| !id.is_empty())
            .ok_or_else(|| format!("{} contains a missing/empty witness id", path.display()))?;
        if !ids.insert(id.to_owned()) {
            return Err(format!("{} contains duplicate id {id}", path.display()));
        }
        if witness.contains_key("expected_error") {
            return Err(format!("{id}: expected numeric bits, found expected_error"));
        }
        let args = witness["args"]
            .as_array()
            .ok_or_else(|| format!("{id}: missing args"))?;
        if args.len() != 2 {
            return Err(format!("{id}: expected two args"));
        }
        let n = f64::from_bits(parse_bits(args[0].as_str().ok_or("n arg is not text")?)?);
        let k = f64::from_bits(parse_bits(args[1].as_str().ok_or("k arg is not text")?)?);
        let truncated_n = n.trunc();
        let truncated_k = k.trunc();
        if !n.is_finite()
            || !k.is_finite()
            || truncated_n < 0.0
            || truncated_k < 0.0
            || truncated_k > truncated_n
        {
            return Err(format!(
                "{id}: args are outside the captured finite COMBIN domain: n={n}, k={k}"
            ));
        }
        let want = parse_bits(
            witness["expected_bits"]
                .as_str()
                .ok_or_else(|| format!("{id}: missing expected_bits"))?,
        )?;
        if !f64::from_bits(want).is_finite() {
            return Err(format!("{id}: expected_bits is not a finite number"));
        }
        let got = combin_kernel(n, k)
            .map_err(|error| format!("{id}: production returned {error:?}"))?
            .to_bits();
        let delta = (got as i128) - (want as i128);
        let delta = i64::try_from(delta).map_err(|_| format!("{id}: delta does not fit i64"))?;
        *histogram.entry(delta).or_default() += 1;
        if delta == 0 {
            exact += 1;
        } else if first_mismatches.len() < 20 {
            first_mismatches.push(format!(
                "{id}: n={n} k={k} got=0x{got:016x} want=0x{want:016x} delta={delta:+}"
            ));
        }
    }
    println!(
        "{}: exact={exact}/{} histogram={histogram:?}",
        path.display(),
        witnesses.len()
    );
    for mismatch in first_mismatches {
        println!("  {mismatch}");
    }
    Ok((exact, witnesses.len()))
}

fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args_os().skip(1).collect();
    if paths.is_empty() {
        return Err("pass one or more WitnessSet JSON paths".to_owned());
    }
    let mut total_exact = 0;
    let mut total_rows = 0;
    for path in paths {
        let (exact, rows) = replay(Path::new(&path))?;
        total_exact += exact;
        total_rows += rows;
    }
    println!("combined: exact={total_exact}/{total_rows}");
    if total_exact != total_rows {
        return Err(format!(
            "production replay has {} mismatches",
            total_rows - total_exact
        ));
    }
    Ok(())
}
