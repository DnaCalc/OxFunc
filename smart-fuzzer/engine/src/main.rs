//! `sf` — the W112 parity driver. Commands land phase by phase; see
//! docs/worksets/W112_PARITY_DRIVER_CONSOLIDATION.md.
//!
//!   sf judge-witnesses <witness.json>... [--max-misses N] [--json]
//!       Offline: run production OxFunc on every banked Excel witness and report
//!       typed-bit agreement, worst severity (ODR-FN-005) and the worst misses.

use sf::judge::judge;
use sf::witness::WitnessSet;

fn usage() -> ! {
    eprintln!("usage: sf judge-witnesses <witness.json>... [--max-misses N] [--json]");
    std::process::exit(2)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(cmd) = args.first() else { usage() };
    match cmd.as_str() {
        "judge-witnesses" => judge_witnesses(&args[1..]),
        _ => usage(),
    }
}

fn judge_witnesses(args: &[String]) {
    let mut files = Vec::new();
    let mut max_misses = 5usize;
    let mut json = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--max-misses" => {
                i += 1;
                max_misses = args.get(i).and_then(|s| s.parse().ok()).unwrap_or_else(|| usage());
            }
            "--json" => json = true,
            f => files.push(f.to_string()),
        }
        i += 1;
    }
    if files.is_empty() {
        usage()
    }
    let mut reports = Vec::new();
    for f in &files {
        let text = std::fs::read_to_string(f).unwrap_or_else(|e| panic!("{f}: {e}"));
        let set: WitnessSet = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{f}: {e}"));
        let report = judge(&set, max_misses).unwrap_or_else(|e| panic!("{f}: {e}"));
        if !json {
            let sev = report
                .severity
                .map(|s| serde_json::to_value(s).unwrap().as_str().unwrap().to_string())
                .unwrap_or_else(|| "none".into());
            println!(
                "{:<24} {:>6}/{:<6} worst={:<10} worst_ulp={:<8} {:?}",
                report.function_id,
                report.rows_agree,
                report.rows_judged,
                sev,
                report.worst_ulp.map(|u| u.to_string()).unwrap_or_else(|| "-".into()),
                report.misses_by_severity
            );
            for m in &report.misses {
                println!(
                    "    {} args={:?} excel={:?} oxfunc={:?} ulp={:?}",
                    m.id, m.args, m.expected, m.actual, m.verdict.ulp
                );
            }
        }
        reports.push(report);
    }
    if json {
        println!("{}", serde_json::to_string_pretty(&reports).unwrap());
    }
}
