//! Demand-driven evaluator for candidate graphs.
//!
//! Values flow as [`Val`]: either a binary64 or an x87 80-bit extended
//! temporary ([`Ext80`]). Conversions are exact upward (f64 -> extended) and
//! round-to-nearest downward (extended -> f64, the store barrier). A `Strict`
//! node consuming an extended input therefore acts as an implicit store
//! barrier — the explicit `store: true` flag exists so a barrier can be placed
//! *between* two x87 nodes, which is the search axis that matters.

use crate::dsl::{CwSpec, EvalModel, Graph, Node, NodeId, Op, Pred, SumOrder};
use oxfunc_core::excel_numeric::research as rx;
use rx::Ext80;

#[derive(Clone, Debug, PartialEq)]
pub enum ArgValue {
    Scalar(f64),
    Array(Vec<f64>),
}

#[derive(Clone, Copy)]
pub enum Val {
    F64(f64),
    Ext(Ext80),
}

impl Val {
    /// Binary64 view (round-to-nearest store for extended values).
    pub fn to_f64(self) -> f64 {
        match self {
            Val::F64(v) => v,
            Val::Ext(e) => rx::ext_to_f64(&e, rx::CW_PC64_RN),
        }
    }
    /// Extended view (exact widening for binary64 values).
    pub fn to_ext(self) -> Ext80 {
        match self {
            Val::F64(v) => rx::ext_from_f64(v),
            Val::Ext(e) => e,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EvalError(pub String);

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "eval error: {}", self.0)
    }
}

fn err<T>(msg: impl Into<String>) -> Result<T, EvalError> {
    Err(EvalError(msg.into()))
}

/// Evaluate a graph to a published binary64 result.
pub fn eval_graph(g: &Graph, args: &[ArgValue]) -> Result<f64, EvalError> {
    Ok(eval_graph_val(g, args)?.to_f64())
}

/// Evaluate a graph to a [`Val`] (used for `FoldSum` bodies so a non-stored
/// x87 body output can flow into an extended accumulator).
pub fn eval_graph_val(g: &Graph, args: &[ArgValue]) -> Result<Val, EvalError> {
    let mut ev = Evaluator {
        g,
        args,
        memo: vec![None; g.nodes.len()],
    };
    ev.node(g.output)
}

struct Evaluator<'a> {
    g: &'a Graph,
    args: &'a [ArgValue],
    memo: Vec<Option<Val>>,
}

impl<'a> Evaluator<'a> {
    fn node(&mut self, id: NodeId) -> Result<Val, EvalError> {
        if id >= self.g.nodes.len() {
            return err(format!("node id {id} out of range"));
        }
        if let Some(v) = self.memo[id] {
            return Ok(v);
        }
        let v = self.compute(&self.g.nodes[id].clone())?;
        self.memo[id] = Some(v);
        Ok(v)
    }

    fn scalar_arg(&self, i: usize) -> Result<f64, EvalError> {
        match self.args.get(i) {
            Some(ArgValue::Scalar(v)) => Ok(*v),
            Some(ArgValue::Array(_)) => err(format!("arg {i} is an array, consumed as scalar")),
            None => err(format!("missing arg {i}")),
        }
    }

    fn array_arg(&self, i: usize) -> Result<&Vec<f64>, EvalError> {
        match self.args.get(i) {
            Some(ArgValue::Array(v)) => Ok(v),
            Some(ArgValue::Scalar(_)) => err(format!("arg {i} is a scalar, consumed as array")),
            None => err(format!("missing arg {i}")),
        }
    }

    fn f64_in(&mut self, id: NodeId) -> Result<f64, EvalError> {
        Ok(self.node(id)?.to_f64())
    }

    fn ext_in(&mut self, id: NodeId) -> Result<Ext80, EvalError> {
        Ok(self.node(id)?.to_ext())
    }

