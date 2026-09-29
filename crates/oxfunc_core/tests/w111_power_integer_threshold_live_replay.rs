//! Public Excel width/signed-decimal controls, compared with exact typed bits.
use oxfunc_core::functions::power_fn::power_kernel;
use serde_json::Value;

fn replay(raw: &str, count: usize) {
    let data:Value=serde_json::from_str(raw).unwrap();
    let rows=data["witnesses"].as_array().unwrap();
    assert_eq!(rows.len(),count);
    for row in rows {
        let args:Vec<_>=row["args"].as_array().unwrap().iter().map(|value|{
            f64::from_bits(u64::from_str_radix(value.as_str().unwrap().trim_start_matches("0x"),16).unwrap())
        }).collect();
        let actual=match power_kernel(args[0],args[1]){
            Ok(value)=>format!("0x{:016x}",value.to_bits()),
            Err(error)=>format!("error:{error:?}"),
        };
        assert_eq!(actual,row["expected_bits"].as_str().unwrap(),"{}",row["id"]);
    }
}

#[test]
fn power_unsigned_width_and_signed_decimal_scale_match_public_controls(){
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/power-integer-thresholds/discovery-answers.json"),13622);
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/power-integer-thresholds/decimal-wrap-answers.json"),1672);
}

#[test]
fn power_frozen_width_candidate_matches_independent_inputs(){
    replay(include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/power-integer-thresholds/heldout-answers.json"),2978);
}
