//! CLI for the W109 calculation-graph racer/scheduler.
//!
//! Subcommands:
//!   race        --candidates <c.json> --witnesses <w.json> [--report <out.json>] [--max-failures N]
//!   distinguish --candidates <c.json> --pool <pool.json> --function <F> --row-id <G*-NN>
//!               --out <batch.json> [--top N]
//!   eliminate   --candidates <c.json> --answers <answered.json> --survivors <out.json>
//!               [--eliminated <kills.jsonl>]
//!
//! File formats (all JSON):
//!   candidates : [Candidate]
//!   witnesses  : WitnessSet { function, witnesses: [{id?, args, expected_bits}] }
//!   pool       : [ProbeCase { id, args }]
//!   answers    : WitnessSet (probe batch answered by Run-W109ProbeBatch.ps1)

use calc_graph_racer::dsl::Candidate;
use calc_graph_racer::scheduler::{
    ProbeBatch, ProbeCase, append_eliminations, eliminate, load_candidates, rank_distinguishing,
    save_candidates,
};
use calc_graph_racer::score::{WitnessSet, race, survivors};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some((cmd, rest)) = args.split_first() else {
        eprintln!("usage: calc_graph_racer <race|distinguish|eliminate> [options]");
        return ExitCode::from(2);
    };
    let opts = match parse_opts(rest) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
    };
    let result = match cmd.as_str() {
        "race" => cmd_race(&opts),
        "distinguish" => cmd_distinguish(&opts),
        "eliminate" => cmd_eliminate(&opts),
        other => Err(format!("unknown subcommand '{other}'")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn parse_opts(rest: &[String]) -> Result<HashMap<String, String>, String> {
    let mut opts = HashMap::new();
    let mut it = rest.iter();
    while let Some(k) = it.next() {
        let key = k
            .strip_prefix("--")
            .ok_or_else(|| format!("expected --option, got '{k}'"))?;
        let v = it
            .next()
            .ok_or_else(|| format!("--{key} needs a value"))?;
        opts.insert(key.to_string(), v.clone());
    }
    Ok(opts)
}

fn required<'a>(opts: &'a HashMap<String, String>, key: &str) -> Result<&'a str, String> {
    opts.get(key)
        .map(|s| s.as_str())
        .ok_or_else(|| format!("missing required option --{key}"))
}

fn read_json<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, String> {
    let text =
        std::fs::read_to_string(path).map_err(|e| format!("read {path}: {e}"))?;
    serde_json::from_str(&text).map_err(|e| format!("parse {path}: {e}"))
}

fn write_json<T: serde::Serialize>(path: &str, value: &T) -> Result<(), String> {
    if let Some(dir) = Path::new(path).parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("mkdir {dir:?}: {e}"))?;
    }
    let text = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    std::fs::write(path, text).map_err(|e| format!("write {path}: {e}"))
}

fn cmd_race(opts: &HashMap<String, String>) -> Result<(), String> {
    let candidates: Vec<Candidate> =
        load_candidates(&PathBuf::from(required(opts, "candidates")?))?;
    let ws: WitnessSet = read_json(required(opts, "witnesses")?)?;
    let max_failures = opts
        .get("max-failures")
        .map(|s| s.parse::<usize>().map_err(|e| e.to_string()))
        .transpose()?
        .unwrap_or(8);
    let results = race(&candidates, &ws.witnesses, max_failures);
    println!(
        "{} candidates x {} witnesses on {}",
        candidates.len(),
        ws.witnesses.len(),
        ws.function
    );
    for r in &results {
        println!(
            "  {:>5}/{:<5} exact  max_ulp={:<8} structural={:<3} complexity={:<4} {}  {}",
            r.exact, r.total, r.score.max_ulp, r.score.structural_mismatches,
            r.score.complexity, r.id, r.description
        );
    }
    let winners = survivors(&results);
    println!(
        "survivors (fully exact): {}",
        if winners.is_empty() {
            "none".to_string()
        } else {
            winners
                .iter()
                .map(|r| r.id.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        }
    );
    if let Some(report) = opts.get("report") {
        write_json(report, &results)?;
        println!("report written to {report}");
    }
    Ok(())
}

fn cmd_distinguish(opts: &HashMap<String, String>) -> Result<(), String> {
    let candidates: Vec<Candidate> =
        load_candidates(&PathBuf::from(required(opts, "candidates")?))?;
    let pool: Vec<ProbeCase> = read_json(required(opts, "pool")?)?;
    let top = opts
        .get("top")
        .map(|s| s.parse::<usize>().map_err(|e| e.to_string()))
        .transpose()?
        .unwrap_or(64);
    let ranked = rank_distinguishing(&candidates, &pool, top);
    let batch = ProbeBatch {
        function: required(opts, "function")?.to_string(),
        row_id: required(opts, "row-id")?.to_string(),
        probes: ranked,
    };
    println!(
        "{} distinguishing probes (from a pool of {}) across {} candidates",
        batch.probes.len(),
        pool.len(),
        candidates.len()
    );
    write_json(required(opts, "out")?, &batch)?;
    Ok(())
}

fn cmd_eliminate(opts: &HashMap<String, String>) -> Result<(), String> {
    let candidates: Vec<Candidate> =
        load_candidates(&PathBuf::from(required(opts, "candidates")?))?;
    let answered: WitnessSet = read_json(required(opts, "answers")?)?;
    let before = candidates.len();
    let (alive, kills) = eliminate(candidates, &answered.witnesses);
    println!(
        "eliminated {} of {} candidates; {} survive",
        kills.len(),
        before,
        alive.len()
    );
    for k in &kills {
        println!(
            "  killed {} ({}) by witness {:?}: expected {} got {:?}",
            k.candidate_id, k.candidate_hash, k.killing_witness_id, k.expected_bits, k.got_bits
        );
    }
    save_candidates(&PathBuf::from(required(opts, "survivors")?), &alive)?;
    if let Some(path) = opts.get("eliminated") {
        append_eliminations(&PathBuf::from(path), &kills)?;
    }
    Ok(())
}
