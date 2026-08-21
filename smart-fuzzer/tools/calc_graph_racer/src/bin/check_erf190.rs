//! W109 G3-01 erf sub-lane: race the NSWC gratio a<1 DIRECT path (branch 190)
//! with TRUE x87 chains (Ext80 fFEXP/fFLN) as Excel's ERF.PRECISE for z < 0.5.
//!
//!   x  = z*z                       (spill axis)
//!   series: an=3; c=x; sum=x/(a+3); loop{an+=1; c=-c*(x/an); t=c/(a+an); sum+=t}
//!   j  = a*x*((sum/6 - 0.5/(a+2))*x + 1/(a+1))
//!   zl = a*ln(x)                   (fFLN; spill axis)
//!   h  = gam1(a); g = 1+h          (NSWC rational; eval/return axes)
//!   w  = exp(zl)                   (fFEXP; spill axis)
//!   ans = w*g*(0.5+(0.5-j))        (association axis)
//!
//! Usage: check_erf190 <work-dir>

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    CW_PC64_RN, Ext80, ext_abs, ext_add, ext_chs, ext_div, ext_f2xm1, ext_from_f64, ext_fyl2x,
    ext_l2e, ext_ln2, ext_mul, ext_one, ext_rndint, ext_scale, ext_sub, ext_to_f64,
};
use std::collections::BTreeMap;

const CW: u16 = CW_PC64_RN;

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn dbl(x: &Ext80) -> f64 {
    ext_to_f64(x, CW)
}

fn exp_ext(x: &Ext80) -> Ext80 {
    let t = ext_mul(x, &ext_l2e(), CW);
    let k = ext_rndint(&t, CW);
    let f = ext_sub(&t, &k, CW);
    let neg = dbl(&f) < 0.0;
    let w = ext_f2xm1(&ext_abs(&f, CW), CW);
    let mut m = ext_add(&w, &ext_one(), CW);
    if neg {
        m = ext_div(&ext_one(), &m, CW);
    }
    ext_scale(&m, &k, CW)
}

fn ln_ext(x: &Ext80) -> Ext80 {
    ext_fyl2x(&ext_ln2(), x, CW)
}

fn ext_le(a: &Ext80, b: &Ext80) -> bool {
    dbl(&ext_sub(a, b, CW)) <= 0.0
}

const GP: [f64; 7] = [
    0.577215664901533e+00,
    -0.409078193005776e+00,
    -0.230975380857675e+00,
    0.597275330452234e-01,
    0.766968181649490e-02,
    -0.514889771323592e-02,
    0.589597428611429e-03,
];
const GQ: [f64; 5] = [
    1.0,
    0.427569613095214e+00,
    0.158451672430138e+00,
    0.261132021441447e-01,
    0.423244297896961e-02,
];

#[derive(Clone, Copy, Debug)]
struct Cfg {
    zz_dbl: bool,
    series_dbl: bool,
    j_dbl: bool,
    zl_dbl: bool,
    gam1_ext: bool, // rational evaluated extended (vs per-op double)
    gam1_ret_dbl: bool,
    g_dbl: bool,
    w_dbl: bool,
    wg_first: bool,  // (w*g)*inner vs w*(g*inner)
    inner_dbl: bool, // 0.5+(0.5-j) rounded to double
}

fn gam1_half(cfg: &Cfg) -> Ext80 {
    let t = ef(0.5);
    let sp = |v: Ext80| -> Ext80 { if cfg.gam1_ext { v } else { ef(dbl(&v)) } };
    let mut top = ef(GP[6]);
    for &c in GP[..6].iter().rev() {
        top = sp(ext_add(&ext_mul(&top, &t, CW), &ef(c), CW));
    }
    let mut bot = ef(GQ[4]);
    for &c in GQ[..4].iter().rev() {
        bot = sp(ext_add(&ext_mul(&bot, &t, CW), &ef(c), CW));
    }
    let w = sp(ext_div(&top, &bot, CW));
    let r = sp(ext_mul(&ef(0.5), &w, CW));
    if cfg.gam1_ret_dbl { ef(dbl(&r)) } else { r }
}

