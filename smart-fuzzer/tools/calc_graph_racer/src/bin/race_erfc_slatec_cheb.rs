//! W109 ERFC body test 2: SLATEC / MATH77 Chebyshev DERFC (Fullerton FNLIB).
//!
//! Coefficients from netlib MATH77 `derf.f` (same ERFCS/ERC2CS/ERFCCS
//! packet as SLATEC FNLIB `DERFC`). Frozen discovery only; heldouts unnamed.
//!
//! Usage (from this crate):
//!   cargo run --release --bin race_erfc_slatec_cheb -- ../../work/w109/G3-01-dist

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research::excel_exp;
use std::collections::BTreeMap;

const FRAC_1_SQRT_2: f64 = f64::from_bits(0x3fe6a09e667f3bcd);
const ERFC_BANKS: [&str; 5] = [
    "answers-erfcp.json",
    "answers-erfcm.json",
    "answers-b7erfc.json",
    "answers-b8erfc.json",
    "answers-b11c.json",
];
const PIN_Z: [f64; 5] = [0.75, 1.28125, 1.875, 2.125, 5.0];

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
const ERC2CS: [f64; 49] = [
    -0.6960134660230950112739150826197e-1,
    -0.4110133936262089348982212084666e-1,
    0.3914495866689626881561143705244e-2,
    -0.4906395650548979161280935450774e-3,
    0.7157479001377036380760894141825e-4,
    -0.1153071634131232833808232847912e-4,
    0.1994670590201997635052314867709e-5,
    -0.3642666471599222873936118430711e-6,
    0.6944372610005012589931277214633e-7,
    -0.1371220902104366019534605141210e-7,
    0.2788389661007137131963860348087e-8,
    -0.5814164724331161551864791050316e-9,
    0.1238920491752753181180168817950e-9,
    -0.2690639145306743432390424937889e-10,
    0.5942614350847910982444709683840e-11,
    -0.1332386735758119579287754420570e-11,
    0.3028046806177132017173697243304e-12,
    -0.6966648814941032588795867588954e-13,
    0.1620854541053922969812893227628e-13,
    -0.3809934465250491999876913057729e-14,
    0.9040487815978831149368971012975e-15,
    -0.2164006195089607347809812047003e-15,
    0.5222102233995854984607980244172e-16,
    -0.1269729602364555336372415527780e-16,
    0.3109145504276197583836227412951e-17,
    -0.7663762920320385524009566714811e-18,
    0.1900819251362745202536929733290e-18,
    -0.4742207279069039545225655999965e-19,
    0.1189649200076528382880683078451e-19,
    -0.3000035590325780256845271313066e-20,
    0.7602993453043246173019385277098e-21,
    -0.1935909447606872881569811049130e-21,
    0.4951399124773337881000042386773e-22,
    -0.1271807481336371879608621989888e-22,
    0.3280049600469513043315841652053e-23,
    -0.8492320176822896568924792422399e-24,
    0.2206917892807560223519879987199e-24,
    -0.5755617245696528498312819507199e-25,
    0.1506191533639234250354144051199e-25,
    -0.3954502959018796953104285695999e-26,
    0.1041529704151500979984645051733e-26,
    -0.2751487795278765079450178901333e-27,
    0.7290058205497557408997703680000e-28,
    -0.1936939645915947804077501098666e-28,
    0.5160357112051487298370054826666e-29,
    -0.1378419322193094099389644800000e-29,
    0.3691326793107069042251093333333e-30,
    -0.9909389590624365420653226666666e-31,
    0.2666491705195388413323946666666e-31,
];
const ERFCCS: [f64; 59] = [
    0.715179310202924774503697709496e-1,
    -0.265324343376067157558893386681e-1,
    0.171115397792085588332699194606e-2,
    -0.163751663458517884163746404749e-3,
    0.198712935005520364995974806758e-4,
    -0.284371241276655508750175183152e-5,
    0.460616130896313036969379968464e-6,
    -0.822775302587920842057766536366e-7,
    0.159214187277090112989358340826e-7,
    -0.329507136225284321486631665072e-8,
    0.722343976040055546581261153890e-9,
    -0.166485581339872959344695966886e-9,
    0.401039258823766482077671768814e-10,
    -0.100481621442573113272170176283e-10,
    0.260827591330033380859341009439e-11,
    -0.699111056040402486557697812476e-12,
    0.192949233326170708624205749803e-12,
    -0.547013118875433106490125085271e-13,
    0.158966330976269744839084032762e-13,
    -0.472689398019755483920369584290e-14,
    0.143587337678498478672873997840e-14,
    -0.444951056181735839417250062829e-15,
    0.140481088476823343737305537466e-15,
    -0.451381838776421089625963281623e-16,
    0.147452154104513307787018713262e-16,
    -0.489262140694577615436841552532e-17,
    0.164761214141064673895301522827e-17,
    -0.562681717632940809299928521323e-18,
    0.194744338223207851429197867821e-18,
    -0.682630564294842072956664144723e-19,
    0.242198888729864924018301125438e-19,
    -0.869341413350307042563800861857e-20,
    0.315518034622808557122363401262e-20,
    -0.115737232404960874261239486742e-20,
    0.428894716160565394623737097442e-21,
    -0.160503074205761685005737770964e-21,
    0.606329875745380264495069923027e-22,
    -0.231140425169795849098840801367e-22,
    0.888877854066188552554702955697e-23,
    -0.344726057665137652230718495566e-23,
    0.134786546020696506827582774181e-23,
    -0.531179407112502173645873201807e-24,
    0.210934105861978316828954734537e-24,
    -0.843836558792378911598133256738e-25,
    0.339998252494520890627359576337e-25,
    -0.137945238807324209002238377110e-25,
    0.563449031183325261513392634811e-26,
    -0.231649043447706544823427752700e-26,
    0.958446284460181015263158381226e-27,
    -0.399072288033010972624224850193e-27,
    0.167212922594447736017228709669e-27,
    -0.704599152276601385638803782587e-28,
    0.297976840286420635412357989444e-28,
    -0.126252246646061929722422632994e-28,
    0.539543870454248793985299653154e-29,
    -0.238099288253145918675346190062e-29,
    0.109905283010276157359726683750e-29,
    -0.486771374164496572732518677435e-30,
    0.152587726411035756763200828211e-30,
];
const PS: [f64; 9] = [
    1.00000000000036828,
    1.87051017604560834,
    1.74642369370058320,
    1.02438464807598001,
    4.07413180167223764e-1,
    1.11870870991098165e-1,
    2.07045775788719818e-2,
    2.37133372752999036e-3,
    1.29992515945788642e-4,
];
const QS: [f64; 10] = [
    1.00000000000000000,
    2.99888934314798253,
    4.13030795287321183,
    3.43830153103630866,
    1.91273588328781533,
    7.40352738163508723e-1,
    2.00387662412610424e-1,
    3.68131014202168126e-2,
    4.20307996290648223e-3,
    2.30405728794132537e-4,
];

