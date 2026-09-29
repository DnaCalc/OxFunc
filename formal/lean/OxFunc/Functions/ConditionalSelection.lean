import OxFunc.Functions.LogicalFold

namespace OxFunc.Functions
open OxFunc

/-- Prepared values only: this model does not establish expression evaluation order. -/
inductive ConditionalValue where
  | scalar (value : CoercionInput)
  | array (rows : List (List CoercionInput))
  deriving DecidableEq, Repr

def conditionalTruth : CoercionInput → Except WorksheetErrorCode Bool
  | .logical b => .ok b
  | .number n => .ok (n ≠ 0)
  | .text s => match parseLogicalFoldText s with
    | some b => .ok b
    | none => .error .value
  | .error code => .error code
  | .missingArg | .emptyCell => .ok false

def conditionalSelectedCell : CoercionInput → CoercionInput
  | .missingArg | .emptyCell => .number 0
  | other => other

def conditionalSelected : ConditionalValue → ConditionalValue
  | .scalar value => .scalar (conditionalSelectedCell value)
  | .array rows => .array (rows.map (List.map conditionalSelectedCell))

def conditionalShape : ConditionalValue → Nat × Nat
  | .scalar _ => (1, 1)
  | .array rows => (rows.length, (rows.headD []).length)

def conditionalUnionShape (values : List ConditionalValue) : Nat × Nat :=
  values.foldl (fun (r, c) value =>
    let (vr, vc) := conditionalShape value
    (max r vr, max c vc)) (1, 1)

/-- Singleton dimensions broadcast; missing coordinates are #N/A, which a
fallback selector can subsequently catch. Rectangular nonempty grids are admitted. -/
def conditionalAt (value : ConditionalValue) (row col : Nat) : CoercionInput :=
  match value with
  | .scalar cell => cell
  | .array rows =>
    let (height, width) := conditionalShape value
    let r := if height = 1 then 0 else row
    let c := if width = 1 then 0 else col
    (rows[r]?.getD [])[c]?.getD (.error .na)

def conditionalGrid (shape : Nat × Nat) (cell : Nat → Nat → CoercionInput) : ConditionalValue :=
  if shape = (1, 1) then .scalar (cell 0 0)
  else .array ((List.range shape.1).map fun r => (List.range shape.2).map fun c => cell r c)

/-- Reference preparation may scalarize a one-cell area; a direct 1x1 array
remains an array selector until output shape has been determined. -/
def conditionalPreparedReference (value : ConditionalValue) : ConditionalValue :=
  if conditionalShape value = (1, 1) then .scalar (conditionalAt value 0 0) else value

def conditionalIf (condition yes no : ConditionalValue) : ConditionalValue :=
  match condition with
  | .scalar value => match conditionalTruth value with
    | .ok takeYes => conditionalSelected (if takeYes then yes else no)
    | .error code => .scalar (.error code)
  | .array _ =>
    conditionalGrid (conditionalUnionShape [condition, yes, no]) fun r c =>
      match conditionalTruth (conditionalAt condition r c) with
      | .ok takeYes => conditionalSelectedCell (conditionalAt (if takeYes then yes else no) r c)
      | .error code => .error code

def conditionalCatches (naOnly : Bool) : CoercionInput → Bool
  | .error code => !naOnly || code == .na
  | _ => false

def conditionalFallback (naOnly : Bool) (primary fallback : ConditionalValue) : ConditionalValue :=
  match primary with
  | .scalar value => conditionalSelected (if conditionalCatches naOnly value then fallback else primary)
  | .array _ =>
    conditionalGrid (conditionalUnionShape [primary, fallback]) fun r c =>
      let value := conditionalAt primary r c
      conditionalSelectedCell (if conditionalCatches naOnly value then conditionalAt fallback r c else value)

theorem conditional_text_binding :
    conditionalTruth (.text "TrUe") = .ok true ∧
    conditionalTruth (.text "1") = .error .value ∧
    conditionalTruth (.text " TRUE") = .error .value := by exact ⟨rfl, rfl, rfl⟩

theorem conditional_if_shape_binding :
    conditionalIf (.array [[.logical true, .logical false]]) (.scalar (.number 7))
      (.array [[.number 11, .number 12, .number 13]]) =
      .array [[.number 7, .number 12, .error .na]] := by native_decide

theorem conditional_if_condition_error_is_per_cell :
    conditionalIf (.array [[.logical true, .error .div0]]) (.scalar (.number 7))
      (.scalar (.number 9)) = .array [[.number 7, .error .div0]] := by native_decide

theorem conditional_fallback_catches_padding :
    conditionalFallback false (.array [[.number 1, .error .na]])
      (.array [[.number 11, .number 12, .number 13]]) =
      .array [[.number 1, .number 12, .number 13]] := by native_decide

theorem conditional_ifna_preserves_other_errors :
    conditionalFallback true (.array [[.number 1, .error .div0]])
      (.array [[.number 11, .number 12, .number 13]]) =
      .array [[.number 1, .error .div0, .number 13]] := by native_decide

theorem conditional_missing_selected_is_zero :
    conditionalSelectedCell .missingArg = .number 0 := by rfl

theorem conditional_direct_unit_array_still_broadcasts :
    conditionalFallback false (.array [[.number 1]]) (.array [[.number 11, .number 12]]) =
      .array [[.number 1, .number 1]] := by native_decide

theorem conditional_reference_unit_array_is_scalar :
    conditionalFallback false (conditionalPreparedReference (.array [[.number 1]]))
      (.array [[.number 11, .number 12]]) = .scalar (.number 1) := by native_decide

end OxFunc.Functions