fn erf190_ext(z: f64, cfg: &Cfg) -> Ext80 {
    let a = ef(0.5);
    let mut x = ext_mul(&ef(z), &ef(z), CW);
    if cfg.zz_dbl {
        x = ef(dbl(&x));
    }
    if dbl(&x) == 0.0 {
        return ef(0.0);
    }
    let sp = |v: Ext80| -> Ext80 { if cfg.series_dbl { ef(dbl(&v)) } else { v } };
    let mut an = ef(3.0);
    let mut c = x;
    let mut sum = sp(ext_div(&x, &ext_add(&a, &ef(3.0), CW), CW));
    let tol = ext_div(
        &ext_mul(&ef(3.0), &ef(5e-15), CW),
        &ext_add(&a, &ext_one(), CW),
        CW,
    );
    for _ in 0..200 {
        an = sp(ext_add(&an, &ext_one(), CW));
        c = sp(ext_chs(&ext_mul(&c, &ext_div(&x, &an, CW), CW), CW));
        let t = sp(ext_div(&c, &ext_add(&a, &an, CW), CW));
        sum = sp(ext_add(&sum, &t, CW));
        if ext_le(&ext_abs(&t, CW), &tol) {
            break;
        }
    }
    let inner_poly = ext_add(
        &ext_mul(
            &ext_sub(
                &ext_div(&sum, &ef(6.0), CW),
                &ext_div(&ef(0.5), &ext_add(&a, &ef(2.0), CW), CW),
                CW,
            ),
            &x,
            CW,
        ),
        &ext_div(&ext_one(), &ext_add(&a, &ext_one(), CW), CW),
        CW,
    );
    let mut j = ext_mul(&ext_mul(&a, &x, CW), &inner_poly, CW);
    if cfg.j_dbl {
        j = ef(dbl(&j));
    }
    let mut zl = ext_mul(&a, &ln_ext(&x), CW);
    if cfg.zl_dbl {
        zl = ef(dbl(&zl));
    }
    let h = gam1_half(cfg);
    let mut g = ext_add(&ext_one(), &h, CW);
    if cfg.g_dbl {
        g = ef(dbl(&g));
    }
    let mut w = exp_ext(&zl);
    if cfg.w_dbl {
        w = ef(dbl(&w));
    }
    let mut inner = ext_add(&ef(0.5), &ext_sub(&ef(0.5), &j, CW), CW);
    if cfg.inner_dbl {
        inner = ef(dbl(&inner));
    }
    if cfg.wg_first {
        ext_mul(&ext_mul(&w, &g, CW), &inner, CW)
    } else {
        ext_mul(&w, &ext_mul(&g, &inner, CW), CW)
    }
}

fn erf190(z: f64, cfg: &Cfg) -> f64 {
    dbl(&erf190_ext(z, cfg))
}

fn dump_ladder(dir: &str) {
    // Dump per-row: z_bits, excel_bits, model RN53(V) bits, and V's fractional
    // position within its ulp (units of ulp) for the all-extended config.
    let cfg = Cfg {
        zz_dbl: false,
        series_dbl: false,
        j_dbl: false,
        zl_dbl: false,
        gam1_ext: true,
        gam1_ret_dbl: true,
        g_dbl: false,
        w_dbl: false,
        wg_first: true,
        inner_dbl: false,
    };
    let ans_name = std::env::args()
        .nth(3)
        .unwrap_or_else(|| "answers-b11.json".into());
    let bits: u32 = std::env::args()
        .nth(4)
        .map(|s| s.parse().unwrap())
        .unwrap_or(304);
    let cfg = Cfg {
        zz_dbl: bits & 1 != 0,
        series_dbl: bits & 2 != 0,
        j_dbl: bits & 4 != 0,
        zl_dbl: bits & 8 != 0,
        gam1_ext: bits & 16 != 0,
        gam1_ret_dbl: bits & 32 != 0,
        g_dbl: bits & 64 != 0,
        w_dbl: bits & 128 != 0,
        wg_first: bits & 256 != 0,
        inner_dbl: bits & 512 != 0,
    };
    let txt = std::fs::read_to_string(format!("{dir}/{ans_name}")).unwrap();
    let ws: WitnessSet = serde_json::from_str(&txt).unwrap();
    for w in &ws.witnesses {
        let Some(id) = &w.id else { continue };
        let z = match &w.args[0] {
            WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
            _ => continue,
        };
        let Some(excel) = parse_bits_hex(&w.expected_bits) else {
            continue;
        };
        let v_ext = erf190_ext(z, &cfg);
        let v53 = dbl(&v_ext);
        // fractional position: (V - RN53(V)) / ulp(RN53(V))
        let frac_num = ext_sub(&v_ext, &ef(v53), CW);
        let ulp = {
            let bits = v53.to_bits();
            let next = f64::from_bits(bits + 1);
            next - v53
        };
        let phase = dbl(&frac_num) / ulp;
        println!(
            "{id} {:016x} {:016x} {:016x} {phase:+.6}",
            z.to_bits(),
            excel.to_bits(),
            v53.to_bits()
        );
    }
}

