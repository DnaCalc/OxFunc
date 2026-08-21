//! The W109 candidate calculation-graph DSL.
//!
//! A [`Candidate`] is data (serde JSON in/out): an expression [`Graph`] whose
//! nodes each carry an [`EvalModel`] (strict binary64 vs x87 extended with an
//! explicit control word and store-barrier flag). Candidates are persisted,
//! diffed, hashed into the ruled-out ledger, and searched over — never code.
//!
//! Layered-search axes this representation exposes directly:
//! * **structure** — which nodes/ops exist (algebraic identity choice);
//! * **association** — the tree shape over the same leaves;
//! * **store boundaries** — `store: true` on any x87 node rounds its result
//!   to binary64 before the consumer sees it;
//! * **constants** — exact binary64 bits vs x87 ROM constants;
//! * **branch thresholds** — [`Op::Branch`] with an explicit predicate;
//! * **accumulation** — [`SumOrder`] on `Sum`/`Prod`/`FoldSum`.

use serde::{Deserialize, Serialize};

pub type NodeId = usize;

/// x87 control word choice (precision control + round-to-nearest, exceptions
/// masked). `Pc64Rn` (0x133F) is Excel's transcendental CW.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CwSpec {
    Pc64Rn,
    Pc53Rn,
    Pc24Rn,
}

impl CwSpec {
    pub fn value(self) -> u16 {
        use oxfunc_core::excel_numeric::research as rx;
        match self {
            CwSpec::Pc64Rn => rx::CW_PC64_RN,
            CwSpec::Pc53Rn => rx::CW_PC53_RN,
            CwSpec::Pc24Rn => rx::CW_PC24_RN,
        }
    }
}

/// Per-node evaluation model.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvalModel {
    /// Strict IEEE binary64 (SSE semantics; one rounding per op).
    Strict,
    /// x87 extended temporary under `cw`. The result stays extended unless
    /// `store` is true, which rounds it to binary64 (the store barrier).
    X87 { cw: CwSpec, store: bool },
}

impl EvalModel {
    /// Excel's transcendental model with an explicit binary64 store.
    pub const X87_STORED: EvalModel = EvalModel::X87 {
        cw: CwSpec::Pc64Rn,
        store: true,
    };
    /// Extended-continuous: no store, value flows on at 64-bit significand.
    pub const X87_CONT: EvalModel = EvalModel::X87 {
        cw: CwSpec::Pc64Rn,
        store: false,
    };
}

/// A constant binding. `F64` carries exact bits (hex, `0x` + 16 digits) so no
/// decimal-text rounding ambiguity can creep in; the ROM variants are the x87
/// instruction constants (64-bit significand, RN).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConstVal {
    F64 { bits_hex: String },
    RomPi,
    RomL2e,
    RomL2t,
    RomLn2,
    RomLg2,
    One,
}

impl ConstVal {
    /// Convenience: exact-bits constant from an `f64`.
    pub fn from_f64(v: f64) -> ConstVal {
        ConstVal::F64 {
            bits_hex: format!("0x{:016x}", v.to_bits()),
        }
    }
}

/// Accumulation order for `Sum` / `Prod` / `FoldSum`.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SumOrder {
    Forward,
    Reverse,
    /// Kahan compensated summation (strict model only).
    KahanForward,
    /// x87-model only: the legacy x87 memory-spill loop. Each step performs
    /// the accumulate in extended and immediately stores the running total to
    /// binary64 — `acc = RN53(RN64(acc + term))` — the pattern emitted by
    /// x87-era compilers for `total += term;` with `total` in memory. Terms
    /// arriving extended (non-stored x87 bodies) enter the add unrounded.
    ForwardStoredStep,
    /// [`SumOrder::ForwardStoredStep`] in reverse element order.
    ReverseStoredStep,
}

/// Branch predicate, evaluated on the binary64 view of its operands.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Pred {
    IsNeg(NodeId),
    Lt(NodeId, NodeId),
    Le(NodeId, NodeId),
    Eq(NodeId, NodeId),
}

