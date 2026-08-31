//! Firehorse implied-F campaign: truncated-CF store masks, piecewise cuts.
//! Scores F_or = Q / excel_exp(-RN53(RN64(z*z))). Heldouts unnamed. No landing.
//!
//!   campaign_erfc_fbody --dir G3-01-dist --out erfc-fbody-campaign --threads 12 --max-hours 96

use calc_graph_racer::erfc_f_packets as f;
use oxfunc_core::excel_numeric::research as rx;
use rayon::prelude::*;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const CHUNK: u32 = 4096;
const TAIL_BAR: usize = 1283;
const MID_BAR: usize = 2389;

#[derive(Clone, Copy)]
enum CfKind {
    Gautschi,
    As714,
}

#[derive(Clone)]
struct CubeJob {
    axis: String,
    kind: CfKind,
    nterms: u32,
    mask_lim: u32,
    note: String,
}

fn cube_jobs() -> Vec<CubeJob> {
    let mut jobs = Vec::new();
    for &(n, two_bit) in &[(12u32, true), (16, false), (20, false), (21, false), (24, false)] {
        let bits = if two_bit { n * 2 } else { n };
        for (kind, tag) in [(CfKind::Gautschi, "gaut"), (CfKind::As714, "as714")] {
            jobs.push(CubeJob {
                axis: format!("CF/{tag}/n{n}{}", if two_bit { "b2" } else { "" }),
                kind,
                nterms: n,
                mask_lim: 1u32 << bits.min(28),
                note: format!(
                    "x87 CF {} n={n} {}-bit store-after-step",
                    tag,
                    bits
                ),
            });
        }
    }
    jobs
}

fn spill(x: Ext80) -> Ext80 {
    ext_from_f64(ext_to_f64(&x, CW_PC64_RN))
}

fn gautschi_mask(x: f64, nterms: u32, mask: u32, two_bit: bool) -> f64 {
    let xe = ext_from_f64(x);
    let mut acc = xe;
    let mut bit = 0u32;
    for k in (1..=nterms).rev() {
        let hn = ext_from_f64(k as f64 * 0.5);
        let mut term = ext_div(&hn, &acc, CW_PC64_RN);
        if two_bit {
            if mask & (1 << bit) != 0 {
                term = spill(term);
            }
            bit += 1;
        }
        acc = ext_add(&xe, &term, CW_PC64_RN);
        if mask & (1 << bit) != 0 {
            acc = spill(acc);
        }
        bit += 1;
    }
    f::RPINV / ext_to_f64(&acc, CW_PC64_RN)
}

fn as714_mask(x: f64, nterms: u32, mask: u32, two_bit: bool) -> f64 {
    let one = ext_from_f64(1.0);
    let a_scale = ext_from_f64(0.5 / (x * x));
    let mut den = one;
    let mut bit = 0u32;
    for k in (1..=nterms).rev() {
        let a = ext_mul(&ext_from_f64(k as f64), &a_scale, CW_PC64_RN);
        let mut q = ext_div(&a, &den, CW_PC64_RN);
        if two_bit {
            if mask & (1 << bit) != 0 {
                q = spill(q);
            }
            bit += 1;
        }
        den = ext_add(&one, &q, CW_PC64_RN);
        if mask & (1 << bit) != 0 {
            den = spill(den);
        }
        bit += 1;
    }
    f::RPINV / x / ext_to_f64(&den, CW_PC64_RN)
}