fn flush(v: f64) -> f64 {
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}

fn initds(os: &[f64], eta: f64) -> usize {
    let mut err = 0.0;
    for i in (0..os.len()).rev() {
        err += os[i].abs();
        if err > eta {
            return (i + 1).max(2);
        }
    }
    2
}

fn dcsevl(x: f64, cs: &[f64], n: usize) -> f64 {
    let twox = x + x;
    let mut b2 = 0.0;
    let mut b1 = 0.0;
    let mut b0 = 0.0;
    for i in (0..n).rev() {
        b2 = b1;
        b1 = b0;
        b0 = twox * b1 - b2 + cs[i];
    }
    0.5 * (b0 - b2)
}

fn horner(cs: &[f64], x: f64) -> f64 {
    let mut acc = 0.0;
    for &c in cs.iter().rev() {
        acc = acc * x + c;
    }
    acc
}

fn derf1(x: f64, nterf: usize) -> f64 {
    x + x * dcsevl(2.0 * x * x - 1.0, &ERFCS, nterf)
}

fn derfc1(y: f64, nterc2: usize, nterfc: usize) -> f64 {
    let ysq = y * y;
    if ysq <= 4.0 {
        0.5 + dcsevl((8.0 / ysq - 5.0) / 3.0, &ERC2CS, nterc2)
    } else {
        0.5 + dcsevl(8.0 / ysq - 1.0, &ERFCCS, nterfc)
    }
}

fn derfe1(y: f64) -> f64 {
    horner(&PS, y) / horner(&QS, y)
}

fn slatec_erfc(z: f64, nterf: usize, nterc2: usize, nterfc: usize) -> f64 {
    let y = z.abs();
    let q = if y <= 1.0 {
        1.0 - derf1(y, nterf)
    } else {
        let f = derfc1(y, nterc2, nterfc);
        flush(excel_exp(-(y * y)) * f / y)
    };
    flush(if z < 0.0 { 2.0 - q } else { q })
}

fn math77_ieee_erfc(z: f64, nterf: usize, nterc2: usize, nterfc: usize) -> f64 {
    let y = z.abs();
    let q = if y <= 0.5 {
        1.0 - derf1(if z < 0.0 { -y } else { y }, nterf)
    } else if y <= 1.0 {
        flush(excel_exp(-(y * y)) * derfe1(y))
    } else {
        let f = derfc1(y, nterc2, nterfc);
        flush(excel_exp(-(y * y)) * f / y)
    };
    flush(if z < 0.0 { 2.0 - q } else { q })
}

struct Acc {
    exact: usize,
    n: usize,
    max_ulp: u64,
    sum_ulp: u128,
}
impl Acc {
    fn new() -> Self {
        Self {
            exact: 0,
            n: 0,
            max_ulp: 0,
            sum_ulp: 0,
        }
    }
    fn add(&mut self, d: u64) {
        self.n += 1;
        if d == 0 {
            self.exact += 1;
        } else {
            self.max_ulp = self.max_ulp.max(d);
            self.sum_ulp += d as u128;
        }
    }
}

