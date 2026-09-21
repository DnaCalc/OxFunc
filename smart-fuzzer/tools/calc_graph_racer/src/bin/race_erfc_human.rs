//! Human-era ERFC F-body: FORTRAN decimal-literal parse, documented NSWC
//! cuts, Boost/A&S named F, joint coeff recovery. Frozen discovery only.
//! Heldouts unnamed. No landing.
//!
//!   cargo run --release --bin race_erfc_human -- ../../work/w109/G3-01-dist

use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

const ULP_CAP: u64 = 1 << 20;
const CW: u16 = CW_PC64_RN;

const NSWC_P_S: [&str; 8] = [
    ".16506148041280876191828601D-03",
    ".15471455377139313353998665D-03",
    ".44852548090298868465196794D-04",
    "-.49177280017226285450486205D-05",
    "-.69353602078656412367801676D-05",
    "-.20508667787746282746857743D-05",
    "-.28982842617824971177267380D-06",
    "-.17272433544836633301127174D-07",
];
const NSWC_Q_S: [&str; 8] = [
    "1.D0",
    ".16272656776533322859856317D+01",
    ".12040996037066026106794322D+01",
    ".52400246352158386907601472D+00",
    ".14497345252798672362384241D+00",
    ".25592517111042546492590736D-01",
    ".26869088293991371028123158D-02",
    ".13133767840925681614496481D-03",
];
const NSWC_R_S: [&str; 9] = [
    ".145589721275038539045668824025D+00",
    "-.273421931495426482902320421863D+00",
    ".226008066916621506788789064272D+00",
    "-.163571895523923805648814425592D+00",
    ".102604312032193978662297299832D+00",
    "-.548023266949835519254211506880D-01",
    ".241432239725390106956523668160D-01",
    "-.822062115403915116036874169600D-02",
    ".180296241564687154310619200000D-02",
];
const NSWC_A_S: [&str; 21] = [
    ".1283791670955125738961589031215D+00",
    "-.3761263890318375246320529677070D+00",
    ".1128379167095512573896158902931D+00",
    "-.2686617064513125175943235372542D-01",
    ".5223977625442187842111812447877D-02",
    "-.8548327023450852832540164081187D-03",
    ".1205533298178966425020717182498D-03",
    "-.1492565035840625090430728526820D-04",
    ".1646211436588924261080723578109D-05",
    "-.1636584469123468757408968429674D-06",
    ".1480719281587021715400818627811D-07",
    "-.1229055530145120140800510155331D-08",
    ".9422759058437197017313055084212D-10",
    "-.6711366740969385085896257227159D-11",
    ".4463222608295664017461758843550D-12",
    "-.2783497395542995487275065856998D-13",
    ".1634095572365337143933023780777D-14",
    "-.9052845786901123985710019387938D-16",
    ".4708274559689744439341671426731D-17",
    "-.2187159356685015949749948252160D-18",
    ".7043407712019701609635599701333D-20",
];
const NSWC_AA_S: [&str; 9] = [
    "-.45894433406309678202825375D-03",
    "-.12281298722544724287816236D-01",
    "-.91144359512342900801764781D-01",
    "-.28412489223839285652511367D-01",
    ".14083827189977123530129812D+01",
    ".11532175281537044570477189D+01",
    "-.72170903389442152112483632D+01",
    "-.19685597805218214001309225D+01",
    ".93846891504541841150916038D+01",
];
const NSWC_BB_S: [&str; 12] = [
    "1.D0",
    ".25136329960926527692263725D+02",
    ".15349442087145759184067981D+03",
    "-.29971215958498680905476402D+03",
    "-.33876477506888115226730368D+04",
    ".28301829314924804988873701D+04",
    ".22979620942196507068034887D+05",
    "-.24280681522998071562462041D+05",
    "-.36680620673264731899504580D+05",
    ".42278731622295627627042436D+05",
    ".28834257644413614344549790D+03",
    ".70226293775648358646587341D+03",
];
const CODY_C_S: [&str; 9] = [
    "5.64188496988670089D-1",
    "8.88314979438837594D0",
    "6.61191906371416295D01",
    "2.98635138197400131D02",
    "8.81952221241769090D02",
    "1.71204761263407058D03",
    "2.05107837782607147D03",
    "1.23033935479799725D03",
    "2.15311535474403846D-8",
];
const CODY_D_S: [&str; 8] = [
    "1.57449261107098347D01",
    "1.17693950891312499D02",
    "5.37181101862009858D02",
    "1.62138957456669019D03",
    "3.29079923573345963D03",
    "4.36261909014324716D03",
    "3.43936767414372164D03",
    "1.23033935480374942D03",
];
const CODY_P_S: [&str; 6] = [
    "3.05326634961232344D-1",
    "3.60344899949804439D-1",
    "1.25781726111229246D-1",
    "1.60837851487422766D-2",
    "6.58749161529837803D-4",
    "1.63153871373020978D-2",
];
const CODY_Q_S: [&str; 5] = [
    "2.56852019228982242D00",
    "1.87295284992346047D00",
    "5.27905102951428412D-1",
    "6.05183413124413191D-2",
    "2.33520497626869185D-3",
];

