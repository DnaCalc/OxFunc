//! Firehorse ERFC-body campaign: named F, then NSWC/Cody Horner-stage
//! store-mask cubes. Checkpoints after every chunk. Heldouts unnamed.
//!
//! Usage:
//!   campaign_erfc_body --dir G3-01-dist --out erfc-campaign --threads 12 --max-hours 96

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::special_dist_family::erfc_precise_kernel;
use rayon::prelude::*;
use rx::{
    CW_PC53_RN, CW_PC64_RN, Ext80, excel_exp, ext_add, ext_div, ext_from_f64, ext_mul, ext_sub,
    ext_to_f64,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const FRAC_1_SQRT_2: f64 = f64::from_bits(0x3fe6a09e667f3bcd);
const ERFC_BANKS: [&str; 5] = [
    "answers-erfcp.json",
    "answers-erfcm.json",
    "answers-b7erfc.json",
    "answers-b8erfc.json",
    "answers-b11c.json",
];
const PIN_Z: [f64; 5] = [0.75, 1.28125, 1.875, 2.125, 5.0];
const CHUNK: u32 = 256;
const P: [f64; 8] = [
    0.16506148041280876191828601e-03,
    0.15471455377139313353998665e-03,
    0.44852548090298868465196794e-04,
    -0.49177280017226285450486205e-05,
    -0.69353602078656412367801676e-05,
    -0.20508667787746282746857743e-05,
    -0.28982842617824971177267380e-06,
    -0.17272433544836633301127174e-07,
];
const Q: [f64; 8] = [
    1.0,
    0.16272656776533322859856317e+01,
    0.12040996037066026106794322e+01,
    0.52400246352158386907601472e+00,
    0.14497345252798672362384241e+00,
    0.25592517111042546492590736e-01,
    0.26869088293991371028123158e-02,
    0.13133767840925681614496481e-03,
];
const R: [f64; 9] = [
    0.145589721275038539045668824025,
    -0.273421931495426482902320421863,
    0.226008066916621506788789064272,
    -0.163571895523923805648814425592,
    0.102604312032193978662297299832,
    -0.548023266949835519254211506880e-01,
    0.241432239725390106956523668160e-01,
    -0.822062115403915116036874169600e-02,
    0.180296241564687154310619200000e-02,
];
const CODY_C: [f64; 9] = [
    0.564188496988670089,
    8.88314979438837594,
    66.1191906371416295,
    298.635138197400131,
    881.95222124176909,
    1712.04761263407058,
    2051.07837782607147,
    1230.33935479799725,
    2.15311535474403846e-8,
];
const CODY_D: [f64; 8] = [
    15.7449261107098347,
    117.693950891312499,
    537.181101862009858,
    1621.38957456669019,
    3290.79923573345963,
    4362.61909014324716,
    3439.36767414372164,
    1230.33935480374942,
];
const CODY_SQRPI: f64 = 0.56418958354775628695;

#[derive(Clone, Copy)]
enum Arith {
    Native,
    X87Cont,
    X87Stage,
}

fn flush(v: f64) -> f64 {
    if v.abs() < f64::MIN_POSITIVE { 0.0 } else { v }
}
fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn dbl(x: &Ext80) -> f64 {
    ext_to_f64(x, CW_PC64_RN)
}
fn spill(x: Ext80) -> Ext80 {
    ef(dbl(&x))
}

fn add(a: &Ext80, b: &Ext80, stage: bool) -> Ext80 {
    let v = ext_add(a, b, CW_PC64_RN);
    if stage { spill(v) } else { v }
}
fn mul(a: &Ext80, b: &Ext80, stage: bool) -> Ext80 {
    let v = ext_mul(a, b, CW_PC64_RN);
    if stage { spill(v) } else { v }
}
fn div(a: &Ext80, b: &Ext80, stage: bool) -> Ext80 {
    let v = ext_div(a, b, CW_PC64_RN);
    if stage { spill(v) } else { v }
}

fn horner_native(cs: &[f64], x: f64) -> f64 {
    let mut acc = 0.0;
    for &c in cs.iter().rev() {
        acc = acc * x + c;
    }
    acc
}

fn horner_mask(cs: &[f64], x: &Ext80, mask: u16, bit0: u32, stage_arith: bool) -> Ext80 {
    let mut acc = ef(*cs.last().unwrap());
    let mut b = bit0;
    for &c in cs.iter().rev().skip(1) {
        acc = add(&mul(&acc, x, stage_arith), &ef(c), stage_arith);
        if mask & (1 << b) != 0 {
            acc = spill(acc);
        }
        b += 1;
    }
    acc
}

fn nswc_native(z: f64) -> f64 {
    let y = z.abs();
    let q = if y <= 1.0 {
        // small series omitted: use libm for R0 contrast only via other graphs
        libm::erfc(y)
    } else {
        let u = horner_native(&P, y);
        let v = horner_native(&Q, y);
        let t = (y - 3.75) / (y + 3.75);
        let mut acc = u / v;
        for &r in R.iter().rev() {
            acc = acc * t + r;
        }
        flush(excel_exp(-(y * y)) * acc)
    };
    flush(if z < 0.0 { 2.0 - q } else { q })
}

fn cody_unsplit_native(z: f64) -> f64 {
    let y = z.abs();
    let f = if y <= 4.0 {
        let mut xnum = CODY_C[8] * y;
        let mut xden = y;
        for i in 0..7 {
            xnum = (xnum + CODY_C[i]) * y;
            xden = (xden + CODY_D[i]) * y;
        }
        (xnum + CODY_C[7]) / (xden + CODY_D[7])
    } else {
        CODY_SQRPI / y
    };
    flush(excel_exp(-(y * y)) * f)
}

fn nswc_mask(z: f64, mask: u16, arith: Arith, zz_dr: bool, store_uv: bool, small_cut: f64) -> f64 {
    let y = z.abs();
    if y <= small_cut {
        return flush(libm::erfc(y));
    }
    let stage = matches!(arith, Arith::X87Stage);
    let ye = ef(y);
    let (u, v) = if matches!(arith, Arith::Native) {
        (ef(horner_native(&P, y)), ef(horner_native(&Q, y)))
    } else {
        (
            horner_mask(&P, &ye, mask, 0, stage),
            horner_mask(&Q, &ye, mask, 8, stage),
        )
    };
    let t = if matches!(arith, Arith::Native) {
        ef((y - 3.75) / (y + 3.75))
    } else {
        div(
            &ext_sub(&ye, &ef(3.75), CW_PC64_RN),
            &ext_add(&ye, &ef(3.75), CW_PC64_RN),
            stage,
        )
    };
    let mut acc = if store_uv {
        spill(div(&u, &v, stage))
    } else if matches!(arith, Arith::Native) {
        ef(dbl(&u) / dbl(&v))
    } else {
        div(&u, &v, stage)
    };
    if !matches!(arith, Arith::Native) {
        for &r in R.iter().rev() {
            acc = add(&mul(&acc, &t, stage), &ef(r), stage);
        }
    } else {
        let tt = dbl(&t);
        let mut a = dbl(&acc);
        for &r in R.iter().rev() {
            a = a * tt + r;
        }
        acc = ef(a);
    }
    let f = dbl(&acc);
    let zz = if zz_dr {
        dbl(&spill(ext_mul(&ye, &ye, CW_PC64_RN)))
    } else {
        y * y
    };
    let q = flush(excel_exp(-zz) * f);
    flush(if z < 0.0 { 2.0 - q } else { q })
}

fn cody_mask(z: f64, mask: u16, arith: Arith) -> f64 {
    let y = z.abs();
    let ye = ef(y);
    let f = if matches!(arith, Arith::Native) {
        let mut xnum = CODY_C[8] * y;
        let mut xden = y;
        for i in 0..7 {
            xnum = (xnum + CODY_C[i]) * y;
            xden = (xden + CODY_D[i]) * y;
        }
        (xnum + CODY_C[7]) / (xden + CODY_D[7])
    } else {
        let stage = matches!(arith, Arith::X87Stage);
        let num = horner_mask(&CODY_C, &ye, mask, 0, stage);
        let den = horner_mask(&CODY_D, &ye, mask, 8, stage);
        dbl(&div(&num, &den, stage))
    };
    flush(excel_exp(-(y * y)) * f)
}

#[derive(Clone, Serialize, Deserialize)]
struct Acc {
    exact: usize,
    n: usize,
    max_ulp: u64,
}

impl Acc {
    fn add(&mut self, d: u64) {
        self.n += 1;
        if d == 0 {
            self.exact += 1;
        } else {
            self.max_ulp = self.max_ulp.max(d);
        }
    }
}

fn score_rows(rows: &[(f64, u64)], eval: impl Fn(f64) -> f64 + Sync) -> (Acc, Acc, Acc, u32) {
    let mut small = Acc { exact: 0, n: 0, max_ulp: 0 };
    let mut mid = Acc { exact: 0, n: 0, max_ulp: 0 };
    let mut tail = Acc { exact: 0, n: 0, max_ulp: 0 };
    let mut pins = 0u32;
    for &(z, exp) in rows {
        let got = eval(z);
        let d = ulp_distance(got, f64::from_bits(exp)).unwrap_or(u64::MAX);
        if z < 0.5 {
            small.add(d);
        } else if z < 4.0 {
            mid.add(d);
        } else {
            tail.add(d);
        }
        if PIN_Z.iter().any(|&p| p == z) && d == 0 {
            pins += 1;
        }
    }
    (small, mid, tail, pins)
}

fn load_rows(dir: &str) -> Vec<(f64, u64)> {
    let mut rows = BTreeMap::new();
    for name in ERFC_BANKS {
        assert!(!name.contains("heldout"));
        let path = format!("{dir}/{name}");
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let bank: WitnessSet = serde_json::from_str(&text).expect(&path);
        for w in &bank.witnesses {
            let z = match &w.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).expect("z"),
                _ => continue,
            };
            let Some(q) = parse_bits_hex(&w.expected_bits) else {
                continue;
            };
            if z.is_finite() && z >= 0.0 {
                rows.entry(z.to_bits()).or_insert(q.to_bits());
            }
        }
    }
    let path = format!("{dir}/answers-b24-normref.json");
    if let Ok(text) = fs::read_to_string(&path) {
        let bank: WitnessSet = serde_json::from_str(&text).expect("normref");
        for w in &bank.witnesses {
            if w.args.len() < 2 {
                continue;
            }
            let x = match &w.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).expect("x"),
                _ => continue,
            };
            let cum = match &w.args[1] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).expect("c"),
                _ => continue,
            };
            if cum != 1.0 || !x.is_sign_negative() {
                continue;
            }
            let Some(ns) = parse_bits_hex(&w.expected_bits) else {
                continue;
            };
            let z = x.abs() * FRAC_1_SQRT_2;
            rows.entry(z.to_bits()).or_insert((ns * 2.0).to_bits());
        }
    }
    rows.into_iter()
        .map(|(z, q)| (f64::from_bits(z), q))
        .collect()
}