    /// Package an extended intermediate per the node's store flag.
    fn finish_x87(&self, e: Ext80, cw: CwSpec, store: bool) -> Val {
        if store {
            Val::F64(rx::ext_to_f64(&e, cw.value()))
        } else {
            Val::Ext(e)
        }
    }

    fn x87_model(&self, node_desc: &str, model: EvalModel) -> Result<(CwSpec, bool), EvalError> {
        match model {
            EvalModel::X87 { cw, store } => Ok((cw, store)),
            EvalModel::Strict => err(format!("{node_desc} is x87-only; model must be x87")),
        }
    }

    fn compute(&mut self, node: &Node) -> Result<Val, EvalError> {
        let model = node.model;
        match &node.op {
            Op::Arg(i) => Ok(Val::F64(self.scalar_arg(*i)?)),
            Op::Elem { arg, index } => {
                let a = self.array_arg(*arg)?;
                match a.get(*index) {
                    Some(v) => Ok(Val::F64(*v)),
                    None => err(format!("elem index {index} out of range for arg {arg}")),
                }
            }
            Op::Len { arg } => Ok(Val::F64(self.array_arg(*arg)?.len() as f64)),
            Op::Const(c) => self.constant(c, model),
            Op::Add(a, b) => self.binop(*a, *b, model, f64_add, ext_add_op),
            Op::Sub(a, b) => self.binop(*a, *b, model, f64_sub, ext_sub_op),
            Op::Mul(a, b) => self.binop(*a, *b, model, f64_mul, ext_mul_op),
            Op::Div(a, b) => self.binop(*a, *b, model, f64_div, ext_div_op),
            Op::Neg(x) => match model {
                EvalModel::Strict => Ok(Val::F64(-self.f64_in(*x)?)),
                EvalModel::X87 { cw, store } => {
                    let e = rx::ext_chs(&self.ext_in(*x)?, cw.value());
                    Ok(self.finish_x87(e, cw, store))
                }
            },
            Op::Abs(x) => match model {
                EvalModel::Strict => Ok(Val::F64(self.f64_in(*x)?.abs())),
                EvalModel::X87 { cw, store } => {
                    let e = rx::ext_abs(&self.ext_in(*x)?, cw.value());
                    Ok(self.finish_x87(e, cw, store))
                }
            },
            Op::Sqrt(x) => match model {
                EvalModel::Strict => Ok(Val::F64(rx::excel_sqrt(self.f64_in(*x)?))),
                EvalModel::X87 { cw, store } => {
                    let e = rx::ext_sqrt(&self.ext_in(*x)?, cw.value());
                    Ok(self.finish_x87(e, cw, store))
                }
            },
            Op::Recip(x) => match model {
                EvalModel::Strict => Ok(Val::F64(1.0 / self.f64_in(*x)?)),
                EvalModel::X87 { cw, store } => {
                    let e = rx::ext_div(&rx::ext_one(), &self.ext_in(*x)?, cw.value());
                    Ok(self.finish_x87(e, cw, store))
                }
            },
            Op::Rndint(x) => {
                let (cw, store) = self.x87_model("rndint", model)?;
                let e = rx::ext_rndint(&self.ext_in(*x)?, cw.value());
                Ok(self.finish_x87(e, cw, store))
            }
            Op::F2xm1(x) => {
                let (cw, store) = self.x87_model("f2xm1", model)?;
                let e = rx::ext_f2xm1(&self.ext_in(*x)?, cw.value());
                Ok(self.finish_x87(e, cw, store))
            }
            Op::Exp(x) => {
                let v = self.f64_in(*x)?;
                Ok(Val::F64(match model {
                    EvalModel::Strict => v.exp(),
                    EvalModel::X87 { .. } => rx::excel_exp(v),
                }))
            }
            Op::Ln(x) => {
                let v = self.f64_in(*x)?;
                Ok(Val::F64(match model {
                    EvalModel::Strict => v.ln(),
                    EvalModel::X87 { .. } => rx::excel_ln(v),
                }))
            }
            Op::Log10(x) => {
                let v = self.f64_in(*x)?;
                Ok(Val::F64(match model {
                    EvalModel::Strict => v.log10(),
                    EvalModel::X87 { .. } => rx::excel_log10(v),
                }))
            }
            Op::Expm1(x) => Ok(Val::F64(rx::excel_expm1(self.f64_in(*x)?))),
            Op::Log1p(x) => Ok(Val::F64(rx::excel_log1p(self.f64_in(*x)?))),
            Op::Sin(x) => self.trig(*x, model, f64::sin, rx::ext_sin),
            Op::Cos(x) => self.trig(*x, model, f64::cos, rx::ext_cos),
            Op::Tan(x) => self.trig(*x, model, f64::tan, rx::ext_tan),
            Op::PowExcelPositive { base, exp } => {
                let b = self.f64_in(*base)?;
                let e = self.f64_in(*exp)?;
                Ok(Val::F64(rx::excel_pow_positive(b, e)))
            }
            Op::PowExcelKernel { base, exp } => {
                let b = self.f64_in(*base)?;
                let e = self.f64_in(*exp)?;
                match oxfunc_core::functions::power_fn::power_kernel(b, e) {
                    Ok(v) => Ok(Val::F64(v)),
                    Err(code) => err(format!("POWER worksheet error {code:?}")),
                }
            }
            Op::PowfStrict { base, exp } => {
                let b = self.f64_in(*base)?;
                let e = self.f64_in(*exp)?;
                Ok(Val::F64(b.powf(e)))
            }
            Op::Fyl2x { y, x } => {
                let (cw, store) = self.x87_model("fyl2x", model)?;
                let ye = self.ext_in(*y)?;
                let xe = self.ext_in(*x)?;
                Ok(self.finish_x87(rx::ext_fyl2x(&ye, &xe, cw.value()), cw, store))
            }
            Op::Fyl2xp1 { y, x } => {
                let (cw, store) = self.x87_model("fyl2xp1", model)?;
                let ye = self.ext_in(*y)?;
                let xe = self.ext_in(*x)?;
                Ok(self.finish_x87(rx::ext_fyl2xp1(&ye, &xe, cw.value()), cw, store))
            }
            Op::Scale { x, k } => {
                let (cw, store) = self.x87_model("scale", model)?;
                let xe = self.ext_in(*x)?;
                let ke = self.ext_in(*k)?;
                Ok(self.finish_x87(rx::ext_scale(&xe, &ke, cw.value()), cw, store))
            }
            Op::Prem { x, modulus } => {
                let (cw, store) = self.x87_model("prem", model)?;
                let xe = self.ext_in(*x)?;
                let me = self.ext_in(*modulus)?;
                Ok(self.finish_x87(rx::ext_prem(&xe, &me, cw.value()), cw, store))
            }
            Op::Prem1 { x, modulus } => {
                let (cw, store) = self.x87_model("prem1", model)?;
                let xe = self.ext_in(*x)?;
                let me = self.ext_in(*modulus)?;
                Ok(self.finish_x87(rx::ext_prem1(&xe, &me, cw.value()), cw, store))
            }
            Op::Sum { terms, order } => {
                let vals = self.terms(terms)?;
                self.accumulate(vals, *order, model, AccKind::Sum)
            }
            Op::Prod { terms, order } => {
                let vals = self.terms(terms)?;
                self.accumulate(vals, *order, model, AccKind::Prod)
            }
            Op::FoldSum { over, body, order } => {
                if over.is_empty() {
                    return err("fold_sum needs at least one array to iterate");
                }
                let len = self.array_arg(over[0])?.len();
                for arg in over.iter().skip(1) {
                    if self.array_arg(*arg)?.len() != len {
                        return err("fold_sum arrays must have equal length");
                    }
                }
                let mut vals = Vec::with_capacity(len);
                for i in 0..len {
                    let mut body_args: Vec<ArgValue> = Vec::with_capacity(over.len() + self.args.len());
                    for arg in over {
                        body_args.push(ArgValue::Scalar(self.array_arg(*arg)?[i]));
                    }
                    body_args.extend(self.args.iter().cloned());
                    vals.push(eval_graph_val(body, &body_args)?);
                }
                self.accumulate(vals, *order, model, AccKind::Sum)
            }
            Op::Branch {
                pred,
                then_node,
                else_node,
            } => {
                let taken = match pred {
                    Pred::IsNeg(x) => self.f64_in(*x)? < 0.0,
                    Pred::Lt(a, b) => self.f64_in(*a)? < self.f64_in(*b)?,
                    Pred::Le(a, b) => self.f64_in(*a)? <= self.f64_in(*b)?,
                    Pred::Eq(a, b) => self.f64_in(*a)? == self.f64_in(*b)?,
                };
                self.node(if taken { *then_node } else { *else_node })
            }
        }
    }

