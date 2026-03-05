use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::io;
use std::path::Path;

const RUNNER_VERSION: &str = "fp-probe-runner-rust/0.2.0";

#[derive(Debug)]
struct Options {
    manifest_path: String,
    out_path: String,
    mode: String,
}

#[derive(Debug)]
struct CsvTable {
    headers: Vec<String>,
    rows: Vec<HashMap<String, String>>,
}

fn main() {
    let code = match run() {
        Ok(()) => 0,
        Err((code, msg)) => {
            eprintln!("Error: {msg}");
            code
        }
    };
    std::process::exit(code);
}

fn run() -> Result<(), (i32, String)> {
    let opts = parse_args().map_err(|m| (2, m))?;
    if !Path::new(&opts.manifest_path).exists() {
        return Err((3, format!("manifest file not found: {}", opts.manifest_path)));
    }

    let table = read_csv(&opts.manifest_path).map_err(|m| (4, m))?;
    if table.rows.is_empty() {
        return Err((4, "manifest has no data rows".to_string()));
    }

    let required = ["scenario_id", "lane", "objective", "status"];
    let header_set: HashSet<String> = table.headers.iter().map(|h| h.to_ascii_lowercase()).collect();
    let missing: Vec<&str> = required
        .iter()
        .copied()
        .filter(|r| !header_set.contains(&r.to_ascii_lowercase()))
        .collect();
    if !missing.is_empty() {
        return Err((5, format!("manifest missing required columns: {}", missing.join(","))));
    }

    let mut out_rows: Vec<Vec<String>> = Vec::new();
    for row in &table.rows {
        let scenario_id = row.get("scenario_id").cloned().unwrap_or_default();
        let lane = row.get("lane").cloned().unwrap_or_default();
        let objective = row.get("objective").cloned().unwrap_or_default();
        let notes = format!("seeded from manifest objective: {objective}");
        out_rows.push(vec![
            scenario_id,
            lane,
            opts.mode.clone(),
            "queued".to_string(),
            "pending_observation".to_string(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            RUNNER_VERSION.to_string(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            notes,
        ]);
    }

    if let Some(parent) = Path::new(&opts.out_path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| (6, format!("failed to create output directory: {e}")))?;
        }
    }

    let headers = [
        "scenario_id",
        "lane",
        "mode",
        "execution_status",
        "observed_class",
        "excel_version",
        "excel_channel",
        "compat_version",
        "locale_profile",
        "runner_version",
        "artifact_ref",
        "primary_cell",
        "primary_formula2",
        "primary_value2",
        "primary_text",
        "observed_cells",
        "comparison_bools",
        "notes",
    ];
    write_csv(&opts.out_path, &headers, &out_rows).map_err(|m| (7, m))?;
    println!("Wrote {} rows to: {}", out_rows.len(), opts.out_path);
    Ok(())
}

fn parse_args() -> Result<Options, String> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 5 {
        return Err(usage());
    }

    let mut manifest_path: Option<String> = None;
    let mut out_path: Option<String> = None;
    let mut mode = "dry-run".to_string();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--manifest" => {
                i += 1;
                if i >= args.len() {
                    return Err(usage());
                }
                manifest_path = Some(args[i].clone());
            }
            "--out" => {
                i += 1;
                if i >= args.len() {
                    return Err(usage());
                }
                out_path = Some(args[i].clone());
            }
            "--mode" => {
                i += 1;
                if i >= args.len() {
                    return Err(usage());
                }
                let m = args[i].as_str();
                if m != "dry-run" && m != "prepare" {
                    return Err("invalid --mode; expected dry-run or prepare".to_string());
                }
                mode = m.to_string();
            }
            other => return Err(format!("unknown argument: {other}\n{}", usage())),
        }
        i += 1;
    }

    let manifest_path = manifest_path.ok_or_else(usage)?;
    let out_path = out_path.ok_or_else(usage)?;
    Ok(Options {
        manifest_path,
        out_path,
        mode,
    })
}

fn usage() -> String {
    "usage: fp_probe_runner --manifest <csv> --out <csv> [--mode dry-run|prepare]".to_string()
}

fn read_csv(path: &str) -> Result<CsvTable, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("failed to read CSV: {e}"))?;
    let mut lines = content.lines();
    let header_line = lines.next().ok_or_else(|| "CSV file is empty".to_string())?;
    let headers = parse_csv_line(header_line)
        .into_iter()
        .map(|h| h.trim().to_string())
        .collect::<Vec<_>>();
    if headers.is_empty() {
        return Err("CSV header row is empty".to_string());
    }

    let mut rows = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let cols = parse_csv_line(line);
        let mut row = HashMap::new();
        for (idx, header) in headers.iter().enumerate() {
            let value = cols.get(idx).cloned().unwrap_or_default();
            row.insert(header.to_ascii_lowercase(), value);
        }
        rows.push(row);
    }
    Ok(CsvTable { headers, rows })
}

fn write_csv(path: &str, headers: &[&str], rows: &[Vec<String>]) -> Result<(), String> {
    let mut out = String::new();
    out.push_str(&headers.iter().map(|h| csv_escape(h)).collect::<Vec<_>>().join(","));
    out.push('\n');
    for row in rows {
        out.push_str(&row.iter().map(|v| csv_escape(v)).collect::<Vec<_>>().join(","));
        out.push('\n');
    }
    fs::write(path, out).map_err(|e| format!("failed to write CSV: {e}"))
}

fn parse_csv_line(line: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0usize;

    while i < chars.len() {
        let ch = chars[i];
        if ch == '"' {
            if in_quotes && i + 1 < chars.len() && chars[i + 1] == '"' {
                cur.push('"');
                i += 1;
            } else {
                in_quotes = !in_quotes;
            }
        } else if ch == ',' && !in_quotes {
            result.push(cur.clone());
            cur.clear();
        } else {
            cur.push(ch);
        }
        i += 1;
    }
    result.push(cur);
    result
}

fn csv_escape(s: &str) -> String {
    let mut value = s.replace('"', "\"\"");
    if value.contains(',') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        value = format!("\"{value}\"");
    }
    value
}

#[allow(dead_code)]
fn _ensure_io_type(_: io::Result<()>) {}
