//! W109 G6-01: END-TO-END log1p race on the dense n=1 sweep (answers-pmt-denselog1p:
//! 384 r x 256 pv, n=1/fv=0/ty=0). Extraction-free: push a candidate log1p through
//! the SOLVED expm1 (internal Kahan) and SOLVED combine (RN(RN(pv/em)*r)) and check
//! the raw PMT bits. A candidate that scores 256/256 pv at an r reproduces Excel there.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_from_f64, ext_fyl2x, ext_fyl2xp1, ext_ln2, ext_one, ext_to_f64,
         CW_PC64_RN};
use std::collections::BTreeMap;

// ---- candidate log1p ----
fn c_cr(r:f64)->f64{ rx::excel_log1p(r) }
fn c_fyl2xp1(r:f64)->f64{ ext_to_f64(&ext_fyl2xp1(&ext_ln2(), &ext_from_f64(r), CW_PC64_RN), CW_PC64_RN) }
fn c_fyl2x_extu(r:f64)->f64{
    let u = ext_add(&ext_one(), &ext_from_f64(r), CW_PC64_RN);
    ext_to_f64(&ext_fyl2x(&ext_ln2(), &u, CW_PC64_RN), CW_PC64_RN)
}
fn c_hybrid(r:f64)->f64{ if r.abs()<0.292893218813452 { c_fyl2xp1(r) } else { c_fyl2x_extu(r) } }

fn pmt_n1(r:f64, pv:f64, log1p:fn(f64)->f64)->f64{
    // n=1, fv=0, ty=0: tau=-log1p(r); em=expm1(tau); pmt=RN(RN(pv/em)*r)
    let tau = -log1p(r);
    let em = rx::excel_expm1_internal(tau);
    let q = pv / em;
    q * r
}

fn main(){
    let path="../../work/w109/G6-solvers/answers-pmt-denselog1p.json";
    let ws:WitnessSet = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    // group by r
    let mut byr:BTreeMap<u64, Vec<(f64,u64)>> = BTreeMap::new(); // r_bits -> (pv, want)
    for w in &ws.witnesses{
        let a:Vec<f64>=w.args.iter().filter_map(|x|match x{WitnessArg::Scalar(s)=>parse_bits_hex(s),_=>None}).collect();
        if a.len()!=5{continue}
        let want = parse_bits_hex(&w.expected_bits).unwrap().to_bits();
        byr.entry(a[0].to_bits()).or_default().push((a[2], want));
    }
    let nr = byr.len();
    println!("distinct r: {}   pv/r: {}", nr, byr.values().next().map(|v|v.len()).unwrap_or(0));

    let cands:[(&str,fn(f64)->f64);4]=[
        ("CR", c_cr), ("FYL2XP1", c_fyl2xp1), ("FYL2Xext", c_fyl2x_extu), ("hybrid", c_hybrid),
    ];
    for (nm,f) in cands{
        let mut full=0u32; let mut totpv=0u32; let mut okpv=0u32;
        for (_rb, rows) in &byr{
            let r=f64::from_bits(*_rb);
            let mut allok=true;
            for (pv,want) in rows{
                let g=pmt_n1(r,*pv,f).to_bits();
                totpv+=1;
                if g==*want{okpv+=1;} else {allok=false;}
            }
            if allok{full+=1;}
        }
        println!("{:<10} full-r {:>3}/{:<3}   pv {:>6}/{:<6}", nm, full, nr, okpv, totpv);
    }

    // For CR: dump per-r how many pv fail + the SIGN of the needed em nudge,
    // to read the dev(r) structure. We infer dev by testing em-shifted by +-1 ulp.
    println!("\n--- CR per-r residual (r_log2 : failing_pv / 256 : em-nudge that fixes) ---");
    let mut printed=0;
    for (_rb, rows) in &byr{
        let r=f64::from_bits(*_rb);
        let mut fail=0;
        for (pv,want) in rows{ if pmt_n1(r,*pv,c_cr).to_bits()!=*want{fail+=1;} }
        if fail==0 { continue; }
        // try em nudged +-1,+-2 ulp (equivalently log1p nudged) to see which sign closes it
        let tau=-c_cr(r); let em0=rx::excel_expm1_internal(tau);
        let mut best=(fail as i64, 0i64);
        for d in [-2i64,-1,1,2]{
            let em = f64::from_bits((em0.to_bits() as i64 + d) as u64);
            let mut fl=0;
            for (pv,want) in rows{ let q=(*pv)/em; if (q*r).to_bits()!=*want{fl+=1;} }
            if (fl as i64) < best.0 { best=(fl as i64, d); }
        }
        if printed<80 {
            println!("  r=2^{:+8.4}  fail {:>3}/256   best em-nudge {:+} -> {} left",
                     r.log2(), fail, best.1, best.0);
            printed+=1;
        }
    }
}