    fn constant(&self, c: &crate::dsl::ConstVal, model: EvalModel) -> Result<Val, EvalError> {
        use crate::dsl::ConstVal as C;
        // ROM constants are inherently extended; publish per the node's model
        // (a strict/stored ROM constant is "the ROM constant rounded to
        // binary64", e.g. FLDPI -> the double π).
        let ext = match c {
            C::F64 { bits_hex } => {
                let v = parse_bits_hex(bits_hex)
                    .ok_or_else(|| EvalError(format!("bad const bits_hex '{bits_hex}'")))?;
                return Ok(Val::F64(v));
            }
            C::RomPi => rx::ext_pi(),
            C::RomL2e => rx::ext_l2e(),
            C::RomL2t => rx::ext_l2t(),
            C::RomLn2 => rx::ext_ln2(),
            C::RomLg2 => rx::ext_lg2(),
            C::One => rx::ext_one(),
        };
        match model {
            EvalModel::Strict => Ok(Val::F64(rx::ext_to_f64(&ext, rx::CW_PC64_RN))),
            EvalModel::X87 { cw, store } => Ok(self.finish_x87(ext, cw, store)),
        }
    }

    fn trig(
        &mut self,
        x: NodeId,
        model: EvalModel,
        strict: fn(f64) -> f64,
        x87: fn(&Ext80, u16) -> Ext80,
    ) -> Result<Val, EvalError> {
        match model {
            EvalModel::Strict => Ok(Val::F64(strict(self.f64_in(x)?))),
            EvalModel::X87 { cw, store } => {
                let e = x87(&self.ext_in(x)?, cw.value());
                Ok(self.finish_x87(e, cw, store))
            }
        }
    }

