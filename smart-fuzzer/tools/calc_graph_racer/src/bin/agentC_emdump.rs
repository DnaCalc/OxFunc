//! agent-C (combine lane): dump Excel-exact em = RN(exp(tau))-1 for the |tau|>=1
//! subset of PMT heldout + fvty witnesses. em depends only on (r,n); for |tau|>=1
//! em=u-1 is exact so any PMT miss is pure COMBINE. Emits CSV for Python search.
use oxfunc_core::excel_numeric::research as rx;

fn fb(h: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(h.trim_start_matches("0x"), 16).unwrap())
}

fn dump(path: &str, tag: &str) {
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("read")).expect("json");
    for w in v["witnesses"].as_array().unwrap() {
        let a = w["args"].as_array().unwrap();
        let r = fb(a[0].as_str().unwrap());
        let nf = fb(a[1].as_str().unwrap());
        let pv = fb(a[2].as_str().unwrap());
        let fv = fb(a[3].as_str().unwrap());
        let ty = fb(a[4].as_str().unwrap());
        let exp_bits = w["expected_bits"].as_str().unwrap();
        // tau = -(n * log1p(r)), log1p CR (FYL2XP1)
        let l1p = rx::excel_log1p(r);
        let tau = -(nf * l1p);
        if tau.abs() < 1.0 {
            continue;
        }
        let u = rx::excel_exp(tau);
        let em = u - 1.0;
        println!(
            "{},{:#018x},{},{:#018x},{:#018x},{:#018x},{:#018x},{:#018x},{:#018x},{}",
            tag,
            r.to_bits(),
            nf as i64,
            pv.to_bits(),
            fv.to_bits(),
            ty.to_bits(),
            em.to_bits(),
            u.to_bits(),
            u64::from_str_radix(exp_bits.trim_start_matches("0x"), 16).unwrap(),
            tau
        );
    }
}

fn main() {
    println!("tag,r,n,pv,fv,ty,em,u,expected,tau");
    dump("../../work/w109/G6-solvers/answers-pmt-heldout.json", "heldout");
    dump("../../work/w109/G6-solvers/answers-pmt-fvty.json", "fvty");
}
