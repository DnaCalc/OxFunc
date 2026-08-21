//! Offline W109 G3-01 `bgrat` Eq-9 association-tree and x87 spill-mask race.
//!
//! Selection evidence only: the banked b25 822-row corpus and the 121-row
//! b21-sharp corpus.  This binary never starts Excel and never uses COM.  The
//! q/u primitive realization is frozen to the currently identified worksheet
//! LN/EXP chain (with `agentM_b25e/f` as historical starting references) while
//! the pure-arithmetic Eq-9 body is raced statement by statement.

use oxfunc_core::excel_numeric::research as rx;
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;

const CW: u16 = rx::CW_PC64_RN;
const EPS: f64 = 15.0e-15;

#[derive(Deserialize)]
struct AnswerBank {
    witnesses: Vec<Witness>,
}

#[derive(Deserialize)]
struct Witness {
    id: String,
    args: Vec<String>,
    expected_bits: String,
}

#[derive(Deserialize)]
struct MetaRow {
    id: String,
    x_bits: String,
    y_bits: String,
    tag: String,
    z0_bits: Option<String>,
}

#[derive(Deserialize)]
struct AgentFRow {
    src: String,
    branch: String,
    id: String,
    a: f64,
    b: f64,
    x: f64,
    y: f64,
    eb: String,
}

#[derive(Clone, Copy)]
struct Pieces {
    bm1: f64,
    nu: f64,
    lnx: f64,
    z: f64,
    r: f64,
    q: f64,
    u: f64,
}

#[derive(Clone)]
struct Row {
    id: String,
    a: f64,
    b: f64,
    expected: u64,
    tag: String,
    z0: Option<u64>,
    pieces: Pieces,
}

fn parse_hex(text: &str) -> u64 {
    u64::from_str_radix(text.trim_start_matches("0x"), 16).unwrap()
}

fn load_b25(dir: &str) -> Vec<Row> {
    let bank: AnswerBank =
        serde_json::from_str(&fs::read_to_string(format!("{dir}/answers-b25-bgrat.json")).unwrap())
            .unwrap();
    let meta: Vec<MetaRow> =
        serde_json::from_str(&fs::read_to_string(format!("{dir}/agentM_b25_meta.json")).unwrap())
            .unwrap();
    let by_id: BTreeMap<String, Witness> = bank
        .witnesses
        .into_iter()
        .map(|witness| (witness.id.clone(), witness))
        .collect();
    let mut rows = Vec::new();
    for m in meta {
        let witness = &by_id[&m.id];
        assert_eq!(witness.args.len(), 4, "{} arity", m.id);
        let x_bits = parse_hex(&witness.args[0]);
        let a_bits = parse_hex(&witness.args[1]);
        let b_bits = parse_hex(&witness.args[2]);
        assert_eq!(x_bits, parse_hex(&m.x_bits), "{} x loader", m.id);
        let x = f64::from_bits(x_bits);
        let y = 1.0 - x;
        assert_eq!(y.to_bits(), parse_hex(&m.y_bits), "{} y loader", m.id);
        let a = f64::from_bits(a_bits);
        let b = f64::from_bits(b_bits);
        rows.push(Row {
            id: m.id,
            a,
            b,
            expected: parse_hex(&witness.expected_bits),
            tag: m.tag,
            z0: m.z0_bits.as_deref().map(parse_hex),
            pieces: frozen_pieces(a, b, x),
        });
    }
    assert_eq!(rows.len(), 822, "b25 loader row count");
    rows
}

fn load_b21_sharp(dir: &str) -> Vec<Row> {
    let bank: AnswerBank =
        serde_json::from_str(&fs::read_to_string(format!("{dir}/answers-b21-beta.json")).unwrap())
            .unwrap();
    let extra: Vec<AgentFRow> =
        serde_json::from_str(&fs::read_to_string(format!("{dir}/agentF_bgrat_rows.json")).unwrap())
            .unwrap();
    let mut rows = Vec::new();
    let mut ids = BTreeSet::new();
    for witness in bank.witnesses {
        let x = f64::from_bits(parse_hex(&witness.args[0]));
        let a = f64::from_bits(parse_hex(&witness.args[1]));
        let b = f64::from_bits(parse_hex(&witness.args[2]));
        if b > 1.0 || !(x > 0.5 && a > 15.0) {
            continue;
        }
        let id = witness.id;
        ids.insert(id.clone());
        rows.push(Row {
            id,
            a,
            b,
            expected: parse_hex(&witness.expected_bits),
            tag: "b21".into(),
            z0: None,
            pieces: frozen_pieces(a, b, x),
        });
    }
    for item in extra {
        if item.src != "b15"
            || item.branch != "bgrat"
            || !(item.x > 0.5 && item.a > 15.0 && item.b <= 1.0)
        {
            continue;
        }
        assert!(
            ids.insert(item.id.clone()),
            "duplicate sharp id {}",
            item.id
        );
        assert_eq!((1.0 - item.x).to_bits(), item.y.to_bits(), "{} y", item.id);
        rows.push(Row {
            id: item.id,
            a: item.a,
            b: item.b,
            expected: parse_hex(&item.eb),
            tag: "b21".into(),
            z0: None,
            pieces: frozen_pieces(item.a, item.b, item.x),
        });
    }
    assert_eq!(rows.len(), 121, "b21-sharp loader row count");
    rows
}

