"""One-off seed for docs/function-lane/EXCEL_PARITY_LEDGER.csv (W111-2, oxf-mwue.2).

Run once to draft the ledger; after that the CSV is the source of truth and is edited in
place (git keeps the history). Kept in the repo so the seed's provenance is inspectable.

One row per catalog function id (crates/oxfunc_core/tests/fixtures/function_meta_golden.txt).
Precedence, highest first, each layer overwriting the ones below it:

  1. open rows of docs/OXFUNC_EXCEL_DISCREPANCY_CATALOG.md          (CATALOG_ROWS table)
  2. explicit findings of docs/function-lane/BIT_EXACT_STOCK_TAKE_20260922.md (STOCK_TAKE table)
  3. smart-fuzzer/planning/FUNCTION_STATUS_MAP.md (2026-07-02) buckets, cross-checked
     against the CURRENT status of each linked BUG-FUNC stream
  4. default: Unverified

blocked_by is set from the wall and harness tables independently of status. A lower layer
that disagrees with a higher one is recorded in exact_domain_note as a conflict.

Severity: the ledger allows `unmeasured` when a function is known Divergent but no stated
ULP or class pins the severity. W111-3 must resolve every `unmeasured` before populating
the Rust specs (the enum has no such variant).

Usage:  python tools/parity-ledger/seed_excel_parity_ledger.py
"""

import csv
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
GOLDEN = ROOT / "crates/oxfunc_core/tests/fixtures/function_meta_golden.txt"
STATUS_MAP = ROOT / "smart-fuzzer/planning/FUNCTION_STATUS_MAP.md"
BUG_REGISTER = ROOT / "docs/bugs/BUG_STREAM_REGISTER.csv"
OUT = ROOT / "docs/function-lane/EXCEL_PARITY_LEDGER.csv"

COLUMNS = ["function_id", "status", "severity", "as_of", "excel_builds_judged", "rows_judged",
           "rows_agree", "worst_ulp", "exact_domain_note", "kernel_story_ref", "blocked_by",
           "evidence_refs"]

STOCK = "docs/function-lane/BIT_EXACT_STOCK_TAKE_20260922.md"
CAT = "docs/OXFUNC_EXCEL_DISCREPANCY_CATALOG.md"
MAP = "smart-fuzzer/planning/FUNCTION_STATUS_MAP.md"
WALLS = "docs/function-lane/W109_WALL_CLUES_LEDGER.md"

# ---------------------------------------------------------------- layer 1: catalog rows
# row key -> (functions, severity, worst_ulp, note). Severity from the row's class and any
# stated ULP; `unmeasured` where the row states neither.
CATALOG_ROWS = {
    "G1-02": (["AND", "OR", "XOR"], "structural", "",
              "direct text other than TRUE/FALSE raises #VALUE! where Excel ignores it; fix owned by oxf-xvt5.15"),
    "G3-01": (["BETAINV", "CHIDIST", "CHIINV", "FDIST", "FINV", "GAMMAINV", "HYPGEOMDIST",
               "NEGBINOMDIST", "TDIST", "TINV", "CONFIDENCE.T"], "unmeasured", "",
              "legacy distribution family; bodies on ERFC / bgrat / private PMF walls"),
    "G3-02": (["GAMMA", "GAMMALN", "GAMMALN.PRECISE"], "gross", "1182",
              "exact slices landed (integers 1..88, half-integers, rational residues); B1 [0.7,1.5) max 1182 ULP, n>=89 max 8 ULP"),
    "G3-03": (["TREND", "LINEST", "LOGEST"], "unmeasured", "",
              "coefficient graph open; FORECAST closed out of this row"),
    "G3-04": (["GROWTH"], "unmeasured", "", "666/1240; kernel unknown"),
    "G3-05": (["CHISQ.TEST", "CHITEST"], "numeric", "8",
              "statistic identified; df=1 via ERFC 154/154; rest inherits GRATIO + ERFC"),
    "G3-06": (["F.TEST", "FTEST"], "unmeasured", "", "33/48; variance schedule + F tail"),
    "G3-07": (["GAUSS", "NORMSDIST", "NORM.S.DIST"], "last_bit", "1",
              "sign-split wrapper landed (G-F3); body inherits the ERFC wall; NORMSDIST(-1) 1 ULP"),
    "G4-04": (["ERF.PRECISE", "ERFC.PRECISE", "ERF", "ERFC"], "unmeasured", "",
              "COMBIN/COMBINA closed out of this row; ERF small-z production max 3 ULP, ERFC mid band best 2389/7741"),
    "G6-01": (["PMT", "PPMT", "IPMT", "CUMIPMT"], "unmeasured", "",
              "expm1 |tau|<1 double-rounding wall; live PMT 10,801/13,752"),
    "G6-03": (["YIELD"], "numeric", "19", "forward kernel exact; solver plateau publication, 19-ULP end-to-end witness"),
    "G6-04": (["ODDFYIELD"], "gross", "3700000", "62k to 3.7M ULP; shares the YIELD solver"),
    "G6-03d": (["PRICE"], "last_bit", "1", "Actual/360 and Actual/365: all residuals exactly -1 ULP (564/600)"),
    "G6-03c": (["DURATION", "MDURATION"], "last_bit", "3", "237/264, max 3 ULP; accumulator graph"),
    "G6-05": (["RATE"], "numeric", "586", "best 2/256; mortgage witness 586 ULP; nper=1 wall"),
    "G6-06": (["IRR"], "gross", "114720", "44/72 leader; witness ~114,720 ULP"),
    "G6-07": (["CUMPRINC"], "unmeasured", "", "shipping 90/540; hidden per-period principal"),
}

