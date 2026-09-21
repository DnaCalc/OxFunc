//! 4 LOW: fdlibm P[0]+4 lead+1 RU ∪ two-factor. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const CW_RU: u16 = CW_PC64_RN | 0x0800;
const M77: [f64; 21] = [
    -0.49046121234691808039984544033376e-1,
    -0.14226120510371364237824741899631e+0,
    0.10035582187599795575754676712933e-1,
    -0.57687646997674847650827025509167e-3,
    0.27419931252196061034422160791471e-4,
    -0.11043175507344507604135381295905e-5,
    0.38488755420345036949961311498174e-7,
    -0.11808582533875466969631751801581e-8,
    0.32334215826050909646402930953354e-10,
    -0.79910159470045487581607374708595e-12,
    0.17990725113961455611967245486634e-13,
    -0.37186354878186926382316828209493e-15,
    0.71035990037142529711689908394666e-17,
    -0.12612455119155225832495424853333e-18,
    0.20916406941769294369170500266666e-20,
    -0.32539731029314072982364160000000e-22,
    0.47668672097976748332373333333333e-24,
    -0.65980120782851343155199999999999e-26,
    0.86550114699637626197333333333333e-28,
    -0.10788925177498064213333333333333e-29,
    0.12811883993017002666666666666666e-31,
];
const PP: [f64; 5] = [
    1.28379167095512558561e-01,
    -3.25042107247001499370e-01,
    -2.84817495755985104766e-02,
    -5.77027029648944159157e-03,
    -2.37630166566501626084e-05,
];
const QQ: [f64; 5] = [
    3.97917223959155352819e-01,
    6.50222499887672944485e-02,
    5.08130628187576562776e-03,
    1.32494738004321644526e-04,
    -3.96022827877536812320e-06,
];
const LOWS: [f64; 4] = [
    0.23046875,
    0.37109375,
    0.4524739583333333,
    0.4716796875,
];

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn poke(x: f64, k: i32) -> f64 {
    let mut v = x;
    if k > 0 {
        for _ in 0..k {
            v = v.next_up();
        }
    } else {
        for _ in 0..(-k) {
            v = v.next_down();
        }
    }
    v
}
fn m856(lead: f64, inner: f64) -> f64 {
    let mut a = M77;
    a[1] = poke(M77[1], -4);
    a[0] = poke(M77[0], 1);
    a[3] = poke(M77[3], 1);
    let xe = ef(inner.abs());
    let t = ext_sub(&ext_mul(&ef(2.0), &ext_mul(&xe, &xe, CW), CW), &ef(1.0), CW);
    let twox = ext_mul(&ef(2.0), &t, CW);
    let mut b0 = ef(0.0);
    let mut b1 = ef(0.0);
    let mut b2 = ef(0.0);
    for i in (0..a.len()).rev() {
        b2 = b1;
        b1 = b0;
        b0 = ext_add(
            &ext_sub(&ext_mul(&twox, &b1, CW), &b2, CW),
            &ef(a[i]),
            CW,
        );
    }
    let y = ext_mul(&ef(0.5), &ext_sub(&b0, &b2, CW), CW);
    ext_to_f64(
        &ext_mul(&ef(lead.abs()), &ext_add(&ef(1.0), &y, CW), CW),
        CW,
    )
}
fn fd4(lead: f64, inner: f64) -> f64 {
    let mut p = PP;
    p[0] = poke(PP[0], 4);
    let ze = ef(inner);
    let zz = ext_mul(&ze, &ze, CW);
    let mut r = ef(0.0);
    for &c in p.iter().rev() {
        r = ext_add(&ext_mul(&r, &zz, CW), &ef(c), CW);
    }
    let mut s = ef(0.0);
    for &c in QQ.iter().rev() {
        s = ext_add(&ext_mul(&s, &zz, CW), &ef(c), CW);
    }
    let den = ext_add(&ef(1.0), &ext_mul(&zz, &s, CW), CW);
    let y = ext_div(&r, &den, CW);
    ext_to_f64(
        &ext_add(&ef(lead), &ext_mul(&ef(lead), &y, CW), CW),
        CW,
    )
}
fn one_y(inner: f64) -> Ext80 {
    let mut p = PP;
    p[0] = poke(PP[0], 4);
    let ze = ef(inner);
    let zz = ext_mul(&ze, &ze, CW);
    let mut r = ef(0.0);
    for &c in p.iter().rev() {
        r = ext_add(&ext_mul(&r, &zz, CW), &ef(c), CW);
    }
    let mut s = ef(0.0);
    for &c in QQ.iter().rev() {
        s = ext_add(&ext_mul(&s, &zz, CW), &ef(c), CW);
    }
    let den = ext_add(&ef(1.0), &ext_mul(&zz, &s, CW), CW);
    let y = ext_div(&r, &den, CW);
    ext_add(&ef(1.0), &y, CW)
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    const BANKS: [&str; 7] = [
        "answers-b9train.json",
        "answers-erfp.json",
        "answers-erfm.json",
        "answers-b8erf.json",
        "answers-b7erf.json",
        "answers-b11.json",
        "answers-b10.json",
    ];
    let mut map = BTreeMap::new();
    for name in BANKS {
        let path = format!("{dir}/{name}");
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let bank: WitnessSet = serde_json::from_str(&text).unwrap();
        for w in &bank.witnesses {
            let x = match &w.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
                _ => continue,
            };
            let Some(e) = parse_bits_hex(&w.expected_bits) else {
                continue;
            };
            if x > 0.0 && x < 0.5 {
                map.entry(x.to_bits()).or_insert(e.to_bits());
            }
        }
    }
    let rows: Vec<_> = map
        .into_iter()
        .map(|(b, e)| (f64::from_bits(b), e))
        .collect();
    let ru = |z: f64| {
        ext_to_f64(&ext_mul(&ef(z.next_up()), &one_y(z), CW_RU), CW_RU)
    };
    let tf = |z: f64| {
        let ow = ext_to_f64(&one_y(z), CW).next_up();
        ext_to_f64(&ext_mul(&ef(z.next_up()), &ef(ow), CW), CW)
    };
    let mut both = 0usize;
    let mut only_ru = 0usize;
    let mut only_tf = 0usize;
    let mut uni = 0usize;
    for &(z, bits) in &rows {
        let t = f64::from_bits(bits);
        let eru = ulp_distance(ru(z), t).unwrap_or(99) == 0;
        let etf = ulp_distance(tf(z), t).unwrap_or(99) == 0;
        match (eru, etf) {
            (true, true) => both += 1,
            (true, false) => only_ru += 1,
            (false, true) => only_tf += 1,
            _ => {}
        }
        if eru || etf {
            uni += 1;
        }
    }
    println!("4 LOW fd P[0]+4 two-mode:");
    let mut hit_ru = 0usize;
    let mut hit_tf = 0usize;
    let mut hit_u = 0usize;
    for &lz in &LOWS {
        let Some((_, bits)) = rows.iter().find(|(z, _)| (*z - lz).abs() < 1e-14) else {
            continue;
        };
        let t = f64::from_bits(*bits);
        let eru = ulp_distance(ru(lz), t).unwrap_or(99) == 0;
        let etf = ulp_distance(tf(lz), t).unwrap_or(99) == 0;
        if eru {
            hit_ru += 1;
        }
        if etf {
            hit_tf += 1;
        }
        if eru || etf {
            hit_u += 1;
        }
        println!("  z={lz:.16} RU={eru} TF={etf} any={}", eru || etf);
    }
    println!(
        "overlap both={both} only_RU={only_ru} only_TF={only_tf} union={uni}/{} leftover RU={hit_ru}/4 TF={hit_tf}/4 union={hit_u}/4",
        rows.len()
    );
}
