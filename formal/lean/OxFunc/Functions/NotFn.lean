import OxFunc.FunctionCore
import OxFunc.Functions.LogicalFold

namespace OxFunc.Functions

open OxFunc

def notMeta : FunctionMeta := {
  functionId := "FUNC.NOT"
  arity := Arity.exact 1
  determinism := .deterministic
  volatility := .nonvolatile
  hostInteraction := .none
  threadSafety := .safePure
  argPreparationProfile := .valuesOnlyPreAdapter
  coercionLiftProfile := .custom
  kernelSignatureClass := .custom
  fecDependencyProfile := .none
  surfaceFecDependencyProfile := .refOnly
}

theorem notMeta_profiles :
    notMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ notMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ notMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [notMeta]

/-- NOT uses the same scalar truth rule after reference preparation and for each array cell.
Unlike AND/OR/XOR, unrecognized text is an error in every origin. -/
def notPreparedCell : CoercionInput → Except WorksheetErrorCode Bool
  | .logical value => .ok (!value)
  | .number value => .ok (value = 0)
  | .text text => match parseLogicalFoldText text with
    | some value => .ok (!value)
    | none => .error .value
  | .emptyCell => .ok true
  | .missingArg => .error .value
  | .error code => .error code

def notPreparedArray (rows : List (List CoercionInput)) := rows.map (List.map notPreparedCell)

theorem not_text_rule (text : String) :
    notPreparedCell (.text text) =
      match parseLogicalFoldText text with
      | some value => .ok (!value)
      | none => .error .value := by
  rfl

theorem not_error_preserved (code : WorksheetErrorCode) :
    notPreparedCell (.error code) = .error code := by rfl

theorem not_logical_text_examples :
    notPreparedCell (.text "TrUe") = .ok false
    ∧ notPreparedCell (.text "fAlSe") = .ok true
    ∧ notPreparedCell (.text "1") = .error .value
    ∧ notPreparedCell (.text " TRUE") = .error .value
    ∧ notPreparedCell .emptyCell = .ok true := by
  exact ⟨rfl, rfl, rfl, rfl, rfl⟩

end OxFunc.Functions