# ---------------------------------------------------------------- layer 2: stock take
# name -> (status, severity, as_of, rows_judged, rows_agree, worst_ulp, note)
C, D, U = "consistent", "divergent", "unverified"
STOCK_TAKE = {
    # section A, no noted miss -> Consistent (Characterized needs a section-4 check; see note)
    "EXP": (C, "", "2026-09-12", "249", "249", "0", "x87 CRT chain; 36 near-midpoint witnesses; characterized-candidate"),
    "LN": (C, "", "2026-09-12", "249", "249", "0", "x87 CRT chain; characterized-candidate"),
    "LOG10": (C, "", "2026-09-12", "249", "249", "0", "x87 CRT chain; characterized-candidate"),
    "LOG": (C, "", "2026-09-12", "249", "249", "0", "LOG(x,base)=ln/ln; characterized-candidate"),
    "POWER": (C, "", "2026-07-04", "715", "715", "0", "715/715 is the fitted matrix; b24 11,045/11,045; re-race 33,145/33,145"),
    "OP_POWER": (C, "", "2026-07-04", "", "", "", "shares the POWER kernel; no separate operator count"),
    "SIN": (C, "", "2026-09-12", "1020", "1020", "0", "held-out fFSIN; trig reduction G4-01 5425/5425; characterized-candidate"),
    "COS": (C, "", "2026-09-12", "2561", "2561", "0", "characterized-candidate"),
    "TAN": (C, "", "2026-09-12", "1020", "1020", "0", "characterized-candidate"),
    "SEC": (C, "", "2026-09-12", "8", "8", "0", "small corpus"),
    "CSC": (C, "", "2026-09-12", "8", "8", "0", "small corpus"),
    "COT": (C, "", "2026-09-12", "10", "10", "0", "small corpus"),
    "ACOT": (C, "", "2026-09-12", "28", "28", "0", "small corpus"),
    "COSH": (C, "", "2026-09-12", "40", "40", "0", ""),
    "SINH": (C, "", "2026-09-12", "37", "37", "0", ""),
    "SECH": (C, "", "2026-09-12", "40", "40", "0", ""),
    "CSCH": (C, "", "2026-09-12", "5", "5", "0", "small corpus"),
    "COTH": (C, "", "2026-09-12", "28", "28", "0", "small corpus"),
    "SQRTPI": (C, "", "2026-09-12", "", "", "", "pow(n*pi, 0.5) landed"),
    "MOD": (C, "", "2026-06-20", "19", "19", "0", "exact IEEE remainder"),
    "ATANH": (C, "", "2026-09-12", "20780", "20780", "0", "closed_signed_off; characterized-candidate"),
    "ACOTH": (C, "", "2026-09-12", "335321", "335321", "0", "incl. held-out 66,552/66,552; closed_signed_off; characterized-candidate"),
    "XNPV": (C, "", "2026-07-11", "1705", "1705", "0", "1530/1530 + 175/175 error rows; x87 spill loop; characterized-candidate"),
    "NPER": (C, "", "2026-07-12", "1293", "1293", "0", ""),
    "FV": (C, "", "2026-07-12", "149", "149", "0", "incl. adversarial"),
    "PV": (C, "", "2026-07-12", "48", "48", "0", ""),
    "YIELDMAT": (C, "", "2026-07-12", "1250", "1250", "0", ""),
    "TBILLYIELD": (C, "", "2026-07-12", "2156", "2156", "0", "fitted matrix, not held-out"),
    "YIELDDISC": (C, "", "2026-06-20", "", "", "", "three-way replay bit-exact; small corpus"),
    "EFFECT": (C, "", "2026-08-20", "160", "160", "0", ""),
    "NOMINAL": (C, "", "2026-08-20", "", "", "", "identified"),
    "PDURATION": (C, "", "2026-09-12", "80", "80", "0", "split-LN form"),
    "CONVERT": (C, "", "2026-07-12", "44607", "44607", "0", "10,418 + 34,189; characterized-candidate"),
    "COMBIN": (C, "", "2026-07-12", "22242", "22242", "0", "characterized-candidate"),
    "COMBINA": (C, "", "2026-07-12", "40330", "40330", "0", "+64,767 replay; characterized-candidate"),
    "PERMUT": (C, "", "2026-07-12", "702", "702", "0", "held-out split not published"),
    "FACT": (C, "", "2026-07-12", "", "", "", "reverse native product landed (<=170)"),
    "FACTDOUBLE": (C, "", "2026-07-12", "", "", "", "reverse native product landed"),
    "EXPON.DIST": (C, "", "2026-07-20", "4000", "4000", "0", "held-out"),
    "PHI": (C, "", "2026-07-20", "764", "764", "0", ""),
    "FORECAST": (C, "", "2026-07-12", "65", "65", "0", "centered kernel"),
    "FORECAST.LINEAR": (C, "", "2026-07-12", "4", "4", "0", "small corpus"),
    "SLOPE": (C, "", "2026-07-12", "4", "4", "0", "small corpus"),
    "INTERCEPT": (C, "", "2026-07-12", "4", "4", "0", "small corpus"),
    "BESSELI": (C, "", "2026-07-12", "18", "18", "0", "small corpus"),
    "BESSELK": (C, "", "2026-07-12", "794", "794", "0", "x>=8 composition"),
    # section A with a noted miss -> Divergent
    "ACOS": (D, "last_bit", "2026-09-12", "34", "33", "1", "ACOS(0.5) libm 1 ULP"),
    "TANH": (D, "unmeasured", "2026-09-12", "11", "8", "", "honest 8/11"),
    "WEIBULL.DIST": (D, "last_bit", "2026-07-20", "6000", "5999", "1", "one chain-microdetail +-1 miss (W109 wall-ledger 'Wall 1' class)"),
    "RRI": (D, "unmeasured", "2026-08-20", "", "", "", "5536/5536 on the kernel corpus; RRI vs POWER 5/6 (W111-10)"),
    "ISPMT": (D, "unmeasured", "2026-09-12", "6", "4", "", "4/6 (W111-10)"),
    "ACCRINT": (D, "unmeasured", "2026-07-12", "146850", "146850", "", "catalog 146,850/146,850 but stock take lists ACCRINT residual rows as not yet exact; conflict to resolve"),
    "ODDFPRICE": (D, "unmeasured", "2026-07-14", "15", "15", "", "30/360 exact (10/10 + 5/5); actual bases not yet exact per stock take, docs disagree (W111-10: measure)"),
    "MINVERSE": (C, "", "2026-07-12", "1599", "1599", "0", "numeric 1599/1599 Doolittle LU (BUG-FUNC-025 closed_signed_off); MINVERSE(5) is an internal 1x1 array in Excel too (nested TYPE probes), final-cell shape is Cat-1 publication CSC-0024 / HO-FN-010, not an OxFunc divergence"),
    "POISSON": (D, "unmeasured", "2026-09-12", "56", "38", "", "k=0 34,000+ exact; k=1 / small lambda 38/56 live, sqrt staging"),
    "POISSON.DIST": (D, "unmeasured", "2026-09-12", "56", "38", "", "as POISSON"),
    "T.DIST": (D, "numeric", "2026-09-12", "", "", "9", "df=1 and df=2 PDFs exact; CDFs 46/75 (df=2); earlier scores invalid (broken COM ULP helper)"),
    "T.DIST.2T": (D, "numeric", "2026-09-12", "", "", "9", "T.DIST.2T(1,1) 9 ULP"),
    "T.DIST.RT": (D, "unmeasured", "2026-09-12", "", "", "", "CDF family"),
    "T.INV.2T": (D, "unmeasured", "2026-09-12", "", "", "", "df=1 9/9 exact; df>=2 blocked on BETA.INV"),
    "T.INV": (D, "unmeasured", "2026-09-12", "", "", "", "blocked on BETA.INV last bit; code comment 135/135 treated as unverified"),
    "BESSELJ": (D, "gross", "2026-07-12", "", "", "100000000", "composition exact at order 0; order>=1 x>=8 1e8-1e11 ULP (BUG-FUNC-024)"),
    "BESSELY": (D, "gross", "2026-07-12", "", "", "100000000", "order>=1 x>=8 1e8-1e11 ULP; 93/93 was fitted"),
    "BINOM.DIST": (D, "unmeasured", "2026-09-12", "", "", "", "k=n and k=0 slices landed; general-k 37.9%, overall 49.59%"),
    "BINOMDIST": (D, "unmeasured", "2026-09-12", "", "", "", "as BINOM.DIST"),
    "GAMMA.DIST": (D, "numeric", "2026-09-12", "446", "337", "8", "CDF 337/446; PDF 4-8 ULP"),
    "GAMMADIST": (D, "numeric", "2026-09-12", "", "", "8", "as GAMMA.DIST"),
    "CHISQ.DIST": (D, "numeric", "2026-09-12", "4100", "1615", "8", "b26 1,615/4,100; PDF 4-8 ULP"),
    "CHISQ.DIST.RT": (D, "unmeasured", "2026-09-12", "", "", "", "CDF tail family"),
    "BETA.DIST": (D, "unmeasured", "2026-07-20", "671", "293", "", "293/671 held-out; spec identity 20,008/20,008"),
    "BETADIST": (D, "unmeasured", "2026-07-20", "", "", "", "as BETA.DIST"),
    "NEGBINOM.DIST": (D, "unmeasured", "2026-09-12", "48", "11", "", "large-n 0/48; k=0 slice 8/8 landed"),
    "HYPGEOM.DIST": (D, "unmeasured", "2026-09-12", "20", "0", "", "0/20 vs every COMBIN association"),
    "MULTINOMIAL": (D, "numeric", "2026-09-12", "10", "1", "22", "private Lanczos lgamma"),
    "F.DIST": (D, "unmeasured", "2026-09-12", "", "", "", "not a BETA identity"),
    "F.DIST.RT": (D, "unmeasured", "2026-09-12", "", "", "", ""),
    "F.INV": (D, "gross", "2026-09-12", "105", "27", "6317", ""),
    "F.INV.RT": (D, "unmeasured", "2026-09-12", "", "", "", ""),
    "CHISQ.INV": (D, "unmeasured", "2026-09-12", "65", "54", "", "df=2 dense 54/65"),
    "CHISQ.INV.RT": (D, "unmeasured", "2026-09-12", "", "", "", ""),
    "GAMMA.INV": (D, "unmeasured", "2026-09-12", "60", "18", "", "incl. near-1 tail/cap (was oxf-acdw.3.1)"),
    "BETA.INV": (D, "unmeasured", "2026-09-12", "30", "12", "", "closed forms refuted"),
    "Z.TEST": (D, "unmeasured", "2026-09-12", "", "", "", "16.7%"),
    "ZTEST": (D, "unmeasured", "2026-09-12", "", "", "", "as Z.TEST"),
    "FISHERINV": (D, "unmeasured", "2026-09-12", "27", "10", "", ""),
    "NORM.INV": (D, "unmeasured", "2026-09-12", "24", "0", "", "kept on Acklam by design; oddness 8/9"),
    "NORMINV": (D, "unmeasured", "2026-09-12", "24", "0", "", "as NORM.INV"),
    "NORM.S.INV": (D, "unmeasured", "2026-09-12", "24", "0", "", "as NORM.INV"),
    "NORMSINV": (D, "unmeasured", "2026-09-12", "24", "0", "", "as NORM.INV"),
    "LOGNORM.INV": (D, "unmeasured", "2026-09-12", "", "", "", "as NORM.INV"),
    "LOGINV": (D, "unmeasured", "2026-09-12", "", "", "", "as NORM.INV"),
    "NORM.DIST": (D, "last_bit", "2026-08-21", "64", "64", "1", "wrapper 64/64; CDF body inherits the ERFC wall"),
    "XIRR": (D, "unmeasured", "2026-07-12", "", "", "", "solver on xnpv_kernel_raw; large-root precision inventory open"),
    "NPV": (D, "last_bit", "2026-07-12", "900", "636", "4", "no exact candidate"),
    "IMEXP": (D, "unmeasured", "2026-09-12", "", "", "", "non-identities recorded; 90-97% in runs"),
    "IMSEC": (D, "unmeasured", "2026-09-12", "", "", "", "non-identities recorded"),
    "IMPOWER": (D, "unmeasured", "2026-09-12", "", "", "", "non-identities recorded"),
    # section D and the W111-5 list -> Unverified, whatever the July map said
    "COUPDAYBS": (U, "", "", "", "", "", "6x1060 Handbook captures, no judgement (W111-5)"),
    "COUPDAYS": (U, "", "", "", "", "", "6x1060 Handbook captures, no judgement (W111-5)"),
    "COUPDAYSNC": (U, "", "", "", "", "", "6x1060 Handbook captures, no judgement (W111-5)"),
    "COUPNCD": (U, "", "", "", "", "", "6x1060 Handbook captures, no judgement (W111-5)"),
    "COUPNUM": (U, "", "", "", "", "", "6x1060 Handbook captures, no judgement (W111-5)"),
    "COUPPCD": (U, "", "", "", "", "", "6x1060 Handbook captures, no judgement (W111-5)"),
    "ACCRINTM": (U, "", "", "", "", "", "single witness (W111-5)"),
    "PRICEMAT": (U, "", "", "", "", "", "July map bit_exact_observed but one witness only (W111-5)"),
    "TBILLEQ": (U, "", "", "", "", "", "W111-5"),
    "TBILLPRICE": (U, "", "", "", "", "", "W111-5"),
    "DOLLARDE": (U, "", "", "", "", "", "W111-5"),
    "DAYS360": (U, "", "", "", "", "", "W111-5"),
    "MDETERM": (U, "", "", "", "", "", "July map bit_exact_observed, no count (W111-5)"),
    "MMULT": (U, "", "", "", "", "", "numeric provisional, no count (W111-5); MMULT(5,2) 1x1 array is Excel-correct, final-cell shape is Cat-1 publication CSC-0025 / HO-FN-010"),
    "ASINH": (U, "", "", "", "", "", "July map bit_exact_observed, no dedicated lane (W111-5)"),
    "ACOSH": (U, "", "", "", "", "", "July map bit_exact_observed, no dedicated lane (W111-5)"),
    "NORMDIST": (U, "", "", "", "", "", "rounded witnesses only; CDF mode inherits the ERFC wall (W111-5)"),
    "LOGNORM.DIST": (U, "", "", "", "", "", "rounded witnesses only; CDF mode inherits the ERFC wall (W111-5)"),
    "KURT": (U, "", "", "", "", "", "0% in runs, untriaged: check the array route before calling it Divergent (W111-5)"),
    "SKEW": (U, "", "", "", "", "", "0% in runs, untriaged: check the array route before calling it Divergent (W111-5)"),
    "YEARFRAC": (U, "", "", "", "", "", "70% historical aggregate, current state unknown (W111-5)"),
    "NETWORKDAYS": (U, "", "", "", "", "", "67.9% historical aggregate, current state unknown (W111-5)"),
    "NETWORKDAYS.INTL": (U, "", "", "", "", "", "historical aggregate only (W111-5)"),
    "WORKDAY": (U, "", "", "", "", "", "historical aggregate only (W111-5)"),
    "WORKDAY.INTL": (U, "", "", "", "", "", "historical aggregate only (W111-5)"),
    "BAHTTEXT": (U, "", "", "", "", "", "excluded in July by scope decision; in scope now (W111-5)"),
}
# locale-seam Cat-1 surfaces: judged under a pinned locale (W111-11)
for _n in ["TEXT", "VALUE", "NUMBERVALUE", "DATEVALUE", "TIMEVALUE", "DOLLAR", "FIXED", "ASC",
           "DBCS", "JIS"]:
    STOCK_TAKE[_n] = (U, "", "", "", "", "", "Cat-1 locale seam; July runs returned #VALUE! at the seam (TEXT 0/78); judge under a pinned locale (W111-11)")