// Public NSWC/TOMS-708 helper, copied algebraically so the q/u axis can stay
// frozen while the Eq-9 body changes.  Transcendentals route through the
// already identified worksheet/internal primitives.
fn alnrel(a: f64) -> f64 {
    const P1: f64 = -0.129418923021993e1;
    const P2: f64 = 0.405303492862024;
    const P3: f64 = -0.178874546012214e-1;
    const Q1: f64 = -0.162752256355323e1;
    const Q2: f64 = 0.747811014037616;
    const Q3: f64 = -0.845104217945565e-1;
    if a.abs() > 0.375 {
        return rx::excel_ln(1.0 + a);
    }
    let t = a / (a + 2.0);
    let t2 = t * t;
    let w = (((P3 * t2 + P2) * t2 + P1) * t2 + 1.0) / (((Q3 * t2 + Q2) * t2 + Q1) * t2 + 1.0);
    2.0 * t * w
}

fn gam1(a: f64) -> f64 {
    const P: [f64; 7] = [
        0.577215664901533,
        -0.409078193005776,
        -0.230975380857675,
        0.0597275330452234,
        0.00766968181649490,
        -0.00514889771323592,
        0.000589597428611429,
    ];
    const Q: [f64; 5] = [
        1.0,
        0.427569613095214,
        0.158451672430138,
        0.0261132021447447,
        0.00423244297896961,
    ];
    const R: [f64; 9] = [
        -0.422784335098468,
        -0.771330383816272,
        -0.244757765222226,
        0.118378989872749,
        0.000930357293360349,
        -0.0118290993445146,
        0.00223047661158249,
        0.000266505979058923,
        -0.000132674909766242,
    ];
    const S1: f64 = 0.273076135303957;
    const S2: f64 = 0.0559398236957378;
    let mut t = a;
    let d = a - 0.5;
    if d > 0.0 {
        t = d - 0.5;
    }
    if t == 0.0 {
        return 0.0;
    }
    if t > 0.0 {
        let top = (((((P[6] * t + P[5]) * t + P[4]) * t + P[3]) * t + P[2]) * t + P[1]) * t + P[0];
        let bot = (((Q[4] * t + Q[3]) * t + Q[2]) * t + Q[1]) * t + 1.0;
        let w = top / bot;
        if d > 0.0 {
            (t / a) * ((w - 0.5) - 0.5)
        } else {
            a * w
        }
    } else {
        let top = (((((((R[8] * t + R[7]) * t + R[6]) * t + R[5]) * t + R[4]) * t + R[3]) * t
            + R[2])
            * t
            + R[1])
            * t
            + R[0];
        let bot = (S2 * t + S1) * t + 1.0;
        let w = top / bot;
        if d > 0.0 {
            t * w / a
        } else {
            a * ((w + 0.5) + 0.5)
        }
    }
}

fn algdiv(a: f64, b: f64) -> f64 {
    const C0: f64 = 0.0833333333333333;
    const C1: f64 = -0.00277777777760991;
    const C2: f64 = 0.000793650666825390;
    const C3: f64 = -0.000595202931351870;
    const C4: f64 = 0.000837308034031215;
    const C5: f64 = -0.00165322962780713;
    let (h, c, x, d) = if a > b {
        let h = b / a;
        (h, 1.0 / (1.0 + h), h / (1.0 + h), a + (b - 0.5))
    } else {
        let h = a / b;
        (h, h / (1.0 + h), 1.0 / (1.0 + h), b + (a - 0.5))
    };
    let _ = h;
    let x2 = x * x;
    let s3 = 1.0 + (x + x2);
    let s5 = 1.0 + (x + x2 * s3);
    let s7 = 1.0 + (x + x2 * s5);
    let s9 = 1.0 + (x + x2 * s7);
    let s11 = 1.0 + (x + x2 * s9);
    let t = (1.0 / b).powi(2);
    let mut w = ((((C5 * s11 * t + C4 * s9) * t + C3 * s7) * t + C2 * s5) * t + C1 * s3) * t + C0;
    w *= c / b;
    let u = d * alnrel(a / b);
    let v = a * (rx::excel_ln(b) - 1.0);
    if u > v { (w - v) - u } else { (w - u) - v }
}

fn rexp(x: f64) -> f64 {
    if x.abs() <= 0.15 {
        const P1: f64 = 0.914041914819518e-9;
        const P2: f64 = 0.0238082361044469;
        const Q1: f64 = -0.499999999085958;
        const Q2: f64 = 0.107141568980644;
        const Q3: f64 = -0.0119041179760821;
        const Q4: f64 = 0.000595130811860248;
        return x * (((P2 * x + P1) * x + 1.0) / ((((Q4 * x + Q3) * x + Q2) * x + Q1) * x + 1.0));
    }
    let w = rx::excel_exp(x);
    if x <= 0.0 {
        (w - 0.5) - 0.5
    } else {
        w * (0.5 + (0.5 - 1.0 / w))
    }
}