#[derive(Serialize, Deserialize, Clone)]
struct Checkpoint {
    /// Next mask to run for each axis key (`R0` done when value>=1;
    /// 16-bit cubes done when value>=0x10000).
    progress: BTreeMap<String, u32>,
    best_exact: usize,
    #[serde(default)]
    best_mid_exact: usize,
    best_label: String,
    configs_done: u64,
    started_unix: u64,
    random_stream: u64,
}

#[derive(Serialize, Deserialize)]
struct StatusJson {
    region: String,
    chunk: String,
    configs_done: u64,
    best_exact: usize,
    best_label: String,
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
    fs::write(&tmp, text).expect("write tmp");
    fs::rename(&tmp, path).expect("rename");
}

fn write_status(
    out: &Path,
    ck: &Checkpoint,
    region: &str,
    chunk: &str,
    runtime_secs: u64,
    max_hours: f64,
    threads: usize,
    stop: bool,
    extra: &str,
) {
    let hours = runtime_secs as f64 / 3600.0;
    let md = format!(
        "# ERFC campaign STATUS\n\n- region: `{region}`\n- chunk: `{chunk}`\n- configs_done: {}\n- best_exact: {}  best_mid: {}  `{best}`\n- runtime_hours: {hours:.2} / {max_hours}\n- threads: {threads}\n- stop_requested: {stop}\n- progress_keys: {}\n\n{extra}\n",
        ck.configs_done,
        ck.best_exact,
        ck.best_mid_exact,
        ck.progress.len(),
        best = ck.best_label,
    );
    write_atomic(&out.join("STATUS.md"), &md);
    let sj = StatusJson {
        region: region.into(),
        chunk: chunk.into(),
        configs_done: ck.configs_done,
        best_exact: ck.best_exact,
        best_label: ck.best_label.clone(),
        runtime_secs,
        max_hours,
        threads,
        stop_requested: stop,
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

fn load_ckpt(out: &Path) -> Checkpoint {
    let p = out.join("checkpoint.json");
    if let Ok(t) = fs::read_to_string(&p) {
        if let Ok(c) = serde_json::from_str(&t) {
            return c;
        }
    }
    Checkpoint {
        progress: BTreeMap::new(),
        best_exact: 0,
        best_mid_exact: 0,
        best_label: String::new(),
        configs_done: 0,
        started_unix: now_unix(),
        random_stream: 0xC0FFEE,
    }
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

fn consider(
    ck: &mut Checkpoint,
    out: &Path,
    label: &str,
    exact: usize,
    mid: bool,
    detail: &str,
) {
    let beat = if mid {
        if exact > ck.best_mid_exact {
            ck.best_mid_exact = exact;
            true
        } else {
            false
        }
    } else if exact > ck.best_exact {
        ck.best_exact = exact;
        true
    } else {
        false
    };
    if beat {
        ck.best_label = label.to_string();
        let _ = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(out.join("leaders.jsonl"))
            .and_then(|mut f| {
                use std::io::Write;
                writeln!(
                    f,
                    "{{\"exact\":{exact},\"label\":{},\"detail\":{}}}",
                    serde_json::to_string(label).unwrap(),
                    serde_json::to_string(detail).unwrap()
                )
            });
    }
}

fn parse_args() -> (String, PathBuf, usize, f64) {
    let mut dir = "../../work/w109/G3-01-dist".to_string();
    let mut out = PathBuf::from("../../work/w109/erfc-campaign");
    let mut threads = 12usize;
    let mut max_hours = 96.0;
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--dir" => dir = it.next().expect("--dir"),
            "--out" => out = PathBuf::from(it.next().expect("--out")),
            "--threads" => threads = it.next().unwrap().parse().unwrap(),
            "--max-hours" => max_hours = it.next().unwrap().parse().unwrap(),
            _ => {}
        }
    }
    assert!(!dir.contains("heldout"));
    (dir, out, threads, max_hours)
}

fn main() {
    let (dir, out, threads, max_hours) = parse_args();
    fs::create_dir_all(&out).unwrap();
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .ok();
    let rows = load_rows(&dir);
    assert!(!rows.is_empty(), "no discovery rows loaded from {dir}");
    let mid: Vec<(f64, u64)> = rows
        .iter()
        .copied()
        .filter(|(z, _)| *z >= 0.5 && *z < 4.0)
        .collect();
    let mut ck = load_ckpt(&out);
    if ck.started_unix == 0 {
        ck.started_unix = now_unix();
    }

    write_status(
        &out,
        &ck,
        "init",
        "-",
        wall_secs(&ck),
        max_hours,
        threads,
        false,
        &format!("rows={} mid={}", rows.len(), mid.len()),
    );

    // ---- R0 named F ----
    if ck.progress.get("R0").copied().unwrap_or(0) == 0 {
        let graphs: [(&str, fn(f64) -> f64); 4] = [
            ("libm::erfc", |z| flush(libm::erfc(z))),
            ("production", |z| {
                flush(erfc_precise_kernel(z).unwrap_or(f64::NAN))
            }),
            ("nswc_native", nswc_native),
            ("cody_unsplit", cody_unsplit_native),
        ];
        let mut extra = String::from("## R0 named F\n\n");
        for (name, f) in graphs {
            let (s, m, t, pins) = score_rows(&rows, f);
            extra.push_str(&format!(
                "- {name}: small {}/{} mid {}/{} tail {}/{} pins {pins}\n",
                s.exact, s.n, m.exact, m.n, t.exact, t.n
            ));
            consider(
                &mut ck,
                &out,
                name,
                s.exact + m.exact + t.exact,
                false,
                &extra,
            );
        }
        ck.progress.insert("R0".into(), 1);
        write_status(
            &out,
            &ck,
            "R0",
            "done",
            wall_secs(&ck),
            max_hours,
            threads,
            false,
            &extra,
        );
    }

    // ---- R1 NSWC 16-bit cube ----
    let ariths = [Arith::X87Cont, Arith::Native, Arith::X87Stage];
    let zz_drs = [false, true];
    let store_uvs = [false, true];
    let cuts = [0.5_f64, 1.0];
    let r1_note = "## R1 NSWC 16-bit Horner-stage store cube\n\nbits[0..7]=P stages, bits[8..15]=Q stages\n";
    'r1: for (ai, arith) in ariths.iter().enumerate() {
        for (zi, zz_dr) in zz_drs.iter().enumerate() {
            for (si, store_uv) in store_uvs.iter().enumerate() {
                for (ci, cut) in cuts.iter().enumerate() {
                    let axis = format!("R1/a{ai}/z{zi}/u{si}/c{ci}");
                    let mut c0 = ck.progress.get(&axis).copied().unwrap_or(0);
                    while c0 < 0x10000 {
                        if timed_out(&ck, max_hours) || stop_requested(&out) {
                            write_status(
                                &out,
                                &ck,
                                "R1",
                                &axis,
                                wall_secs(&ck),
                                max_hours,
                                threads,
                                stop_requested(&out),
                                &format!("{r1_note}\nstopped at {axis} c0={c0:04x}\n"),
                            );
                            break 'r1;
                        }
                        let c1 = (c0 + CHUNK).min(0x10000);
                        let scores: Vec<(u16, usize, u64)> = (c0..c1)
                            .into_par_iter()
                            .map(|m| {
                                let mask = m as u16;
                                let mut exact = 0usize;
                                let mut maxu = 0u64;
                                for &(z, exp) in &mid {
                                    let got =
                                        nswc_mask(z, mask, *arith, *zz_dr, *store_uv, *cut);
                                    let d = ulp_distance(got, f64::from_bits(exp))
                                        .unwrap_or(u64::MAX);
                                    if d == 0 {
                                        exact += 1;
                                    } else {
                                        maxu = maxu.max(d);
                                    }
                                }
                                (mask, exact, maxu)
                            })
                            .collect();
                        ck.configs_done += scores.len() as u64;
                        for (mask, exact, maxu) in &scores {
                            consider(
                                &mut ck,
                                &out,
                                &format!("{axis}/mask={mask:04x}"),
                                *exact,
                                true,
                                &format!("mid_exact={exact} max_ulp={maxu}"),
                            );
                        }
                        c0 = c1;
                        ck.progress.insert(axis.clone(), c0);
                        write_status(
                            &out,
                            &ck,
                            "R1",
                            &format!("{axis}/{c0:04x}"),
                            wall_secs(&ck),
                            max_hours,
                            threads,
                            false,
                            &format!(
                                "{r1_note}\nlast {axis} next={c0:04x} chunk_best {}\n",
                                scores.iter().map(|s| s.1).max().unwrap_or(0)
                            ),
                        );
                    }
                }
            }
        }
    }

    // ---- R2 Cody 16-bit ----
    if !timed_out(&ck, max_hours) && !stop_requested(&out) {
        let axis = "R2";
        let mut c0 = ck.progress.get(axis).copied().unwrap_or(0);
        while c0 < 0x10000 {
            if timed_out(&ck, max_hours) || stop_requested(&out) {
                break;
            }
            let c1 = (c0 + CHUNK).min(0x10000);
            let scores: Vec<(u16, usize)> = (c0..c1)
                .into_par_iter()
                .map(|m| {
                    let mask = m as u16;
                    let mut exact = 0usize;
                    for &(z, exp) in &mid {
                        let got = cody_mask(z, mask, Arith::X87Cont);
                        if ulp_distance(got, f64::from_bits(exp)) == Some(0) {
                            exact += 1;
                        }
                    }
                    (mask, exact)
                })
                .collect();
            ck.configs_done += scores.len() as u64;
            for (mask, exact) in &scores {
                consider(
                    &mut ck,
                    &out,
                    &format!("R2/mask={mask:04x}"),
                    *exact,
                    true,
                    "cody x87 stage-mask",
                );
            }
            c0 = c1;
            ck.progress.insert(axis.into(), c0);
            write_status(
                &out,
                &ck,
                "R2",
                &format!("{c0:04x}"),
                wall_secs(&ck),
                max_hours,
                threads,
                false,
                "R2 Cody C/D Horner-stage 16-bit cube, arith=X87Cont",
            );
        }
    }

    // ---- R1d remaining-time random 32-bit masks on X87Cont / zz native / uv store / cut 0.5
    while !timed_out(&ck, max_hours) && !stop_requested(&out) {
        let mut masks = Vec::with_capacity(CHUNK as usize);
        for _ in 0..CHUNK {
            ck.random_stream = ck
                .random_stream
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1);
            masks.push(ck.random_stream as u16);
        }
        let scores: Vec<(u16, usize, u64)> = masks
            .into_par_iter()
            .map(|mask| {
                let mut exact = 0usize;
                let mut maxu = 0u64;
                for &(z, exp) in &mid {
                    let got = nswc_mask(z, mask, Arith::X87Cont, false, true, 0.5);
                    let d = ulp_distance(got, f64::from_bits(exp)).unwrap_or(u64::MAX);
                    if d == 0 {
                        exact += 1;
                    } else {
                        maxu = maxu.max(d);
                    }
                }
                (mask, exact, maxu)
            })
            .collect();
        ck.configs_done += scores.len() as u64;
        for (mask, exact, maxu) in &scores {
            consider(
                &mut ck,
                &out,
                &format!("R1d/mask={mask:04x}"),
                *exact,
                true,
                &format!("random mid_exact={exact} max_ulp={maxu}"),
            );
        }
        ck.progress
            .insert("R1d".into(), ck.progress.get("R1d").copied().unwrap_or(0) + CHUNK);
        write_status(
            &out,
            &ck,
            "R1d",
            "random",
            wall_secs(&ck),
            max_hours,
            threads,
            false,
            "R1d uniform-random 16-bit NSWC masks (X87Cont, zz native, store uv, cut 0.5) until max-hours or STOP",
        );
    }

    write_status(
        &out,
        &ck,
        "exit",
        "-",
        wall_secs(&ck),
        max_hours,
        threads,
        stop_requested(&out),
        "campaign process exiting (time, STOP, or regions finished). resume = same command.",
    );
}

// silence unused import on non-x87 benches
#[allow(dead_code)]
fn _pc53() -> u16 {
    CW_PC53_RN
}
