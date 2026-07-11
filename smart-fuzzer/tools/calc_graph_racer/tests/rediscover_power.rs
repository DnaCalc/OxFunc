//! Racer acceptance fixture 2: from witnesses alone, the racer must uniquely
//! rediscover POWER's signed-off x87 staging (BUG-FUNC-042, 715/715 live):
//!
//!   y < 0  ->  RN53(RN64(1 / pos))  where pos = POWER-positive(x, |y|)
//!   else   ->  POWER-positive(x, y)
//!   POWER-positive: |y| == 0.5 -> SQRTSD(x), else x87 exp(RN53(RN64(y·ln x)))
//!
//! against the plausible rivals: platform powf, x87 exp/ln with a
//! single-rounded (SSE) product, x87 exp/ln with the double-rounded product
//! but no sqrt/reciprocal staging, and the confirmed graph with a strict
//! (single-rounded) reciprocal. Witness bits come from the signed-off
//! production composition; discriminating inputs are *searched for* offline —
//! the same distinguishing-input discipline the scheduler uses.

use calc_graph_racer::dsl::{Candidate, EvalModel, GraphBuilder, Op, Pred};
use calc_graph_racer::eval::format_bits_hex;
use calc_graph_racer::scheduler::{ProbeCase, rank_distinguishing};
use calc_graph_racer::score::{Witness, WitnessArg, race, survivors};
use oxfunc_core::excel_numeric::research as rx;

/// The signed-off production composition (the fixture's oracle stand-in).
fn excel_power_ref(x: f64, y: f64) -> f64 {
    if y < 0.0 {
        rx::x87_recip(rx::excel_pow_positive(x, -y))
    } else {
        rx::excel_pow_positive(x, y)
    }
}

fn scalar(v: f64) -> WitnessArg {
    WitnessArg::Scalar(format_bits_hex(v))
}

fn witness(id: String, x: f64, y: f64) -> Witness {
    Witness {
        id: Some(id),
        args: vec![scalar(x), scalar(y)],
        expected_bits: format_bits_hex(excel_power_ref(x, y)),
    }
}