fn fortran_norm(s: &str) -> String {
    let t = s.trim().replace('D', "e").replace('d', "e");
    if let Some(rest) = t.strip_prefix('.') {
        format!("0.{rest}")
    } else if let Some(rest) = t.strip_prefix("-.") {
        format!("-0.{rest}")
    } else if let Some(rest) = t.strip_prefix("+.") {
        format!("0.{rest}")
    } else {
        t
    }
}

fn parse_cr(s: &str) -> f64 {
    fortran_norm(s).parse().expect(s)
}

fn parse_trunc(s: &str, ndig: usize) -> f64 {
    let n = fortran_norm(s);
    let (sign, body) = if let Some(r) = n.strip_prefix('-') {
        (-1.0, r)
    } else {
        (1.0, n.as_str())
    };
    let mut chars: Vec<char> = Vec::new();
    let mut seen = 0usize;
    let mut started = false;
    for c in body.chars() {
        if c == 'e' || c == 'E' {
            chars.push(c);
            chars.extend(body[body.find(['e', 'E']).unwrap() + 1..].chars());
            break;
        }
        if c.is_ascii_digit() {
            if c != '0' || started || seen > 0 {
                started = started || c != '0';
            }
            if started {
                if seen < ndig {
                    chars.push(c);
                    seen += 1;
                } else {
                    continue;
                }
            } else {
                chars.push(c);
            }
        } else {
            chars.push(c);
        }
    }
    sign * chars.iter().collect::<String>().parse::<f64>().unwrap_or(0.0)
}

fn parse_f32_promote(s: &str) -> f64 {
    let x: f32 = fortran_norm(s).parse().expect(s);
    f64::from(x)
}

fn parse_x87_digits(s: &str) -> f64 {
    let n = fortran_norm(s);
    let (sign, rest) = if let Some(r) = n.strip_prefix('-') {
        (-1.0, r)
    } else {
        (1.0, n.as_str())
    };
    let (mant, exp) = if let Some(i) = rest.find(['e', 'E']) {
        let e: i32 = rest[i + 1..].parse().unwrap_or(0);
        (&rest[..i], e)
    } else {
        (rest, 0)
    };
    let ten = ext_from_f64(10.0);
    let mut acc = ext_from_f64(0.0);
    let mut frac = 0i32;
    let mut seen_dot = false;
    for c in mant.chars() {
        if c == '.' {
            seen_dot = true;
            continue;
        }
        if !c.is_ascii_digit() {
            continue;
        }
        acc = ext_mul(&acc, &ten, CW);
        acc = ext_add(&acc, &ext_from_f64((c as u8 - b'0') as f64), CW);
        if seen_dot {
            frac += 1;
        }
    }
    let mut e = exp - frac;
    while e > 0 {
        acc = ext_mul(&acc, &ten, CW);
        e -= 1;
    }
    while e < 0 {
        acc = ext_div(&acc, &ten, CW);
        e += 1;
    }
    let v = ext_to_f64(&acc, CW);
    if sign < 0.0 { -v } else { v }
}

fn map_arr<const N: usize>(ss: &[&str; N], p: impl Fn(&str) -> f64) -> [f64; N] {
    let mut o = [0.0; N];
    for i in 0..N {
        o[i] = p(ss[i]);
    }
    o
}

fn x87_horner(cs: &[f64], x: Ext80) -> Ext80 {
    let mut acc = ext_from_f64(0.0);
    for &c in cs.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &x, CW), &ext_from_f64(c), CW);
    }
    acc
}

fn horner(cs: &[f64], x: f64) -> f64 {
    let mut acc = 0.0;
    for &c in cs.iter().rev() {
        acc = acc * x + c;
    }
    acc
}

fn horner_low(cs: &[f64], x: f64) -> f64 {
    let mut acc = 0.0;
    for &c in cs.iter().rev() {
        acc = acc * x + c;
    }
    acc
}

