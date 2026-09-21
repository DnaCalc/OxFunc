//! leftover-low 13 vs MATH77 / Schonfelder. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const ERFCS: [f64; 21] = [
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
const SCH: [f64; 18] = [
    1.4831105640848035818894480790578,
    -0.3010710733865949424707310463118,
    0.0689948306898315662466031807188,
    -0.0139162712647221876825465256678,
    0.0024207995224334636628916782398,
    -0.0003658639685848086446493825778,
    4.86209844323190482828875688e-5,
    -5.7492565580356848350542158e-6,
    6.113243578434764697067588e-7,
    -5.89910153129584343908468e-8,
    5.2070090920686482404558e-9,
    -4.232975879965543268108e-10,
    3.18811350664917497488e-11,
    -2.2361550188326842738e-12,
    1.467329847991084928e-13,
    -9.0440019853817478e-15,
    5.254813715470928e-16,
    -2.88742612228498e-17,
];

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
/// MATH77/SLATEC DCSEVL: B2=B1; B1=B0; B0=2x*B1-B2+A(I); result=0.5*(B0-B2)
fn dcsevl(t: Ext80, cs: &[f64]) -> Ext80 {
    let twox = ext_mul(&ef(2.0), &t, CW);
    let mut b0 = ef(0.0);
    let mut b1 = ef(0.0);
    let mut b2 = ef(0.0);
    for i in (0..cs.len()).rev() {
        b2 = b1;
        b1 = b0;
        b0 = ext_add(
            &ext_sub(&ext_mul(&twox, &b1, CW), &b2, CW),
            &ef(cs[i]),
            CW,
        );
    }
    ext_mul(&ef(0.5), &ext_sub(&b0, &b2, CW), CW)
}
/// Primed Clenshaw: t*d1 - d2 + 0.5*a0  (a[0] is primed)
fn clenshaw_prime(t: Ext80, a: &[f64]) -> Ext80 {
    let two = ef(2.0);
    let mut d1 = ef(0.0);
    let mut d2 = ef(0.0);
    for k in (1..a.len()).rev() {
        let dk = ext_add(
            &ext_sub(&ext_mul(&ext_mul(&two, &t, CW), &d1, CW), &d2, CW),
            &ef(a[k]),
            CW,
        );
        d2 = d1;
        d1 = dk;
    }
    ext_add(
        &ext_sub(&ext_mul(&t, &d1, CW), &d2, CW),
        &ext_mul(&ef(0.5), &ef(a[0]), CW),
        CW,
    )
}
fn erf_m77_1p(z: f64) -> f64 {
    let xe = ef(z.abs());
    let t = ext_sub(&ext_mul(&ef(2.0), &ext_mul(&xe, &xe, CW), CW), &ef(1.0), CW);
    let y = dcsevl(t, &ERFCS);
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &y, CW), CW), CW)
}
fn erf_m77_x(z: f64) -> f64 {
    let xe = ef(z.abs());
    let t = ext_sub(&ext_mul(&ef(2.0), &ext_mul(&xe, &xe, CW), CW), &ef(1.0), CW);
    let y = dcsevl(t, &ERFCS);
    ext_to_f64(&ext_mul(&xe, &y, CW), CW)
}
fn erf_m77_halfx2(z: f64) -> f64 {
    let xe = ef(z.abs());
    let t = ext_sub(&ext_mul(&ef(0.5), &ext_mul(&xe, &xe, CW), CW), &ef(1.0), CW);
    let y = dcsevl(t, &ERFCS);
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &y, CW), CW), CW)
}
fn erf_sch(z: f64) -> f64 {
    let xe = ef(z.abs());
    let t = ext_sub(&ext_mul(&ef(0.5), &ext_mul(&xe, &xe, CW), CW), &ef(1.0), CW);
    let y = clenshaw_prime(t, &SCH);
    ext_to_f64(&ext_mul(&xe, &y, CW), CW)
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
            if x < 0.0 || x >= 0.5 {
                continue;
            }
            let Some(p) = parse_bits_hex(&w.expected_bits) else {
                continue;
            };
            map.insert(x.to_bits(), (x, p.to_bits()));
        }
    }
    let rows: Vec<_> = map.into_values().collect();
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
    let poke = |x: f64, k: i32| {
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
    };
    let erf_j = |z: f64| {
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
        ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &acc, CW), CW), CW)
    };
    let mut erfcs_a1m4 = ERFCS;
    erfcs_a1m4[1] = poke(ERFCS[1], -4);
    let erf_a1m4 = |z: f64| {
        let xe = ef(z.abs());
        let t = ext_sub(&ext_mul(&ef(2.0), &ext_mul(&xe, &xe, CW), CW), &ef(1.0), CW);
        let y = dcsevl(t, &erfcs_a1m4);
        ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &y, CW), CW), CW)
    };
    let lows: Vec<(f64, u64)> = rows
        .iter()
        .copied()
        .filter(|&(z, pbits)| {
            let t = f64::from_bits(pbits);
            let g = erf_j(z);
            ulp_distance(g, t).unwrap_or(99) >= 2 && g < t
        })
        .collect();
    println!("leftover-low n={}", lows.len());
    let graphs: [(&str, Box<dyn Fn(f64) -> f64>); 5] = [
        ("Schonfelder", Box::new(erf_sch)),
        ("M77 x*(1+dcsevl)", Box::new(erf_m77_1p)),
        ("M77 a1-4", Box::new(erf_a1m4)),
        ("M77 x*dcsevl", Box::new(erf_m77_x)),
        ("M77 0.5x2-1", Box::new(erf_m77_halfx2)),
    ];
    for (name, ev) in &graphs {
        let mut hit = 0usize;
        let mut u1 = 0usize;
        let mut keep = 0usize;
        for &(z, pbits) in &rows {
            if ulp_distance(ev(z), f64::from_bits(pbits)).unwrap_or(99) == 0 {
                keep += 1;
            }
        }
        for &(z, pbits) in &lows {
            let d = ulp_distance(ev(z), f64::from_bits(pbits)).unwrap_or(99);
            if d == 0 {
                hit += 1;
                println!("  HIT {name} z={z:.16}");
            } else if d == 1 {
                u1 += 1;
            }
        }
        println!("{name:22} hard_hit={hit}/{} ulp1={u1} keep={keep}/{}", lows.len(), rows.len());
    }
}
