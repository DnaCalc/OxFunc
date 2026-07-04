#!/usr/bin/env python3
"""Regenerate `crates/oxfunc_core/src/excel_numeric/tables.rs` (W108 numeric core).

Extracts the glibc-2.29 __exp_data / __log_data tables from the FROZEN
glibc-2.29 probe C source (see PROBE below) and emits the Rust `tables.rs`. All
numeric literals are copied VERBATIM (as C hex-float / hex-int strings) so the
transcription is byte-faithful.

`tables.rs` is machine output and MUST NOT be hand-edited — change the frozen C
source (or this script) and regenerate instead. Usage:

    python gen_tables.py ../tables.rs   # (path to tables.rs to (re)write)

Rust f64 literals do not support hex-float syntax, so we convert every C literal
to its exact IEEE-754 u64 bit pattern (via Python struct) and emit
`f64::from_bits(0x....)` for doubles, guaranteeing bit-identical constants.
"""
import re, struct, sys

PROBE = r"C:/Work/DnaCalc/OxFunc/smart-fuzzer/runs/w108a-reference-hunt/glibc229_probe.c"

def c_double_to_bits(tok: str) -> int:
    tok = tok.strip().rstrip(',').strip()
    # Python's float() parses C99 hex-float like 0x1.71547652b82fep0
    v = float.fromhex(tok) if ('0x' in tok.lower() or 'p' in tok.lower()) else float(tok)
    return struct.unpack('<Q', struct.pack('<d', v))[0]

def c_u64_to_int(tok: str) -> int:
    tok = tok.strip().rstrip(',').strip()
    return int(tok, 16) if tok.lower().startswith('0x') else int(tok)

src = open(PROBE, encoding='utf-8', errors='replace').read()

# ---- Extract exp_data ----
# exp_data = { ... };  (first block, from line ~39 to ~180)
exp_block = re.search(r'\}\s*exp_data\s*=\s*\{(.*?)\n\};', src, re.S).group(1)

def field_doubles(block, name, count):
    m = re.search(r'\.%s\s*=' % re.escape(name), block)
    # grab up to `count` C-doubles after the '='
    tail = block[m.end():]
    toks = re.findall(r'-?0x[0-9a-fA-F.]+p[+-]?\d+|-?\d+\.\d+(?:[eE][+-]?\d+)?|-?0x[0-9a-fA-F.]+', tail)
    return [c_double_to_bits(t) for t in toks[:count]]

# invln2N is written as `0x1.71547652b82fep0 * N` — evaluate it
m = re.search(r'\.invln2N\s*=\s*([^;,]+)', exp_block)
inv_expr = m.group(1).strip()
# form: 0x1.71547652b82fep0 * N   where N=128
inv_val = float.fromhex('0x1.71547652b82fep0') * 128
invln2N_bits = struct.unpack('<Q', struct.pack('<d', inv_val))[0]

def one_double(block, name):
    m = re.search(r'\.%s\s*=\s*(-?0x[0-9a-fA-F.]+p[+-]?\d+|-?[0-9.]+)' % re.escape(name), block)
    return c_double_to_bits(m.group(1))

exp_shift   = one_double(exp_block, 'shift')
exp_negln2hi= one_double(exp_block, 'negln2hiN')
exp_negln2lo= one_double(exp_block, 'negln2loN')

# exp poly (4 doubles inside .poly = { ... })
poly_txt = re.search(r'\.poly\s*=\s*\{(.*?)\}', exp_block, re.S).group(1)
exp_poly = [c_double_to_bits(t) for t in re.findall(r'-?0x[0-9a-fA-F.]+p[+-]?\d+', poly_txt)]
assert len(exp_poly) == 4, exp_poly

# exp tab: 2*128 = 256 uint64 hex entries inside .tab = { ... }
exp_tab_txt = re.search(r'\.tab\s*=\s*\{(.*?)\}', exp_block, re.S).group(1)
exp_tab = [c_u64_to_int(t) for t in re.findall(r'0x[0-9a-fA-F]+', exp_tab_txt)]
assert len(exp_tab) == 256, len(exp_tab)

