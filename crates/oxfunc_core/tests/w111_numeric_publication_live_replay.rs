//! Exact black-box numerical publication and overflow observations.
use oxfunc_core::functions::surface_dispatch::eval_surface_value_call;
use oxfunc_core::resolver::NULL_REFERENCE_SYSTEM_PROVIDER;
use oxfunc_core::value::{CalcValue, CoreValue};
fn replay(source: &str) {
    let bank:serde_json::Value=serde_json::from_str(source).unwrap();
    let function=format!("FUNC.{}",bank["function"].as_str().unwrap());
    let mut failures=Vec::new();
    for row in bank["witnesses"].as_array().unwrap() {
        let args:Vec<_>=row["args"].as_array().unwrap().iter().map(|x| CalcValue::number(f64::from_bits(u64::from_str_radix(x.as_str().unwrap().trim_start_matches("0x"),16).unwrap()))).collect();
        let actual=match eval_surface_value_call(&function,&args,&NULL_REFERENCE_SYSTEM_PROVIDER,None,None,None,None) {
            Err(e)=>format!("error:{e:?}"),
            Ok(v)=>match v.core {CoreValue::Number(n)=>format!("0x{:016x}",n.to_bits()),CoreValue::Error(e)=>format!("error:{e:?}"),other=>panic!("{other:?}")}
        };
        if actual!=row["expected_bits"].as_str().unwrap() { failures.push(format!("{} {} {} expected {}",row["id"],row["args"],actual,row["expected_bits"])); }
    }
    assert!(failures.is_empty(),"{function}: {} mismatches of {}:\n{}",failures.len(),bank["witnesses"].as_array().unwrap().len(),failures.iter().take(30).cloned().collect::<Vec<_>>().join("\n"));
}
macro_rules! discovery {($name:ident,$file:literal)=>{#[test] fn $name(){replay(include_str!(concat!("../../../docs/function-lane/evidence/w111-broad-20260929/integer-publication/discovery/answers/answers-",$file,".json")));}}}
discovery!(sqrt_staged_discovery,"sqrt");
discovery!(phi_overflow_discovery,"phi");
discovery!(norm_standard_density_discovery,"norm.s.dist");
discovery!(norm_density_discovery,"norm.dist");
discovery!(radians_tiny_discovery,"radians");
discovery!(standardize_publication_discovery,"standardize");
discovery!(sech_tiny_discovery,"sech");
discovery!(expon_dist_publication_discovery,"expon.dist");
discovery!(expondist_publication_discovery,"expondist");
macro_rules! heldout {($name:ident,$file:literal)=>{#[test] fn $name(){replay(include_str!(concat!("../../../docs/function-lane/evidence/w111-broad-20260929/integer-publication/heldout/answers/answers-",$file,".json")));}}}
heldout!(sqrt_staged_independent,"sqrt");
heldout!(phi_overflow_independent,"phi");
heldout!(norm_standard_density_independent,"norm.s.dist");
heldout!(norm_density_independent,"norm.dist");
heldout!(radians_tiny_independent,"radians");
heldout!(standardize_publication_independent,"standardize");
heldout!(sech_tiny_independent,"sech");
heldout!(expon_dist_publication_independent,"expon.dist");
heldout!(expondist_publication_independent,"expondist");
#[test]fn radians_staged_second_independent(){replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/integer-publication/radians-sech-heldout2/answers/answers-radians.json"));}
#[test]fn sech_staged_second_independent(){replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/integer-publication/radians-sech-heldout2/answers/answers-sech.json"));}