/// Lane-6 merge test: slide the chain argument zl in ulp64 steps through the
/// C10r pipeline; does SOME argument reproduce Excel's published bits?
/// C10r (agentJ resting state): x = ext(z*z); series/j at RN53;
/// inner = RN53(0.5 + RN53(0.5 - j)); g = g_x (pinned 64-bit mantissa
/// 0x906eba8214db6c6f, exp 0x3FFF); w = chain(zl_j) extended;
/// ans = RN53( ext( w * ext(g_x * inner) ) ).
fn jscan(dir: &str) {
    const J: i64 = 240;
    let g_x = {
        let m: u64 = 0x906e_ba82_14db_6c6f;
        let mut b = [0u8; 10];
        b[..8].copy_from_slice(&m.to_le_bytes());
        b[8] = 0xFF;
        b[9] = 0x3F; // sign 0, biased exp 0x3FFF (value in [1,2))
        Ext80(b)
    };
    let mut rows: BTreeMap<u64, u64> = BTreeMap::new();
    for name in [
        "answers-b9train.json",
        "answers-erfp.json",
        "answers-erfm.json",
        "answers-b8erf.json",
        "answers-b7erf.json",
        "answers-b11.json",
        "answers-b10.json",
    ] {
        let Ok(txt) = std::fs::read_to_string(format!("{dir}/{name}")) else {
            continue;
        };
        let ws: WitnessSet = serde_json::from_str(&txt).unwrap();
        for w in &ws.witnesses {
            let z = match &w.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
                _ => continue,
            };
            let Some(expected) = parse_bits_hex(&w.expected_bits) else {
                continue;
            };
            if z > 0.0 && z < 0.5 {
                rows.insert(z.to_bits(), expected.to_bits());
            }
        }
    }
    println!(
        "{} distinct z<0.5 dev rows (b9heldout untouched)",
        rows.len()
    );

    let cfg = Cfg {
        zz_dbl: false,
        series_dbl: true,
        j_dbl: true,
        zl_dbl: false,
        gam1_ext: true,
        gam1_ret_dbl: false,
        g_dbl: false,
        w_dbl: false,
        wg_first: false,
        inner_dbl: false,
    };
    let (mut no_window, mut hit0, mut windowed) = (0u32, 0u32, 0u32);
    let mut centers: Vec<f64> = Vec::new();
    let mut widths: Vec<i64> = Vec::new();
    for (&zb, &eb) in &rows {
        let z = f64::from_bits(zb);
        // pipeline up to zl (C10r): x extended, series RN53, j RN53
        let a = ef(0.5);
        let x = ext_mul(&ef(z), &ef(z), CW);
        if dbl(&x) == 0.0 {
            continue;
        }
        let sp = |v: Ext80| -> Ext80 { ef(dbl(&v)) };
        let mut an = ef(3.0);
        let mut c = x;
        let mut sum = sp(ext_div(&x, &ext_add(&a, &ef(3.0), CW), CW));
        let tol = ext_div(
            &ext_mul(&ef(3.0), &ef(5e-15), CW),
            &ext_add(&a, &ext_one(), CW),
            CW,
        );
        for _ in 0..200 {
            an = sp(ext_add(&an, &ext_one(), CW));
            c = sp(ext_chs(&ext_mul(&c, &ext_div(&x, &an, CW), CW), CW));
            let t = sp(ext_div(&c, &ext_add(&a, &an, CW), CW));
            sum = sp(ext_add(&sum, &t, CW));
            if ext_le(&ext_abs(&t, CW), &tol) {
                break;
            }
        }
        let inner_poly = ext_add(
            &ext_mul(
                &ext_sub(
                    &ext_div(&sum, &ef(6.0), CW),
                    &ext_div(&ef(0.5), &ext_add(&a, &ef(2.0), CW), CW),
                    CW,
                ),
                &x,
                CW,
            ),
            &ext_div(&ext_one(), &ext_add(&a, &ext_one(), CW), CW),
            CW,
        );
        let j_d = dbl(&ext_mul(&ext_mul(&a, &x, CW), &inner_poly, CW));
        let inner53 = 0.5 + (0.5 - j_d); // RN53(0.5 + RN53(0.5 - j))
        let gi = ext_mul(&g_x, &ef(inner53), CW); // RN64
        let zl0 = ext_mul(&a, &ln_ext(&x), CW); // extended argument
        // ulp64 of zl: 2^(exp-63)
        let zl_d = dbl(&zl0);
        let u64_step = {
            let e = zl_d.abs().log2().floor() as i32;
            (2.0_f64).powi(e - 63)
        };
        let mut js: Vec<i64> = Vec::new();
        for j in -J..=J {
            let zj = ext_add(&zl0, &ef(j as f64 * u64_step), CW);
            let w = exp_ext(&zj);
            let ans = dbl(&ext_mul(&w, &gi, CW));
            if ans.to_bits() == eb {
                js.push(j);
            }
        }
        if js.is_empty() {
            no_window += 1;
        } else {
            windowed += 1;
            if js.contains(&0) {
                hit0 += 1;
            }
            centers.push(0.5 * (js[0] + js[js.len() - 1]) as f64);
            widths.push(js[js.len() - 1] - js[0]);
        }
    }
    let n = no_window + windowed;
    println!(
        "windowed {windowed}/{n} ({:.1}%)  no-window {no_window} ({:.1}%)  j=0 exact {hit0} ({:.1}%)",
        100.0 * windowed as f64 / n as f64,
        100.0 * no_window as f64 / n as f64,
        100.0 * hit0 as f64 / n as f64
    );
    centers.sort_by(|a, b| a.partial_cmp(b).unwrap());
    widths.sort();
    if !centers.is_empty() {
        let m = centers.len();
        println!(
            "centers: min {:.0} q1 {:.0} med {:.0} q3 {:.0} max {:.0}",
            centers[0],
            centers[m / 4],
            centers[m / 2],
            centers[3 * m / 4],
            centers[m - 1]
        );
        println!(
            "widths:  min {} q1 {} med {} q3 {} max {}",
            widths[0],
            widths[m / 4],
            widths[m / 2],
            widths[3 * m / 4],
            widths[m - 1]
        );
        // center histogram, 24-ulp64 bins
        let mut hist: BTreeMap<i64, u32> = BTreeMap::new();
        for c in &centers {
            *hist.entry((c / 24.0).floor() as i64 * 24).or_default() += 1;
        }
        println!("center histogram (24-ulp64 bins): {:?}", hist);
    }
}

