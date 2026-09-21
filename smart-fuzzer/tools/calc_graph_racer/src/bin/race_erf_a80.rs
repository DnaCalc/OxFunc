//! P-side A21 / MATH77 / Schonfelder coeffs as 80-bit RN of the printed decimals. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    ext_add, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN,
};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
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
const A21_80: [Ext80; 21] = [
    Ext80([0x6c, 0x44, 0xdb, 0xa6, 0x10, 0xd4, 0x75, 0x83, 0xfc, 0x3f]),
    Ext80([0x12, 0x36, 0xcf, 0x1b, 0x58, 0xa3, 0x93, 0xc0, 0xfd, 0xbf]),
    Ext80([0xaf, 0x0d, 0x5f, 0x21, 0xd0, 0x90, 0x17, 0xe7, 0xfb, 0x3f]),
    Ext80([0xf0, 0x86, 0x5a, 0x44, 0x89, 0x71, 0x16, 0xdc, 0xf9, 0xbf]),
    Ext80([0x2c, 0xf7, 0x29, 0x35, 0x87, 0xe6, 0x2d, 0xab, 0xf7, 0x3f]),
    Ext80([0x4d, 0x2c, 0x5c, 0x20, 0x00, 0xda, 0x16, 0xe0, 0xf4, 0xbf]),
    Ext80([0xd2, 0xbb, 0x4d, 0xdc, 0x7c, 0x93, 0xd1, 0xfc, 0xf1, 0x3f]),
    Ext80([0x9b, 0x5a, 0x17, 0x06, 0x1f, 0x2e, 0x69, 0xfa, 0xee, 0xbf]),
    Ext80([0xeb, 0x01, 0x8d, 0x32, 0xee, 0x64, 0xf3, 0xdc, 0xeb, 0x3f]),
    Ext80([0x08, 0x04, 0x36, 0x1f, 0x62, 0x17, 0xba, 0xaf, 0xe8, 0xbf]),
    Ext80([0xaf, 0xdc, 0xb6, 0x11, 0x03, 0xb9, 0x62, 0xfe, 0xe4, 0x3f]),
    Ext80([0x1f, 0xbe, 0xdc, 0x1e, 0x0e, 0x8c, 0xeb, 0xa8, 0xe1, 0xbf]),
    Ext80([0xa0, 0x32, 0x67, 0x62, 0xf0, 0x6a, 0x35, 0xcf, 0xdd, 0x3f]),
    Ext80([0x41, 0xd9, 0x98, 0xf9, 0x14, 0x9e, 0x22, 0xec, 0xd9, 0xbf]),
    Ext80([0x2c, 0x96, 0x46, 0x33, 0x0a, 0xd1, 0x41, 0xfb, 0xd5, 0x3f]),
    Ext80([0xff, 0xf5, 0xa3, 0xe0, 0x81, 0x14, 0xb7, 0xfa, 0xd1, 0xbf]),
    Ext80([0x89, 0x76, 0x01, 0x4d, 0x52, 0x7c, 0x7f, 0xeb, 0xcd, 0x3f]),
    Ext80([0x97, 0x26, 0x78, 0x16, 0xc4, 0x91, 0xbe, 0xd0, 0xc9, 0xbf]),
    Ext80([0xa6, 0x39, 0xc0, 0xb1, 0x5c, 0x65, 0xb4, 0xad, 0xc5, 0x3f]),
    Ext80([0x2a, 0x49, 0x4f, 0xf3, 0xef, 0x6a, 0x1b, 0x81, 0xc1, 0xbf]),
    Ext80([0x89, 0x7f, 0xcf, 0x72, 0x6b, 0xd4, 0x0b, 0x85, 0xbc, 0x3f]),
];
const M77_80: [Ext80; 21] = [
    Ext80([0xa8, 0x32, 0xae, 0x2d, 0xeb, 0x95, 0xe4, 0xc8, 0xfa, 0xbf]),
    Ext80([0xab, 0x9d, 0xd6, 0xa3, 0xdd, 0xeb, 0xac, 0x91, 0xfc, 0xbf]),
    Ext80([0xac, 0xb4, 0x21, 0xb1, 0x52, 0x48, 0x6c, 0xa4, 0xf8, 0x3f]),
    Ext80([0x27, 0xdc, 0x85, 0x1e, 0x4a, 0x86, 0x39, 0x97, 0xf4, 0xbf]),
    Ext80([0xbf, 0x81, 0x51, 0x4f, 0x9f, 0xda, 0x03, 0xe6, 0xef, 0x3f]),
    Ext80([0xcf, 0x69, 0x41, 0xfe, 0xe6, 0x0f, 0x38, 0x94, 0xeb, 0xbf]),
    Ext80([0x09, 0xc9, 0xb4, 0x1a, 0x89, 0xd5, 0x4e, 0xa5, 0xe6, 0x3f]),
    Ext80([0x29, 0xeb, 0x2a, 0x72, 0x94, 0xc1, 0x4b, 0xa2, 0xe1, 0xbf]),
    Ext80([0xb4, 0xa3, 0x43, 0xab, 0x30, 0x17, 0x35, 0x8e, 0xdc, 0x3f]),
    Ext80([0xa5, 0xc8, 0x9a, 0x38, 0x9b, 0x56, 0xed, 0xe0, 0xd6, 0xbf]),
    Ext80([0x04, 0x60, 0x6f, 0xfb, 0xa8, 0xc9, 0x0b, 0xa2, 0xd1, 0x3f]),
    Ext80([0xaf, 0x04, 0xa8, 0xf7, 0xab, 0x5f, 0x5d, 0xd6, 0xcb, 0xbf]),
    Ext80([0x0b, 0x67, 0x70, 0x6e, 0x3f, 0xcc, 0x09, 0x83, 0xc6, 0x3f]),
    Ext80([0xde, 0xf5, 0xf2, 0xb5, 0x7d, 0xce, 0xe6, 0x94, 0xc0, 0xbf]),
    Ext80([0x27, 0xa1, 0x67, 0x6f, 0x0d, 0x37, 0x0a, 0x9e, 0xba, 0x3f]),
    Ext80([0x07, 0x88, 0x4d, 0x84, 0x64, 0x3c, 0x5a, 0x9d, 0xb4, 0xbf]),
    Ext80([0xf4, 0x40, 0x38, 0xd9, 0x53, 0x03, 0x87, 0x93, 0xae, 0x3f]),
    Ext80([0xc4, 0x92, 0xf3, 0xbe, 0x58, 0xe5, 0xaf, 0x82, 0xa8, 0xbf]),
    Ext80([0xf4, 0x4f, 0xaf, 0xe8, 0x6f, 0x3c, 0x6e, 0xdb, 0xa1, 0x3f]),
    Ext80([0x9e, 0xe2, 0xdc, 0xbe, 0x1c, 0x71, 0x0f, 0xaf, 0x9b, 0xbf]),
    Ext80([0x89, 0x7f, 0xcf, 0x72, 0x6b, 0xd4, 0x0b, 0x85, 0x95, 0x3f]),
];
const SCH_80: [Ext80; 18] = [
    Ext80([0x51, 0xfc, 0x54, 0x8c, 0x24, 0x91, 0xd6, 0xbd, 0xff, 0x3f]),
    Ext80([0x9f, 0x07, 0x20, 0xef, 0xdb, 0xfc, 0x25, 0x9a, 0xfd, 0xbf]),
    Ext80([0x9d, 0xf3, 0x3f, 0x3f, 0x6b, 0x29, 0x4d, 0x8d, 0xfb, 0x3f]),
    Ext80([0x99, 0x57, 0x36, 0xb6, 0x7d, 0x12, 0x01, 0xe4, 0xf8, 0xbf]),
    Ext80([0x93, 0x58, 0x20, 0x6e, 0xc7, 0x46, 0xa6, 0x9e, 0xf6, 0x3f]),
    Ext80([0x78, 0xe3, 0x6a, 0x25, 0x3d, 0x6e, 0xd1, 0xbf, 0xf3, 0xbf]),
    Ext80([0x62, 0x71, 0x1d, 0x2f, 0x6f, 0x62, 0xee, 0xcb, 0xf0, 0x3f]),
    Ext80([0x8c, 0x16, 0xa9, 0x8d, 0xdf, 0xbc, 0xe9, 0xc0, 0xed, 0xbf]),
    Ext80([0x28, 0x26, 0xb7, 0x2f, 0xd6, 0xe3, 0x19, 0xa4, 0xea, 0x3f]),
    Ext80([0xaa, 0xe7, 0x6c, 0x4e, 0xa9, 0x4e, 0x5d, 0xfd, 0xe6, 0xbf]),
    Ext80([0x6f, 0xe5, 0xf3, 0x76, 0x1a, 0x56, 0xe9, 0xb2, 0xe3, 0x3f]),
    Ext80([0x27, 0x07, 0x3f, 0x65, 0xe0, 0xd6, 0xb5, 0xe8, 0xdf, 0xbf]),
    Ext80([0x5e, 0xef, 0x5c, 0x67, 0x8d, 0xf7, 0x36, 0x8c, 0xdc, 0x3f]),
    Ext80([0x04, 0xd7, 0xea, 0xdb, 0xd5, 0xfc, 0x5a, 0x9d, 0xd8, 0xbf]),
    Ext80([0x6a, 0x79, 0x0d, 0x63, 0x45, 0xe7, 0x34, 0xa5, 0xd4, 0x3f]),
    Ext80([0xf4, 0x9b, 0x88, 0x31, 0xf6, 0x18, 0xec, 0xa2, 0xd0, 0xbf]),
    Ext80([0x91, 0x5a, 0xbe, 0x28, 0x77, 0xae, 0x75, 0x97, 0xcc, 0x3f]),
    Ext80([0xc0, 0xf3, 0xd0, 0x88, 0xfa, 0xb5, 0x28, 0x85, 0xc8, 0xbf]),
];

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn erf_a21_f64(z: f64) -> f64 {
    let xe = ef(z.abs());
    let u = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in AS0.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &u, CW), &ef(c), CW);
    }
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &acc, CW), CW), CW)
}
fn erf_a21_80(z: f64) -> f64 {
    let xe = ef(z.abs());
    let u = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for c in A21_80.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &u, CW), c, CW);
    }
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &acc, CW), CW), CW)
}
fn dcsevl(t: Ext80, cs: &[Ext80]) -> Ext80 {
    let twox = ext_mul(&ef(2.0), &t, CW);
    let mut b0 = ef(0.0);
    let mut b1 = ef(0.0);
    let mut b2 = ef(0.0);
    for i in (0..cs.len()).rev() {
        b2 = b1;
        b1 = b0;
        b0 = ext_add(&ext_sub(&ext_mul(&twox, &b1, CW), &b2, CW), &cs[i], CW);
    }
    ext_mul(&ef(0.5), &ext_sub(&b0, &b2, CW), CW)
}
fn erf_m77_f64(z: f64) -> f64 {
    let xe = ef(z.abs());
    let t = ext_sub(&ext_mul(&ef(2.0), &ext_mul(&xe, &xe, CW), CW), &ef(1.0), CW);
    let cs: [Ext80; 21] = M77.map(ef);
    let y = dcsevl(t, &cs);
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &y, CW), CW), CW)
}
fn erf_m77_80(z: f64) -> f64 {
    let xe = ef(z.abs());
    let t = ext_sub(&ext_mul(&ef(2.0), &ext_mul(&xe, &xe, CW), CW), &ef(1.0), CW);
    let y = dcsevl(t, &M77_80);
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &y, CW), CW), CW)
}
fn clenshaw_prime(t: Ext80, a: &[Ext80]) -> Ext80 {
    let two = ef(2.0);
    let mut d1 = ef(0.0);
    let mut d2 = ef(0.0);
    for k in (1..a.len()).rev() {
        let dk = ext_add(
            &ext_sub(&ext_mul(&ext_mul(&two, &t, CW), &d1, CW), &d2, CW),
            &a[k],
            CW,
        );
        d2 = d1;
        d1 = dk;
    }
    ext_add(
        &ext_sub(&ext_mul(&t, &d1, CW), &d2, CW),
        &ext_mul(&ef(0.5), &a[0], CW),
        CW,
    )
}
fn erf_sch_f64(z: f64) -> f64 {
    let xe = ef(z.abs());
    let t = ext_sub(&ext_mul(&ef(0.5), &ext_mul(&xe, &xe, CW), CW), &ef(1.0), CW);
    let cs: [Ext80; 18] = SCH.map(ef);
    let y = clenshaw_prime(t, &cs);
    ext_to_f64(&ext_mul(&xe, &y, CW), CW)
}
fn erf_sch_80(z: f64) -> f64 {
    let xe = ef(z.abs());
    let t = ext_sub(&ext_mul(&ef(0.5), &ext_mul(&xe, &xe, CW), CW), &ef(1.0), CW);
    let y = clenshaw_prime(t, &SCH_80);
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
    println!(
        "A21[0] f64-wide {:016x} 80store {:016x}",
        ext_to_f64(&ef(AS0[0]), CW).to_bits(),
        ext_to_f64(&A21_80[0], CW).to_bits()
    );
    let graphs: [(&str, fn(f64) -> f64); 6] = [
        ("A21 f64-wide Horner", erf_a21_f64),
        ("A21 80-bit decimal Horner", erf_a21_80),
        ("M77 f64-wide DCSEVL", erf_m77_f64),
        ("M77 80-bit decimal DCSEVL", erf_m77_80),
        ("Schon f64-wide Clenshaw", erf_sch_f64),
        ("Schon 80-bit decimal Clenshaw", erf_sch_80),
    ];
    println!("P-side rows={}", rows.len());
    for (name, ev) in graphs {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut maxu = 0u64;
        for &(z, pbits) in &rows {
            let pg = ev(z);
            if !pg.is_finite() {
                continue;
            }
            let d = ulp_distance(pg, f64::from_bits(pbits)).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            n += 1;
            if d == 0 {
                ex += 1;
            } else {
                maxu = maxu.max(d);
            }
        }
        println!("{name:32} {ex}/{n} max={maxu}");
    }
}