/// Graph operations.
///
/// Model semantics per op:
/// * Arithmetic (`Add`..`Recip`, `Sum`, `Prod`, `FoldSum`): `Strict` = one
///   IEEE binary64 rounding; `X87` = extended arithmetic under `cw`.
/// * `Exp`/`Ln`/`Log10`/`Sin`/`Cos`/`Tan`: `Strict` = platform libm (the
///   current OxFunc comparator), `X87` = the Excel-parity x87 op for
///   exp/ln/log10 and the raw `FSIN`/`FCOS`/`FPTAN` microcode for trig.
///   These publish binary64 (exp/ln/log10) or honor `store` (trig).
/// * `PowExcelPositive` is the signed-off Excel `POWER` positive staging
///   (`|y|==0.5 -> sqrt`, else `exp(RN53(RN64(y·ln x)))`); `PowfStrict` is the
///   platform `powf`. Both publish binary64 regardless of model.
/// * x87-only instructions (`Fyl2x`, `Fyl2xp1`, `F2xm1`, `Scale`, `Prem`,
///   `Prem1`, `Rndint`) always run on the x87; the model's `cw`/`store` apply
///   (`Strict` on these is an evaluation error).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Op {
    /// Top-level argument by index (scalar unless consumed by `Elem`/`FoldSum`).
    Arg(usize),
    /// Fixed element of an array argument.
    Elem {
        arg: usize,
        index: usize,
    },
    /// Length of an array argument, as f64.
    Len {
        arg: usize,
    },
    Const(ConstVal),
    Add(NodeId, NodeId),
    Sub(NodeId, NodeId),
    Mul(NodeId, NodeId),
    Div(NodeId, NodeId),
    Neg(NodeId),
    Abs(NodeId),
    Sqrt(NodeId),
    /// `1/x` (strict) or the x87 `fdiv` reciprocal (Excel's POWER staging
    /// reciprocal when `store: true`).
    Recip(NodeId),
    Rndint(NodeId),
    F2xm1(NodeId),
    Exp(NodeId),
    Ln(NodeId),
    Log10(NodeId),
    Expm1(NodeId),
    Log1p(NodeId),
    Sin(NodeId),
    Cos(NodeId),
    Tan(NodeId),
    /// Legacy CRT π-reduction chains (x87-only): `FPREM1(x, FLDPI)` with the
    /// quotient's low bit from the status flags, then the trig instruction on
    /// the residue with the `(-1)^Q` parity fixup (`sin`/`cos`; `tan` has
    /// period π and needs none). Result honors the model's `store` flag.
    SinPiParity(NodeId),
    CosPiParity(NodeId),
    TanPiReduced(NodeId),
    /// Legacy CRT π/2-reduction chains (x87-only): `FPREM1(x, FLDPI/2)` with
    /// the quotient low bits selecting the quadrant:
    /// sin -> {sin, cos, -sin, -cos}[q mod 4](r);
    /// cos -> {cos, -sin, -cos, sin}[q mod 4](r);
    /// tan -> tan(r) on even q, -1/tan(r) (extended divide) on odd q.
    SinPi2Quadrant(NodeId),
    CosPi2Quadrant(NodeId),
    TanPi2Quadrant(NodeId),
    /// The `87trig.asm` `fFCOS` shape: `u = x + π/2` formed in EXTENDED
    /// (unstored), then the `fFSIN` π-parity chain on `u`.
    CosViaSinShift(NodeId),
    PowExcelPositive {
        base: NodeId,
        exp: NodeId,
    },
    /// The FULL production Excel `POWER` kernel (BUG-FUNC-042): exact-integer
    /// exponents publish via plain-binary64 binary exponentiation; fractional
    /// exponents take the x87 `exp(RN53(RN64(y·ln x)))` staging with the
    /// `|y|==0.5 -> sqrt` case and the `y<0` reciprocal staging; worksheet
    /// error lanes (#NUM!/#DIV/0!) surface as evaluation errors.
    PowExcelKernel {
        base: NodeId,
        exp: NodeId,
    },
    PowfStrict {
        base: NodeId,
        exp: NodeId,
    },
    Fyl2x {
        y: NodeId,
        x: NodeId,
    },
    Fyl2xp1 {
        y: NodeId,
        x: NodeId,
    },
    Scale {
        x: NodeId,
        k: NodeId,
    },
    Prem {
        x: NodeId,
        modulus: NodeId,
    },
    Prem1 {
        x: NodeId,
        modulus: NodeId,
    },
    Sum {
        terms: Vec<NodeId>,
        order: SumOrder,
    },
    Prod {
        terms: Vec<NodeId>,
        order: SumOrder,
    },
    /// Lockstep fold over array arguments. The `body` graph is evaluated once
    /// per element with its own argument space: body `Arg(k)` for
    /// `k < over.len()` is the current element of `over[k]`; body
    /// `Arg(over.len() + j)` is the outer argument `j` (any kind). Body
    /// results are accumulated per `order` under this node's model — a
    /// non-stored x87 body output flows into an extended accumulator without
    /// an intermediate binary64 store.
    FoldSum {
        over: Vec<usize>,
        body: Graph,
        order: SumOrder,
    },
    /// Product fold over array arguments (same body/arg conventions as
    /// [`Op::FoldSum`], accumulation is multiplication).
    FoldProd {
        over: Vec<usize>,
        body: Graph,
        order: SumOrder,
    },
    /// The legacy interleaved multiply–divide loop
    /// `acc = (acc * nums[i]) / dens[i]` (combinatorial kernels). `order`
    /// picks direction and per-step store behavior; the model picks
    /// strict / extended arithmetic. Starts from `acc = 1.0`.
    FoldMulDiv {
        nums: usize,
        dens: usize,
        order: SumOrder,
    },
    Branch {
        pred: Pred,
        then_node: NodeId,
        else_node: NodeId,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Node {
    pub op: Op,
    pub model: EvalModel,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub output: NodeId,
}

impl Graph {
    /// Total node count including `FoldSum` bodies (complexity metric).
    pub fn complexity(&self) -> u32 {
        let mut n = 0u32;
        for node in &self.nodes {
            n += 1;
            if let Op::FoldSum { body, .. } | Op::FoldProd { body, .. } = &node.op {
                n += body.complexity();
            }
        }
        n
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Candidate {
    pub id: String,
    pub description: String,
    pub graph: Graph,
}

/// Stable dependency-free FNV-1a 64 hash of the candidate's graph JSON — the
/// `candidate_hash` recorded in DISCREPANCY_RULED_OUT_LEDGER.csv.
pub fn candidate_hash(c: &Candidate) -> String {
    let json = serde_json::to_string(&c.graph).expect("graph serializes");
    let mut h: u64 = 0xcbf29ce484222325;
    for b in json.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{h:016x}")
}

/// Incremental graph construction helper for enumerators and fixtures.
#[derive(Default)]
pub struct GraphBuilder {
    nodes: Vec<Node>,
}

impl GraphBuilder {
    pub fn new() -> Self {
        GraphBuilder { nodes: Vec::new() }
    }
    pub fn push(&mut self, op: Op, model: EvalModel) -> NodeId {
        self.nodes.push(Node { op, model });
        self.nodes.len() - 1
    }
    pub fn strict(&mut self, op: Op) -> NodeId {
        self.push(op, EvalModel::Strict)
    }
    pub fn finish(self, output: NodeId) -> Graph {
        Graph {
            nodes: self.nodes,
            output,
        }
    }
}
