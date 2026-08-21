//! Generate the LOG1P-LANE discriminating capture batch (Run-W109BulkBatch format).
//! Selects INEXACT-(1+r) general rates where std_ln1p disagrees with CR log1p
//! (the discriminating signature), plus exact-(1+r) controls. For each r emits:
//!   - n=1 and n=2   (|tau|<1, FULL log1p sensitivity, wall-gated by n-consistency)
//!   - n=n_clean     (smallest 2^j with |tau|>=1, MODEL-EXACT exp-only confirmatory)
//! each with a 128-pv ladder (pv = 128 consecutive doubles near 1.0) so the
//! combine pmt=RN(RN(pv/em)*r) pins em to one double.  fv=0, type=0.
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    CW_PC64_RN as CW, ext_add, ext_from_f64 as ef, ext_fyl2x, ext_fyl2xp1, ext_ln2, ext_one,
    ext_to_f64,
};

fn l1p_cr(r: f64) -> f64 {
    rx::excel_log1p(r)
}
fn l1p_std(r: f64) -> f64 {
    r.ln_1p()
}
fn l1p_fyl(r: f64) -> f64 {
    if r.abs() < 0.292893218813452 {
        ext_to_f64(&ext_fyl2xp1(&ext_ln2(), &ef(r), CW), CW)
    } else {
        ext_to_f64(
            &ext_fyl2x(&ext_ln2(), &ext_add(&ext_one(), &ef(r), CW), CW),
            CW,
        )
    }
}
fn exact_1pr(r: f64) -> bool {
    (1.0 + r) - 1.0 == r
}
fn h(x: f64) -> String {
    format!("0x{:016x}", x.to_bits())
}

fn main() {
    let mut probes = String::new();
    let mut idx = 0u32;
    let mut push = |probes: &mut String, id: String, r: f64, n: f64, pv: f64| {
        if !probes.is_empty() {
            probes.push(',');
        }
        probes.push_str(&format!("{{\"probe\":{{\"id\":\"{}\",\"args\":[\"{}\",\"{}\",\"{}\",\"0x0000000000000000\",\"0x0000000000000000\"]}}}}",
            id, h(r), h(n), h(pv)));
    };
    // select r: log-uniform 3e-4..0.29, keep INEXACT-(1+r); target 44 std!=CR + 6 std==CR controls
    let mut seed = 0xC0FFEE1234567u64;
    let mut disc: Vec<f64> = Vec::new();
    let mut ctl: Vec<f64> = Vec::new();
    let mut fyl_diff: Vec<f64> = Vec::new();
    while disc.len() < 44 || ctl.len() < 6 || fyl_diff.len() < 4 {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        let frac = (seed >> 11) as f64 / (1u64 << 53) as f64;
        let r = 3e-4 * (0.29f64 / 3e-4).powf(frac);
        if exact_1pr(r) {
            continue;
        }
        let cr = l1p_cr(r);
        let sd = l1p_std(r);
        let fy = l1p_fyl(r);
        if fy.to_bits() != cr.to_bits() && fyl_diff.len() < 4 {
            fyl_diff.push(r);
            continue;
        }
        if sd.to_bits() != cr.to_bits() {
            if disc.len() < 44 {
                disc.push(r);
            }
        } else if ctl.len() < 6 {
            ctl.push(r);
        }
        if seed == 0 {
            seed = 1;
        }
    }
    let mut all: Vec<(f64, &str)> = Vec::new();
    for r in &disc {
        all.push((*r, "disc"));
    }
    for r in &ctl {
        all.push((*r, "ctl"));
    }
    for r in &fyl_diff {
        all.push((*r, "fyldiff"));
    }

    let base = 1.0f64.to_bits();
    let mut meta = String::from("r_hex,class,n1,n2,n_clean,cr_l1p,std_dev_ulp,fyl_dev_ulp\n");
    for (r, cls) in &all {
        let cr = l1p_cr(*r);
        // n_clean = smallest 2^j with n*log1p(r) >= 1.0
        let mut j = 0u32;
        while (2f64.powi(j as i32) * cr.abs()) < 1.0 {
            j += 1;
        }
        let n_clean = 2f64.powi(j as i32);
        for (tag, n) in [("n1", 1.0), ("n2", 2.0), ("nc", n_clean)] {
            for k in 0..128u64 {
                let pv = f64::from_bits(base + k);
                push(
                    &mut probes,
                    format!("l1p-{}-{}-{:03}", cls, tag, idx),
                    *r,
                    n,
                    pv,
                );
                idx += 1;
            }
        }
        let sd_ulp = l1p_std(*r).to_bits() as i64 - cr.to_bits() as i64;
        let fy_ulp = l1p_fyl(*r).to_bits() as i64 - cr.to_bits() as i64;
        meta.push_str(&format!(
            "{},{},1,2,{},0x{:016x},{},{}\n",
            h(*r),
            cls,
            n_clean as u64,
            cr.to_bits(),
            sd_ulp,
            fy_ulp
        ));
    }
    let batch = format!("{{\"function\":\"PMT\",\"probes\":[{}]}}", probes);
    std::fs::write("../../work/w109/G6-solvers/batch-log1p-lane.json", &batch).unwrap();
    std::fs::write(
        "../../work/w109/G6-solvers/batch-log1p-lane-meta.csv",
        &meta,
    )
    .unwrap();
    println!(
        "wrote batch-log1p-lane.json : {} probes ({} r-values: {} disc, {} ctl, {} fyldiff; x3 n x 128 pv)",
        idx,
        all.len(),
        disc.len(),
        ctl.len(),
        fyl_diff.len()
    );
    println!(
        "meta csv rows (r, class, n1/n2/n_clean, CR-log1p bits, std-CR ulp, fyl-CR ulp) written."
    );
}
