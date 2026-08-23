//! Cross-host x87 last-bit comparison (W108 Phase A).
//!
//! Dumps the live-Excel EXP/LN ground-truth corpus plus a last-bit EXP
//! midpoint ladder, raw `F2XM1`/`FYL2X`, and x87 double-rounded multiply.
//! Run on two x86_64 machines and `diff` the JSONL (skip the `host` line).
//!
//! The interesting question is whether `F2XM1`/`FYL2X` microcode agrees
//! bit-for-bit. Ordinary IEEE add/mul will; the ~1-in-30 hardest EXP rows
//! may not. See `docs/worksets/W108_EXCEL_NUMERIC_CORE_AND_FINANCIAL_POWER_EXACTNESS.md`
//! §10 and `crates/oxfunc_core/src/excel_numeric/x87.rs`.
//!
//! Usage (from this crate):
//!   cargo run --release --bin compare_x87_host -- dump [out.jsonl]
//!   cargo run --release --bin compare_x87_host -- diff a.jsonl b.jsonl

use oxfunc_core::excel_numeric::research as rx;
use rx::{
    CW_PC64_RN, excel_exp, excel_ln, excel_log10, ext_f2xm1, ext_from_f64, ext_fyl2x, ext_ln2,
    ext_to_f64, x87_mul,
};
use std::io::{self, Write};
use std::path::Path;

const GROUND_TRUTH: &str =
    include_str!("../../../../../crates/oxfunc_core/src/excel_numeric/x87_excel_ground_truth.tsv");

fn hex_f64(x: f64) -> String {
    format!("{:016x}", x.to_bits())
}

fn emit(w: &mut dyn Write, rec: &str) -> io::Result<()> {
    writeln!(w, "{rec}")
}

fn json_str(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

fn host_record() -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let (vendor, model, extra) = host_cpu();
    format!(
        "{{\"kind\":\"host\",\"os\":{},\"arch\":{},\"vendor\":{},\"model\":{},\"extra\":{}}}",
        json_str(os),
        json_str(arch),
        json_str(&vendor),
        json_str(&model),
        json_str(&extra)
    )
}

fn host_cpu() -> (String, String, String) {
    #[cfg(target_os = "linux")]
    {
        let text = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
        let mut vendor = String::new();
        let mut model = String::new();
        let mut family = String::new();
        let mut cpu_model = String::new();
        let mut stepping = String::new();
        let mut flags = String::new();
        for line in text.lines() {
            let Some((k, v)) = line.split_once(':') else {
                continue;
            };
            let k = k.trim();
            let v = v.trim();
            match k {
                "vendor_id" if vendor.is_empty() => vendor = v.to_string(),
                "model name" if model.is_empty() => model = v.to_string(),
                "cpu family" if family.is_empty() => family = v.to_string(),
                "model" if cpu_model.is_empty() => cpu_model = v.to_string(),
                "stepping" if stepping.is_empty() => stepping = v.to_string(),
                "flags" if flags.is_empty() => flags = v.to_string(),
                _ => {}
            }
        }
        let hypervisor = flags.split_whitespace().any(|f| f == "hypervisor");
        let extra = format!(
            "family={family} model={cpu_model} stepping={stepping} hypervisor={hypervisor}"
        );
        (vendor, model, extra)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let ident = std::env::var("PROCESSOR_IDENTIFIER").unwrap_or_default();
        let extra = format!("PROCESSOR_IDENTIFIER={ident}");
        (String::new(), ident, extra)
    }
}

fn row(
    suite: &str,
    func: &str,
    x: f64,
    host: f64,
    excel: Option<f64>,
) -> String {
    let match_excel = excel.map(|e| e.to_bits() == host.to_bits());
    let excel_hex = excel.map(hex_f64).unwrap_or_else(|| "null".into());
    let match_s = match match_excel {
        Some(true) => "true",
        Some(false) => "false",
        None => "null",
    };
    format!(
        "{{\"kind\":\"row\",\"suite\":{},\"func\":{},\"x\":\"0x{}\",\"host\":\"0x{}\",\"excel\":{},\"match_excel\":{}}}",
        json_str(suite),
        json_str(func),
        hex_f64(x),
        hex_f64(host),
        if excel.is_some() {
            format!("\"0x{excel_hex}\"")
        } else {
            "null".into()
        },
        match_s
    )
}

