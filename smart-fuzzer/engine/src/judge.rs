//! Offline judge: production OxFunc on every banked witness, typed-bit compared.

use crate::classify::{Outcome, RowVerdict, Severity, compare};
use crate::witness::{WitnessSet, function_id, parse_expected, parse_hex_bits};
use oxfunc_core::functions::surface_dispatch::eval_surface_value_call;
use oxfunc_core::resolver::NULL_REFERENCE_SYSTEM_PROVIDER;
use oxfunc_core::value::{CalcValue, CoreValue};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Serialize)]
pub struct Miss {
    pub id: String,
    pub args: Vec<String>,
    pub expected: Outcome,
    pub actual: Outcome,
    pub verdict: RowVerdict,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub function_id: String,
    pub rows_judged: usize,
    pub rows_agree: usize,
    /// Worst class over all rows; None when every row agrees.
    pub severity: Option<Severity>,
    pub worst_ulp: Option<u64>,
    pub misses_by_severity: BTreeMap<String, usize>,
    /// Up to `max_misses` misses, worst first.
    pub misses: Vec<Miss>,
    pub capture_provenance: Option<serde_json::Value>,
}

pub fn outcome_of(result: Result<CalcValue, oxfunc_core::value::WorksheetErrorCode>) -> Outcome {
    match result {
        Err(code) => Outcome::Error { code: format!("{code:?}") },
        Ok(value) => match value.core {
            CoreValue::Number(x) => Outcome::Number { bits: x.to_bits() },
            CoreValue::Error(code) => Outcome::Error { code: format!("{code:?}") },
            CoreValue::Logical(b) => Outcome::Logical { value: b },
            CoreValue::Text(t) => Outcome::Text { value: t.to_string_lossy() },
            other => Outcome::Other { detail: format!("{other:?}") },
        },
    }
}

pub fn eval_production(fid: &str, args: &[f64]) -> Outcome {
    let values: Vec<CalcValue> = args.iter().map(|x| CalcValue::number(*x)).collect();
    outcome_of(eval_surface_value_call(
        fid,
        &values,
        &NULL_REFERENCE_SYSTEM_PROVIDER,
        None,
        None,
        None,
        None,
    ))
}

pub fn judge(set: &WitnessSet, max_misses: usize) -> Result<Report, String> {
    let fid = function_id(&set.function);
    let mut rows_agree = 0;
    let mut misses = Vec::new();
    let mut by_sev: BTreeMap<String, usize> = BTreeMap::new();
    let mut worst: Option<Severity> = None;
    let mut worst_ulp: Option<u64> = None;
    for w in &set.witnesses {
        let args = w
            .args
            .iter()
            .map(|a| parse_hex_bits(a).map(f64::from_bits))
            .collect::<Result<Vec<_>, _>>()?;
        let expected = parse_expected(&w.expected_bits)?;
        let actual = eval_production(&fid, &args);
        let verdict = compare(&expected, &actual);
        if verdict.agree {
            rows_agree += 1;
            continue;
        }
        let sev = verdict.severity.expect("a disagreeing row has a severity");
        *by_sev
            .entry(serde_json::to_value(sev).unwrap().as_str().unwrap().to_string())
            .or_default() += 1;
        worst = worst.max(Some(sev));
        if let Some(u) = verdict.ulp {
            worst_ulp = worst_ulp.max(Some(u));
        }
        misses.push(Miss { id: w.id.clone(), args: w.args.clone(), expected, actual, verdict });
    }
    misses.sort_by(|a, b| {
        b.verdict
            .severity
            .cmp(&a.verdict.severity)
            .then(b.verdict.ulp.cmp(&a.verdict.ulp))
    });
    misses.truncate(max_misses);
    Ok(Report {
        function_id: fid,
        rows_judged: set.witnesses.len(),
        rows_agree,
        severity: worst,
        worst_ulp,
        misses_by_severity: by_sev,
        misses,
        capture_provenance: set.capture_provenance.clone(),
    })
}