struct Rng(u64);
impl Rng {
    fn next_u64(&mut self) -> u64 {
        let mut v = self.0;
        v ^= v >> 12;
        v ^= v << 25;
        v ^= v >> 27;
        self.0 = v;
        v.wrapping_mul(0x2545F4914F6CDD1D)
    }
    fn uniform(&mut self, lo: f64, hi: f64) -> f64 {
        let u = (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
        lo + u * (hi - lo)
    }
}

/// Confirmed graph: branch on sign, reciprocal staging through the x87
/// double-rounded reciprocal, positive path = the Excel POWER-positive kernel.
fn confirmed() -> Candidate {
    let mut b = GraphBuilder::new();
    let x = b.strict(Op::Arg(0));
    let y = b.strict(Op::Arg(1));
    let abs_y = b.strict(Op::Abs(y));
    let pos = b.strict(Op::PowExcelPositive { base: x, exp: abs_y });
    let rec = b.push(Op::Recip(pos), EvalModel::X87_STORED);
    let direct = b.strict(Op::PowExcelPositive { base: x, exp: y });
    let out = b.strict(Op::Branch {
        pred: Pred::IsNeg(y),
        then_node: rec,
        else_node: direct,
    });
    Candidate {
        id: "confirmed-x87-recip-staging".into(),
        description: "y<0 -> x87recip(pow_pos(x,|y|)); else pow_pos(x,y)".into(),
        graph: b.finish(out),
    }
}

/// Rival: identical staging but a strict (single-rounded SSE) reciprocal.
fn strict_recip_variant() -> Candidate {
    let mut b = GraphBuilder::new();
    let x = b.strict(Op::Arg(0));
    let y = b.strict(Op::Arg(1));
    let abs_y = b.strict(Op::Abs(y));
    let pos = b.strict(Op::PowExcelPositive { base: x, exp: abs_y });
    let rec = b.strict(Op::Recip(pos));
    let direct = b.strict(Op::PowExcelPositive { base: x, exp: y });
    let out = b.strict(Op::Branch {
        pred: Pred::IsNeg(y),
        then_node: rec,
        else_node: direct,
    });
    Candidate {
        id: "strict-recip-staging".into(),
        description: "y<0 -> (1/pow_pos) single-rounded; else pow_pos".into(),
        graph: b.finish(out),
    }
}

/// Rival: x87 exp/ln, single-rounded product, no sqrt case, direct negative y.
fn single_rounded_mul() -> Candidate {
    let mut b = GraphBuilder::new();
    let x = b.strict(Op::Arg(0));
    let y = b.strict(Op::Arg(1));
    let ln = b.push(Op::Ln(x), EvalModel::X87_STORED);
    let t = b.strict(Op::Mul(y, ln));
    let out = b.push(Op::Exp(t), EvalModel::X87_STORED);
    Candidate {
        id: "x87-exp-ln-sse-mul".into(),
        description: "exp(y*ln x), x87 exp/ln, single-rounded product".into(),
        graph: b.finish(out),
    }
}

/// Rival: x87 exp/ln with the double-rounded product, but no sqrt special
/// case and no reciprocal staging.
fn double_rounded_mul_no_staging() -> Candidate {
    let mut b = GraphBuilder::new();
    let x = b.strict(Op::Arg(0));
    let y = b.strict(Op::Arg(1));
    let ln = b.push(Op::Ln(x), EvalModel::X87_STORED);
    let t = b.push(Op::Mul(y, ln), EvalModel::X87_STORED);
    let out = b.push(Op::Exp(t), EvalModel::X87_STORED);
    Candidate {
        id: "x87-exp-ln-x87-mul-direct".into(),
        description: "exp(RN53(RN64(y*ln x))), no sqrt/recip staging".into(),
        graph: b.finish(out),
    }
}

/// Rival: the platform powf (current-OxFunc-before-W108 comparator).
fn platform_powf() -> Candidate {
    let mut b = GraphBuilder::new();
    let x = b.strict(Op::Arg(0));
    let y = b.strict(Op::Arg(1));
    let out = b.strict(Op::PowfStrict { base: x, exp: y });
    Candidate {
        id: "platform-powf".into(),
        description: "platform powf(x, y)".into(),
        graph: b.finish(out),
    }
}

#[test]
fn rediscovers_power_staging() {
    let candidates = vec![
        confirmed(),
        strict_recip_variant(),
        single_rounded_mul(),
        double_rounded_mul_no_staging(),
        platform_powf(),
    ];

    // ---- witness set ----
    let mut witnesses = Vec::new();
    // Known discriminators from the sign-off evidence.
    for (i, (x, y)) in [
        (2.0, 0.5),   // sqrt special case (exp chain is 1 ULP off)
        (2.0, -0.5),  // sqrt + reciprocal staging
        (10.0, 0.3),
        (2.0, 10.0),
        (1.5, -3.0),
        (7.5, -0.25),
    ]
    .iter()
    .enumerate()
    {
        witnesses.push(witness(format!("special-{i}"), *x, *y));
    }
    // Broad random coverage.
    let mut rng = Rng(0x0109_DA7A_5EED_0001);
    for i in 0..4000 {
        let x = rng.uniform(0.05, 40.0);
        let mut y = rng.uniform(-6.0, 6.0);
        if y == 0.0 {
            y = 0.75;
        }
        witnesses.push(witness(format!("rand-{i}"), x, y));
    }
    // Searched discriminators: reciprocal double-rounding windows (kills the
    // strict-reciprocal rival). y = -1 puts the entire difference in the
    // reciprocal staging.
    let mut recip_hits = 0;
    for i in 0..400_000 {
        let x = rng.uniform(1.0, 2.0);
        let pos = rx::excel_pow_positive(x, 1.0);
        if (1.0 / pos).to_bits() != rx::x87_recip(pos).to_bits() {
            witnesses.push(witness(format!("recip-window-{i}"), x, -1.0));
            recip_hits += 1;
            if recip_hits >= 8 {
                break;
            }
        }
    }
    assert!(
        recip_hits >= 1,
        "no reciprocal double-rounding discriminator found — widen the search"
    );
    // Searched discriminators: product double-rounding windows (kills the
    // single-rounded-mul rival on positive y, isolating the product staging).
    let mut mul_hits = 0;
    for i in 0..400_000 {
        let x = rng.uniform(0.1, 30.0);
        let y = rng.uniform(0.05, 5.0);
        let ln = rx::excel_ln(x);
        if (y * ln).to_bits() != rx::x87_mul(y, ln).to_bits() {
            witnesses.push(witness(format!("mul-window-{i}"), x, y));
            mul_hits += 1;
            if mul_hits >= 8 {
                break;
            }
        }
    }
    assert!(
        mul_hits >= 1,
        "no product double-rounding discriminator found — widen the search"
    );

    // ---- race: the confirmed staging must be the unique exact survivor ----
    let results = race(&candidates, &witnesses, 4);
    let alive = survivors(&results);
    assert_eq!(
        alive.len(),
        1,
        "exactly one candidate must survive; survivors: {:?}",
        alive.iter().map(|r| &r.id).collect::<Vec<_>>()
    );
    assert_eq!(alive[0].id, "confirmed-x87-recip-staging");
    for r in &results {
        if r.id != "confirmed-x87-recip-staging" {
            assert!(
                r.score.inexact > 0 || r.score.structural_mismatches > 0,
                "rival '{}' was not discriminated",
                r.id
            );
        }
    }

    // ---- scheduler sanity: distinguishing-input search must rank a known
    // discriminator (sqrt special case) above an agreeing input. ----
    let pool = vec![
        ProbeCase {
            id: "agree".into(),
            args: vec![scalar(2.0), scalar(2.0)], // all rivals agree on 4.0
        },
        ProbeCase {
            id: "sqrt-case".into(),
            args: vec![scalar(2.0), scalar(0.5)],
        },
    ];
    let ranked = rank_distinguishing(&candidates, &pool, 10);
    assert!(
        ranked.iter().any(|r| r.probe.id == "sqrt-case"),
        "sqrt discriminator must be surfaced"
    );
    assert!(
        !ranked.iter().any(|r| r.probe.id == "agree"),
        "zero-information probes must be dropped"
    );
}
