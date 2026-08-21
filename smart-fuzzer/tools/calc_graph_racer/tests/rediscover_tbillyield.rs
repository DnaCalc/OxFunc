//! Racer acceptance fixture 1: from witnesses alone, the association
//! enumerator + bit-exact scoring must uniquely rediscover TBILLYIELD's
//! signed-off calculation path
//!
//!     ((100 - price) / price) * (360 / days)
//!
//! among all 14 associations of `100 - p / p * 360 / days`. The witness bits
//! are generated from that pinned association (proven bit-identical to live
//! Excel across the 2156-case closure sweep, signed off 2026-07-10); the
//! former left-associative OxFunc path is among the candidates and must be
//! killed, as must every other tree.

use calc_graph_racer::dsl::{Candidate, ConstVal, EvalModel, GraphBuilder, NodeId, Op};
use calc_graph_racer::enumerate::{BinKind, describe_association, enumerate_associations};
use calc_graph_racer::eval::format_bits_hex;
use calc_graph_racer::score::{Witness, WitnessArg, race, survivors};

/// The pinned Excel association (closure-sweep evidence, G6-10 closed row).
fn tbillyield_excel(price: f64, days: f64) -> f64 {
    ((100.0 - price) / price) * (360.0 / days)
}

/// Deterministic xorshift64* for reproducible witness grids.
struct Rng(u64);
impl Rng {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }
    fn uniform(&mut self, lo: f64, hi: f64) -> f64 {
        let u = (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
        lo + u * (hi - lo)
    }
    fn int_in(&mut self, lo: u64, hi: u64) -> u64 {
        lo + self.next_u64() % (hi - lo + 1)
    }
}

fn scalar(v: f64) -> WitnessArg {
    WitnessArg::Scalar(format_bits_hex(v))
}

#[test]
fn rediscovers_tbillyield_association() {
    // ---- candidate space: all associations of 100 - p / p * 360 / days ----
    let leaf_100 =
        |b: &mut GraphBuilder| -> NodeId { b.strict(Op::Const(ConstVal::from_f64(100.0))) };
    let leaf_p = |b: &mut GraphBuilder| -> NodeId { b.strict(Op::Arg(0)) };
    let leaf_360 =
        |b: &mut GraphBuilder| -> NodeId { b.strict(Op::Const(ConstVal::from_f64(360.0))) };
    let leaf_days = |b: &mut GraphBuilder| -> NodeId { b.strict(Op::Arg(1)) };
    let leaves: Vec<&dyn Fn(&mut GraphBuilder) -> NodeId> =
        vec![&leaf_100, &leaf_p, &leaf_p, &leaf_360, &leaf_days];
    let ops = [BinKind::Sub, BinKind::Div, BinKind::Mul, BinKind::Div];
    let labels = ["100", "p", "p", "360", "days"];

    let graphs = enumerate_associations(&leaves, &ops, EvalModel::Strict);
    assert_eq!(graphs.len(), 14, "Catalan(4) association trees expected");

    let candidates: Vec<Candidate> = graphs
        .into_iter()
        .enumerate()
        .map(|(i, graph)| {
            let description = describe_association(&graph, &labels);
            Candidate {
                id: format!("assoc-{i:02}"),
                description,
                graph,
            }
        })
        .collect();

    let confirmed_desc = "(((100 - p) / p) * (360 / days))";
    assert!(
        candidates.iter().any(|c| c.description == confirmed_desc),
        "the confirmed association must be in the enumerated space; got: {:?}",
        candidates
            .iter()
            .map(|c| &c.description)
            .collect::<Vec<_>>()
    );

    // ---- witnesses: the closure-sweep shape (settl/duration/price grid) ----
    let mut rng = Rng(0x5EED_7B11_71E1D);
    let mut witnesses = Vec::new();
    // Structured sweep echoing the sign-off grid.
    for days in [
        7u64, 14, 28, 35, 56, 91, 120, 150, 182, 210, 245, 280, 320, 364,
    ] {
        for k in 0..11 {
            let price = 85.0 + 1.383 * k as f64 + 0.0137 * days as f64 / 364.0;
            let expected = tbillyield_excel(price, days as f64);
            witnesses.push(Witness {
                id: Some(format!("grid-{days}-{k}")),
                args: vec![scalar(price), scalar(days as f64)],
                expected_bits: format_bits_hex(expected),
            });
        }
    }
    // Random fill to separate every inequivalent tree.
    for i in 0..3000 {
        let price = rng.uniform(80.0, 99.999);
        let days = rng.int_in(1, 364) as f64;
        let expected = tbillyield_excel(price, days);
        witnesses.push(Witness {
            id: Some(format!("rand-{i}")),
            args: vec![scalar(price), scalar(days)],
            expected_bits: format_bits_hex(expected),
        });
    }

    // ---- race ----
    let results = race(&candidates, &witnesses, 4);
    let alive = survivors(&results);
    assert_eq!(
        alive.len(),
        1,
        "exactly one association must survive; survivors: {:?}",
        alive.iter().map(|r| &r.description).collect::<Vec<_>>()
    );
    assert_eq!(
        alive[0].description, confirmed_desc,
        "the survivor must be the signed-off Excel association"
    );

    // Every wrong tree must have been killed by a real bit mismatch (not a
    // structural artifact), i.e. the witness grid genuinely discriminates.
    for r in &results {
        if r.description == confirmed_desc {
            assert_eq!(r.exact, r.total, "confirmed path must be 100% exact");
        } else {
            assert!(
                r.score.inexact > 0,
                "candidate '{}' was not discriminated by the grid",
                r.description
            );
            assert_eq!(r.score.structural_mismatches, 0);
        }
    }
}
