# Follow-up audit of the broad 998-row typed capture

State: scope_completeness=scope_partial; target_completeness=target_partial;
integration_completeness=partial. Open lanes include the residual semantic
families, context-provider coverage and one-cell array publication seam.

The report replays the retained `w111-broad-typed-20260929-002` Excel observations
through the then-current local evaluator executable. Its exact binary SHA-256
is recorded in `report.json`; this is a reproducible intermediate snapshot, not
a live counter after later edits. It records 885 exact rows, 33 one-cell array
publication differences and 80 semantic or unclassified differences. No oracle
answers or historical capture were changed.

Priority after the decimal-parser validation:

1. COUNT and COUNTBLANK reference errors: two simple scalar discrepancies with
   direct source evidence. The focused 476-row follow-up reveals additional
   direct-error and omitted-slot behavior affecting COUNT and COUNTA. The
   subsequent candidate and evidence live in `../count-family/`.
2. IF numeric text and IFERROR/IFNA lifted arrays: three structural discrepancies
   passed to the logical agent. They should use that family's own logical and
   elementwise policies, not a global numeric-text preconversion.
3. LEFT/RIGHT/LEFTB/RIGHTB explicitly omitted length: all four return empty text
   in Excel and #VALUE! locally. LEN/LENB/EXACT/ENCODEURL and regex families also
   have array-admission differences. A shared lift proposal needs per-position
   broadcast and error probes before changing metadata.
4. TEXTBEFORE/TEXTAFTER/TEXTSPLIT optional/empty input behavior: eight residual
   rows distinguish empty source, explicit optional omissions, fallback and
   array results. These deserve a combined optional-argument packet.
5. Array transformation error publication: CHOOSECOLS/CHOOSEROWS/DROP/EXPAND/
   FILTER/HSTACK/VSTACK/SORTBY/WRAPCOLS need scalar error versus computed-array
   probes. Some preserve/pad an input error locally where Excel returns one
   scalar error, so a global blanket error rule would be unsafe.

AGGREGATE/SUBTOTAL, ASC/DBCS/JIS and FORMULATEXT/ISFORMULA still include provider,
locale or catalog/admission questions. The unsupported JIS spelling returns
#NAME? in this Excel baseline. These rows should not all be labeled pure kernel
bugs from the local #VALUE! fallback alone. DSUM's no-match negative-zero output
is an independent numeric publication discrepancy already visible in this bank.