    fn binop(
        &mut self,
        a: NodeId,
        b: NodeId,
        model: EvalModel,
        strict: fn(f64, f64) -> f64,
        x87: fn(&Ext80, &Ext80, u16) -> Ext80,
    ) -> Result<Val, EvalError> {
        match model {
            EvalModel::Strict => Ok(Val::F64(strict(self.f64_in(a)?, self.f64_in(b)?))),
            EvalModel::X87 { cw, store } => {
                let ae = self.ext_in(a)?;
                let be = self.ext_in(b)?;
                Ok(self.finish_x87(x87(&ae, &be, cw.value()), cw, store))
            }
        }
    }

    fn terms(&mut self, ids: &[NodeId]) -> Result<Vec<Val>, EvalError> {
        ids.iter().map(|id| self.node(*id)).collect()
    }

    fn accumulate(
        &mut self,
        mut vals: Vec<Val>,
        order: SumOrder,
        model: EvalModel,
        kind: AccKind,
    ) -> Result<Val, EvalError> {
        if matches!(order, SumOrder::Reverse | SumOrder::ReverseStoredStep) {
            vals.reverse();
        }
        if vals.is_empty() {
            return Ok(Val::F64(match kind {
                AccKind::Sum => 0.0,
                AccKind::Prod => 1.0,
            }));
        }
        match model {
            EvalModel::Strict => {
                if matches!(
                    order,
                    SumOrder::ForwardStoredStep | SumOrder::ReverseStoredStep
                ) {
                    return err("stored-step accumulation is x87-only");
                }
                if order == SumOrder::KahanForward {
                    if kind != AccKind::Sum {
                        return err("kahan order is sum-only");
                    }
                    let mut s = 0.0f64;
                    let mut c = 0.0f64;
                    for v in vals {
                        let y = v.to_f64() - c;
                        let t = s + y;
                        c = (t - s) - y;
                        s = t;
                    }
                    return Ok(Val::F64(s));
                }
                let mut it = vals.into_iter();
                let mut acc = it.next().unwrap().to_f64();
                for v in it {
                    acc = match kind {
                        AccKind::Sum => acc + v.to_f64(),
                        AccKind::Prod => acc * v.to_f64(),
                    };
                }
                Ok(Val::F64(acc))
            }
            EvalModel::X87 { cw, store } => {
                if order == SumOrder::KahanForward {
                    return err("kahan order is strict-only");
                }
                if matches!(
                    order,
                    SumOrder::ForwardStoredStep | SumOrder::ReverseStoredStep
                ) {
                    // Legacy x87 memory-spill loop: extended op, then an
                    // immediate binary64 store of the running total per step.
                    let mut it = vals.into_iter();
                    let mut acc = it.next().unwrap().to_f64();
                    for v in it {
                        let acc_e = rx::ext_from_f64(acc);
                        let ve = v.to_ext();
                        let r = match kind {
                            AccKind::Sum => rx::ext_add(&acc_e, &ve, cw.value()),
                            AccKind::Prod => rx::ext_mul(&acc_e, &ve, cw.value()),
                        };
                        acc = rx::ext_to_f64(&r, cw.value());
                    }
                    return Ok(Val::F64(acc));
                }
                let mut it = vals.into_iter();
                let mut acc = it.next().unwrap().to_ext();
                for v in it {
                    let ve = v.to_ext();
                    acc = match kind {
                        AccKind::Sum => rx::ext_add(&acc, &ve, cw.value()),
                        AccKind::Prod => rx::ext_mul(&acc, &ve, cw.value()),
                    };
                }
                Ok(self.finish_x87(acc, cw, store))
            }
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
enum AccKind {
    Sum,
    Prod,
}

fn f64_add(a: f64, b: f64) -> f64 {
    a + b
}
fn f64_sub(a: f64, b: f64) -> f64 {
    a - b
}
fn f64_mul(a: f64, b: f64) -> f64 {
    a * b
}
fn f64_div(a: f64, b: f64) -> f64 {
    a / b
}
fn ext_add_op(a: &Ext80, b: &Ext80, cw: u16) -> Ext80 {
    rx::ext_add(a, b, cw)
}
fn ext_sub_op(a: &Ext80, b: &Ext80, cw: u16) -> Ext80 {
    rx::ext_sub(a, b, cw)
}
fn ext_mul_op(a: &Ext80, b: &Ext80, cw: u16) -> Ext80 {
    rx::ext_mul(a, b, cw)
}
fn ext_div_op(a: &Ext80, b: &Ext80, cw: u16) -> Ext80 {
    rx::ext_div(a, b, cw)
}

/// Parse `0x`-prefixed 16-digit hex into the exact f64 bit pattern.
pub fn parse_bits_hex(s: &str) -> Option<f64> {
    let hex = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X"))?;
    if hex.len() != 16 {
        return None;
    }
    u64::from_str_radix(hex, 16).ok().map(f64::from_bits)
}

/// Format an f64 as the canonical `0x` + 16-hex-digit bit pattern.
pub fn format_bits_hex(v: f64) -> String {
    format!("0x{:016x}", v.to_bits())
}
