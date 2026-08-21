//! Witness sets, bit-exact scoring, and candidate ranking.
//!
//! Scoring is lexicographic per the W109 doctrine: (1) fewest structural
//! mismatches, (2) most exact-bit matches, (3) lowest max ULP, (4) lowest
//! graph complexity. Average accuracy vs a correctly-rounded reference is
//! deliberately NOT a criterion — Excel may be intentionally less accurate,
//! and the target is Excel's bits, not mathematics.

use crate::dsl::Candidate;
use crate::eval::{ArgValue, eval_graph, parse_bits_hex};
use serde::{Deserialize, Serialize};

/// One oracle witness: exact input bits and Excel's exact result bits.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Witness {
    #[serde(default)]
    pub id: Option<String>,
    pub args: Vec<WitnessArg>,
    /// Excel's published result, `0x` + 16 hex digits.
    pub expected_bits: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum WitnessArg {
    Scalar(String),
    Array(Vec<String>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WitnessSet {
    pub function: String,
    pub witnesses: Vec<Witness>,
}

impl Witness {
    pub fn arg_values(&self) -> Result<Vec<ArgValue>, String> {
        self.args
            .iter()
            .map(|a| match a {
                WitnessArg::Scalar(s) => parse_bits_hex(s)
                    .map(ArgValue::Scalar)
                    .ok_or_else(|| format!("bad scalar bits '{s}'")),
                WitnessArg::Array(items) => items
                    .iter()
                    .map(|s| parse_bits_hex(s).ok_or_else(|| format!("bad array bits '{s}'")))
                    .collect::<Result<Vec<f64>, _>>()
                    .map(ArgValue::Array),
            })
            .collect()
    }

    pub fn expected(&self) -> Result<f64, String> {
        parse_bits_hex(&self.expected_bits)
            .ok_or_else(|| format!("bad expected bits '{}'", self.expected_bits))
    }
}

/// Monotonic ordering key over f64 bit patterns (finite and infinite values;
/// NaN has no key).
fn ordered_key(v: f64) -> Option<u64> {
    if v.is_nan() {
        return None;
    }
    let bits = v.to_bits();
    Some(if bits & 0x8000_0000_0000_0000 != 0 {
        !bits
    } else {
        bits | 0x8000_0000_0000_0000
    })
}

/// ULP distance between two non-NaN doubles (0 = identical bits). `None`
/// marks a structural mismatch (either side NaN).
pub fn ulp_distance(a: f64, b: f64) -> Option<u64> {
    let (ka, kb) = (ordered_key(a)?, ordered_key(b)?);
    Some(ka.abs_diff(kb))
}

/// Lexicographic candidate score. Lower is better under `Ord`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Score {
    /// Evaluation errors + NaN-kind mismatches.
    pub structural_mismatches: u32,
    /// Witnesses NOT matched bit-exactly (so `Ord` can be derive-friendly).
    pub inexact: u32,
    /// Largest ULP distance over inexact numeric rows.
    pub max_ulp: u64,
    /// Node count (incl. fold bodies) — the simplicity tiebreaker.
    pub complexity: u32,
}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (
            self.structural_mismatches,
            self.inexact,
            self.max_ulp,
            self.complexity,
        )
            .cmp(&(
                other.structural_mismatches,
                other.inexact,
                other.max_ulp,
                other.complexity,
            ))
    }
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Per-witness failure detail for reports and the ruled-out ledger.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WitnessFailure {
    pub witness_index: usize,
    pub witness_id: Option<String>,
    pub expected_bits: String,
    pub got_bits: Option<String>,
    pub ulp: Option<u64>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CandidateResult {
    pub id: String,
    pub hash: String,
    pub description: String,
    pub score: Score,
    pub exact: u32,
    pub total: u32,
    pub failures: Vec<WitnessFailure>,
}

/// Score one candidate over a witness set.
pub fn score_candidate(
    c: &Candidate,
    witnesses: &[Witness],
    max_failures: usize,
) -> CandidateResult {
    let mut structural = 0u32;
    let mut inexact = 0u32;
    let mut exact = 0u32;
    let mut max_ulp = 0u64;
    let mut failures = Vec::new();
    for (i, w) in witnesses.iter().enumerate() {
        let expected = match w.expected() {
            Ok(v) => v,
            Err(e) => {
                structural += 1;
                if failures.len() < max_failures {
                    failures.push(WitnessFailure {
                        witness_index: i,
                        witness_id: w.id.clone(),
                        expected_bits: w.expected_bits.clone(),
                        got_bits: None,
                        ulp: None,
                        error: Some(e),
                    });
                }
                continue;
            }
        };
        let outcome = w
            .arg_values()
            .map_err(crate::eval::EvalError)
            .and_then(|args| eval_graph(&c.graph, &args));
        match outcome {
            Err(e) => {
                structural += 1;
                if failures.len() < max_failures {
                    failures.push(WitnessFailure {
                        witness_index: i,
                        witness_id: w.id.clone(),
                        expected_bits: w.expected_bits.clone(),
                        got_bits: None,
                        ulp: None,
                        error: Some(e.0),
                    });
                }
            }
            Ok(got) => {
                if got.to_bits() == expected.to_bits() {
                    exact += 1;
                    continue;
                }
                match ulp_distance(got, expected) {
                    Some(d) => {
                        inexact += 1;
                        max_ulp = max_ulp.max(d);
                        if failures.len() < max_failures {
                            failures.push(WitnessFailure {
                                witness_index: i,
                                witness_id: w.id.clone(),
                                expected_bits: w.expected_bits.clone(),
                                got_bits: Some(crate::eval::format_bits_hex(got)),
                                ulp: Some(d),
                                error: None,
                            });
                        }
                    }
                    None => {
                        structural += 1;
                        if failures.len() < max_failures {
                            failures.push(WitnessFailure {
                                witness_index: i,
                                witness_id: w.id.clone(),
                                expected_bits: w.expected_bits.clone(),
                                got_bits: Some(crate::eval::format_bits_hex(got)),
                                ulp: None,
                                error: Some("nan-kind mismatch".into()),
                            });
                        }
                    }
                }
            }
        }
    }
    CandidateResult {
        id: c.id.clone(),
        hash: crate::dsl::candidate_hash(c),
        description: c.description.clone(),
        score: Score {
            structural_mismatches: structural,
            inexact,
            max_ulp,
            complexity: c.graph.complexity(),
        },
        exact,
        total: witnesses.len() as u32,
        failures,
    }
}

/// Race a candidate set: score all, return results sorted best-first.
pub fn race(
    candidates: &[Candidate],
    witnesses: &[Witness],
    max_failures: usize,
) -> Vec<CandidateResult> {
    let mut results: Vec<CandidateResult> = candidates
        .iter()
        .map(|c| score_candidate(c, witnesses, max_failures))
        .collect();
    results.sort_by(|a, b| a.score.cmp(&b.score).then_with(|| a.id.cmp(&b.id)));
    results
}

/// Fully-exact survivors (the promotion pool).
pub fn survivors(results: &[CandidateResult]) -> Vec<&CandidateResult> {
    results
        .iter()
        .filter(|r| r.score.structural_mismatches == 0 && r.score.inexact == 0)
        .collect()
}