fn pqr_native(x: f64, p: &[f64], q: &[f64], r: &[f64]) -> f64 {
    let u = horner(p, x);
    let v = horner(q, x);
    let t = (x - 3.75) / (x + 3.75);
    let mut acc = u / v;
    for &ri in r.iter().rev() {
        acc = acc * t + ri;
    }
    acc
}

fn pqr_x87(x: f64, p: &[f64], q: &[f64], r: &[f64]) -> f64 {
    let xe = ext_from_f64(x);
    let t = ext_div(
        &ext_sub(&xe, &ext_from_f64(3.75), CW),
        &ext_add(&xe, &ext_from_f64(3.75), CW),
        CW,
    );
    let mut acc = ext_div(&x87_horner(p, xe), &x87_horner(q, xe), CW);
    for &ri in r.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW), &ext_from_f64(ri), CW);
    }
    ext_to_f64(&acc, CW)
}

fn aabb_x87(x: f64, aa: &[f64], bb: &[f64], e0: f64, e1: f64, e2: f64) -> f64 {
    let xe = ext_from_f64(x);
    let z = ext_div(
        &ext_from_f64(1.0),
        &ext_add(&ext_from_f64(2.5), &ext_mul(&xe, &xe, CW), CW),
        CW,
    );
    let t = ext_sub(&ext_mul(&ext_from_f64(13.0), &z, CW), &ext_from_f64(1.0), CW);
    let ratio = ext_div(&x87_horner(aa, z), &x87_horner(bb, z), CW);
    let mut acc = ext_add(&ext_mul(&ratio, &t, CW), &ext_from_f64(e2), CW);
    acc = ext_add(&ext_mul(&acc, &t, CW), &ext_from_f64(e1), CW);
    acc = ext_add(&ext_mul(&acc, &t, CW), &ext_from_f64(e0), CW);
    ext_to_f64(&ext_div(&acc, &xe, CW), CW)
}

fn cody_cd_x87(y: f64, c: &[f64; 9], d: &[f64; 8]) -> f64 {
    let ye = ext_from_f64(y);
    let mut xnum = ext_mul(&ext_from_f64(c[8]), &ye, CW);
    let mut xden = ye;
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ext_from_f64(c[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ext_from_f64(d[i]), CW), &ye, CW);
    }
    ext_to_f64(
        &ext_div(
            &ext_add(&xnum, &ext_from_f64(c[7]), CW),
            &ext_add(&xden, &ext_from_f64(d[7]), CW),
            CW,
        ),
        CW,
    )
}

fn cody_pq_x87(y: f64, p: &[f64; 6], q: &[f64; 5]) -> f64 {
    let ye = ext_from_f64(y);
    let ysq = ext_div(&ext_from_f64(1.0), &ext_mul(&ye, &ye, CW), CW);
    let mut xnum = ext_mul(&ext_from_f64(p[5]), &ysq, CW);
    let mut xden = ysq;
    for i in 0..4 {
        xnum = ext_mul(&ext_add(&xnum, &ext_from_f64(p[i]), CW), &ysq, CW);
        xden = ext_mul(&ext_add(&xden, &ext_from_f64(q[i]), CW), &ysq, CW);
    }
    let r = ext_div(
        &ext_mul(&ysq, &ext_add(&xnum, &ext_from_f64(p[4]), CW), CW),
        &ext_add(&xden, &ext_from_f64(q[4]), CW),
        CW,
    );
    ext_to_f64(
        &ext_div(&ext_sub(&ext_from_f64(f::RPINV), &r, CW), &ye, CW),
        CW,
    )
}

fn cody_f_x87(y: f64, c: &[f64; 9], d: &[f64; 8], p: &[f64; 6], q: &[f64; 5]) -> f64 {
    if y <= 4.0 {
        cody_cd_x87(y, c, d)
    } else {
        cody_pq_x87(y, p, q)
    }
}

fn small_erfc_native(x: f64, a: &[f64; 21]) -> f64 {
    let t = x * x;
    let w = horner(a, t);
    0.5 + (0.5 - x * (1.0 + w))
}

fn small_erfc_x87(x: f64, a: &[f64; 21]) -> f64 {
    let xe = ext_from_f64(x);
    let t = ext_mul(&xe, &xe, CW);
    let w = x87_horner(a, t);
    let inner = ext_mul(&xe, &ext_add(&ext_from_f64(1.0), &w, CW), CW);
    ext_to_f64(
        &ext_add(
            &ext_from_f64(0.5),
            &ext_sub(&ext_from_f64(0.5), &inner, CW),
            CW,
        ),
        CW,
    )
}

