# Value Universe Research and Open Questions

Status: `active-research`

## 1. Purpose
Capture admissible evidence and unresolved design questions for Workset 3:
`WORKSET_TUX1000_VALUE_UNIVERSE_AND_EXTENDED_TYPES.md`.

## 2. Evidence Anchors (Public Sources)
1. Excel C API data types:
   - `xltypeNum`, `xltypeStr`, `xltypeBool`, `xltypeErr`, `xltypeMulti`, `xltypeRef`, `xltypeSRef`, `xltypeMissing`, `xltypeNil`.
   - Source: Microsoft Learn, "Data types used by Excel".
2. `xlfRegister` arg typing includes optional (`U`) and omitted-arg behavior with `xltypeMissing`.
   - Source: Microsoft Learn, `xlfRegister (Form 1)`.
3. `xlCoerce` notes `xltypeNil` for empty cells and `xltypeMissing` for omitted args.
   - Source: Microsoft Learn, `xlCoerce`.
4. Worksheet limits include:
   - cell character limit (32,767),
   - formula length limit (8,192 characters).
   - Source: Microsoft Support, "Excel specifications and limits".
5. 3D references are explicit worksheet reference forms (`Sheet1:SheetN!A1`).
   - Source: Microsoft Support, "Create a 3-D reference to the same cell range on multiple worksheets".
6. LAMBDA behavior:
   - uncalled LAMBDA entry returns `#CALC!`,
   - lambda function can be named and called.
   - Source: Microsoft Support, "LAMBDA function".
7. Error families:
   - legacy worksheet errors are represented in C API/VBA enums (`xlErr...`, `XlCVError`),
   - newer worksheet errors (`#CALC!`, `#FIELD!`, `#BLOCKED!`, `#CONNECT!`) have dedicated support docs and may not map 1:1 with legacy enum sets.
   - Source: Microsoft Learn (`XlCVError`) and Microsoft Support error docs.

## 3. Working Interpretation (Provisional)
1. `missing` and `nil/empty` are clearly present at API/interpreter boundary.
2. Worksheet visible value model must distinguish:
   - empty cell state,
   - omitted argument marker,
   - error scalar values.
3. `null` as scalar worksheet value is not yet evidenced as a first-class worksheet value token:
   - `#NULL!` is an error value, not scalar null.
4. Error taxonomy must be versioned and split by boundary:
   - worksheet-visible error family,
   - XLL/UDF-transferable error subset.
5. LAMBDA-like values likely belong to intermediate/function value domain, not ordinary cell-result scalar domain.
6. 3D references should be modeled as reference-domain entities (sheet-set references), not as immediate scalar values.

## 4. Open Questions (Need Empirical Resolution)
1. Can formulas/materialization paths expose distinct behavior for "empty" vs "omitted" in all function families?
2. How does worksheet pipeline normalize newer errors when crossing XLL/UDF boundaries?
3. Are there observable contexts where `-0` survives as distinguishable value token?
4. How should spilled-array error metadata be represented in `ExtendedValue` wrappers?
5. What exact value shape does a function observe when passed a 3D reference in supported functions?

## 5. Immediate TODOs
1. Define draft `ValueTag` algebra with:
   - scalar, error, array, reference, missing, empty, extended wrappers.
2. Create boundary matrix:
   - worksheet cell content,
   - formula eval result,
   - function argument,
   - XLL argument/return.
3. Add version-scoped error registry draft:
   - legacy set,
   - modern extension set,
   - transferability to XLL/UDF surfaces.
4. Link resulting decisions into conformance rows and correlation ledger.

## 6. Sources
1. https://learn.microsoft.com/en-us/office/client-developer/excel/data-types-used-by-excel
2. https://learn.microsoft.com/en-us/office/client-developer/excel/xlfregister-form-1
3. https://learn.microsoft.com/en-us/office/client-developer/excel/xlcoerce
4. https://support.microsoft.com/en-us/office/excel-specifications-and-limits-1672b34d-7043-467e-8e27-269d656771c3
5. https://support.microsoft.com/en-us/office/create-a-3-d-reference-to-the-same-cell-range-on-multiple-worksheets-40ca91ff-9dcb-4ad1-99d2-787d0bc888b6
6. https://support.microsoft.com/en-gb/office/lambda-function-bd212d27-1cd1-4321-a34a-ccbf254b8b67
7. https://learn.microsoft.com/en-us/office/vba/api/excel.xlcverror
8. https://support.microsoft.com/en-us/office/how-to-correct-a-calc-error-d6ee03c5-daf6-426a-8df5-4b284730ab1b
9. https://support.microsoft.com/en-us/office/how-to-correct-a-field-error-836bc3b0-26a3-4e31-a4b5-9133b4c59071
10. https://support.microsoft.com/en-us/office/how-to-correct-a-blocked-error-13be1179-5f7a-4f3e-8b55-d290a8c67dfc
11. https://support.microsoft.com/en-us/office/how-to-correct-a-connect-error-f6d37f9b-9c8f-4773-9f26-8bbca3f6c3a5
