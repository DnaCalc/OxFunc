//! Offline race of public arithmetic graphs against retained black-box ATAN rows.
#[path="support/w111_fpatan.rs"] mod primitive;
use primitive::direct as atan_kernel;
fn main() {
    let path=std::env::args().nth(1).expect("answer bank path");
    let bank:serde_json::Value=serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let names=["direct_fpatan","positive_fpatan_sign_restore","std_atan","positive_std_sign_restore","reciprocal_fpatan","half_angle_fpatan"];
    let mut matches=[0usize;6];let mut misses=Vec::new();
    for row in bank["witnesses"].as_array().unwrap() {
        let x=f64::from_bits(u64::from_str_radix(row["args"][0].as_str().unwrap().trim_start_matches("0x"),16).unwrap());
        let expected=u64::from_str_radix(row["expected_bits"].as_str().unwrap().trim_start_matches("0x"),16).unwrap();
        let a=x.abs();let p=atan_kernel(a);
        let recip=if a>1.0 {std::f64::consts::FRAC_PI_2-atan_kernel(1.0/a)} else {p};
        let half=if a<=1e150 {2.0*atan_kernel(a/(1.0+(1.0+a*a).sqrt()))} else {p};
        let values=[atan_kernel(x),p.copysign(x),x.atan(),a.atan().copysign(x),recip.copysign(x),half.copysign(x)];
        for i in 0..6 {matches[i]+=usize::from(values[i].to_bits()==expected);}
        if values[0].to_bits()!=expected {
            misses.push(serde_json::json!({"id":row["id"],"args":row["args"],"expected_bits":row["expected_bits"],"candidates":names.iter().zip(values).map(|(name,v)|((*name).to_owned(),serde_json::Value::String(format!("0x{:016x}",v.to_bits())))).collect::<serde_json::Map<_,_>>()}));
        }
    }
    println!("{}",serde_json::to_string_pretty(&serde_json::json!({"rows":bank["witnesses"].as_array().unwrap().len(),"exact":names.iter().zip(matches).map(|(n,c)|((*n).to_owned(),serde_json::json!(c))).collect::<serde_json::Map<_,_>>(),"direct_misses":misses})).unwrap());
}