fn band_idx(z: f64) -> usize {
    if z < 0.5 {
        0
    } else if z < 4.0 {
        1
    } else {
        2
    }
}

fn score(rows: &[(f64, u64)], eval: impl Fn(f64) -> f64) -> ([Acc; 3], Acc) {
    let mut bands = [Acc::new(), Acc::new(), Acc::new()];
    let mut all = Acc::new();
    for &(z, expected) in rows {
        let got = eval(z);
        let d = ulp_distance(got, f64::from_bits(expected)).unwrap_or(u64::MAX);
        all.add(d);
        bands[band_idx(z)].add(d);
    }
    (bands, all)
}

fn fmt(a: &Acc) -> String {
    format!("{}/{} max={} sum={}", a.exact, a.n, a.max_ulp, a.sum_ulp)
}

fn load_rows(dir: &str) -> Vec<(f64, u64)> {
    let mut rows = BTreeMap::new();
    for name in ERFC_BANKS {
        let path = format!("{dir}/{name}");
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let bank: WitnessSet = serde_json::from_str(&text).expect(&path);
        for w in &bank.witnesses {
            let z = match &w.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).expect("z"),
                _ => continue,
            };
            let Some(q) = parse_bits_hex(&w.expected_bits) else {
                continue;
            };
            if z.is_finite() && z >= 0.0 {
                rows.entry(z.to_bits()).or_insert(q.to_bits());
            }
        }
    }
    let path = format!("{dir}/answers-b24-normref.json");
    if let Ok(text) = std::fs::read_to_string(&path) {
        let bank: WitnessSet = serde_json::from_str(&text).expect("normref");
        for w in &bank.witnesses {
            if w.args.len() < 2 {
                continue;
            }
            let x = match &w.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).expect("x"),
                _ => continue,
            };
            let cum = match &w.args[1] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).expect("c"),
                _ => continue,
            };
            if cum != 1.0 || !x.is_sign_negative() {
                continue;
            }
            let Some(ns) = parse_bits_hex(&w.expected_bits) else {
                continue;
            };
            let z = x.abs() * FRAC_1_SQRT_2;
            rows.entry(z.to_bits()).or_insert((ns * 2.0).to_bits());
        }
    }
    rows.into_iter()
        .map(|(z, q)| (f64::from_bits(z), q))
        .collect()
}

fn main() {
    let dir = std::env::args().nth(1).expect("G3-01-dist directory");
    assert!(!dir.contains("heldout"));
    let rows = load_rows(&dir);
    let eta = 0.1 * f64::EPSILON;
    let nterf = initds(&ERFCS, eta);
    let nterc2 = initds(&ERC2CS, eta);
    let nterfc = initds(&ERFCCS, eta);
    println!(
        "{} rows; INITDS nterf={nterf} nterc2={nterc2} nterfc={nterfc} (MATH77 IEEE table 12/24/25); heldout absent",
        rows.len()
    );
    println!(
        "{:<28} {:>22} {:>22} {:>22} {:>22}",
        "graph", "small z<0.5", "mid [0.5,4)", "tail z>=4", "all"
    );

    let (b, a) = score(&rows, |z| slatec_erfc(z, nterf, nterc2, nterfc));
    println!(
        "{:<28} {:>22} {:>22} {:>22} {:>22}",
        "slatec_cheb_unsplit",
        fmt(&b[0]),
        fmt(&b[1]),
        fmt(&b[2]),
        fmt(&a)
    );
    let (b2, a2) = score(&rows, |z| math77_ieee_erfc(z, nterf, nterc2, nterfc));
    println!(
        "{:<28} {:>22} {:>22} {:>22} {:>22}",
        "math77_ieee_unsplit",
        fmt(&b2[0]),
        fmt(&b2[1]),
        fmt(&b2[2]),
        fmt(&a2)
    );
    // MATH77 documented IEEE term counts, in case INITDS differs.
    let (b3, a3) = score(&rows, |z| slatec_erfc(z, 12, 24, 25));
    println!(
        "{:<28} {:>22} {:>22} {:>22} {:>22}",
        "slatec_nter_12_24_25",
        fmt(&b3[0]),
        fmt(&b3[1]),
        fmt(&b3[2]),
        fmt(&a3)
    );

    println!("pins slatec_cheb_unsplit:");
    for &z in &PIN_Z {
        if let Some((_, expected)) = rows.iter().find(|(zz, _)| *zz == z) {
            let got = slatec_erfc(z, nterf, nterc2, nterfc);
            let d = ulp_distance(got, f64::from_bits(*expected)).unwrap_or(u64::MAX);
            println!(
                "  z={z} got={:#x} excel={expected:#x} ulp={d}",
                got.to_bits()
            );
        }
    }
}
