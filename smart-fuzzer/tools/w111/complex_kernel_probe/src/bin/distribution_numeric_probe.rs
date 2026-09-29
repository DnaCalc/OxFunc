//! Exact replay of captured public distribution outcomes; retains all residuals.
use oxfunc_core::functions::{discrete_dist_family as d,normal_log_family as n};
use serde_json::{Value,json};
fn main(){
    let path=std::env::args().nth(1).unwrap();
    let data:Value=serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let name=data["function"].as_str().unwrap();let mut misses=Vec::new();
    for r in data["witnesses"].as_array().unwrap(){
        let a:Vec<f64>=r["args"].as_array().unwrap().iter().map(|x|f64::from_bits(u64::from_str_radix(x.as_str().unwrap().trim_start_matches("0x"),16).unwrap())).collect();
        let value=match name{
            "NORM.DIST"=>n::norm_dist_kernel(a[0],a[1],a[2],a[3]!=0.),
            "NORM.S.DIST"=>n::norm_s_dist_kernel(a[0],a[1]!=0.),
            "GAUSS"=>Ok(n::identified_gauss(a[0])),
            "BINOM.INV"=>d::binom_inv_kernel(a[0],a[1],a[2]),
            "NEGBINOM.DIST"=>d::negbinom_dist_kernel(a[0],a[1],a[2],a[3]!=0.),
            "HYPGEOM.DIST"=>d::hypergeom_dist_kernel(a[0],a[1],a[2],a[3],a[4]!=0.),
            "POISSON.DIST"=>d::poisson_dist_kernel(a[0],a[1],a[2]!=0.),
            _=>panic!("unsupported {name}")
        };
        let actual=match value{Ok(n)=>format!("0x{:016x}",n.to_bits()),Err(e)=>format!("error:{e:?}")};
        if actual!=r["expected_bits"].as_str().unwrap(){misses.push(json!({"id":r["id"],"args":r["args"],"expected":r["expected_bits"],"actual":actual}));}
    }
    let rows=data["witnesses"].as_array().unwrap().len();
    println!("{}",json!({"function":name,"rows":rows,"matches":rows-misses.len(),"misses":misses}));
}
