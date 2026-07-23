//! W109 G6-01: identify Excel's non-CR log1p directly against agent-P's extracted
//! deviation map (agentP_log1p_devmap.json -> effective_log1p = Excel's exact bits).
//! Key hypothesis: Excel's PMT log1p is the RAW x87 FYL2XP1 hardware instruction
//! (or FYL2X on the exact extended 1+r), whose ~1-ext-ulp microcode imprecision,
//! rounded once to double, is faithful-but-non-CR. ext_fyl2xp1/ext_fyl2x are real
//! inline x87 asm on THIS silicon, so a hardware match is a bit-exact identification.
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_from_f64, ext_fyl2x, ext_fyl2xp1, ext_ln2, ext_mul, ext_one, ext_to_f64,
         CW_PC64_RN, CW_PC53_RN};
use serde_json::Value;

fn parse_hex(s:&str)->u64{ u64::from_str_radix(s.trim_start_matches("0x"),16).unwrap() }

// --- candidate log1p implementations ---
fn c_cr(r:f64)->f64{ rx::excel_log1p(r) }                 // portable correctly-rounded
fn c_ln_dbl(r:f64)->f64{ rx::excel_ln(1.0+r) }            // ln(1+r), 1+r formed in double
fn c_fyl2xp1_64(r:f64)->f64{                               // raw FYL2XP1 hw, PC64
    ext_to_f64(&ext_fyl2xp1(&ext_ln2(), &ext_from_f64(r), CW_PC64_RN), CW_PC64_RN)
}
fn c_fyl2xp1_53(r:f64)->f64{                               // raw FYL2XP1 hw, PC53
    ext_to_f64(&ext_fyl2xp1(&ext_ln2(), &ext_from_f64(r), CW_PC53_RN), CW_PC53_RN)
}
fn c_fyl2x_extu(r:f64)->f64{                               // FYL2X on EXACT ext(1+r), PC64
    let u = ext_add(&ext_one(), &ext_from_f64(r), CW_PC64_RN);   // exact for |r|<1
    ext_to_f64(&ext_fyl2x(&ext_ln2(), &u, CW_PC64_RN), CW_PC64_RN)
}
fn c_fyl2x_dblu(r:f64)->f64{                               // FYL2X on double 1+r, PC64
    let u = ext_from_f64(1.0+r);
    ext_to_f64(&ext_fyl2x(&ext_ln2(), &u, CW_PC64_RN), CW_PC64_RN)
}
// FYL2XP1 valid only for |r| < 1 - sqrt(2)/2 ~ 0.2929; fall back to FYL2X(1+r) above.
fn c_hybrid(r:f64)->f64{
    if r.abs() < 0.292893218813452 { c_fyl2xp1_64(r) } else { c_fyl2x_extu(r) }
}
// log2 path: y=1 -> log2(1+r) ext, then *ln2 ext (separate mul), round once
fn c_fyl2xp1_sepln2(r:f64)->f64{
    let l2 = ext_fyl2xp1(&ext_one(), &ext_from_f64(r), CW_PC64_RN); // log2(1+r) ext
    ext_to_f64(&ext_mul(&l2, &ext_ln2(), CW_PC64_RN), CW_PC64_RN)
}

fn main(){
    let path = "../../work/w109/G6-solvers/agentP_log1p_devmap.json";
    let txt = std::fs::read_to_string(path).expect("read devmap");
    let map:Value = serde_json::from_str(&txt).unwrap();
    let obj = map.as_object().unwrap();

    let cands:[(&str, fn(f64)->f64);7] = [
        ("CR(portable)", c_cr),
        ("ln(1+r)dbl",   c_ln_dbl),
        ("FYL2XP1 pc64", c_fyl2xp1_64),
        ("FYL2XP1 pc53", c_fyl2xp1_53),
        ("FYL2X extU",   c_fyl2x_extu),
        ("FYL2X dblU",   c_fyl2x_dblu),
        ("FYL2XP1sepLn2",c_fyl2xp1_sepln2),
    ];

    let mut tot_unamb = 0u32;
    let mut score = [0u32; 7];
    // also track: of the rows where CR is WRONG (Excel != CR), how many each cand gets
    let mut nonc = 0u32;
    let mut nonc_hit = [0u32; 7];
    // collect mismatches for the best hardware candidate for inspection
    let mut fy_miss: Vec<(f64,i64)> = Vec::new(); // (r, excel_bits - fyl2xp1_bits)

    for (rk, v) in obj {
        let eff = v["effective_log1p"].as_array().unwrap();
        if eff.len()!=1 { continue; }   // only unambiguous rows
        tot_unamb += 1;
        let r = f64::from_bits(parse_hex(rk));
        let excel = parse_hex(eff[0].as_str().unwrap());
        let cr = parse_hex(v["cr_log1p"].as_str().unwrap());
        let is_nonc = excel != cr;
        if is_nonc { nonc += 1; }
        for (i,(_,f)) in cands.iter().enumerate() {
            let g = f(r).to_bits();
            if g == excel { score[i]+=1; if is_nonc { nonc_hit[i]+=1; } }
        }
        // record fyl2xp1 signed miss
        let g = c_fyl2xp1_64(r).to_bits();
        let d = excel as i64 - g as i64;
        if d != 0 { fy_miss.push((r, d)); }
    }

    println!("unambiguous rows: {}   (non-CR among them: {})", tot_unamb, nonc);
    println!("{:<16} {:>8} {:>10}", "candidate", "total", "non-CR");
    for (i,(nm,_)) in cands.iter().enumerate() {
        println!("{:<16} {:>4}/{:<4} {:>5}/{:<4}", nm, score[i], tot_unamb, nonc_hit[i], nonc);
    }
    println!("\nFYL2XP1 pc64 misses (r_log2, excel-hw ulps): {}", fy_miss.len());
    fy_miss.sort_by(|a,b| a.0.partial_cmp(&b.0).unwrap());
    for (r,d) in fy_miss.iter().take(60) {
        println!("  r=2^{:+7.4}  d={}", r.log2(), d);
    }
}