# ---- Extract log_data ----
log_block = re.search(r'\}\s*log_data\s*=\s*\{(.*?)\n\};', src, re.S).group(1)
log_ln2hi = one_double(log_block, 'ln2hi')
log_ln2lo = one_double(log_block, 'ln2lo')
poly_txt  = re.search(r'\.poly\s*=\s*\{(.*?)\}', log_block, re.S).group(1)
log_poly  = [c_double_to_bits(t) for t in re.findall(r'-?0x[0-9a-fA-F.]+p[+-]?\d+', poly_txt)]
assert len(log_poly) == 5, len(log_poly)
poly1_txt = re.search(r'\.poly1\s*=\s*\{(.*?)\}', log_block, re.S).group(1)
log_poly1 = [c_double_to_bits(t) for t in re.findall(r'-?0x[0-9a-fA-F.]+(?:p[+-]?\d+)?', poly1_txt)]
assert len(log_poly1) == 11, len(log_poly1)

# tab: 128 pairs {invc, logc}
tab_txt = re.search(r'\.tab\s*=\s*\{(.*?)\n\s*\}', log_block, re.S).group(1)
tab_pairs = re.findall(r'\{\s*(-?0x[0-9a-fA-F.]+p[+-]?\d+)\s*,\s*(-?0x[0-9a-fA-F.]+p[+-]?\d+)\s*\}', tab_txt)
assert len(tab_pairs) == 128, len(tab_pairs)
log_tab = [(c_double_to_bits(a), c_double_to_bits(b)) for a,b in tab_pairs]

# tab2: 128 pairs {chi, clo}
tab2_txt = re.search(r'\.tab2\s*=\s*\{(.*?)\n\s*\}', log_block, re.S).group(1)
tab2_pairs = re.findall(r'\{\s*(-?0x[0-9a-fA-F.]+p[+-]?\d+)\s*,\s*(-?0x[0-9a-fA-F.]+p[+-]?\d+)\s*\}', tab2_txt)
assert len(tab2_pairs) == 128, len(tab2_pairs)
log_tab2 = [(c_double_to_bits(a), c_double_to_bits(b)) for a,b in tab2_pairs]

# ---- Emit Rust ----
def d(bits): return f"f64::from_bits(0x{bits:016x})"

out = []
out.append("// AUTO-GENERATED from glibc229_probe.c (verified no-FMA glibc-2.29 __exp/__log).")
out.append("// All f64 constants are exact IEEE-754 bit patterns copied from the C source.")
out.append("// DO NOT EDIT BY HAND.\n")

out.append("// ===== exp_data =====")
out.append(f"pub const EXP_INVLN2N: f64 = {d(invln2N_bits)};")
out.append(f"pub const EXP_SHIFT: f64 = {d(exp_shift)};")
out.append(f"pub const EXP_NEGLN2HIN: f64 = {d(exp_negln2hi)};")
out.append(f"pub const EXP_NEGLN2LON: f64 = {d(exp_negln2lo)};")
out.append("pub const EXP_POLY: [f64; 4] = [")
for b in exp_poly: out.append(f"    {d(b)},")
out.append("];")
out.append("pub const EXP_TAB: [u64; 256] = [")
for i in range(0, 256, 2):
    out.append(f"    0x{exp_tab[i]:016x}, 0x{exp_tab[i+1]:016x},")
out.append("];\n")

out.append("// ===== log_data =====")
out.append(f"pub const LOG_LN2HI: f64 = {d(log_ln2hi)};")
out.append(f"pub const LOG_LN2LO: f64 = {d(log_ln2lo)};")
out.append("pub const LOG_POLY: [f64; 5] = [")
for b in log_poly: out.append(f"    {d(b)},")
out.append("];")
out.append("pub const LOG_POLY1: [f64; 11] = [")
for b in log_poly1: out.append(f"    {d(b)},")
out.append("];")
out.append("// tab[i] = (invc, logc)")
out.append("pub const LOG_TAB: [(f64, f64); 128] = [")
for a,b in log_tab: out.append(f"    ({d(a)}, {d(b)}),")
out.append("];")
out.append("// tab2[i] = (chi, clo)")
out.append("pub const LOG_TAB2: [(f64, f64); 128] = [")
for a,b in log_tab2: out.append(f"    ({d(a)}, {d(b)}),")
out.append("];")

open(sys.argv[1], 'w').write("\n".join(out) + "\n")
print("OK: exp_tab=%d log_tab=%d log_tab2=%d exp_poly=%d log_poly=%d log_poly1=%d"
      % (len(exp_tab), len(log_tab), len(log_tab2), len(exp_poly), len(log_poly), len(log_poly1)))
print("invln2N=%r shift=%r" % (inv_val, struct.unpack('<d', struct.pack('<Q', exp_shift))[0]))
