# Rounding preparation and evaluator impact

State: in_progress; scope_completeness=scope_partial;
target_completeness=target_partial; integration_completeness=partial.
Open lanes: HO-FN-028 receiving acknowledgement,
contextual numeric-text grammar, initial15 precision, and HO-FN-029 numeric
overflow publication. The arithmetic and prepared-value questions are separate.

The 691-row typed capture contains 658 exact rows, 32 semantic differences, and
one rejected formula. The rejected TRUNC() spelling arose from a sole explicit
Missing argument and supplies no admitted worksheet observation. The remaining
690 observations preserve numeric bits, text ordinal content and typed fixtures.

All four functions admit an explicit missing numeric or digit argument as zero.
TRUNC's omitted second argument also defaults to zero. Logical, numeric text,
blank reference and mixed reference-array controls follow the already exercised
scalar numeric coercion for these functions. Error checking proceeds in argument
order. A missing array coordinate supplies NA at that argument position; it does
not replace an earlier present coercion error. For example, an earlier text "x"
and a later padded coordinate publish VALUE, while the generic broadcast helper
currently collapses the whole pair to NA.

The planned repair is confined to the four rounding functions. Reference
resolution still uses the existing values-only preparation. A rounding-local
helper supplies missing scalar zero, per-position NA padding and left-to-right
coercion. Singleton dimensions broadcast and larger dimensions use their union.
Generic numeric adapters, other functions, context providers and reference
resolution policy remain unchanged. The existing numeric kernel count conversion
and finite-result policy are reused; the raw overflow publication issue remains
explicitly unresolved under HO-FN-029.

ROUND's UnaryNumericScalarOnly declaration does not describe its exercised
two-argument surface. Parent authorized changing it to Custom after this
assessment, together with its Lean metadata and exact golden metadata row.
ROUNDUP, ROUNDDOWN and TRUNC already declare Custom and need no declaration
change. Their kernel class remains NumsToNum, and values-only preparation plus
the reference capability dependency remain accurate for this admitted surface.

OxFml consumers must retain explicit Missing until the function-local policy is
applied, preserve each argument's padded error position, and use the native
prepared surface instead of generic unary lifting. The parent owns extension
and registration of HO-FN-028, receiving acknowledgement and canonical status.
Filing or extending that handoff supplies no acknowledgement by itself.
