//! joint-866 vs m856 two-mode overlap on 4 LOW. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const CW_RU: u16 = CW_PC64_RN | 0x0800;
const AS0: [f64; 21] = [
    0.1283791670955125738961589031215,
    -0.3761263890318375246320529677070,
    0.1128379167095512573896158902931,
    -0.2686617064513125175943235372542e-01,
    0.5223977625442187842111812447877e-02,
    -0.8548327023450852832540164081187e-03,
    0.1205533298178966425020717182498e-03,
    -0.1492565035840625090430728526820e-04,
    0.1646211436588924261080723578109e-05,
    -0.1636584469123468757408968429674e-06,
    0.1480719281587021715400818627811e-07,
    -0.1229055530145120140800510155331e-08,
    0.9422759058437197017313055084212e-10,
    -0.6711366740969385085896257227159e-11,
    0.4463222608295664017461758843550e-12,
    -0.2783497395542995487275065856998e-13,
    0.1634095572365337143933023780777e-14,
    -0.9052845786901123985710019387938e-16,
    0.4708274559689744439341671426731e-17,
    -0.2187159356685015949749948252160e-18,
    0.7043407712019701609635599701333e-20,
];
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
fn one_w(z: f64) -> Ext80 {
    let mut a = AS0;
    a[0] = poke(AS0[0], 4);
    a[1] = poke(AS0[1], -2);
    a[2] = poke(AS0[2], -5);
    a[3] = poke(AS0[3], 1);
    let xe = ef(z);
    let t = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW), &ef(c), CW);
    }
    ext_add(&ef(1.0), &acc, CW)
}
fn one_y(inner: f64) -> Ext80 {
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
    let jn = |z: f64| ext_to_f64(&ext_mul(&ef(z), &one_w(z), CW), CW);
    let ru = |z: f64| ext_to_f64(&ext_mul(&ef(z.next_up()), &one_y(z), CW_RU), CW_RU);
    let tf = |z: f64| {
        let ow = ext_to_f64(&one_y(z), CW).next_up();
        ext_to_f64(&ext_mul(&ef(z.next_up()), &ef(ow), CW), CW)
    };
    let mut both = 0usize;
    let mut only_j = 0usize;
    let mut only_m = 0usize;
    let mut only_m_zs = Vec::new();
    for &(z, bits) in &rows {
        let t = f64::from_bits(bits);
        let ej = ulp_distance(jn(z), t).unwrap_or(99) == 0;
        let em = ulp_distance(ru(z), t).unwrap_or(99) == 0
            || ulp_distance(tf(z), t).unwrap_or(99) == 0;
        match (ej, em) {
            (true, true) => both += 1,
            (true, false) => only_j += 1,
            (false, true) => {
                only_m += 1;
                only_m_zs.push(z);
            }
            _ => {}
        }
    }
    println!("4 LOW joint vs m856 two-mode:");
    let mut hit_j = 0usize;
    let mut hit_m = 0usize;
    let mut hit_u = 0usize;
    for &lz in &LOWS {
        let Some((_, bits)) = rows.iter().find(|(z, _)| (*z - lz).abs() < 1e-14) else {
            continue;
        };
        let t = f64::from_bits(*bits);
        let ej = ulp_distance(jn(lz), t).unwrap_or(99);
        let eru = ulp_distance(ru(lz), t).unwrap_or(99) == 0;
        let etf = ulp_distance(tf(lz), t).unwrap_or(99) == 0;
        let em = eru || etf;
        if ej == 0 {
            hit_j += 1;
        }
        if em {
            hit_m += 1;
        }
        if ej == 0 || em {
            hit_u += 1;
        }
        println!(
            "  z={lz:.16} joint_ulp={ej} RU={eru} TF={etf} any={}",
            ej == 0 || em
        );
    }
    println!(
        "overlap both={both} only_joint={only_j} only_m856={only_m} union={}/{} leftover J={hit_j}/4 M={hit_m}/4 union={hit_u}/4",
        both + only_j + only_m,
        rows.len()
    );
    println!("only_m856 count={}", only_m_zs.len());
}
