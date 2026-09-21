//! ccdd E0+1 vs PQR R[1]+1 leftover overlap and z-cuts. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use std::env;

const SIX: [f64; 6] = [
    4.0520833333333330,
    4.0625000000000000,
    5.2395833333333330,
    5.2708333333333330,
    5.3333333333333330,
    6.0000000000000000,
];
const P0: [f64; 8] = [
    0.16506148041280876191828601e-03,
    0.15471455377139313353998665e-03,
    0.44852548090298868465196794e-04,
    -0.49177280017226285450486205e-05,
    -0.69353602078656412367801676e-05,
    -0.20508667787746282746857743e-05,
    -0.28982842617824971177267380e-06,
    -0.17272433544836633301127174e-07,
];
const Q0: [f64; 8] = [
    1.0,
    0.16272656776533322859856317e+01,
    0.12040996037066026106794322e+01,
    0.52400246352158386907601472e+00,
    0.14497345252798672362384241e+00,
    0.25592517111042546492590736e-01,
    0.26869088293991371028123158e-02,
    0.13133767840925681614496481e-03,
];
const R0: [f64; 9] = [
    0.145589721275038539045668824025,
    -0.273421931495426482902320421863,
    0.226008066916621506788789064272,
    -0.163571895523923805648814425592,
    0.102604312032193978662297299832,
    -0.548023266949835519254211506880e-01,
    0.241432239725390106956523668160e-01,
    -0.822062115403915116036874169600e-02,
    0.180296241564687154310619200000e-02,
];
const CC0: [f64; 9] = [
    -0.7040906288250128001000086e-04,
    -0.3858822461760510359506941e-02,
    -0.7708202127512212359395078e-01,
    -0.6713655014557429480440263e+00,
    -0.2081992124162995545731882e+01,
    0.2898831421475282558867888e+01,
    0.2199509380600429331650192e+02,
    0.2907064664404115316722996e+01,
    -0.4766208741588182425380950e+02,
];
const DD0: [f64; 10] = [
    1.0,
    0.5238852785508439144747174e+02,
    0.9646843357714742409535148e+03,
    0.7007152775135939601804416e+04,
    0.8515386792259821780601162e+04,
    -0.1002360095177164564992134e+06,
    -0.2065250031331232815791912e+06,
    0.5695324805290370358175984e+06,
    0.6589752493461331195697873e+06,
    -0.1192930193156561957631462e+07,
];
const E0: f64 = 0.540464821348814822409610122136;
const E1: f64 = -0.261515522487415653487049835220e-01;
const E2: f64 = -0.288573438386338758794591212600e-02;
const E3: f64 = -0.529353396945788057720258856000e-03;

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
fn horner(cs: &[f64], x: f64) -> f64 {
    let mut a = 0.0;
    for &c in cs.iter().rev() {
        a = a * x + c;
    }
    a
}
fn pqr_r1(x: f64) -> f64 {
    let mut r = R0;
    r[1] = poke(R0[1], 1);
    let u = horner(&P0, x);
    let v = horner(&Q0, x);
    let t = (x - 3.75) / (x + 3.75);
    let mut acc = u / v;
    for &c in r.iter().rev() {
        acc = acc * t + c;
    }
    acc
}
fn e0p(x: f64) -> f64 {
    let z = 1.0 / (2.5 + x * x);
    let t = 13.0 * z - 1.0;
    let acc = (((horner(&CC0, z) / horner(&DD0, z) * t + E3) * t + E2) * t + E1) * t + poke(E0, 1);
    acc / x
}
fn qw(z: f64, ff: f64) -> f64 {
    let v = f::w_rn53(z) * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let tail: Vec<&f::QRow> = rows.iter().filter(|r| r.direct && r.z >= 4.0).collect();
    let mut both = 0usize;
    let mut only_e = 0usize;
    let mut only_r = 0usize;
    println!("6 leftover-low E0+1 vs R[1]+1:");
    for &lz in &SIX {
        let Some(r) = tail.iter().find(|rr| (rr.z - lz).abs() < 1e-12) else {
            continue;
        };
        let t = f64::from_bits(r.qbits);
        let ge = qw(lz, e0p(lz));
        let gr = qw(lz, pqr_r1(lz));
        let de = ulp_distance(ge, t).unwrap_or(99);
        let dr = ulp_distance(gr, t).unwrap_or(99);
        let se = if ge < t {
            "L"
        } else if ge > t {
            "H"
        } else {
            "="
        };
        let sr = if gr < t {
            "L"
        } else if gr > t {
            "H"
        } else {
            "="
        };
        println!("  z={lz:.16} E0+1={de}{se} R1={dr}{sr}");
    }
    for r in &tail {
        let t = f64::from_bits(r.qbits);
        let ee = ulp_distance(qw(r.z, e0p(r.z)), t).unwrap_or(99) == 0;
        let er = ulp_distance(qw(r.z, pqr_r1(r.z)), t).unwrap_or(99) == 0;
        match (ee, er) {
            (true, true) => both += 1,
            (true, false) => only_e += 1,
            (false, true) => only_r += 1,
            _ => {}
        }
    }
    println!(
        "DIRECT both={both} only_E0={only_e} only_R1={only_r} union={} n={}",
        both + only_e + only_r,
        tail.len()
    );
    let mut qe_e = 0usize;
    let mut qe_r = 0usize;
    let mut qe_u = 0usize;
    for row in rows.iter().filter(|rr| rr.z >= 4.0) {
        let t = f64::from_bits(row.qbits);
        let ee = ulp_distance(qw(row.z, e0p(row.z)), t).unwrap_or(99) == 0;
        let er = ulp_distance(qw(row.z, pqr_r1(row.z)), t).unwrap_or(99) == 0;
        if ee {
            qe_e += 1;
        }
        if er {
            qe_r += 1;
        }
        if ee || er {
            qe_u += 1;
        }
    }
    println!("Q z>=4 E0+1={qe_e} R1={qe_r} union={qe_u}");
    println!("cuts:");
    let cuts = [4.05, 4.1, 4.5, 5.0, 5.2, 5.35, 5.5, 6.0, 8.0];
    for &c in &cuts {
        let mut er_q = 0usize;
        let mut er_d = 0usize;
        let mut re_q = 0usize;
        let mut re_d = 0usize;
        let mut hit_er = 0usize;
        let mut hit_re = 0usize;
        for row in rows.iter().filter(|rr| rr.z >= 4.0) {
            let t = f64::from_bits(row.qbits);
            let ge = qw(row.z, e0p(row.z));
            let gr = qw(row.z, pqr_r1(row.z));
            let g = if row.z < c { ge } else { gr };
            let h = if row.z < c { gr } else { ge };
            if ulp_distance(g, t).unwrap_or(99) == 0 {
                er_q += 1;
                if row.direct {
                    er_d += 1;
                }
            }
            if ulp_distance(h, t).unwrap_or(99) == 0 {
                re_q += 1;
                if row.direct {
                    re_d += 1;
                }
            }
        }
        for &lz in &SIX {
            let Some(r) = tail.iter().find(|rr| (rr.z - lz).abs() < 1e-12) else {
                continue;
            };
            let t = f64::from_bits(r.qbits);
            let g = if lz < c {
                qw(lz, e0p(lz))
            } else {
                qw(lz, pqr_r1(lz))
            };
            let h = if lz < c {
                qw(lz, pqr_r1(lz))
            } else {
                qw(lz, e0p(lz))
            };
            if ulp_distance(g, t).unwrap_or(99) == 0 {
                hit_er += 1;
            }
            if ulp_distance(h, t).unwrap_or(99) == 0 {
                hit_re += 1;
            }
        }
        println!(
            "  cut={c} E0-then-R1 Q={er_q} d={er_d} hit={hit_er}/6  R1-then-E0 Q={re_q} d={re_d} hit={hit_re}/6"
        );
    }
}
