//! Check pin UNIQUENESS: for each collision config, how many distinct em in a wide window satisfy
//! ALL 128 pv rows? If >1, the "distinct em per group" is partly pin softness, not pure signal.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
fn main() {
    let ws: WitnessSet = serde_json::from_str(
        &std::fs::read_to_string("../../work/w109/G6-solvers/answers-pmt-collide.json").unwrap(),
    )
    .unwrap();
    let wl = &ws.witnesses;
    let ncfg = wl.len() / 128;
    let mut widths = std::collections::BTreeMap::<usize, usize>::new();
    let mut pinned = 0;
    for ci in 0..ncfg {
        let mut rows = Vec::new();
        let mut rn = (0.0f64, 0.0f64);
        for j in 0..128 {
            let w = &wl[ci * 128 + j];
            let a: Vec<f64> = w
                .args
                .iter()
                .filter_map(|x| match x {
                    WitnessArg::Scalar(s) => parse_bits_hex(s),
                    _ => None,
                })
                .collect();
            rn = (a[0], a[1]);
            rows.push((a[2], parse_bits_hex(&w.expected_bits).unwrap().to_bits()));
        }
        let (r, n) = rn;
        let center = rx::excel_expm1_internal(-(n * rx::excel_log1p(r)));
        let cb = center.to_bits() as i64;
        let mut cnt = 0;
        for d in -30..=30i64 {
            let e = f64::from_bits((cb + d) as u64);
            if e >= 0.0 {
                continue;
            }
            if rows
                .iter()
                .all(|(pv, want)| ((pv / e) * r).to_bits() == *want)
            {
                cnt += 1;
            }
        }
        if cnt > 0 {
            pinned += 1;
            *widths.entry(cnt).or_default() += 1;
        }
    }
    println!("configs with a valid pin: {}", pinned);
    println!(
        "pin-window multiplicity histogram (#em satisfying all 128 : #configs): {:?}",
        widths
    );
}