/// Lane-6c joint solve, erf side: end-to-end C10r variants differing ONLY in
/// sub-double composition of the chain argument:
///   V0  x extended into ln (C10r baseline)
///   V1  x SPILLED (RN53) into ln, extended x kept for the series
///   V2  x spilled into ln AND series (uniform, = zz_dbl visible axis)
///   V3  x extended into ln, spilled into series
fn hyp_race(dir: &str) {
    let g_x = {
        let m: u64 = 0x906e_ba82_14db_6c6f;
        let mut b = [0u8; 10];
        b[..8].copy_from_slice(&m.to_le_bytes());
        b[8] = 0xFF;
        b[9] = 0x3F;
        Ext80(b)
    };
    let mut rows: BTreeMap<u64, u64> = BTreeMap::new();
    for name in [
        "answers-b9train.json",
        "answers-erfp.json",
        "answers-erfm.json",
        "answers-b8erf.json",
        "answers-b7erf.json",
        "answers-b11.json",
        "answers-b10.json",
    ] {
        let Ok(txt) = std::fs::read_to_string(format!("{dir}/{name}")) else {
            continue;
        };
        let ws: WitnessSet = serde_json::from_str(&txt).unwrap();
        for w in &ws.witnesses {
            let z = match &w.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
                _ => continue,
            };
            let Some(expected) = parse_bits_hex(&w.expected_bits) else {
                continue;
            };
            if z > 0.0 && z < 0.5 {
                rows.insert(z.to_bits(), expected.to_bits());
            }
        }
    }
    for (vname, ln_spill, ser_spill) in [
        ("V0_ext/ext", false, false),
        ("V1_ln53/serE", true, false),
        ("V2_ln53/ser53", true, true),
        ("V3_lnE/ser53", false, true),
    ] {
        let mut exact = 0u32;
        let mut n = 0u32;
        for (&zb, &eb) in &rows {
            let z = f64::from_bits(zb);
            let a = ef(0.5);
            let x_ext = ext_mul(&ef(z), &ef(z), CW);
            if dbl(&x_ext) == 0.0 {
                continue;
            }
            n += 1;
            let x_ln = if ln_spill { ef(dbl(&x_ext)) } else { x_ext };
            let x_sr = if ser_spill { ef(dbl(&x_ext)) } else { x_ext };
            let sp = |v: Ext80| -> Ext80 { ef(dbl(&v)) };
            let mut an = ef(3.0);
            let mut c = x_sr;
            let mut sum = sp(ext_div(&x_sr, &ext_add(&a, &ef(3.0), CW), CW));
            let tol = ext_div(
                &ext_mul(&ef(3.0), &ef(5e-15), CW),
                &ext_add(&a, &ext_one(), CW),
                CW,
            );
            for _ in 0..200 {
                an = sp(ext_add(&an, &ext_one(), CW));
                c = sp(ext_chs(&ext_mul(&c, &ext_div(&x_sr, &an, CW), CW), CW));
                let t = sp(ext_div(&c, &ext_add(&a, &an, CW), CW));
                sum = sp(ext_add(&sum, &t, CW));
                if ext_le(&ext_abs(&t, CW), &tol) {
                    break;
                }
            }
            let inner_poly = ext_add(
                &ext_mul(
                    &ext_sub(
                        &ext_div(&sum, &ef(6.0), CW),
                        &ext_div(&ef(0.5), &ext_add(&a, &ef(2.0), CW), CW),
                        CW,
                    ),
                    &x_sr,
                    CW,
                ),
                &ext_div(&ext_one(), &ext_add(&a, &ext_one(), CW), CW),
                CW,
            );
            let j_d = dbl(&ext_mul(&ext_mul(&a, &x_sr, CW), &inner_poly, CW));
            let inner53 = 0.5 + (0.5 - j_d);
            let gi = ext_mul(&g_x, &ef(inner53), CW);
            let zl = ext_mul(&a, &ln_ext(&x_ln), CW);
            let w = exp_ext(&zl);
            let ans = dbl(&ext_mul(&w, &gi, CW));
            if ans.to_bits() == eb {
                exact += 1;
            }
        }
        println!(
            "{vname:14} {exact:4}/{n} ({:.1}%)",
            100.0 * exact as f64 / n as f64
        );
    }
}

