//! Experiment scheduler: surviving-candidate state per catalog row,
//! offline distinguishing-input search, and elimination against oracle bits.
//!
//! The active-learning loop this supports:
//! 1. `race` a candidate space over the known witnesses (score + survivors);
//! 2. `distinguish` — rank a probe pool by how many *distinct* output bit
//!    patterns the surviving candidates produce on each input (zero Excel
//!    cost), and emit the top probes as a batch file;
//! 3. the PS driver answers the batch through OracleCache;
//! 4. `eliminate` — kill every candidate that misses any answered bit, and
//!    append the kills to the row's elimination ledger.
//!
//! State layout under `smart-fuzzer/work/w109/<row_id>/`:
//! * `candidates.json`  — surviving [`Candidate`] set (input + output);
//! * `eliminated.jsonl` — append-only kill records (feed the ruled-out ledger).

use crate::dsl::{Candidate, candidate_hash};
use crate::eval::{eval_graph, format_bits_hex};
use crate::score::{Witness, WitnessArg};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// An unanswered probe: exact input bits, no expectation yet.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProbeCase {
    pub id: String,
    pub args: Vec<WitnessArg>,
}

/// One ranked entry of the distinguishing-input search.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RankedProbe {
    pub probe: ProbeCase,
    /// Number of distinct output bit patterns across surviving candidates
    /// (evaluation errors count as one extra pattern).
    pub distinct_outputs: usize,
    /// The distinct patterns, for the record.
    pub outputs: Vec<String>,
}

/// The probe batch handed to the PS driver (`Run-W109ProbeBatch.ps1`).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProbeBatch {
    pub function: String,
    pub row_id: String,
    pub probes: Vec<RankedProbe>,
}

/// Elimination record appended to `eliminated.jsonl`.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Elimination {
    pub candidate_id: String,
    pub candidate_hash: String,
    pub description: String,
    pub killing_witness_id: Option<String>,
    pub expected_bits: String,
    pub got_bits: Option<String>,
    pub error: Option<String>,
    pub unix_time: u64,
}

/// Rank probe-pool inputs by candidate disagreement, best first. Probes on
/// which every candidate agrees carry zero information and are dropped.
pub fn rank_distinguishing(
    candidates: &[Candidate],
    pool: &[ProbeCase],
    top: usize,
) -> Vec<RankedProbe> {
    let mut ranked: Vec<RankedProbe> = pool
        .iter()
        .filter_map(|probe| {
            let shim = Witness {
                id: Some(probe.id.clone()),
                args: probe.args.clone(),
                expected_bits: "0x0000000000000000".into(),
            };
            let args = shim.arg_values().ok()?;
            let mut outputs: Vec<String> = candidates
                .iter()
                .map(|c| match eval_graph(&c.graph, &args) {
                    Ok(v) => format_bits_hex(v),
                    Err(e) => format!("error:{}", e.0),
                })
                .collect();
            outputs.sort();
            outputs.dedup();
            if outputs.len() < 2 {
                return None;
            }
            Some(RankedProbe {
                probe: probe.clone(),
                distinct_outputs: outputs.len(),
                outputs,
            })
        })
        .collect();
    ranked.sort_by(|a, b| {
        b.distinct_outputs
            .cmp(&a.distinct_outputs)
            .then_with(|| a.probe.id.cmp(&b.probe.id))
    });
    ranked.truncate(top);
    ranked
}

/// Eliminate candidates against answered witnesses. Returns the survivors and
/// the kill records (a candidate is killed by its first mismatching witness).
pub fn eliminate(
    candidates: Vec<Candidate>,
    answered: &[Witness],
) -> (Vec<Candidate>, Vec<Elimination>) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let mut survivors = Vec::new();
    let mut kills = Vec::new();
    'candidates: for c in candidates {
        for w in answered {
            let expected = match w.expected() {
                Ok(v) => v,
                Err(_) => continue, // unanswerable row cannot kill
            };
            let outcome = w
                .arg_values()
                .map_err(crate::eval::EvalError)
                .and_then(|args| eval_graph(&c.graph, &args));
            let (got_bits, error, dead) = match outcome {
                Ok(v) => (
                    Some(format_bits_hex(v)),
                    None,
                    v.to_bits() != expected.to_bits(),
                ),
                Err(e) => (None, Some(e.0), true),
            };
            if dead {
                kills.push(Elimination {
                    candidate_id: c.id.clone(),
                    candidate_hash: candidate_hash(&c),
                    description: c.description.clone(),
                    killing_witness_id: w.id.clone(),
                    expected_bits: w.expected_bits.clone(),
                    got_bits,
                    error,
                    unix_time: now,
                });
                continue 'candidates;
            }
        }
        survivors.push(c);
    }
    (survivors, kills)
}

// ---------------------------------------------------------------------------
// Row-state persistence
// ---------------------------------------------------------------------------

pub fn load_candidates(path: &Path) -> Result<Vec<Candidate>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("read {path:?}: {e}"))?;
    serde_json::from_str(&text).map_err(|e| format!("parse {path:?}: {e}"))
}

pub fn save_candidates(path: &Path, candidates: &[Candidate]) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("mkdir {dir:?}: {e}"))?;
    }
    let text = serde_json::to_string_pretty(candidates).map_err(|e| e.to_string())?;
    std::fs::write(path, text).map_err(|e| format!("write {path:?}: {e}"))
}

pub fn append_eliminations(path: &Path, kills: &[Elimination]) -> Result<(), String> {
    if kills.is_empty() {
        return Ok(());
    }
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("mkdir {dir:?}: {e}"))?;
    }
    let mut lines = String::new();
    for k in kills {
        lines.push_str(&serde_json::to_string(k).map_err(|e| e.to_string())?);
        lines.push('\n');
    }
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("open {path:?}: {e}"))?;
    f.write_all(lines.as_bytes())
        .map_err(|e| format!("append {path:?}: {e}"))
}