# ---------------------------------------------------------------- blocked_by tables
WALL = {
    "wall:erf-erfc-body": ["ERF", "ERF.PRECISE", "ERFC", "ERFC.PRECISE", "NORMSDIST", "NORM.S.DIST",
                           "NORM.DIST", "NORMDIST", "GAUSS", "LOGNORM.DIST", "LOGNORMDIST", "NORM.INV",
                           "NORMINV", "NORM.S.INV", "NORMSINV", "LOGNORM.INV", "LOGINV", "CHIDIST",
                           "CHISQ.TEST", "CHITEST", "Z.TEST", "ZTEST", "CONFIDENCE", "CONFIDENCE.NORM"],
    "wall:pmt-expm1": ["PMT", "IPMT", "PPMT", "CUMIPMT", "CUMPRINC"],
    "wall:irr-rate-solver": ["IRR", "RATE"],
    "wall:bgrat": ["BETA.DIST", "BETADIST", "BETA.INV", "BETAINV", "T.DIST", "T.DIST.2T", "T.DIST.RT",
                   "TDIST", "T.INV", "T.INV.2T", "TINV", "F.DIST", "F.DIST.RT", "FDIST", "F.INV",
                   "F.INV.RT", "FINV", "F.TEST", "FTEST", "CONFIDENCE.T", "GAMMA.DIST", "GAMMADIST",
                   "CHISQ.DIST", "CHISQ.DIST.RT", "CHIINV", "CHISQ.INV", "CHISQ.INV.RT", "GAMMA.INV",
                   "GAMMAINV"],
    "wall:gammaln-b1b2": ["GAMMALN", "GAMMALN.PRECISE", "GAMMA"],
    "wall:private-pmf": ["NEGBINOM.DIST", "NEGBINOMDIST", "HYPGEOM.DIST", "HYPGEOMDIST", "MULTINOMIAL",
                         "GAMMA.DIST", "GAMMADIST", "CHISQ.DIST"],
}
HARNESS = {
    "harness:workbook": ["AGGREGATE", "SUBTOTAL", "XLOOKUP", "OFFSET", "SHEET", "SHEETS", "ISFORMULA",
                         "FORMULATEXT", "CELL", "INDIRECT", "INFO", "OP_TRIM_REF_BOTH",
                         "OP_TRIM_REF_LEADING", "OP_TRIM_REF_TRAILING", "OP_SPILL_REF", "OP_RANGE_REF",
                         "OP_INTERSECTION_REF", "OP_UNION_REF", "OP_IMPLICIT_INTERSECTION"],
    "harness:random": ["RAND", "RANDARRAY", "RANDBETWEEN", "NOW", "TODAY"],
    "harness:lambda": ["MAP", "SCAN", "BYROW", "BYCOL", "REDUCE", "MAKEARRAY", "ISOMITTED", "GROUPBY",
                       "PIVOTBY", "LET", "LAMBDA"],
    "harness:locale": ["TEXT", "VALUE", "NUMBERVALUE", "DATEVALUE", "TIMEVALUE", "DOLLAR", "FIXED",
                       "ASC", "DBCS", "JIS", "BAHTTEXT"],
}
DEFERRED = {"WEBSERVICE", "STOCKHISTORY"}  # plus every CUBE*