fn gratio_cf_q(a: f64, x: f64, r: f64) -> f64 {
    let acc = 5.0e-15;
    let tol = (5.0 * f64::EPSILON).max(acc);
    let mut a2nm1 = 1.0;
    let mut a2n = 1.0;
    let mut b2nm1 = x;
    let mut b2n = x + (1.0 - a);
    let mut c = 1.0;
    loop {
        a2nm1 = x * a2n + c * a2nm1;
        b2nm1 = x * b2n + c * b2nm1;
        let am0 = a2nm1 / b2nm1;
        c += 1.0;
        let cma = c - a;
        a2n = a2nm1 + cma * a2n;
        b2n = b2nm1 + cma * b2n;
        let an0 = a2n / b2n;
        if (an0 - am0).abs() < tol * an0 {
            return r * an0;
        }
    }
}

/// Frozen q primitive: TOMS-654 GRATIO, worksheet LN/EXP, RZ publication at
/// the continued-fraction r-site.  The selected corpora have 0 < a=b <= 1.
fn frozen_q(a: f64, x: f64) -> f64 {
    let acc = 5.0e-15;
    if x == 0.0 {
        return 1.0;
    }
    if a == 1.0 {
        return rx::excel_exp(-x);
    }
    assert!(a > 0.0 && a < 1.0, "q corpus outside small-a path: a={a}");
    if x < 1.1 {
        let mut an = 3.0;
        let mut c = x;
        let mut sum = x / (a + 3.0);
        let tol = 3.0 * acc / (a + 1.0);
        loop {
            an += 1.0;
            c = -c * (x / an);
            let term = c / (a + an);
            sum += term;
            if term.abs() <= tol {
                break;
            }
        }
        let j = a * x * ((sum / 6.0 - 0.5 / (a + 2.0)) * x + 1.0 / (a + 1.0));
        let z = a * rx::excel_ln(x);
        let h = gam1(a);
        let g = 1.0 + h;
        let go200 = if x < 0.25 { z > -0.13394 } else { a < x / 2.59 };
        if go200 {
            let l = rexp(z);
            let w = 0.5 + (0.5 + l);
            return (w * j - l) * g - h;
        }
        let p = rx::excel_exp(z) * g * (0.5 + (0.5 - j));
        return 0.5 + (0.5 - p);
    }
    let t1 = a * rx::excel_ln(x) - x;
    let u = a * rx::excel_exp_rz(t1);
    let r = u * (1.0 + gam1(a));
    gratio_cf_q(a, x, r)
}

fn frozen_pieces(a: f64, b: f64, x: f64) -> Pieces {
    let bm1 = (b - 0.5) - 0.5;
    let nu = a + 0.5 * bm1;
    let lnx = rx::excel_ln(x);
    let z = -nu * lnx;
    let mut r = b * (1.0 + gam1(b)) * rx::excel_exp(b * rx::excel_ln(z));
    r = r * rx::excel_exp(a * lnx) * rx::excel_exp(0.5 * bm1 * lnx);
    let ua = algdiv(b, a) + b * rx::excel_ln(nu);
    let u = r * rx::excel_exp(-ua);
    Pieces {
        bm1,
        nu,
        lnx,
        z,
        r,
        q: frozen_q(b, z),
        u,
    }
}

#[derive(Clone, Copy)]
struct V(rx::Ext80);

impl V {
    fn from_f64(value: f64) -> Self {
        Self(rx::ext_from_f64(value))
    }

    fn to_f64(self) -> f64 {
        rx::ext_to_f64(&self.0, CW)
    }

    fn spill(self) -> Self {
        Self::from_f64(self.to_f64())
    }
}

fn xadd(a: V, b: V, spill: bool) -> V {
    let value = V(rx::ext_add(&a.0, &b.0, CW));
    if spill { value.spill() } else { value }
}

fn xsub(a: V, b: V, spill: bool) -> V {
    let value = V(rx::ext_sub(&a.0, &b.0, CW));
    if spill { value.spill() } else { value }
}

fn xmul(a: V, b: V, spill: bool) -> V {
    let value = V(rx::ext_mul(&a.0, &b.0, CW));
    if spill { value.spill() } else { value }
}

fn xdiv(a: V, b: V, spill: bool) -> V {
    let value = V(rx::ext_div(&a.0, &b.0, CW));
    if spill { value.spill() } else { value }
}

fn nadd(a: V, b: V) -> V {
    V::from_f64(a.to_f64() + b.to_f64())
}