fn dump_ground_truth(w: &mut dyn Write) -> io::Result<(u32, u32)> {
    let mut n = 0u32;
    let mut miss = 0u32;
    for line in GROUND_TRUTH.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut it = line.split('\t');
        let func = it.next().unwrap();
        let xb = u64::from_str_radix(it.next().unwrap(), 16).expect("x hex");
        let eb = u64::from_str_radix(it.next().unwrap(), 16).expect("excel hex");
        let x = f64::from_bits(xb);
        let excel = f64::from_bits(eb);
        let host = match func {
            "EXP" => excel_exp(x),
            "LN" => excel_ln(x),
            "LOG10" => excel_log10(x),
            _ => continue,
        };
        if host.to_bits() != eb {
            miss += 1;
        }
        n += 1;
        emit(w, &row("ground_truth", func, x, host, Some(excel)))?;
    }
    Ok((n, miss))
}

/// EXP arguments with `x/ln2` sitting on a half-integer — FRNDINT midpoint,
/// invert-branch edge, F2XM1 last-bit. This is the W108 "hard ~1-in-30" class.
fn dump_exp_midpoints(w: &mut dyn Write) -> io::Result<u32> {
    let ln2 = std::f64::consts::LN_2;
    let mut n = 0u32;
    for k in -40i32..=40 {
        let base = (k as f64 + 0.5) * ln2;
        for d in -16i32..=16 {
            let x = if d == 0 {
                base
            } else if d > 0 {
                let mut v = base;
                for _ in 0..d {
                    v = v.next_up();
                }
                v
            } else {
                let mut v = base;
                for _ in 0..(-d) {
                    v = v.next_down();
                }
                v
            };
            if !x.is_finite() {
                continue;
            }
            emit(w, &row("exp_midpoint", "EXP", x, excel_exp(x), None))?;
            n += 1;
        }
    }
    Ok(n)
}

fn dump_raw_f2xm1(w: &mut dyn Write) -> io::Result<u32> {
    let mut n = 0u32;
    let cw = CW_PC64_RN;
    // Domain of F2XM1 is |x| <= 1.
    for k in 0..=53u32 {
        let x = 2f64.powi(-(k as i32));
        for &s in &[x, -x, 1.0 - x] {
            if s.abs() > 1.0 || s == 0.0 {
                continue;
            }
            let y = ext_to_f64(&ext_f2xm1(&ext_from_f64(s), cw), cw);
            emit(w, &row("raw_f2xm1", "F2XM1", s, y, None))?;
            n += 1;
        }
    }
    Ok(n)
}

fn dump_raw_fyl2x(w: &mut dyn Write) -> io::Result<u32> {
    let mut n = 0u32;
    let cw = CW_PC64_RN;
    let ln2 = ext_ln2();
    for e in -20i32..=20 {
        let x = 2f64.powi(e);
        let y = ext_to_f64(&ext_fyl2x(&ln2, &ext_from_f64(x), cw), cw);
        emit(w, &row("raw_fyl2x", "FYL2X_LN2", x, y, None))?;
        n += 1;
    }
    // 1 + n ulp, the FYL2XP1-adjacent band but through FYL2X.
    let mut x = 1.0_f64;
    for _ in 0..32 {
        x = x.next_up();
        let y = ext_to_f64(&ext_fyl2x(&ln2, &ext_from_f64(x), cw), cw);
        emit(w, &row("raw_fyl2x", "FYL2X_LN2", x, y, None))?;
        n += 1;
    }
    Ok(n)
}

fn dump_x87_mul(w: &mut dyn Write) -> io::Result<u32> {
    let mut n = 0u32;
    // Double-rounding window: products whose exact value sits between a
    // binary64 midpoint and the PC=64 midpoint.
    for e in 0..64u32 {
        let a = 1.0 + f64::from_bits(e as u64); // 1, 1+tiny
        let b = 1.0 + (e as f64) * f64::EPSILON;
        let y = x87_mul(a, b);
        emit(w, &row("x87_mul", "MUL", a, y, None))?;
        n += 1;
        let y2 = x87_mul(a, std::f64::consts::PI);
        emit(w, &row("x87_mul", "MUL_PI", a, y2, None))?;
        n += 1;
    }
    Ok(n)
}