def catalog_ids():
    ids = []
    for line in GOLDEN.read_text(encoding="utf-8").splitlines():
        if " => FunctionMeta {" in line:
            ids.append(line.split(" => ")[0])
    return sorted(ids)


def status_map():
    """surface -> (bucket, streams, runs_seen, last_seen)"""
    out, bucket = {}, None
    for line in STATUS_MAP.read_text(encoding="utf-8").splitlines():
        m = re.match(r"^## (\w+) \((\d+)\)", line)
        if m:
            bucket = m.group(1)
            continue
        if bucket in ("bit_exact_observed", "deferred") and line and not line.startswith(("#", "_")):
            for name in (n.strip() for n in line.split(",")):
                if name:
                    out[name] = (bucket, "", "", "")
            continue
        if bucket and line.startswith("| `"):
            cells = [c.strip() for c in line.strip().strip("|").split("|")]
            name = cells[0].strip("`")
            streams = cells[1] if len(cells) > 1 else ""
            runs = cells[2] if len(cells) > 2 else ""
            last = cells[3] if len(cells) > 3 else ""
            out[name] = (bucket, streams, runs, last)
    return out


def stream_status():
    rows = list(csv.DictReader(BUG_REGISTER.open(encoding="utf-8")))
    return {r["bug_id"]: (r["status"], r["last_reviewed"]) for r in rows}