fn nmul(a: V, b: V) -> V {
    V::from_f64(a.to_f64() * b.to_f64())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Choice {
    tree: u8,
    mask: u16,
}

const NATIVE: Choice = Choice {
    tree: u8::MAX,
    mask: 0,
};

fn bit(choice: Choice, index: u8) -> bool {
    choice.mask & (1_u16 << index) != 0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Config {
    v: Choice,
    t2: Choice,
    j0: Choice,
    recurrence: Choice,
    cn: Choice,
    s: Choice,
    d: Choice,
    sum: Choice,
    final_mul: Choice,
}

impl Config {
    fn native() -> Self {
        Self {
            v: NATIVE,
            t2: NATIVE,
            j0: NATIVE,
            recurrence: NATIVE,
            cn: NATIVE,
            s: NATIVE,
            d: NATIVE,
            sum: NATIVE,
            final_mul: NATIVE,
        }
    }
}

fn eval_v(choice: Choice, nu: f64) -> V {
    if choice == NATIVE {
        return V::from_f64(0.25 * (1.0 / nu).powi(2));
    }
    let one = V::from_f64(1.0);
    let quarter = V::from_f64(0.25);
    let half = V::from_f64(0.5);
    let nu = V::from_f64(nu);
    match choice.tree {
        0 => {
            let r = xdiv(one, nu, bit(choice, 0));
            let square = xmul(r, r, bit(choice, 1));
            xmul(quarter, square, bit(choice, 2))
        }
        1 => {
            let square = xmul(nu, nu, bit(choice, 0));
            xdiv(quarter, square, bit(choice, 1))
        }
        2 => {
            let r = xdiv(half, nu, bit(choice, 0));
            xmul(r, r, bit(choice, 1))
        }
        3 => {
            let r = xdiv(quarter, nu, bit(choice, 0));
            xdiv(r, nu, bit(choice, 1))
        }
        _ => unreachable!(),
    }
}

fn eval_t2(choice: Choice, lnx: f64) -> V {
    if choice == NATIVE {
        return V::from_f64(0.25 * lnx * lnx);
    }
    let l = V::from_f64(lnx);
    let q = V::from_f64(0.25);
    let h = V::from_f64(0.5);
    match choice.tree {
        0 => {
            let left = xmul(q, l, bit(choice, 0));
            xmul(left, l, bit(choice, 1))
        }
        1 => {
            let square = xmul(l, l, bit(choice, 0));
            xmul(q, square, bit(choice, 1))
        }
        2 => {
            let half = xmul(h, l, bit(choice, 0));
            xmul(half, half, bit(choice, 1))
        }
        3 => {
            let square = xmul(l, l, bit(choice, 0));
            xdiv(square, V::from_f64(4.0), bit(choice, 1))
        }
        _ => unreachable!(),
    }
}

fn update_t(choice: Choice, t: V, t2: V) -> V {
    if choice == NATIVE {
        nmul(t, t2)
    } else {
        xmul(t, t2, bit(choice, 2))
    }
}

fn eval_j0(choice: Choice, q: f64, r: f64) -> V {
    if choice == NATIVE {
        V::from_f64(q / r)
    } else {
        xdiv(V::from_f64(q), V::from_f64(r), bit(choice, 0))
    }
}

fn eval_recurrence(choice: Choice, b: f64, n2: f64, z: f64, j: V, t: V, v: V) -> V {
    if choice == NATIVE {
        let bp = b + n2;
        let value = (bp * (bp + 1.0) * j.to_f64() + (z + bp + 1.0) * t.to_f64()) * v.to_f64();
        return V::from_f64(value);
    }
    let bp = xadd(V::from_f64(b), V::from_f64(n2), bit(choice, 0));
    let p_left = xadd(bp, V::from_f64(1.0), bit(choice, 1));
    let left_kind = choice.tree % 3;
    let right_kind = (choice.tree / 3) % 3;
    let distributed = choice.tree >= 9;
    let left = match left_kind {
        0 => {
            let first = xmul(bp, p_left, bit(choice, 2));
            xmul(first, j, bit(choice, 3))
        }
        1 => {
            let first = xmul(p_left, j, bit(choice, 2));
            xmul(bp, first, bit(choice, 3))
        }
        _ => {
            let first = xmul(bp, j, bit(choice, 2));
            xmul(first, p_left, bit(choice, 3))
        }
    };
    let right = match right_kind {
        0 => {
            let first = xadd(V::from_f64(z), bp, bit(choice, 4));
            let second = xadd(first, V::from_f64(1.0), bit(choice, 4));
            xmul(second, t, bit(choice, 5))
        }
        1 => {
            let p = xadd(bp, V::from_f64(1.0), bit(choice, 4));
            let first = xadd(V::from_f64(z), p, bit(choice, 4));
            xmul(first, t, bit(choice, 5))
        }
        _ => {
            let zt = xmul(V::from_f64(z), t, bit(choice, 4));
            let p = xadd(bp, V::from_f64(1.0), bit(choice, 4));
            let pt = xmul(p, t, bit(choice, 4));
            xadd(zt, pt, bit(choice, 5))
        }
    };
    if distributed {
        let lv = xmul(left, v, bit(choice, 6));
        let rv = xmul(right, v, bit(choice, 6));
        xadd(lv, rv, bit(choice, 7))
    } else {
        let numerator = xadd(left, right, bit(choice, 6));
        xmul(numerator, v, bit(choice, 7))
    }
}

fn eval_cn(choice: Choice, cn: V, n2: f64) -> V {
    if choice == NATIVE {
        return V::from_f64(cn.to_f64() / (n2 * (n2 + 1.0)));
    }
    let n = V::from_f64(n2);
    let np1 = xadd(n, V::from_f64(1.0), bit(choice, 0));
    match choice.tree {
        0 => {
            let den = xmul(n, np1, bit(choice, 1));
            xdiv(cn, den, bit(choice, 2))
        }
        1 => {
            let first = xdiv(cn, n, bit(choice, 1));
            xdiv(first, np1, bit(choice, 2))
        }
        2 => {
            let first = xdiv(cn, np1, bit(choice, 1));
            xdiv(first, n, bit(choice, 2))
        }
        3 => {
            let r1 = xdiv(V::from_f64(1.0), n, bit(choice, 1));
            let r2 = xdiv(V::from_f64(1.0), np1, bit(choice, 1));
            let first = xmul(cn, r1, bit(choice, 2));
            xmul(first, r2, bit(choice, 2))
        }
        _ => unreachable!(),
    }
}

fn eval_s_term(choice: Choice, sum: V, coef: V, c: V, d: V) -> V {
    if choice == NATIVE {
        return V::from_f64(sum.to_f64() + coef.to_f64() * c.to_f64() * d.to_f64());
    }
    let term = match choice.tree {
        0 => {
            let first = xmul(coef, c, bit(choice, 0));
            xmul(first, d, bit(choice, 1))
        }
        1 => {
            let first = xmul(c, d, bit(choice, 0));
            xmul(coef, first, bit(choice, 1))
        }
        2 => {
            let first = xmul(coef, d, bit(choice, 0));
            xmul(first, c, bit(choice, 1))
        }
        _ => unreachable!(),
    };
    xadd(sum, term, bit(choice, 2))
}

fn update_coef(choice: Choice, coef: V, b: f64) -> V {
    if choice == NATIVE {
        nadd(coef, V::from_f64(b))
    } else {
        xadd(coef, V::from_f64(b), bit(choice, 3))
    }
}

fn initial_coef(choice: Choice, b: f64, n: usize) -> V {
    if choice == NATIVE {
        V::from_f64(b - n as f64)
    } else {
        xsub(V::from_f64(b), V::from_f64(n as f64), bit(choice, 3))
    }
}

fn eval_d(choice: Choice, bm1: f64, cn: V, s: V, n: usize) -> V {
    if choice == NATIVE {
        return V::from_f64(bm1 * cn.to_f64() + s.to_f64() / n as f64);
    }
    let n = V::from_f64(n as f64);
    match choice.tree {
        0 => {
            let left = xmul(V::from_f64(bm1), cn, bit(choice, 0));
            let right = xdiv(s, n, bit(choice, 1));
            xadd(left, right, bit(choice, 2))
        }
        1 => {
            let left = xmul(V::from_f64(bm1), cn, bit(choice, 0));
            let scaled = xmul(left, n, bit(choice, 1));
            let numerator = xadd(scaled, s, bit(choice, 2));
            xdiv(numerator, n, bit(choice, 3))
        }
        2 => {
            let scaled = xmul(V::from_f64(bm1), n, bit(choice, 0));
            let left = xmul(scaled, cn, bit(choice, 1));
            let numerator = xadd(left, s, bit(choice, 2));
            xdiv(numerator, n, bit(choice, 3))
        }
        _ => unreachable!(),
    }
}

fn eval_sum(choice: Choice, sum: V, d: V, j: V) -> (V, V) {
    if choice == NATIVE {
        let dj = nmul(d, j);
        return (nadd(sum, dj), dj);
    }
    let dj = xmul(d, j, bit(choice, 0));
    (xadd(sum, dj, bit(choice, 1)), dj)
}

fn eval_final(choice: Choice, u: f64, sum: V) -> f64 {
    if choice == NATIVE {
        u * sum.to_f64()
    } else {
        xmul(V::from_f64(u), sum, false).to_f64()
    }
}

fn eval_body(row: &Row, cfg: &Config) -> Option<(f64, usize)> {
    let p = row.pieces;
    let v = eval_v(cfg.v, p.nu);
    let t2 = eval_t2(cfg.t2, p.lnx);
    let mut j = eval_j0(cfg.j0, p.q, p.r);
    let mut sum = j;
    let mut t = V::from_f64(1.0);
    let mut cn = V::from_f64(1.0);
    let mut n2 = 0.0;
    let mut c = [V::from_f64(0.0); 31];
    let mut d = [V::from_f64(0.0); 31];
    for n in 1..=30 {
        j = eval_recurrence(cfg.recurrence, row.b, n2, p.z, j, t, v);
        n2 += 2.0;
        t = update_t(cfg.t2, t, t2);
        cn = eval_cn(cfg.cn, cn, n2);
        c[n] = cn;
        let mut s = V::from_f64(0.0);
        if n != 1 {
            let mut coef = initial_coef(cfg.s, row.b, n);
            for i in 1..n {
                s = eval_s_term(cfg.s, s, coef, c[i], d[n - i]);
                coef = update_coef(cfg.s, coef, row.b);
            }
        }
        d[n] = eval_d(cfg.d, p.bm1, cn, s, n);
        let (new_sum, dj) = eval_sum(cfg.sum, sum, d[n], j);
        sum = new_sum;
        let sum_f = sum.to_f64();
        let dj_f = dj.to_f64();
        if sum_f <= 0.0 {
            return None;
        }
        if dj_f.abs() <= EPS * sum_f {
            return Some((eval_final(cfg.final_mul, p.u, sum), n));
        }
    }
    Some((eval_final(cfg.final_mul, p.u, sum), 30))
}

fn ordered(bits: u64) -> i128 {
    if bits & (1_u64 << 63) != 0 {
        -((bits ^ u64::MAX) as i128)
    } else {
        bits as i128
    }
}

fn ulp_delta(got: u64, expected: u64) -> i128 {
    ordered(got) - ordered(expected)
}

#[derive(Clone)]
struct Score {
    name: String,
    cfg: Config,
    exact: usize,
    worst: u128,
    sum: u128,
    b25: usize,
    b21: usize,
    tag_exact: BTreeMap<String, usize>,
    tag_total: BTreeMap<String, usize>,
    hit_words: Vec<u64>,
}

fn score(rows: &[Row], name: String, cfg: Config) -> Score {
    let mut exact = 0;
    let mut worst = 0_u128;
    let mut sum = 0_u128;
    let mut b25 = 0;
    let mut b21 = 0;
    let mut tag_exact = BTreeMap::new();
    let mut tag_total = BTreeMap::new();
    let mut hit_words = vec![0_u64; rows.len().div_ceil(64)];
    for (index, row) in rows.iter().enumerate() {
        *tag_total.entry(row.tag.clone()).or_insert(0) += 1;
        let got = eval_body(row, &cfg)
            .map(|pair| pair.0.to_bits())
            .unwrap_or(f64::NAN.to_bits());
        let delta = ulp_delta(got, row.expected).unsigned_abs();
        worst = worst.max(delta);
        sum = sum.saturating_add(delta);
        if got == row.expected {
            exact += 1;
            if row.tag == "b21" {
                b21 += 1;
            } else {
                b25 += 1;
            }
            *tag_exact.entry(row.tag.clone()).or_insert(0) += 1;
            hit_words[index / 64] |= 1_u64 << (index % 64);
        }
    }
    Score {
        name,
        cfg,
        exact,
        worst,
        sum,
        b25,
        b21,
        tag_exact,
        tag_total,
        hit_words,
    }
}

fn rank(scores: &mut [Score]) {
    scores.sort_by(|a, b| {
        b.exact
            .cmp(&a.exact)
            .then(a.worst.cmp(&b.worst))
            .then(a.sum.cmp(&b.sum))
            .then(config_complexity(&a.cfg).cmp(&config_complexity(&b.cfg)))
            .then(a.name.cmp(&b.name))
    });
}

fn config_complexity(cfg: &Config) -> (usize, u32) {
    let choices = [
        cfg.v,
        cfg.t2,
        cfg.j0,
        cfg.recurrence,
        cfg.cn,
        cfg.s,
        cfg.d,
        cfg.sum,
        cfg.final_mul,
    ];
    (
        choices.iter().filter(|choice| **choice != NATIVE).count(),
        choices.iter().map(|choice| choice.mask.count_ones()).sum(),
    )
}

fn print_score(item: &Score, n: usize) {
    println!(
        "{}: {}/{} exact b25={} b21={} worst={} sum={} tags={:?}/{:?}",
        item.name,
        item.exact,
        n,
        item.b25,
        item.b21,
        item.worst,
        item.sum,
        item.tag_exact,
        item.tag_total
    );
}

fn hit(score: &Score, index: usize) -> bool {
    score.hit_words[index / 64] & (1_u64 << (index % 64)) != 0
}

fn print_group_intersection(rows: &[Row], scores: &[Score], label: &str) {
    let mut groups = BTreeMap::<(u64, u64, u64), Vec<usize>>::new();
    for (index, row) in rows.iter().enumerate() {
        if row.tag == "A" {
            groups
                .entry((row.b.to_bits(), row.z0.unwrap(), row.pieces.z.to_bits()))
                .or_default()
                .push(index);
        }
    }
    let groups: Vec<_> = groups
        .into_values()
        .filter(|items| items.len() >= 2)
        .collect();
    let mut ok = 0;
    let mut covered = 0;
    let total_rows: usize = groups.iter().map(Vec::len).sum();
    let mut survivor_hist = BTreeMap::<usize, usize>::new();
    for group in &groups {
        let survivors = scores
            .iter()
            .filter(|candidate| group.iter().all(|index| hit(candidate, *index)))
            .count();
        *survivor_hist.entry(survivors.min(9)).or_insert(0) += 1;
        if survivors > 0 {
            ok += 1;
            covered += group.len();
        }
    }
    println!(
        "group-intersection {label}: subgroups={} any-common-config={} rows-covered={}/{} survivor-count-hist(9=9+)={survivor_hist:?}",
        groups.len(),
        ok,
        covered,
        total_rows
    );
}

fn print_residual_evidence(rows: &[Row], cfg: &Config, label: &str) {
    let mut histogram = BTreeMap::<i128, usize>::new();
    let mut misses = Vec::<(u128, i128, String, u64, u64)>::new();
    let mut coarse = BTreeMap::<String, (usize, usize)>::new();
    for row in rows {
        let got = eval_body(row, cfg)
            .map(|pair| pair.0.to_bits())
            .unwrap_or(f64::NAN.to_bits());
        let delta = ulp_delta(got, row.expected);
        *histogram.entry(delta.clamp(-16, 16)).or_insert(0) += 1;
        let key = if row.tag == "A" {
            format!("A/b={:016x}/z0={:016x}", row.b.to_bits(), row.z0.unwrap())
        } else {
            row.tag.clone()
        };
        let entry = coarse.entry(key).or_insert((0, 0));
        entry.1 += 1;
        if got == row.expected {
            entry.0 += 1;
        } else {
            misses.push((
                delta.unsigned_abs(),
                delta,
                row.id.clone(),
                got,
                row.expected,
            ));
        }
    }
    misses.sort_by(|a, b| b.0.cmp(&a.0).then(a.2.cmp(&b.2)));
    let mut groups: Vec<_> = coarse.into_iter().collect();
    groups.sort_by(|a, b| {
        let (ae, an) = a.1;
        let (be, bn) = b.1;
        (be * an).cmp(&(ae * bn)).then(a.0.cmp(&b.0))
    });
    println!("residual-evidence {label}: clipped-hist(-16/+16=tails)={histogram:?}");
    println!("  weakest coarse groups:");
    for (name, (exact, total)) in groups.iter().take(12) {
        println!("    {name}: {exact}/{total}");
    }
    println!("  largest residual rows:");
    for (_, delta, id, got, expected) in misses.iter().take(16) {
        println!("    {id}: delta={delta:+} got=0x{got:016x} want=0x{expected:016x}");
    }
}

fn family_choices(family: &str) -> Vec<Choice> {
    let mut out = vec![NATIVE];
    let specs: &[(u8, u8)] = match family {
        "v" => &[(0, 3), (1, 2), (2, 2), (3, 2)],
        "t2" => &[(0, 3), (1, 3), (2, 3), (3, 3)],
        "j0" => &[(0, 1)],
        "recurrence" => &[
            (0, 8),
            (1, 8),
            (2, 8),
            (3, 8),
            (4, 8),
            (5, 8),
            (9, 8),
            (10, 8),
            (11, 8),
            (12, 8),
            (13, 8),
            (14, 8),
        ],
        "cn" => &[(0, 3), (1, 3), (2, 3), (3, 3)],
        "s" => &[(0, 4), (1, 4), (2, 4)],
        "d" => &[(0, 3), (1, 4), (2, 4)],
        "sum" => &[(0, 2)],
        "final" => &[(0, 0)],
        _ => unreachable!(),
    };
    for (tree, bits) in specs {
        for mask in 0..(1_u16 << bits) {
            out.push(Choice { tree: *tree, mask });
        }
    }
    out.sort();
    out.dedup();
    out
}

fn set_family(mut cfg: Config, family: &str, choice: Choice) -> Config {
    match family {
        "v" => cfg.v = choice,
        "t2" => cfg.t2 = choice,
        "j0" => cfg.j0 = choice,
        "recurrence" => cfg.recurrence = choice,
        "cn" => cfg.cn = choice,
        "s" => cfg.s = choice,
        "d" => cfg.d = choice,
        "sum" => cfg.sum = choice,
        "final" => cfg.final_mul = choice,
        _ => unreachable!(),
    }
    cfg
}

fn choice_name(family: &str, choice: Choice) -> String {
    if choice == NATIVE {
        format!("{family}:native")
    } else {
        format!("{family}:t{}m{:03x}", choice.tree, choice.mask)
    }
}

fn race_family(rows: &[Row], base: Config, family: &str) -> Vec<Score> {
    let choices = family_choices(family);
    println!(
        "\nrace {family}: {} tree/mask configurations",
        choices.len()
    );
    let mut scores: Vec<Score> = choices
        .par_iter()
        .map(|choice| {
            let cfg = set_family(base, family, *choice);
            score(rows, choice_name(family, *choice), cfg)
        })
        .collect();
    rank(&mut scores);
    for item in scores.iter().take(12) {
        print_score(item, rows.len());
    }
    print_group_intersection(rows, &scores, family);
    scores
}

fn main() {
    let dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "../../work/w109/G3-01-dist".to_string());
    let mut b25 = load_b25(&dir);
    let mut b21 = load_b21_sharp(&dir);
    println!(
        "loaders: b25={} b21-sharp={} selection-only; no heldout",
        b25.len(),
        b21.len()
    );
    let mut rows = Vec::new();
    rows.append(&mut b25);
    rows.append(&mut b21);

    let baseline_cfg = Config::native();
    let baseline = score(
        &rows,
        "baseline:frozen-q/u-native-source".into(),
        baseline_cfg,
    );
    print_score(&baseline, rows.len());
    for row in rows.iter().take(3).chain(rows.iter().skip(822).take(3)) {
        let p = row.pieces;
        let got = eval_body(row, &baseline_cfg).unwrap().0.to_bits();
        println!(
            "primitive-gate {} bm1={:016x} nu={:016x} lnx={:016x} z={:016x} r={:016x} q={:016x} u={:016x} got=0x{got:016x} want=0x{:016x}",
            row.id,
            p.bm1.to_bits(),
            p.nu.to_bits(),
            p.lnx.to_bits(),
            p.z.to_bits(),
            p.r.to_bits(),
            p.q.to_bits(),
            p.u.to_bits(),
            row.expected
        );
        let g = gam1(row.b);
        let e1 = rx::excel_exp(row.b * rx::excel_ln(p.z));
        let e2 = rx::excel_exp(row.a * p.lnx);
        let e3 = rx::excel_exp(0.5 * p.bm1 * p.lnx);
        let r0 = row.b * (1.0 + g) * e1;
        let r1 = r0 * e2;
        let alg = algdiv(row.b, row.a);
        let ua = alg + row.b * rx::excel_ln(p.nu);
        let eu = rx::excel_exp(-ua);
        println!(
            "  sub g={:016x} e1={:016x} e2={:016x} e3={:016x} r0={:016x} r1={:016x} alg={:016x} ua={:016x} eu={:016x}",
            g.to_bits(),
            e1.to_bits(),
            e2.to_bits(),
            e3.to_bits(),
            r0.to_bits(),
            r1.to_bits(),
            alg.to_bits(),
            ua.to_bits(),
            eu.to_bits()
        );
    }
    assert_eq!(
        (baseline.b25, baseline.b21),
        (53, 2),
        "identified-primitive/source-body gate"
    );

    let families = [
        "v",
        "t2",
        "j0",
        "recurrence",
        "cn",
        "s",
        "d",
        "sum",
        "final",
    ];
    let mut family_tops = BTreeMap::<String, Vec<Choice>>::new();
    let mut sum_seed = baseline_cfg;
    for family in families {
        let scores = race_family(&rows, baseline_cfg, family);
        if family == "sum" {
            sum_seed = scores[0].cfg;
        }
        let mut choices = vec![NATIVE];
        let mut per_tree = BTreeMap::<u8, usize>::new();
        for item in &scores {
            let choice = match family {
                "v" => item.cfg.v,
                "t2" => item.cfg.t2,
                "j0" => item.cfg.j0,
                "recurrence" => item.cfg.recurrence,
                "cn" => item.cfg.cn,
                "s" => item.cfg.s,
                "d" => item.cfg.d,
                "sum" => item.cfg.sum,
                "final" => item.cfg.final_mul,
                _ => unreachable!(),
            };
            if choice == NATIVE {
                continue;
            }
            let count = per_tree.entry(choice.tree).or_insert(0);
            if *count < 4 && !choices.contains(&choice) {
                choices.push(choice);
                *count += 1;
            }
        }
        family_tops.insert(family.to_string(), choices);
    }

    let coordinate_evaluations: usize = families
        .iter()
        .filter(|family| **family != "sum")
        .map(|family| family_choices(family).len())
        .sum::<usize>()
        * 2;
    println!(
        "\ncoordinate descent seeded by best isolated sum/dj graph: evaluations={coordinate_evaluations}"
    );
    let mut coordinate = sum_seed;
    for pass in 0..2 {
        for family in families {
            if family == "sum" {
                continue;
            }
            let choices = family_choices(family);
            let mut scored: Vec<Score> = choices
                .par_iter()
                .map(|choice| {
                    let cfg = set_family(coordinate, family, *choice);
                    score(&rows, choice_name(family, *choice), cfg)
                })
                .collect();
            rank(&mut scored);
            coordinate = scored[0].cfg;
            println!("coordinate pass={pass} family={family}:");
            print_score(&scored[0], rows.len());
        }
    }
    let coordinate_score = score(&rows, "coordinate-final".into(), coordinate);
    print_score(&coordinate_score, rows.len());
    println!("  cfg={:?}", coordinate_score.cfg);
    print_residual_evidence(&rows, &coordinate, "coordinate-final");

    println!("\nbeam composition: width=64, stratified top-four per tree plus native");
    let mut beam = vec![baseline_cfg];
    for family in families {
        let choices = &family_tops[family];
        let candidates: BTreeSet<Config> = beam
            .iter()
            .flat_map(|cfg| {
                choices
                    .iter()
                    .map(|choice| set_family(*cfg, family, *choice))
            })
            .collect();
        let candidates: Vec<Config> = candidates.into_iter().collect();
        println!(
            "beam candidates family={family}: choices={} unique-full-configs={}",
            choices.len(),
            candidates.len()
        );
        let mut scored: Vec<Score> = candidates
            .par_iter()
            .enumerate()
            .map(|(index, cfg)| score(&rows, format!("beam:{family}:{index}"), *cfg))
            .collect();
        rank(&mut scored);
        scored.truncate(64);
        println!("beam after {family}:");
        for item in scored.iter().take(5) {
            print_score(item, rows.len());
            println!("  cfg={:?}", item.cfg);
        }
        print_group_intersection(&rows, &scored, &format!("beam-{family}"));
        beam = scored.into_iter().map(|item| item.cfg).collect();
    }

    let mut final_scores: Vec<Score> = beam
        .par_iter()
        .enumerate()
        .map(|(index, cfg)| score(&rows, format!("final-beam:{index}"), *cfg))
        .collect();
    println!("final beam rescoring: {} full configs", final_scores.len());
    rank(&mut final_scores);
    println!("\nfinal ranked beam:");
    for item in final_scores.iter().take(20) {
        print_score(item, rows.len());
        println!("  cfg={:?}", item.cfg);
    }
    print_group_intersection(&rows, &final_scores, "final-beam");
    let winner = if coordinate_score.exact > final_scores[0].exact
        || (coordinate_score.exact == final_scores[0].exact
            && (coordinate_score.worst, coordinate_score.sum)
                < (final_scores[0].worst, final_scores[0].sum))
    {
        coordinate_score
    } else {
        final_scores[0].clone()
    };
    println!("\noverall bounded winner:");
    print_score(&winner, rows.len());
    println!("  cfg={:?}", winner.cfg);
    print_residual_evidence(&rows, &winner.cfg, "bounded-winner");
}
