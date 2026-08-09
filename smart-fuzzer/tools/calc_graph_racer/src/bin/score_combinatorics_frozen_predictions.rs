//! Score frozen selected/control typed predictions without model refinement.

use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
struct Frozen {
    function: String,
    predictions: Vec<Prediction>,
}

#[derive(Deserialize)]
struct Prediction {
    id: String,
    args: [String; 2],
    selected: String,
    controls: BTreeMap<String, String>,
}

#[derive(Deserialize)]
struct Answers {
    function: String,
    witnesses: Vec<Witness>,
    #[serde(flatten)]
    _rest: BTreeMap<String, Value>,
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

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err(
            "usage: score_combinatorics_frozen_predictions <predictions> <answers>".to_owned(),
        );
    }
    let frozen: Frozen = read(Path::new(&args[0]))?;
    let answers: Answers = read(Path::new(&args[1]))?;
    if frozen.function != answers.function || frozen.predictions.len() != answers.witnesses.len() {
        return Err("function/count mismatch".to_owned());
    }
    let mut scores = BTreeMap::<String, usize>::new();
    for (index, (prediction, answer)) in frozen
        .predictions
        .iter()
        .zip(&answers.witnesses)
        .enumerate()
    {
        if prediction.id != answer.id || prediction.args != answer.args {
            return Err(format!("ordered id/args mismatch at row {index}"));
        }
        if prediction.selected == answer.expected_bits {
            *scores.entry("selected".to_owned()).or_default() += 1;
        }
        for (name, value) in &prediction.controls {
            if value == &answer.expected_bits {
                *scores.entry(name.clone()).or_default() += 1;
            }
        }
    }
    println!("{} rows={}", frozen.function, frozen.predictions.len());
    for (name, exact) in scores {
        println!("{name}: {exact}/{}", frozen.predictions.len());
    }
    Ok(())
}