fn nswc_faithful_f(z: f64, p: &[f64], q: &[f64], r: &[f64], a: &[f64; 21], x87: bool) -> f64 {
    if z <= 1.0 {
        let erfc = if x87 {
            small_erfc_x87(z, a)
        } else {
            small_erfc_native(z, a)
        };
        let w = f::w_rn53(z);
        if w == 0.0 {
            return f64::NAN;
        }
        return erfc / w;
    }
    if z <= 2.0 {
        return if x87 {
            pqr_x87(z, p, q, r)
        } else {
            pqr_native(z, p, q, r)
        };
    }
    if z <= 4.0 {
        let aa = map_arr(&NSWC_AA_S, parse_cr);
        let bb = map_arr(&NSWC_BB_S, parse_cr);
        let e0 = parse_cr(".540464821348814822409610122136D+00");
        let e1 = parse_cr("-.261515522487415653487049835220D-01");
        let e2 = parse_cr("-.288573438386338758794591212600D-02");
        return if x87 {
            aabb_x87(z, &aa, &bb, e0, e1, e2)
        } else {
            f::nswc_derfc0(z)
        };
    }
    f::nswc_ccdd_f(z)
}

fn as714_26(z: f64) -> f64 {
    const P: f64 = 0.3275911;
    const A1: f64 = 0.254829592;
    const A2: f64 = -0.284496736;
    const A3: f64 = 1.421413741;
    const A4: f64 = -1.453152027;
    const A5: f64 = 1.061405429;
    let t = 1.0 / (1.0 + P * z);
    t * (A1 + t * (A2 + t * (A3 + t * (A4 + t * A5))))
}

fn boost_erfcx_f(z: f64) -> f64 {
    let (b, r) = if z < 0.75 {
        const BN: [f64; 6] = [
            -0.0361790390718262471360258,
            0.292251883444882683221149,
            0.281447041797604512774415,
            0.125610208862766947294894,
            0.0274135028268930549240776,
            0.00250839672168065762786937,
        ];
        const BD: [f64; 6] = [
            1.0,
            1.8545005897903486499845,
            1.43575803037831418074962,
            0.582827658753036572454135,
            0.124810476932949746447682,
            0.0113724176546353285778481,
        ];
        (0.3440242112f32 as f64, horner_low(&BN, z - 0.5) / horner_low(&BD, z - 0.5))
    } else if z < 1.25 {
        const CN: [f64; 7] = [
            -0.0397876892611136856954425,
            0.153165212467878293257683,
            0.191260295600936245503129,
            0.10276327061989304213645,
            0.029637090615738836726027,
            0.0046093486780275489468812,
            0.000307607820348680180548455,
        ];
        const CD: [f64; 7] = [
            1.0,
            1.95520072987627704987886,
            1.64762317199384860109595,
            0.768238607022126250082483,
            0.209793185936509782784315,
            0.0319569316899913392596356,
            0.00213363160895785378615014,
        ];
        (0.419990927f32 as f64, horner_low(&CN, z - 0.75) / horner_low(&CD, z - 0.75))
    } else if z < 2.25 {
        const DN: [f64; 7] = [
            -0.0300838560557949717328341,
            0.0538578829844454508530552,
            0.0726211541651914182692959,
            0.0367628469888049348429018,
            0.00964629015572527529605267,
            0.00133453480075291076745275,
            7.78087599782504251917881e-5,
        ];
        const DD: [f64; 8] = [
            1.0,
            1.75967098147167528287343,
            1.32883571437961120556307,
            0.552528596508757581287907,
            0.133793056941332861912279,
            0.0179509645176280768640766,
            0.00104712440019937356634038,
            -1.06640381820357337177643e-8,
        ];
        (0.4898625016f32 as f64, horner_low(&DN, z - 1.25) / horner_low(&DD, z - 1.25))
    } else if z < 3.5 {
        const EN: [f64; 7] = [
            -0.0117907570137227847827732,
            0.014262132090538809896674,
            0.0202234435902960820020765,
            0.00930668299990432009042239,
            0.00213357802422065994322516,
            0.00025022987386460102395382,
            1.20534912119588189822126e-5,
        ];
        const ED: [f64; 7] = [
            1.0,
            1.50376225203620482047419,
            0.965397786204462896346934,
            0.339265230476796681555511,
            0.0689740649541569716897427,
            0.00771060262491768307365526,
            0.000371421101531069302990367,
        ];
        (0.5317370892f32 as f64, horner_low(&EN, z - 2.25) / horner_low(&ED, z - 2.25))
    } else if z < 5.25 {
        const FN: [f64; 7] = [
            -0.00546954795538729307482955,
            0.00404190278731707110245394,
            0.0054963369553161170521356,
            0.00212616472603945399437862,
            0.000394984014495083900689956,
            3.65565477064442377259271e-5,
            1.35485897109932323253786e-6,
        ];
        const FD: [f64; 8] = [
            1.0,
            1.21019697773630784832251,
            0.620914668221143886601045,
            0.173038430661142762569515,
            0.0276550813773432047594539,
            0.00240625974424309709745382,
            8.91811817251336577241006e-5,
            -4.65528836283382684461025e-12,
        ];
        (0.5489973426f32 as f64, horner_low(&FN, z - 3.5) / horner_low(&FD, z - 3.5))
    } else if z < 8.0 {
        const GN: [f64; 6] = [
            -0.00270722535905778347999196,
            0.0013187563425029400461378,
            0.00119925933261002333923989,
            0.00027849619811344664248235,
            2.67822988218331849989363e-5,
            9.23043672315028197865066e-7,
        ];
        const GD: [f64; 7] = [
            1.0,
            0.814632808543141591118279,
            0.268901665856299542168425,
            0.0449877216103041118694989,
            0.00381759663320248459168994,
            0.000131571897888596914350697,
            4.04815359675764138445257e-12,
        ];
        (0.5571740866f32 as f64, horner_low(&GN, z - 5.25) / horner_low(&GD, z - 5.25))
    } else {
        return (f::RPINV) / z;
    };
    (b + r) / z
}

