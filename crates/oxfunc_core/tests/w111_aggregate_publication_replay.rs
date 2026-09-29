//! Aggregate numeric candidates against retained, exactly admitted Excel data.
use oxfunc_core::functions::surface_dispatch::eval_surface_value_call;
use oxfunc_core::resolver::NULL_REFERENCE_SYSTEM_PROVIDER;
use oxfunc_core::value::{CalcValue,CoreValue};
fn replay(source:&str){
 let bank:serde_json::Value=serde_json::from_str(source).unwrap();let function=format!("FUNC.{}",bank["function"].as_str().unwrap());let mut failures=Vec::new();
 for row in bank["witnesses"].as_array().unwrap(){
  let args:Vec<_>=row["args"].as_array().unwrap().iter().map(|x|CalcValue::number(f64::from_bits(u64::from_str_radix(x.as_str().unwrap().trim_start_matches("0x"),16).unwrap()))).collect();
  let actual=match eval_surface_value_call(&function,&args,&NULL_REFERENCE_SYSTEM_PROVIDER,None,None,None,None){
   Err(e)=>format!("error:{e:?}"),Ok(v)=>match v.core{CoreValue::Number(n)=>format!("0x{:016x}",n.to_bits()),CoreValue::Logical(v)=>format!("logical:{}",if v{"True"}else{"False"}),CoreValue::Error(e)=>format!("error:{e:?}"),other=>panic!("{other:?}")}
  };
  if actual!=row["expected_bits"].as_str().unwrap(){failures.push(format!("{} {} {} expected {}",row["id"],row["args"],actual,row["expected_bits"]));}
 }
 assert!(failures.is_empty(),"{function}: {} mismatches of {}:\n{}",failures.len(),bank["witnesses"].as_array().unwrap().len(),failures.iter().take(25).cloned().collect::<Vec<_>>().join("\n"));
}
#[test]
fn harmean_discovery(){replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/aggregate-publication/discovery/answers-harmean.json"));}
#[test]
fn devsq_discovery(){replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/aggregate-publication/discovery/answers-devsq.json"));}
#[test]
fn harmean_heldout(){replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/aggregate-publication/heldout/answers-harmean.json"));}
#[test]
fn devsq_heldout(){replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/aggregate-publication/heldout/answers-devsq.json"));}
