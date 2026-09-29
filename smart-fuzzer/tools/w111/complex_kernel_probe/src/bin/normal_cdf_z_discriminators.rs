//! Select fresh arithmetic discriminators without consulting Excel outputs.
use oxfunc_core::excel_numeric::research::*;
use serde_json::json;
use std::collections::BTreeMap;
fn main() {
    let out=std::path::PathBuf::from(std::env::args().nth(1).unwrap());std::fs::create_dir_all(&out).unwrap();
    let seed=2026092936_u64;let mut state=seed;let mut next=||{state^=state<<13;state^=state>>7;state^=state<<17;state};
    let c=f64::from_bits(0x3fe6a09e667f3bcd);
    let staged=|x|ext_to_f64(&ext_mul(&ext_from_f64(x),&ext_from_f64(c),CW_PC64_RN),CW_PC64_RN);
    let mut selected=Vec::new();let mut seen=std::collections::BTreeSet::new();let mut draws=0;
    while selected.len()<128 {
        draws+=1;assert!(draws<3_000_000);
        let raw=0x3fe0000000000000_u64+next()%(0x4042c00000000000_u64-0x3fe0000000000000);
        let x=f64::from_bits(raw);
        if (x*c).to_bits()!=staged(x).to_bits() && seen.insert(raw) {selected.push((x,"z-staging-discriminator"));}
    }
    for _ in 0..128 {
        let raw=0x3fe0000000000000_u64+next()%(0x4042c00000000000_u64-0x3fe0000000000000);
        if seen.insert(raw){selected.push((f64::from_bits(raw),"fresh-control"));}
    }
    let mut packets=BTreeMap::<&str,Vec<serde_json::Value>>::new();let mut zseen=std::collections::BTreeSet::new();
    let mut emit=|name,args:Vec<f64>,kind:&str|{let rows=packets.entry(name).or_default();rows.push(json!({"probe":{"id":format!("cdf-z-{}-{:05}-{kind}",name,rows.len()),"args":args.into_iter().map(|v|format!("0x{:016x}",v.to_bits())).collect::<Vec<_>>()}}));};
    let mut relationships=Vec::new();
    for (x,kind) in selected {
        for sign in [-1.,1.] {emit("NORM.S.DIST",vec![sign*x,1.],kind);emit("GAUSS",vec![sign*x],kind);}
        for z in [x*c,staged(x)] {if zseen.insert(z.to_bits()){emit("ERFC.PRECISE",vec![z],kind);}}
        relationships.push(json!({"x_bits":format!("0x{:016x}",x.to_bits()),"kind":kind,
            "z_native_bits":format!("0x{:016x}",(x*c).to_bits()),"z_staged_bits":format!("0x{:016x}",staged(x).to_bits())}));
    }
    let mut batches=Vec::new();for(name,rows)in packets {let path=format!("batch-{}.json",name.to_lowercase());std::fs::write(out.join(&path),serde_json::to_string_pretty(&json!({"function":name,"probes":rows})).unwrap()).unwrap();batches.push(json!({"function":name,"path":path,"rows":rows.len()}));}
    std::fs::write(out.join("relationships.json"),serde_json::to_string_pretty(&relationships).unwrap()).unwrap();
    let manifest=json!({"generator":"normal_cdf_z_discriminators.rs","seed":seed,"draws":draws,
        "authority":"fresh_mathematical_normal_z_discriminators_without_oracle_selection","batches":batches});
    std::fs::write(out.join("manifest.json"),serde_json::to_string_pretty(&manifest).unwrap()).unwrap();println!("{manifest}");
}
