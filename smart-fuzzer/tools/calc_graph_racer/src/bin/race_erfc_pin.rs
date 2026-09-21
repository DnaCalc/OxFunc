//! PIN_Z ±1 ULP Excel Q bits vs 0x210 / libm. Flattening vs F-body. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;

const CW: u16 = CW_PC64_RN;
const C0: [f64; 9] = [
    0.564188496988670089,
    8.88314979438837594,
    66.1191906371416295,
    298.635138197400131,
    881.95222124176909,
    1712.04761263407058,
    2051.07837782607147,
    1230.33935479799725,
    2.15311535474403846e-8,
];
const D0: [f64; 8] = [
    15.7449261107098347,
    117.693950891312499,
    537.181101862009858,
    1621.38957456669019,
    3290.79923573345963,
    4362.61909014324716,
    3439.36767414372164,
    1230.33935480374942,
];
const PINS: [f64; 11] = [
    0.46875, 0.5, 0.75, 1.0, 1.28125, 1.875, 2.0, 2.125, 4.0, 5.0, 8.0,
];

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn maybe(x: Ext80, mask: u32, bit: u32) -> Ext80 {
    if bit < 32 && mask & (1u32 << bit) != 0 {
        ef(ext_to_f64(&x, CW))
    } else {
        x
    }
}
fn cody_cd(y: f64, mask: u32) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C0[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, mask, 0);
    xden = maybe(xden, mask, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C0[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &ye, CW);
        xnum = maybe(xnum, mask, 2 + 2 * i as u32);
        xden = maybe(xden, mask, 3 + 2 * i as u32);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(C0[7]), CW),
        &ext_add(&xden, &ef(D0[7]), CW),
        CW,
    );
    q = maybe(q, mask, 16);
    ext_to_f64(&q, CW)
}
fn qwf(z: f64, ff: f64) -> f64 {
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
    let mut map: BTreeMap<u64, (bool, u64)> = BTreeMap::new();
    for r in &rows {
        map.insert(r.z.to_bits(), (r.direct, r.qbits));
    }
    println!("pin  z  dir  excel  210  libm  vs_pin_excel  graph(pin) vs excel(z)");
    for &p in &PINS {
        let nbrs = [p.next_down(), p, p.next_up()];
        let pin_excel = map.get(&p.to_bits()).map(|t| t.1);
        for z in nbrs {
            let Some(&(dir, excel)) = map.get(&z.to_bits()) else {
                println!(
                    "  {:8.5} z={:.16}  -- missing from banks",
                    p, z
                );
                continue;
            };
            let t = f64::from_bits(excel);
            let g210 = qwf(z, cody_cd(z, 0x210));
            let d210 = ulp_distance(g210, t).unwrap_or(99);
            let dl = ulp_distance(libm::erfc(z), t).unwrap_or(99);
            let vs_pin = match pin_excel {
                Some(pb) if pb == excel => "SAME",
                Some(pb) => {
                    let d = ulp_distance(f64::from_bits(pb), t).unwrap_or(99);
                    if d > 20 {
                        "DIFF>20"
                    } else {
                        "DIFF"
                    }
                }
                None => "nopin",
            };
            let gpin = qwf(p, cody_cd(p, 0x210));
            let dgp = ulp_distance(gpin, t).unwrap_or(99);
            println!(
                "  p={p:.5} z={z:.16} dir={dir} excel={excel:#x} 210={d210}{} libm={dl}{} vs_pin={vs_pin} 210(p)vs_ex(z)={dgp}",
                if g210 < t { "L" } else if g210 > t { "H" } else { "=" },
                if libm::erfc(z) < t { "L" } else if libm::erfc(z) > t { "H" } else { "=" },
            );
        }
    }
}