fn eval_job(j: &CubeJob, mask: u32, z: f64) -> f64 {
    let two_bit = j.axis.ends_with("b2");
    match j.kind {
        CfKind::Gautschi => gautschi_mask(z, j.nterms, mask, two_bit),
        CfKind::As714 => as714_mask(z, j.nterms, mask, two_bit),
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct Checkpoint {
    progress: BTreeMap<String, u32>,
    best_mid_exact: usize,
    best_tail_exact: usize,
    best_mid_label: String,
    best_tail_label: String,
    configs_done: u64,
    started_unix: u64,
}

#[derive(Serialize, Deserialize)]
struct StatusJson {
    region: String,
    chunk: String,
    configs_done: u64,
    best_mid_exact: usize,
    best_tail_exact: usize,
    best_mid_label: String,
    best_tail_label: String,
    named_mid_bar: usize,
    named_tail_bar: usize,
    runtime_secs: u64,
    max_hours: f64,
    threads: usize,
    stop_requested: bool,
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
fn write_atomic(path: &Path, text: &str) {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, text).unwrap();
    fs::rename(&tmp, path).unwrap();
}
fn wall_secs(ck: &Checkpoint) -> u64 {
    now_unix().saturating_sub(ck.started_unix)
}
fn timed_out(ck: &Checkpoint, max_hours: f64) -> bool {
    wall_secs(ck) as f64 / 3600.0 >= max_hours
}
fn stop_requested(out: &Path) -> bool {
    out.join("STOP").exists()
}

fn load_ckpt(out: &Path) -> Checkpoint {
    let p = out.join("checkpoint.json");
    if let Ok(t) = fs::read_to_string(&p) {
        if let Ok(c) = serde_json::from_str(&t) {
            return c;
        }
    }
    Checkpoint {
        progress: BTreeMap::new(),
        best_mid_exact: 0,
        best_tail_exact: 0,
        best_mid_label: String::new(),
        best_tail_label: String::new(),
        configs_done: 0,
        started_unix: now_unix(),
    }
}

fn consider(ck: &mut Checkpoint, out: &Path, label: &str, mid: usize, tail: usize) {
    if mid > ck.best_mid_exact {
        ck.best_mid_exact = mid;
        ck.best_mid_label = label.to_string();
        let _ = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(out.join("leaders.jsonl"))
            .and_then(|mut f| writeln!(f, "{{\"mid\":{mid},\"tail\":{tail},\"label\":{label:?}}}"));
    }
    if tail > ck.best_tail_exact {
        ck.best_tail_exact = tail;
        ck.best_tail_label = label.to_string();
        let _ = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(out.join("leaders.jsonl"))
            .and_then(|mut f| writeln!(f, "{{\"mid\":{mid},\"tail\":{tail},\"label\":{label:?}}}"));
        if tail > TAIL_BAR {
            let _ = fs::write(
                out.join("HIT_TAIL"),
                format!("{label} tail={tail} bar={TAIL_BAR}\n"),
            );
        }
    }
    if mid > MID_BAR {
        let _ = fs::write(
            out.join("HIT_MID"),
            format!("{label} mid={mid} bar={MID_BAR}\n"),
        );
    }
}

fn write_status(
    out: &Path,
    ck: &Checkpoint,
    jobs: &[CubeJob],
    region: &str,
    chunk: &str,
    max_hours: f64,
    threads: usize,
    extra: &str,
) {
    let hours = wall_secs(ck) as f64 / 3600.0;
    let mut map = String::from("# F-body campaign REGION_MAP\n\n| id | space | next | note |\n|---|---|---|---|\n");
    for j in jobs {
        let v = ck.progress.get(&j.axis).copied().unwrap_or(0);
        let next = if v >= j.mask_lim {
            "done".into()
        } else {
            format!("0x{v:07x}/0x{:07x}", j.mask_lim)
        };
        map.push_str(&format!(
            "| {} | 0..0x{:07x} | {next} | {} |\n",
            j.axis, j.mask_lim, j.note
        ));
    }
    map.push_str(&format!(
        "\nbest_mid {} / bar {MID_BAR} `{}`\nbest_tail {} / bar {TAIL_BAR} `{}`\n",
        ck.best_mid_exact, ck.best_mid_label, ck.best_tail_exact, ck.best_tail_label
    ));
    let md = format!(
        "# F-body campaign STATUS\n\n\
         - region: `{region}`\n- chunk: `{chunk}`\n- configs_done: {}\n\
         - best_mid: {} / bar {MID_BAR} `{}`\n\
         - best_tail: {} / bar {TAIL_BAR} `{}`\n\
         - runtime_hours: {hours:.2} / {max_hours}\n- threads: {threads}\n\
         - stop_requested: {}\n\n{extra}\n\n{map}\n",
        ck.configs_done,
        ck.best_mid_exact,
        ck.best_mid_label,
        ck.best_tail_exact,
        ck.best_tail_label,
        stop_requested(out)
    );
    write_atomic(&out.join("STATUS.md"), &md);
    write_atomic(&out.join("REGION_MAP.md"), &map);
    let sj = StatusJson {
        region: region.into(),
        chunk: chunk.into(),
        configs_done: ck.configs_done,
        best_mid_exact: ck.best_mid_exact,
        best_tail_exact: ck.best_tail_exact,
        best_mid_label: ck.best_mid_label.clone(),
        best_tail_label: ck.best_tail_label.clone(),
        named_mid_bar: MID_BAR,
        named_tail_bar: TAIL_BAR,
        runtime_secs: wall_secs(ck),
        max_hours,
        threads,
        stop_requested: stop_requested(out),
    };
    write_atomic(
        &out.join("status.json"),
        &serde_json::to_string_pretty(&sj).unwrap(),
    );
    write_atomic(
        &out.join("checkpoint.json"),
        &serde_json::to_string_pretty(ck).unwrap(),
    );
}

fn parse_args() -> (String, PathBuf, usize, f64, Vec<String>) {
    let mut dir = "../../work/w109/G3-01-dist".into();
    let mut out = PathBuf::from("../../work/w109/erfc-fbody-campaign");
    let mut threads = 12usize;
    let mut max_hours = 96.0;
    let mut only = Vec::new();
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--dir" => dir = it.next().expect("--dir"),
            "--out" => out = PathBuf::from(it.next().expect("--out")),
            "--threads" => threads = it.next().unwrap().parse().unwrap(),
            "--max-hours" => max_hours = it.next().unwrap().parse().unwrap(),
            "--only" => {
                only = it
                    .next()
                    .expect("--only")
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
            _ => {}
        }
    }
    assert!(!dir.contains("heldout"));
    (dir, out, threads, max_hours, only)
}

fn selected(axis: &str, only: &[String]) -> bool {
    only.is_empty() || only.iter().any(|p| axis == p || axis.starts_with(&format!("{p}/")))
}

fn main() {
    let (dir, out, threads, max_hours, only) = parse_args();
    fs::create_dir_all(&out).unwrap();
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .ok();
    let rows = f::load_q_rows(&dir);
    let jobs = cube_jobs();
    let mut ck = load_ckpt(&out);
    if ck.started_unix == 0 {
        ck.started_unix = now_unix();
    }

    if ck.progress.get("R0").copied().unwrap_or(0) == 0 && selected("R0", &only) {
        let mut extra = String::from("## R0 named F under w_rn53\n\n");
        for (name, eval) in [
            ("nswc_derfc0", f::nswc_derfc0 as fn(f64) -> f64),
            ("cody_erfcx", f::cody_erfcx_f),
            ("cephes_erfce", f::cephes_f),
            ("cdflib_erfc1", f::cdflib_erfc1_f),
            ("cf_as714_n80", f::cf_as714_f),
            ("cf_gautschi_n80", f::cf_gautschi_f),
        ] {
            let (m, t) = f::score_f(&rows, eval);
            extra.push_str(&format!(
                "- {name}: mid {} tail {}\n",
                f::fmt_acc(&m),
                f::fmt_acc(&t)
            ));
            consider(&mut ck, &out, name, m.exact, t.exact);
        }
        extra.push_str("\n## piecewise NSWC / CF-as714-n80\n");
        for k in 0..=80 {
            let cut = 0.5 + k as f64 * 0.1;
            if cut > 8.5 {
                break;
            }
            let (m, t) = f::score_f(&rows, |z| {
                if z < cut {
                    f::nswc_derfc0(z)
                } else {
                    f::cf_as714_f(z)
                }
            });
            extra.push_str(&format!(
                "- cut={cut:.1}: mid {} tail {}\n",
                f::fmt_acc(&m),
                f::fmt_acc(&t)
            ));
            consider(
                &mut ck,
                &out,
                &format!("piece/cut={cut:.1}"),
                m.exact,
                t.exact,
            );
        }
        write_atomic(&out.join("R0.md"), &extra);
        ck.progress.insert("R0".into(), 1);
        write_status(&out, &ck, &jobs, "R0", "done", max_hours, threads, &extra);
    }

    for job in &jobs {
        if timed_out(&ck, max_hours) || stop_requested(&out) {
            break;
        }
        if !selected(&job.axis, &only) {
            continue;
        }
        let mut c0 = ck.progress.get(&job.axis).copied().unwrap_or(0);
        while c0 < job.mask_lim {
            if timed_out(&ck, max_hours) || stop_requested(&out) {
                write_status(
                    &out,
                    &ck,
                    &jobs,
                    &job.axis,
                    &format!("stop@{c0:07x}"),
                    max_hours,
                    threads,
                    &job.note,
                );
                return;
            }
            let c1 = (c0 + CHUNK).min(job.mask_lim);
            let scores: Vec<(u32, usize, usize)> = (c0..c1)
                .into_par_iter()
                .map(|mask| {
                    let (m, t) = f::score_f(&rows, |z| eval_job(job, mask, z));
                    (mask, m.exact, t.exact)
                })
                .collect();
            ck.configs_done += scores.len() as u64;
            for (mask, mid, tail) in &scores {
                consider(
                    &mut ck,
                    &out,
                    &format!("{} /mask={mask:07x}", job.axis),
                    *mid,
                    *tail,
                );
            }
            c0 = c1;
            ck.progress.insert(job.axis.clone(), c0);
            let chunk_best_tail = scores.iter().map(|s| s.2).max().unwrap_or(0);
            write_status(
                &out,
                &ck,
                &jobs,
                &job.axis,
                &format!("{c0:07x}"),
                max_hours,
                threads,
                &format!("{}\nchunk_best_tail={chunk_best_tail}", job.note),
            );
        }
    }

    let exhausted = jobs
        .iter()
        .filter(|j| selected(&j.axis, &only))
        .all(|j| ck.progress.get(&j.axis).copied().unwrap_or(0) >= j.mask_lim);
    write_status(
        &out,
        &ck,
        &jobs,
        if exhausted { "exit-regions" } else { "exit" },
        "-",
        max_hours,
        threads,
        if exhausted {
            "selected CF cubes finished"
        } else {
            "exiting (time/STOP). resume = same command"
        },
    );
}
