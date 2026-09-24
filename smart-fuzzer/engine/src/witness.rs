//! Banked Excel answers in the racer's WitnessSet shape, as written by
//! `smart-fuzzer/tools/Run-W109BulkBatch.ps1`:
//!
//! ```json
//! { "function": "COUPDAYS", "witnesses": [ { "id": "...", "args": ["0x40e5...", ...],
//!   "expected_bits": "0x4066..." | "error:Num" | "logical:True" | "text:..." } ],
//!   "capture_provenance": { ... } }
//! ```
//!
//! Scalar numeric arguments only (hex f64 bits), which is what the bulk engine captures.

use crate::classify::Outcome;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct WitnessSet {
    pub function: String,
    pub witnesses: Vec<Witness>,
    #[serde(default)]
    pub capture_provenance: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct Witness {
    pub id: String,
    pub args: Vec<String>,
    pub expected_bits: String,
}

pub fn parse_hex_bits(s: &str) -> Result<u64, String> {
    let t = s.trim().trim_start_matches("0x").trim_start_matches("0X");
    u64::from_str_radix(t, 16).map_err(|e| format!("bad hex bits '{s}': {e}"))
}

pub fn parse_expected(s: &str) -> Result<Outcome, String> {
    if let Some(code) = s.strip_prefix("error:") {
        return Ok(Outcome::Error { code: code.to_string() });
    }
    if let Some(v) = s.strip_prefix("logical:") {
        return Ok(Outcome::Logical { value: v.eq_ignore_ascii_case("true") });
    }
    if let Some(v) = s.strip_prefix("text:") {
        return Ok(Outcome::Text { value: v.to_string() });
    }
    Ok(Outcome::Number { bits: parse_hex_bits(s)? })
}

/// Surface name ("COUPDAYS") or id ("FUNC.COUPDAYS") to a catalog function id.
pub fn function_id(name: &str) -> String {
    if name.starts_with("FUNC.") {
        name.to_string()
    } else {
        format!("FUNC.{}", name.to_ascii_uppercase())
    }
}