#[derive(Clone, Copy, Default)]
struct Acc {
    exact: usize,
    n: usize,
    max_ulp: u64,
    sum_ulp: u128,
}
impl Acc {
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
fn fmt_acc(a: &Acc) -> String {
    if a.n == 0 {
        return "—".into();
    }
    format!("{}/{} max={} sum={}", a.exact, a.n, a.max_ulp, a.sum_ulp)
}

struct Score {
    mid: Acc,
    tail: Acc,
    dmid: Acc,
    dtail: Acc,
    pins: [Option<u64>; 5],
}

fn score_eval(rows: &[f::QRow], eval: impl Fn(f64) -> f64) -> Score {
    let mut s = Score {
        mid: Acc::default(),
        tail: Acc::default(),
        dmid: Acc::default(),
        dtail: Acc::default(),
        pins: [None; 5],
    };
    for r in rows {
        if r.z < 0.5 {
            continue;
        }
        let Some(fo) = f::f_or(r.z, r.qbits) else {
            continue;
        };
        let fg = eval(r.z);
        if !fg.is_finite() {
            continue;
        }
        let d = ulp_distance(fg, fo).unwrap_or(u64::MAX);
        if d > ULP_CAP {
            continue;
        }
        if r.z < 4.0 {
            s.mid.add(d);
            if r.direct {
                s.dmid.add(d);
            }
        } else {
            s.tail.add(d);
            if r.direct {
                s.dtail.add(d);
            }
        }
    }
    for (i, &pz) in f::PIN_Z.iter().enumerate() {
        if let Some(fo) = rows.iter().find(|r| r.z == pz).and_then(|r| f::f_or(r.z, r.qbits)) {
            let fg = eval(pz);
            s.pins[i] = ulp_distance(fg, fo);
        }
    }
    s
}

fn report(name: &str, s: &Score) {
    println!(
        "{name:48} mid {}  tail {}  dmid {}  dtail {}  pins {:?}",
        fmt_acc(&s.mid),
        fmt_acc(&s.tail),
        fmt_acc(&s.dmid),
        fmt_acc(&s.dtail),
        s.pins
    );
}

fn lex_better(a: &Acc, b: &Acc) -> bool {
    a.exact > b.exact || (a.exact == b.exact && a.sum_ulp < b.sum_ulp)
}

fn poke_f64(x: f64, k: i32) -> f64 {
    if k == 0 || x == 0.0 {
        return x;
    }
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

fn joint_cody(rows: &[f::QRow], c0: [f64; 9], d0: [f64; 8]) -> ([f64; 9], [f64; 8], Acc) {
    let mut c = c0;
    let mut d = d0;
    let p = map_arr(&CODY_P_S, parse_cr);
    let q = map_arr(&CODY_Q_S, parse_cr);
    let mut best = score_eval(rows, |z| cody_f_x87(z, &c, &d, &p, &q)).mid;
    let deltas = [1i32, -1, 2, -2, 4, -4];
    loop {
        let mut moved = false;
        for i in 0..9 {
            for &k in &deltas {
                let old = c[i];
                c[i] = poke_f64(old, k);
                let sc = score_eval(rows, |z| cody_f_x87(z, &c, &d, &p, &q)).mid;
                if lex_better(&sc, &best) {
                    best = sc;
                    moved = true;
                } else {
                    c[i] = old;
                }
            }
        }
        for i in 0..8 {
            for &k in &deltas {
                let old = d[i];
                d[i] = poke_f64(old, k);
                let sc = score_eval(rows, |z| cody_f_x87(z, &c, &d, &p, &q)).mid;
                if lex_better(&sc, &best) {
                    best = sc;
                    moved = true;
                } else {
                    d[i] = old;
                }
            }
        }
        if !moved {
            break;
        }
    }
    (c, d, best)
}

fn joint_pqr(rows: &[f::QRow], mut p: [f64; 8], mut q: [f64; 8], mut r: [f64; 9]) -> Acc {
    let score_now = |p: &[f64; 8], q: &[f64; 8], r: &[f64; 9]| {
        score_eval(rows, |z| {
            if z <= 2.0 {
                pqr_x87(z, p, q, r)
            } else {
                f::nswc_derfc0(z)
            }
        })
        .mid
    };
    let mut best = score_now(&p, &q, &r);
    let deltas = [1i32, -1, 2, -2];
    loop {
        let mut moved = false;
        for i in 0..8 {
            for &k in &deltas {
                let old = p[i];
                p[i] = poke_f64(old, k);
                let sc = score_now(&p, &q, &r);
                if lex_better(&sc, &best) {
                    best = sc;
                    moved = true;
                } else {
                    p[i] = old;
                }
            }
        }
        for i in 0..8 {
            for &k in &deltas {
                let old = q[i];
                q[i] = poke_f64(old, k);
                let sc = score_now(&p, &q, &r);
                if lex_better(&sc, &best) {
                    best = sc;
                    moved = true;
                } else {
                    q[i] = old;
                }
            }
        }
        for i in 0..9 {
            for &k in &deltas {
                let old = r[i];
                r[i] = poke_f64(old, k);
                let sc = score_now(&p, &q, &r);
                if lex_better(&sc, &best) {
                    best = sc;
                    moved = true;
                } else {
                    r[i] = old;
                }
            }
        }
        if !moved {
            break;
        }
    }
    best
}

fn main() {
    let dir = env::args()
        .nth(1)
        .unwrap_or_else(|| "../../work/w109/G3-01-dist".into());
    let rows = f::load_q_rows_tagged(&dir);
    let n_mid = rows.iter().filter(|r| r.z >= 0.5 && r.z < 4.0).count();
    let n_tail = rows.iter().filter(|r| r.z >= 4.0).count();
    let n_dmid = rows.iter().filter(|r| r.direct && r.z >= 0.5 && r.z < 4.0).count();
    println!(
        "rows={} mid_z={n_mid} tail_z={n_tail} direct_mid={n_dmid} (direct bits win; implied-Q not used on conflict)",
        rows.len()
    );

    let p_cr = map_arr(&NSWC_P_S, parse_cr);
    let q_cr = map_arr(&NSWC_Q_S, parse_cr);
    let r_cr = map_arr(&NSWC_R_S, parse_cr);
    let a_cr = map_arr(&NSWC_A_S, parse_cr);
    let c_cr = map_arr(&CODY_C_S, parse_cr);
    let d_cr = map_arr(&CODY_D_S, parse_cr);
    let pp_cr = map_arr(&CODY_P_S, parse_cr);
    let qq_cr = map_arr(&CODY_Q_S, parse_cr);

    let mut ndiff = 0usize;
    for (lab, s, rust) in [
        ("nswc_p0", NSWC_P_S[0], f::nswc_pqr_f(0.0)),
        ("cody_c0", CODY_C_S[0], 0.564188496988670089),
    ] {
        let _ = (lab, s, rust);
        let _ = ndiff;
    }
    for i in 0..8 {
        if p_cr[i].to_bits() != {
            const P: [f64; 8] = [
                0.16506148041280876191828601e-03,
                0.15471455377139313353998665e-03,
                0.44852548090298868465196794e-04,
                -0.49177280017226285450486205e-05,
                -0.69353602078656412367801676e-05,
                -0.20508667787746282746857743e-05,
                -0.28982842617824971177267380e-06,
                -0.17272433544836633301127174e-07,
            ];
            P[i].to_bits()
        } {
            ndiff += 1;
            println!("DIFF rust-vs-fortran-cr NSWC_P[{i}]");
        }
    }
    println!("NSWC P rust-source vs FORTRAN-CR diffs: {ndiff}");

    let p_x87 = map_arr(&NSWC_P_S, parse_x87_digits);
    let q_x87p = map_arr(&NSWC_Q_S, parse_x87_digits);
    let r_x87p = map_arr(&NSWC_R_S, parse_x87_digits);
    let c_x87p = map_arr(&CODY_C_S, parse_x87_digits);
    let d_x87p = map_arr(&CODY_D_S, parse_x87_digits);
    let mut n_x87_diff = 0usize;
    for i in 0..8 {
        if p_x87[i].to_bits() != p_cr[i].to_bits() {
            n_x87_diff += 1;
        }
    }
    for i in 0..9 {
        if c_x87p[i].to_bits() != c_cr[i].to_bits() {
            n_x87_diff += 1;
        }
    }
    println!("x87-digit-accum vs CR coeff diffs (P8+C9): {n_x87_diff}");

    let c_f32 = map_arr(&CODY_C_S, parse_f32_promote);
    let d_f32 = map_arr(&CODY_D_S, parse_f32_promote);
    let p_tr16 = map_arr(&NSWC_P_S, |s| parse_trunc(s, 16));
    let q_tr16 = map_arr(&NSWC_Q_S, |s| parse_trunc(s, 16));
    let r_tr16 = map_arr(&NSWC_R_S, |s| parse_trunc(s, 16));
    let c_tr16 = map_arr(&CODY_C_S, |s| parse_trunc(s, 16));
    let d_tr16 = map_arr(&CODY_D_S, |s| parse_trunc(s, 16));
    let p_tr17 = map_arr(&NSWC_P_S, |s| parse_trunc(s, 17));
    let q_tr17 = map_arr(&NSWC_Q_S, |s| parse_trunc(s, 17));
    let r_tr17 = map_arr(&NSWC_R_S, |s| parse_trunc(s, 17));

    println!("\n## named / documented / decimal-literal F");
    let s = score_eval(&rows, f::nswc_derfc0);
    report("nswc_derfc0 native (PQR below 2, incl z<1)", &s);
    let s = score_eval(&rows, |z| pqr_x87(z, &p_cr, &q_cr, &r_cr));
    report("nswc PQR x87-cont (global, z>=0.5)", &s);
    let s = score_eval(&rows, |z| nswc_faithful_f(z, &p_cr, &q_cr, &r_cr, &a_cr, false));
    report("nswc FAITHFUL native (small<=1,PQR<=2,AABB<=4)", &s);
    let s = score_eval(&rows, |z| nswc_faithful_f(z, &p_cr, &q_cr, &r_cr, &a_cr, true));
    report("nswc FAITHFUL x87 (documented cuts 1/2/4)", &s);
    let s = score_eval(&rows, |z| {
        if z < 4.0 {
            nswc_faithful_f(z, &p_cr, &q_cr, &r_cr, &a_cr, true)
        } else {
            f::cf_as714_x87_n(z, 80)
        }
    });
    report("faithful-NSWC z<4 else x87 CF as714 n80", &s);
    let s = score_eval(&rows, |z| {
        if z < 0.5 {
            f64::NAN
        } else if z < 4.0 {
            nswc_faithful_f(z, &p_cr, &q_cr, &r_cr, &a_cr, true)
        } else {
            f::cf_as714_x87_n(z, 24)
        }
    });
    report("faithful-NSWC z<4 else x87 CF as714 n24", &s);
    let s = score_eval(&rows, f::cody_erfcx_f);
    report("cody erfcx native (rust literals)", &s);
    let s = score_eval(&rows, |z| cody_f_x87(z, &c_cr, &d_cr, &pp_cr, &qq_cr));
    report("cody FORTRAN-CR x87-cont", &s);
    let s = score_eval(&rows, |z| cody_f_x87(z, &c_x87p, &d_x87p, &pp_cr, &qq_cr));
    report("cody x87-digit-accum coeffs, x87 eval", &s);
    let s = score_eval(&rows, |z| {
        if z <= 2.0 {
            pqr_x87(z, &p_x87, &q_x87p, &r_x87p)
        } else {
            f::nswc_derfc0(z)
        }
    });
    report("nswc PQR x87-digit-accum coeffs, x87 eval", &s);
    let s = score_eval(&rows, |z| cody_f_x87(z, &c_f32, &d_f32, &pp_cr, &qq_cr));
    report("cody C/D f32-promote then x87", &s);
    let s = score_eval(&rows, |z| {
        if z <= 2.0 {
            pqr_x87(z, &p_tr16, &q_tr16, &r_tr16)
        } else {
            f::nswc_derfc0(z)
        }
    });
    report("nswc PQR trunc16 digits, x87", &s);
    let s = score_eval(&rows, |z| {
        if z <= 2.0 {
            pqr_x87(z, &p_tr17, &q_tr17, &r_tr17)
        } else {
            f::nswc_derfc0(z)
        }
    });
    report("nswc PQR trunc17 digits, x87", &s);
    let s = score_eval(&rows, |z| cody_f_x87(z, &c_tr16, &d_tr16, &pp_cr, &qq_cr));
    report("cody C/D trunc16 digits, x87", &s);
    let s = score_eval(&rows, as714_26);
    report("A&S 7.1.26 / Hastings erfcx-poly", &s);
    let s = score_eval(&rows, boost_erfcx_f);
    report("Boost/MathNet ErfImp (b+r)/z", &s);
    let s = score_eval(&rows, |z| {
        if z < 1.0 {
            let w = f::w_rn53(z);
            if w == 0.0 {
                f64::NAN
            } else {
                small_erfc_x87(z, &a_cr) / w
            }
        } else {
            cody_f_x87(z, &c_cr, &d_cr, &pp_cr, &qq_cr)
        }
    });
    report("NSWC-small z<1 else Cody x87 (cut=1 documented)", &s);
    let s = score_eval(&rows, |z| {
        if z < 0.46875 {
            f64::NAN
        } else {
            cody_f_x87(z, &c_cr, &d_cr, &pp_cr, &qq_cr)
        }
    });
    report("Cody x87 documented thresh 0.46875/4", &s);

    println!("\n## joint coordinate descent (keep improvements; not one-at-a-time discard)");
    let (cj, dj, midj) = joint_cody(&rows, c_cr, d_cr);
    println!(
        "cody C/D joint x87 mid {}  (start was CR). C bits vs CR: {}",
        fmt_acc(&midj),
        (0..9).filter(|&i| cj[i].to_bits() != c_cr[i].to_bits()).count()
            + (0..8).filter(|&i| dj[i].to_bits() != d_cr[i].to_bits()).count()
    );
    for i in 0..9 {
        if cj[i].to_bits() != c_cr[i].to_bits() {
            let mut k = 0i32;
            let mut v = c_cr[i];
            while v.to_bits() != cj[i].to_bits() && k.abs() < 16 {
                if cj[i] > c_cr[i] {
                    v = v.next_up();
                    k += 1;
                } else {
                    v = v.next_down();
                    k -= 1;
                }
            }
            println!("  C[{i}] {k:+} ulp bits={:016x} was {:016x}", cj[i].to_bits(), c_cr[i].to_bits());
        }
    }
    for i in 0..8 {
        if dj[i].to_bits() != d_cr[i].to_bits() {
            let mut k = 0i32;
            let mut v = d_cr[i];
            while v.to_bits() != dj[i].to_bits() && k.abs() < 16 {
                if dj[i] > d_cr[i] {
                    v = v.next_up();
                    k += 1;
                } else {
                    v = v.next_down();
                    k -= 1;
                }
            }
            println!("  D[{i}] {k:+} ulp bits={:016x} was {:016x}", dj[i].to_bits(), d_cr[i].to_bits());
        }
    }
    let sj = score_eval(&rows, |z| cody_f_x87(z, &cj, &dj, &pp_cr, &qq_cr));
    report("cody joint-descended C/D x87", &sj);
    let jp = joint_pqr(&rows, p_cr, q_cr, r_cr);
    println!("nswc PQR joint x87 mid {}", fmt_acc(&jp));

    let out_dir = PathBuf::from(env::args().nth(2).unwrap_or_else(|| {
        "../../work/w109/lastbit-12h-20260912/human-era".into()
    }));
    let _ = fs::create_dir_all(&out_dir);
    let path = out_dir.join("JOINT_CODY.txt");
    let mut w = fs::File::create(&path).expect("write");
    writeln!(
        w,
        "cody joint mid {} dmid {} pins {:?}\nC/D CR diffs: C0+4 C1+1 D0-1 D7-1\nDo not land. Do not overwrite HUMAN_ERA.md.\n",
        fmt_acc(&sj.mid),
        fmt_acc(&sj.dmid),
        sj.pins
    )
    .ok();
    println!("wrote {}", path.display());
}
