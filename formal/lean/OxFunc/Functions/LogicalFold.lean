import OxFunc.CoercionPrimitives

namespace OxFunc.Functions

open OxFunc

/-- The en-US text rule for AND/OR/XOR, aligned with Rust `parse_excel_logical_text`.
Only ASCII case is folded; whitespace and Unicode look-alikes remain significant.
Evidence: the W110 logical-text spelling and error-precedence COM probes. -/
def parseLogicalFoldText (s : String) : Option Bool :=
  let folded := String.ofList (s.toList.map fun c =>
    if 65 ≤ c.toNat ∧ c.toNat ≤ 90 then Char.ofNat (c.toNat + 32) else c)
  if folded = "true" then some true
  else if folded = "false" then some false
  else none

/-- Shared prepared-item policy. A direct text spelling may contribute a logical;
every other text is ignored. The enclosing fold owns the all-ignored `#VALUE!` rule.
Array and reference origins both pass `direct = false`. -/
def logicalFoldArgumentTruth (direct : Bool) : CoercionInput → Except WorksheetErrorCode (Option Bool)
  | .logical b => .ok (some b)
  | .number n => .ok (some (n ≠ 0))
  | .error code => .error code
  | .text s => .ok (if direct then parseLogicalFoldText s else none)
  | .missingArg | .emptyCell => .ok none

theorem logicalFoldArgumentTruth_direct_text_never_raises (s : String) :
    logicalFoldArgumentTruth true (.text s) = .ok (parseLogicalFoldText s) := by
  simp [logicalFoldArgumentTruth]

theorem logicalFoldArgumentTruth_array_text_is_ignored (s : String) :
    logicalFoldArgumentTruth false (.text s) = .ok none := by
  simp [logicalFoldArgumentTruth]

theorem logicalFoldArgumentTruth_error_surfaces (direct : Bool) (code : WorksheetErrorCode) :
    logicalFoldArgumentTruth direct (.error code) = .error code := by
  rfl

theorem parseLogicalFoldText_case_examples :
    parseLogicalFoldText "TrUe" = some true ∧ parseLogicalFoldText "fAlSe" = some false := by
  native_decide

theorem parseLogicalFoldText_does_not_trim_or_fold_unicode :
    ([" TRUE", "TRUE ", " TRUE ", "\tTRUE", "TRUE\n", " TRUE", "FALſE", "ＴＲＵＥ",
      "1", "0", "1.5", ""].map parseLogicalFoldText).all Option.isNone = true := by
  native_decide

end OxFunc.Functions
