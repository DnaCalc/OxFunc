# Function Slice - Web Text/XML Local Functions Contract (Prelim)

Status: `provisional`
Workset: `W41`

## 1. Purpose
1. define the admitted current-baseline local slice for `ENCODEURL` and `FILTERXML`,
2. separate these local utility semantics from the truly external/provider-bound functions that also live in `W41`,
3. keep the packet honest about what is implemented and what is only classified.

## 2. In-Scope Members
1. `ENCODEURL`
2. `FILTERXML`

## 3. Admitted Slice
1. both functions are admitted on the current local baseline without a live external service dependency.
2. `ENCODEURL` is admitted for scalar-to-text coercion followed by percent-encoding of UTF-8 bytes outside the unreserved set.
3. `FILTERXML` is admitted for well-formed XML text plus XPath expressions that evaluate to a node-set.
4. `FILTERXML` projects a single matching node to a scalar cell and multiple nodes to a vertical spill.

## 4. Main Rules Pinned In This Packet
1. `ENCODEURL("a b+c") = "a%20b%2Bc"`.
2. scalar numeric and logical inputs to `ENCODEURL` first coerce to worksheet text.
3. malformed XML yields `#VALUE!`.
4. an XPath that yields an empty node-set yields `#VALUE!`.
5. an XPath result that is not a node-set on the admitted slice also yields `#VALUE!`.
6. multi-node `FILTERXML` results spill vertically in document order on the admitted slice.

## 5. Out Of Scope
1. platform/version availability outside the current installed baseline,
2. web fetch or service discovery; that belongs with `WEBSERVICE`,
3. richer XPath scalar-result support not observed on the current baseline.

## 6. Boundary Notes
1. this slice is local worksheet semantics, not a provider-subscription seam.
2. the remaining `W41` functions stay interesting because they depend on cube/provider/add-in truth; `ENCODEURL` and the admitted `FILTERXML` slice do not.
