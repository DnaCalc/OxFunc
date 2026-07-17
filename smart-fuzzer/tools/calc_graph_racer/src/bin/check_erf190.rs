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
    gam1_ext: bool,   // rational evaluated extended (vs per-op double)
    gam1_ret_dbl: bool,
    g_dbl: bool,
    w_dbl: bool,
    wg_first: bool,   // (w*g)*inner vs w*(g*inner)
    inner_dbl: bool,  // 0.5+(0.5-j) rounded to double
}

fn gam1_half(cfg: &Cfg) -> Ext80 {
    let t = ef(0.5);
    let sp = |v: Ext80| -> Ext80 {
        if cfg.gam1_ext { v } else { ef(dbl(&v)) }
    };
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

fn erf190(z: f64, cfg: &Cfg) -> f64 {
    let a = ef(0.5);
    let mut x = ext_mul(&ef(z), &ef(z), CW);
    if cfg.zz_dbl {
        x = ef(dbl(&x));
    }
    if dbl(&x) == 0.0 {
        return 0.0;
    }
    let sp = |v: Ext80| -> Ext80 {
        if cfg.series_dbl { ef(dbl(&v)) } else { v }
    };
    let mut an = ef(3.0);
    let mut c = x;
    let mut sum = sp(ext_div(&x, &ext_add(&a, &ef(3.0), CW), CW));
    let tol = ext_div(&ext_mul(&ef(3.0), &ef(5e-15), CW), &ext_add(&a, &ext_one(), CW), CW);
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
            &ext_sub(&ext_div(&sum, &ef(6.0), CW), &ext_div(&ef(0.5), &ext_add(&a, &ef(2.0), CW), CW), CW),
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
    let ans = if cfg.wg_first {
        ext_mul(&ext_mul(&w, &g, CW), &inner, CW)
    } else {
        ext_mul(&w, &ext_mul(&g, &inner, CW), CW)
    };
    dbl(&ans)
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
    let ans_name = std::env::args().nth(3).unwrap_or_else(|| "answers-b11.json".into());
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
        // recompute V as Ext80 (mirror erf190 without the final dbl)
        let a = ef(0.5);
        let x = ext_mul(&ef(z), &ef(z), CW);
        let sp = |v: Ext80| -> Ext80 { v };
        let mut an = ef(3.0);
        let mut c = x;
        let mut sum = sp(ext_div(&x, &ext_add(&a, &ef(3.0), CW), CW));
        let tol = ext_div(&ext_mul(&ef(3.0), &ef(5e-15), CW), &ext_add(&a, &ext_one(), CW), CW);
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
                &ext_sub(&ext_div(&sum, &ef(6.0), CW), &ext_div(&ef(0.5), &ext_add(&a, &ef(2.0), CW), CW), CW),
                &x,
                CW,
            ),
            &ext_div(&ext_one(), &ext_add(&a, &ext_one(), CW), CW),
            CW,
        );
        let j = ext_mul(&ext_mul(&a, &x, CW), &inner_poly, CW);
        let zl = ext_mul(&a, &ln_ext(&x), CW);
        let h = gam1_half(&cfg);
        let g = ext_add(&ext_one(), &h, CW);
        let wv = exp_ext(&zl);
        let inner = ext_add(&ef(0.5), &ext_sub(&ef(0.5), &j, CW), CW);
        let v_ext = ext_mul(&ext_mul(&wv, &g, CW), &inner, CW);
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

fn main() {
    let dir = std::env::args().nth(1).expect("work dir");
    if std::env::args().nth(2).as_deref() == Some("dump") {
        dump_ladder(&dir);
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