def mdy_to_iso(s):
    m = re.match(r"(\d{2})/(\d{2})/(\d{4})", s or "")
    return f"{m.group(3)}-{m.group(1)}-{m.group(2)}" if m else "2026-07-02"


def main():
    ids = catalog_ids()
    short = {i: i[len("FUNC."):] for i in ids}
    by_short = {v: k for k, v in short.items()}
    smap = status_map()
    streams = stream_status()
    rows = {}
    unknown_names = set()

    def row(fid):
        return rows.setdefault(fid, {c: "" for c in COLUMNS} | {"function_id": fid, "status": "unverified"})

    def note(r, text):
        r["exact_domain_note"] = "; ".join(x for x in [r["exact_domain_note"], text] if x)

    def ref(r, text):
        r["evidence_refs"] = ";".join(x for x in [r["evidence_refs"], text] if x)

    for fid in ids:
        row(fid)

    # layer 3: July status map crossed with current stream status
    for name, (bucket, stream_cell, runs, last) in smap.items():
        fid = by_short.get(name)
        if not fid:
            continue
        r = row(fid)
        ids_in_cell = re.findall(r"BUG-FUNC-\d+", stream_cell)
        states = [streams.get(b, ("unknown", ""))[0] for b in ids_in_cell]
        ref(r, f"{MAP}#{bucket}")
        for b in ids_in_cell:
            ref(r, f"BUG_STREAM_REGISTER:{b}")
        if bucket == "bit_exact_observed":
            r.update(status="consistent", as_of="2026-07-02")
            note(r, "July map bit_exact_observed (array_rollup runs, no non-match; not a closure claim)")
        elif bucket in ("structural_bug_open", "numeric_drift_open"):
            sev = "structural" if bucket == "structural_bug_open" else "unmeasured"
            if states and all(s == "closed_signed_off" for s in states):
                r.update(status="consistent", as_of="2026-07-02")
                note(r, f"July {bucket} via {','.join(ids_in_cell)}; streams now closed_signed_off")
            elif states and all(s in ("closed", "validated_local", "closed_signed_off", "handed_off") for s in states):
                r.update(status="unverified", as_of="")
                note(r, f"July {bucket} via {','.join(ids_in_cell)}; streams now {','.join(sorted(set(states)))} (fix not re-judged live): re-judge in W111-6")
            else:
                r.update(status="divergent", severity=sev, as_of=mdy_to_iso(last))
                note(r, f"July {bucket} via {','.join(ids_in_cell) or 'no stream'}; stream status {','.join(sorted(set(states))) or 'n/a'}")
        elif bucket == "mixed_or_open":
            r.update(status="divergent", severity="unmeasured", as_of=mdy_to_iso(last))
            note(r, "July mixed_or_open: genuine non-match rows, no linked stream, untriaged (W111-6)")
        elif bucket in ("harness_pending", "excluded"):
            r.update(status="unverified", as_of="")
            note(r, f"July {bucket}: {stream_cell}")
        elif bucket == "deferred":
            r.update(status="unverified", as_of="")
            note(r, "July deferred; in scope unless CUBE*/WEBSERVICE/STOCKHISTORY")
        if runs:
            r["rows_judged"] = r["rows_judged"] or ""

    # layer 2: stock take
    for name, (status, sev, as_of, judged, agree, ulp, text) in STOCK_TAKE.items():
        fid = by_short.get(name)
        if not fid:
            unknown_names.add(name)
            continue
        r = row(fid)
        prev = r["status"]
        if prev not in ("unverified", status):
            note(r, f"conflict: July map said {prev}, stock take says {status}; stock take wins")
        r.update(status=status, severity=sev, as_of=as_of, rows_judged=judged, rows_agree=agree,
                 worst_ulp=ulp)
        note(r, text)
        ref(r, STOCK)

    # layer 1: catalog open rows
    for key, (names, sev, ulp, text) in CATALOG_ROWS.items():
        for name in names:
            fid = by_short.get(name)
            if not fid:
                unknown_names.add(name)
                continue
            r = row(fid)
            if r["status"] not in ("divergent", "unverified"):
                note(r, f"conflict: {r['status']} below, open catalog row {key} above; catalog wins")
            r["status"] = "divergent"
            old = r["severity"]
            order = ["last_bit", "numeric", "gross", "structural"]
            if sev == "unmeasured":
                r["severity"] = old or "unmeasured"
            elif old in order and order.index(old) > order.index(sev):
                pass  # keep the worse class already known
            else:
                r["severity"] = sev
            if ulp and (not r["worst_ulp"] or int(ulp) > int(r["worst_ulp"] or 0)):
                r["worst_ulp"] = ulp
            r["as_of"] = max(r["as_of"] or "", "2026-09-15")  # catalog reconciled 2026-09-15
            note(r, f"catalog {key}: {text}")
            ref(r, f"{CAT}#{key}")

    # deferred by policy
    for fid, s in short.items():
        if s.startswith("CUBE") or s in DEFERRED:
            rows[fid].update(status="deferred", severity="", as_of="2026-09-22")
            note(rows[fid], "deferred by policy (ODR-FN-005)")

    # LET / LAMBDA: not catalog functions yet (oxf-v8ui D1 decides they become ones); rows now so
    # they are not lost. W111-3's join must tolerate exactly these two until the catalog has them.
    for name in ("LET", "LAMBDA"):
        fid = "FUNC." + name
        r = row(fid)
        by_short[name] = fid
        note(r, "not yet a catalog FunctionMeta (oxf-v8ui D1); OxFml evaluates it today")
        ref(r, "oxf-v8ui")

    # blocked_by
    for table in (WALL, HARNESS):
        for key, names in table.items():
            for name in names:
                fid = by_short.get(name)
                if not fid:
                    unknown_names.add(name)
                    continue
                cur = [x for x in rows[fid]["blocked_by"].split(";") if x]
                if key not in cur:
                    cur.append(key)
                rows[fid]["blocked_by"] = ";".join(cur)
                if key.startswith("wall:"):
                    ref(rows[fid], f"{WALLS}")

    # builds judged: honest about what we know
    for r in rows.values():
        if r["status"] in ("consistent", "divergent", "characterized") and not r["excel_builds_judged"]:
            r["excel_builds_judged"] = "pre-20430 (20026..20326; per-row build in evidence)"
        # dedupe refs
        seen, out = set(), []
        for x in r["evidence_refs"].split(";"):
            if x and x not in seen:
                seen.add(x)
                out.append(x)
        r["evidence_refs"] = ";".join(out)
        if r["status"] != "divergent":
            r["severity"] = ""

    with OUT.open("w", encoding="utf-8", newline="") as f:
        w = csv.DictWriter(f, fieldnames=COLUMNS, lineterminator="\n")
        w.writeheader()
        for fid in sorted(rows):
            w.writerow(rows[fid])

    from collections import Counter
    print("rows", len(rows))
    print("status", dict(Counter(r["status"] for r in rows.values())))
    print("severity", dict(Counter(r["severity"] for r in rows.values() if r["status"] == "divergent")))
    print("names not in catalog (ignored):", sorted(unknown_names))


if __name__ == "__main__":
    main()
