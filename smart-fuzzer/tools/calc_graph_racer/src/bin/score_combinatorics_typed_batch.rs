//! Replay typed COMBIN/COMBINA batches against the compiled production kernels.

use oxfunc_core::functions::combin::combin_kernel;
use oxfunc_core::functions::combina::combina_kernel;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
struct Batch {
    function: String,
    probes: Vec<BatchProbe>,
}

#[derive(Deserialize)]
struct BatchProbe {
    probe: Probe,
}

#[derive(Deserialize)]
struct Probe {
    id: String,
    args: [String; 2],
}

#[derive(Deserialize)]
struct Answers {
    function: String,
    witnesses: Vec<Witness>,
}

#[derive(Deserialize)]
struct Witness {
    id: String,
    args: [String; 2],
    expected_bits: String,
}

fn read<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))
}

fn decode(raw: &str) -> Result<f64, String> {
    let raw = raw
        .strip_prefix("0x")
        .ok_or_else(|| format!("not a binary64 bit string: {raw}"))?;
    let bits = u64::from_str_radix(raw, 16).map_err(|error| error.to_string())?;
    Ok(f64::from_bits(bits))
}

fn replay(function: &str, args: &[String; 2]) -> Result<String, String> {
    let n = decode(&args[0])?;
    let k = decode(&args[1])?;
    let result = match function {
        "COMBIN" => combin_kernel(n, k),
        "COMBINA" => combina_kernel(n, k),
        _ => return Err(format!("unsupported function: {function}")),
    };
    Ok(match result {
        Ok(value) => format!("0x{:016x}", value.to_bits()),
        Err(error) => format!("error:{error:?}"),
    })
}

fn score(batch_path: &Path, answers_path: &Path) -> Result<(usize, usize), String> {
    let batch: Batch = read(batch_path)?;
    let answers: Answers = read(answers_path)?;
    if batch.function != answers.function {
        return Err(format!(
            "function mismatch: batch={} answers={}",
            batch.function, answers.function
        ));
    }
    if batch.probes.len() != answers.witnesses.len() {
        return Err(format!(
            "count mismatch: batch={} answers={}",
            batch.probes.len(),
            answers.witnesses.len()
        ));
    }
    let mut ids = BTreeSet::new();
    let mut exact = 0_usize;
    let mut misses = Vec::new();
    for (index, (probe, witness)) in batch.probes.iter().zip(&answers.witnesses).enumerate() {
        if probe.probe.id != witness.id || probe.probe.args != witness.args {
            return Err(format!("ordered id/args mismatch at row {index}"));
        }
        if !ids.insert(&witness.id) {
            return Err(format!("duplicate id: {}", witness.id));
        }
        let actual = replay(&batch.function, &witness.args)?;
        if actual == witness.expected_bits {
            exact += 1;
        } else if misses.len() < 40 {
            misses.push(format!(
                "{} {:?}: expected={} actual={actual}",
                witness.id, witness.args, witness.expected_bits
            ));
        }
    }
    println!(
        "{} {exact}/{} exact: {} vs {}",
        batch.function,
        batch.probes.len(),
        batch_path.display(),
        answers_path.display()
    );
    for miss in misses {
        println!("  MISS {miss}");
    }
    Ok((exact, batch.probes.len()))
}

fn main() -> Result<(), String> {
    let args: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    if args.is_empty() || args.len() % 2 != 0 {
        return Err(
            "usage: score_combinatorics_typed_batch <batch.json> <answers.json> [...]".to_owned(),
        );
    }
    let mut exact = 0_usize;
    let mut total = 0_usize;
    for pair in args.chunks_exact(2) {
        let (pair_exact, pair_total) = score(&pair[0], &pair[1])?;
        exact += pair_exact;
        total += pair_total;
    }
    println!("combined {exact}/{total} exact");
    if exact != total {
        return Err(format!("production replay has {} misses", total - exact));
    }
    Ok(())
}