fn main() {
    let dir = std::env::args().nth(1).expect("work dir");
    if std::env::args().nth(2).as_deref() == Some("dump") {
        dump_ladder(&dir);
        return;
    }
    if std::env::args().nth(2).as_deref() == Some("jscan") {
        jscan(&dir);
        return;
    }
    if std::env::args().nth(2).as_deref() == Some("hyp") {
        hyp_race(&dir);
        return;
    }
    let mut rows: BTreeMap<u64, f64> = BTreeMap::new();
    for name in [
        "answers-b9train.json",
        "answers-erfp.json",
        "answers-erfm.json",
        "answers-b8erf.json",
        "answers-b7erf.json",
        "answers-b11.json",
        "answers-b10.json",
    ] {
        let Ok(txt) = std::fs::read_to_string(format!("{dir}/{name}")) else {
            continue;
        };
        let ws: WitnessSet = serde_json::from_str(&txt).unwrap();
        for w in &ws.witnesses {
            let z = match &w.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
                _ => continue,
            };
            let Some(expected) = parse_bits_hex(&w.expected_bits) else {
                continue;
            };
            if z > 0.0 && z < 0.5 {
                rows.insert(z.to_bits(), expected);
            }
        }
    }
    let rows: Vec<(f64, u64)> = rows
        .iter()
        .map(|(zb, e)| (f64::from_bits(*zb), e.to_bits()))
        .collect();
    println!("{} distinct z<0.5 rows", rows.len());

    let mut results: Vec<(usize, Cfg, Vec<(f64, i64)>)> = Vec::new();
    for bits in 0u32..1024 {
        let cfg = Cfg {
            zz_dbl: bits & 1 != 0,
            series_dbl: bits & 2 != 0,
            j_dbl: bits & 4 != 0,
            zl_dbl: bits & 8 != 0,
            gam1_ext: bits & 16 != 0,
            gam1_ret_dbl: bits & 32 != 0,
            g_dbl: bits & 64 != 0,
            w_dbl: bits & 128 != 0,
            wg_first: bits & 256 != 0,
            inner_dbl: bits & 512 != 0,
        };
        let mut exact = 0usize;
        let mut misses = Vec::new();
        for &(z, eb) in &rows {
            let v = erf190(z, &cfg).to_bits();
            if v == eb {
                exact += 1;
            } else if misses.len() < 400 {
                fn key(i: u64) -> i64 {
                    let i = i as i64;
                    if i < 0 { !i } else { i }
                }
                misses.push((z, key(eb) - key(v)));
            }
        }
        results.push((exact, cfg, misses));
    }
    results.sort_by(|a, b| b.0.cmp(&a.0));
    println!("top configs:");
    for (exact, cfg, _) in results.iter().take(8) {
        println!("  {exact:4}/{} {:?}", rows.len(), cfg);
    }
    let (exact, cfg, misses) = &results[0];
    println!("\nbest: {exact}/{} {:?}", rows.len(), cfg);
    print!("miss pattern (first 40): ");
    for (z, d) in misses.iter().take(40) {
        print!("{d:+}@{z:.4e} ");
    }
    println!();
}
