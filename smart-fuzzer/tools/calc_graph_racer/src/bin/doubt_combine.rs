//! W109 G6-01 DOUBT PROBE: is the collision em-variation real, or a general-rate COMBINE
//! artifact? At general r, pmt=combine(pv,em,r). I pinned under SSE2 RN(RN(pv/em)*r). If the
//! real combine keeps the quotient EXTENDED before *r, the r-dependent rounding is
//! misattributed to em -> fake variation. Test: under each combine model, does a SINGLE em
//! reproduce ALL configs in a tau_double group (=> em IS g(tau_double), variation was artifact)?
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_div, ext_from_f64 as ef, ext_mul, ext_to_f64, CW_PC64_RN as CW};
use std::collections::BTreeMap;

// combine models: given pv, em, r -> pmt
fn c_sse(pv:f64,em:f64,r:f64)->f64{ (pv/em)*r }                       // RN(RN(pv/em)*r)
fn c_numr(pv:f64,em:f64,r:f64)->f64{ (pv*r)/em }                      // RN(RN(pv*r)/em)
fn c_afac(pv:f64,em:f64,r:f64)->f64{ pv/(em/r) }                      // annuity factor a=em/r first: RN(pv/RN(em/r))
fn c_rem(pv:f64,em:f64,r:f64)->f64{ pv*(r/em) }                       // RN(pv*RN(r/em))
fn c_x87(pv:f64,em:f64,r:f64)->f64{                                    // quotient & product at PC64, store 53
    ext_to_f64(&ext_mul(&ext_div(&ef(pv),&ef(em),CW),&ef(r),CW),CW)
}
fn c_qext(pv:f64,em:f64,r:f64)->f64{                                   // quotient extended, *r then store
    let q=ext_div(&ef(pv),&ef(em),CW);
    ext_to_f64(&ext_mul(&q,&ef(r),CW),CW)
}
fn c_afac_x87(pv:f64,em:f64,r:f64)->f64{                               // a=RN64(em/r) ext, pv/a store 53
    ext_to_f64(&ext_div(&ef(pv),&ext_div(&ef(em),&ef(r),CW),CW),CW)
}

fn main(){
    let ws:WitnessSet=serde_json::from_str(&std::fs::read_to_string(
        "../../work/w109/G6-solvers/answers-pmt-collide.json").unwrap()).unwrap();
    // meta groups: parse collide-meta for gi. Instead, group by tau_double computed from (r,n).
    let wl=&ws.witnesses; let ncfg=wl.len()/128;
    // build configs
    struct Cfg{ r:f64, n:f64, rows:Vec<(f64,u64)> }
    let mut cfgs=Vec::new();
    for ci in 0..ncfg{
        let mut rows=Vec::new(); let mut rn=(0.0,0.0);
        for j in 0..128{
            let w=&wl[ci*128+j];
            let a:Vec<f64>=w.args.iter().filter_map(|x|match x{WitnessArg::Scalar(s)=>parse_bits_hex(s),_=>None}).collect();
            rn=(a[0],a[1]); rows.push((a[2],parse_bits_hex(&w.expected_bits).unwrap().to_bits()));
        }
        cfgs.push(Cfg{r:rn.0,n:rn.1,rows});
    }
    // group by tau_double
    let mut groups:BTreeMap<u64,Vec<usize>>=BTreeMap::new();
    for (i,c) in cfgs.iter().enumerate(){
        let tau=-(c.n*rx::excel_log1p(c.r));
        groups.entry(tau.to_bits()).or_default().push(i);
    }
    let combines:[(&str,fn(f64,f64,f64)->f64);7]=[("SSE2",c_sse),("num*r",c_numr),("afac a=em/r",c_afac),
        ("pv*(r/em)",c_rem),("x87-dr",c_x87),("q-ext",c_qext),("afac-x87",c_afac_x87)];
    for (cn,cf) in combines{
        // per group: does a SINGLE em reproduce ALL configs' all-128 pv under this combine?
        let mut single_ok=0u32; let mut ngroups=0u32; let mut per_config_pin_distinct=0u32; let mut tot_cfg=0u32;
        for (tb,idxs) in &groups{
            ngroups+=1;
            let tau=f64::from_bits(*tb);
            // candidate em around excel_expm1_internal(tau)
            let base=rx::excel_expm1_internal(tau).to_bits() as i64;
            // find em that works for ALL configs in the group
            let mut found=false;
            for d in -20..=20i64{
                let em=f64::from_bits((base+d) as u64);
                if em>=0.0{continue}
                let ok=idxs.iter().all(|&i|{
                    cfgs[i].rows.iter().all(|(pv,want)| cf(*pv,em,cfgs[i].r).to_bits()==*want )
                });
                if ok{ found=true; break; }
            }
            if found{ single_ok+=1; }
            // also: per-config distinct em count (under this combine)
            let mut ems=std::collections::BTreeSet::new();
            for &i in idxs{
                tot_cfg+=1;
                for d in -20..=20i64{
                    let em=f64::from_bits((base+d) as u64);
                    if em>=0.0{continue}
                    if cfgs[i].rows.iter().all(|(pv,want)| cf(*pv,em,cfgs[i].r).to_bits()==*want){ ems.insert(em.to_bits()); break; }
                }
            }
            per_config_pin_distinct+=ems.len() as u32;
        }
        println!("combine={:<7}: groups where a SINGLE em fits ALL configs: {}/{}   (total distinct-em across groups: {})",
                 cn, single_ok, ngroups, per_config_pin_distinct);
    }
    println!("\nIf x87-dr or q-ext gives single-em-fits-all for most groups -> variation was a COMBINE artifact,");
    println!("em IS g(tau_double), and the 'extended argument' breakthrough is RETRACTED.");
}
