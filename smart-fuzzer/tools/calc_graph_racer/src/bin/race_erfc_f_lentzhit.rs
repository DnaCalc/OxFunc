//! Replay firehorse HIT_TAIL lentz/gaut/n21 mask=0x22. Heldouts unnamed. No landing.
//!
//!   cargo run --release --bin race_erfc_f_lentzhit -- ../../work/w109/G3-01-dist

use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};

fn spill(x: Ext80) -> Ext80 {
    ext_from_f64(ext_to_f64(&x, CW_PC64_RN))
}

fn lentz_gaut_n21_mask(x: f64, mask: u32) -> f64 {
    let xe = ext_from_f64(x);
    let one = ext_from_f64(1.0);
    let mut facc = xe;
    let mut c = xe;
    let mut d = ext_from_f64(0.0);
    let mut bit = 0u32;
    for j in 1..=21u32 {
        let a = ext_from_f64(j as f64 * 0.5);
        let den = ext_add(&xe, &ext_mul(&a, &d, CW_PC64_RN), CW_PC64_RN);
        d = ext_div(&one, &den, CW_PC64_RN);
        let cn = ext_add(&xe, &ext_div(&a, &c, CW_PC64_RN), CW_PC64_RN);
        c = cn;
        if mask & (1 << bit) != 0 {
            c = spill(c);
        }
        bit += 1;
        facc = ext_mul(&facc, &ext_mul(&c, &d, CW_PC64_RN), CW_PC64_RN);
    }
    f::RPINV / ext_to_f64(&facc, CW_PC64_RN)
}

fn report(label: &str, rows: &[(f64, u64)], eval: impl Fn(f64) -> f64) {
    let (m, t) = f::score_f(rows, eval);
    println!("  {label:<40} mid {}  tail {}", f::fmt_acc(&m), f::fmt_acc(&t));
}

fn main() {
    let dir = std::env::args()
        .nth(1)
        .expect("G3-01-dist (heldout files are not named)");
    assert!(!dir.contains("heldout"));
    let tagged = f::load_q_rows_tagged(&dir);
    let rows: Vec<(f64, u64)> = tagged.iter().map(|r| (r.z, r.qbits)).collect();
    let direct: Vec<(f64, u64)> = tagged
        .iter()
        .filter(|r| r.direct)
        .map(|r| (r.z, r.qbits))
        .collect();

    println!("## replay lentz/gaut/n21 masks");
    report("mask=0 (named x87 lentz gaut n21)", &rows, |z| {
        lentz_gaut_n21_mask(z, 0)
    });
    report("mask=0x22", &rows, |z| lentz_gaut_n21_mask(z, 0x22));
    report("mask=0xa", &rows, |z| lentz_gaut_n21_mask(z, 0xa));
    report("mask=0x2588 HIT", &rows, |z| lentz_gaut_n21_mask(z, 0x2588));
    report("evenodd x87 n=12", &rows, |z| f::cf_evenodd_as714_x87_n(z, 12));
    report("as714 x87 n=24", &rows, |z| f::cf_as714_x87_n(z, 24));
    report("gaut x87 n=24", &rows, |z| f::cf_gautschi_x87_n(z, 24));

    println!("\n## direct-only tail/mid for mask=0x22");
    report("mask=0x2588 direct", &direct, |z| lentz_gaut_n21_mask(z, 0x2588));
    report("mask=0x22 direct", &direct, |z| lentz_gaut_n21_mask(z, 0x22));
    report("mask=0 direct", &direct, |z| lentz_gaut_n21_mask(z, 0));
    report("evenodd x87 n12 direct", &direct, |z| {
        f::cf_evenodd_as714_x87_n(z, 12)
    });

    println!("\n## pins vs mask=0x22");
    for &z in &f::PIN_Z {
        let Some(r) = tagged.iter().find(|r| r.z == z) else {
            println!("  z={z} not in banks");
            continue;
        };
        let Some(fo) = f::f_or(z, r.qbits) else {
            continue;
        };
        let fg = lentz_gaut_n21_mask(z, 0x22);
        let d = ulp_distance(fg, fo).unwrap_or(u64::MAX);
        println!("  z={z} direct={} ulp={d}", r.direct as u8);
    }

    println!("\n## extra exact tail rows vs evenodd n12 (implied vs direct)");
    let mut extra_d = 0usize;
    let mut extra_i = 0usize;
    let mut shown = 0usize;
    for r in &tagged {
        if r.z < 4.0 {
            continue;
        }
        let Some(fo) = f::f_or(r.z, r.qbits) else {
            continue;
        };
        let d22 = ulp_distance(lentz_gaut_n21_mask(r.z, 0x22), fo).unwrap_or(1);
        let deo = ulp_distance(f::cf_evenodd_as714_x87_n(r.z, 12), fo).unwrap_or(1);
        if d22 == 0 && deo != 0 {
            if r.direct {
                extra_d += 1;
            } else {
                extra_i += 1;
            }
            if shown < 12 {
                println!(
                    "  extra z={:.16e} direct={} evenodd_ulp={deo}",
                    r.z, r.direct as u8
                );
                shown += 1;
            }
        }
    }
    println!("  extra exact vs evenodd n12: direct={extra_d} implied={extra_i}");
    println!("mask 0x22 = spill Lentz C after steps j=2 and j=6 (0-based bits 1 and 5)");
}
