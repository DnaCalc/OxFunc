//! agent-P: on the sensitive lane, feed the EXACT prec-64 extended tau
//! (agentP_tauext64.json, Ext80 hex) through the x87 exp chain and compare
//! em = exp(tau_ext)-1 to gold. If this matches, the sensitive-lane op-graph is
//! "correctly-rounded-to-64-bit tau delivered EXTENDED into fFEXP", and fyl2xp1
//! simply isn't CR-to-64-bit.
use oxfunc_core::excel_numeric::research as rx;
use std::collections::{BTreeMap, HashMap};

const CW: u16 = rx::CW_PC64_RN;

fn fb(h: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(h.trim_start_matches("0x"), 16).unwrap())
}
fn parse_ext80(h: &str) -> rx::Ext80 {
    let v = u128::from_str_radix(h.trim_start_matches("0x"), 16).unwrap();
    let mut b = [0u8; 10];
    for i in 0..10 {
        b[i] = ((v >> (8 * i)) & 0xff) as u8;
    }
    rx::Ext80(b)
}
fn tf(x: &rx::Ext80) -> f64 {
    rx::ext_to_f64(x, CW)
}
fn exp_ext(arg: &rx::Ext80) -> rx::Ext80 {
    let z = rx::ext_mul(arg, &rx::ext_l2e(), CW);
    let k = rx::ext_rndint(&z, CW);
    let f = rx::ext_sub(&z, &k, CW);
    let neg = tf(&f) < 0.0;
    let w = rx::ext_f2xm1(&rx::ext_abs(&f, CW), CW);
    let mut m = rx::ext_add(&w, &rx::ext_one(), CW);
    if neg {
        m = rx::ext_div(&rx::ext_one(), &m, CW);
    }
    rx::ext_scale(&m, &k, CW)
}
fn load(path: &str) -> Vec<(String, f64, i64, f64)> {
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("read")).expect("json");
    let mut out = Vec::new();
    for (k, val) in v.as_object().unwrap() {
        if val.is_null() {
            continue;
        }
        let (rh, nh) = k.split_once('|').unwrap();
        out.push((
            k.clone(),
            fb(rh),
            nh.parse().unwrap(),
            fb(val.as_str().unwrap()),
        ));
    }
    out
}
fn hist(name: &str, v: &[i64]) {
    let mut m: BTreeMap<i64, u32> = BTreeMap::new();
    for &x in v {
        *m.entry(x).or_default() += 1;
    }
    let n = v.len();
    let exact = *m.get(&0).unwrap_or(&0);
    print!(
        "  {:34} {:4}/{:4} ({:5.1}%)  ",
        name,
        exact,
        n,
        100.0 * exact as f64 / n as f64
    );
    let mut big = 0;
    for (k, c) in &m {
        if k.abs() <= 2 {
            print!("{}:{} ", k, c);
        } else {
            big += c;
        }
    }
    if big > 0 {
        print!("|>2|:{}", big);
    }
    println!();
}

fn main() {
    let rows = load("../../work/w109/G6-solvers/pmt_em_gold_pv1.json");
    let te: HashMap<String, rx::Ext80> = {
        let v: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string("../../work/w109/G6-solvers/agentP_tauext64.json").unwrap(),
        )
        .unwrap();
        v.as_object()
            .unwrap()
            .iter()
            .map(|(k, val)| (k.clone(), parse_ext80(val.as_str().unwrap())))
            .collect()
    };
    let mut r_extf = Vec::new(); // exp(tau_ext64); u f64; u-1
    let mut r_exte = Vec::new(); // exp(tau_ext64); RN53(u_ext-1)
    for (k, r, n, em_x) in &rows {
        if let Some(taue) = te.get(k) {
            let _ = (r, n);
            let uext = exp_ext(taue);
            r_extf.push((tf(&uext) - 1.0).to_bits() as i64 - em_x.to_bits() as i64);
            r_exte.push(
                tf(&rx::ext_sub(&uext, &rx::ext_one(), CW)).to_bits() as i64
                    - em_x.to_bits() as i64,
            );
        }
    }
    println!("sensitive rows with prec-64 ext tau: {}", r_extf.len());
    hist("exp(tau_ext64); u(f64)-1", &r_extf);
    hist("exp(tau_ext64); RN53(u_ext-1)", &r_exte);
}