fn dump(path: Option<&str>) -> io::Result<()> {
    let mut file;
    let w: &mut dyn Write = if let Some(p) = path {
        file = std::fs::File::create(p)?;
        &mut file
    } else {
        &mut io::stdout()
    };
    emit(w, &host_record())?;
    let (gt_n, gt_miss) = dump_ground_truth(w)?;
    let mid_n = dump_exp_midpoints(w)?;
    let f2_n = dump_raw_f2xm1(w)?;
    let fy_n = dump_raw_fyl2x(w)?;
    let mul_n = dump_x87_mul(w)?;
    emit(
        w,
        &format!(
            "{{\"kind\":\"summary\",\"ground_truth_rows\":{gt_n},\"ground_truth_excel_miss\":{gt_miss},\"exp_midpoint_rows\":{mid_n},\"f2xm1_rows\":{f2_n},\"fyl2x_rows\":{fy_n},\"x87_mul_rows\":{mul_n}}}"
        ),
    )?;
    eprintln!(
        "dumped ground_truth={gt_n} excel_miss={gt_miss} exp_mid={mid_n} f2xm1={f2_n} fyl2x={fy_n} mul={mul_n}"
    );
    Ok(())
}

fn payload_key(line: &str) -> Option<(String, String)> {
    // Identity on suite+func+x so host/summary lines are skipped.
    if !line.contains("\"kind\":\"row\"") {
        return None;
    }
    let suite = capture(line, "\"suite\":")?;
    let func = capture(line, "\"func\":")?;
    let x = capture(line, "\"x\":")?;
    Some((format!("{suite}\t{func}\t{x}"), line.to_string()))
}

fn capture(line: &str, key: &str) -> Option<String> {
    let i = line.find(key)?;
    let rest = &line[i + key.len()..];
    let rest = rest.trim_start_matches(':').trim_start();
    if rest.starts_with('"') {
        let end = rest[1..].find('"')?;
        Some(rest[1..=end].to_string())
    } else {
        None
    }
}

fn host_of(path: &str) -> String {
    let text = std::fs::read_to_string(path).unwrap_or_default();
    text.lines()
        .find(|l| l.contains("\"kind\":\"host\""))
        .unwrap_or("")
        .to_string()
}

fn diff(a: &str, b: &str) -> io::Result<()> {
    let ta = std::fs::read_to_string(a)?;
    let tb = std::fs::read_to_string(b)?;
    let mut ma = std::collections::BTreeMap::new();
    let mut mb = std::collections::BTreeMap::new();
    for line in ta.lines() {
        if let Some((k, v)) = payload_key(line) {
            ma.insert(k, v);
        }
    }
    for line in tb.lines() {
        if let Some((k, v)) = payload_key(line) {
            mb.insert(k, v);
        }
    }
    println!("A host: {}", host_of(a));
    println!("B host: {}", host_of(b));
    let mut only_a = 0u32;
    let mut only_b = 0u32;
    let mut differ = 0u32;
    let mut same = 0u32;
    let mut shown = 0u32;
    for (k, va) in &ma {
        match mb.get(k) {
            None => only_a += 1,
            Some(vb) if va == vb => same += 1,
            Some(vb) => {
                differ += 1;
                if shown < 40 {
                    println!("DIFF {k}");
                    println!("  A {va}");
                    println!("  B {vb}");
                    shown += 1;
                }
            }
        }
    }
    for k in mb.keys() {
        if !ma.contains_key(k) {
            only_b += 1;
        }
    }
    println!(
        "same={same} differ={differ} only_a={only_a} only_b={only_b} (first {shown} diffs shown)"
    );
    Ok(())
}

fn main() {
    let mut args = std::env::args().skip(1);
    let cmd = args.next().unwrap_or_else(|| "dump".into());
    match cmd.as_str() {
        "dump" => {
            let out = args.next();
            dump(out.as_deref()).expect("dump");
        }
        "diff" => {
            let a = args.next().expect("diff A.jsonl");
            let b = args.next().expect("diff B.jsonl");
            assert!(Path::new(&a).exists() && Path::new(&b).exists());
            diff(&a, &b).expect("diff");
        }
        _ => {
            eprintln!("usage: compare_x87_host dump [out.jsonl]");
            eprintln!("       compare_x87_host diff a.jsonl b.jsonl");
            std::process::exit(2);
        }
    }
}
