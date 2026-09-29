import OxFunc.CoercionPrimitives
import OxFunc.ElementaryPublication

namespace OxFunc.ElementaryPrepared

/-- Observed LOG/POWER scalar coercion after reference preparation. -/
def number : CoercionInput → Except CoercionError Rat
  | .missingArg | .emptyCell => .ok 0
  | other => coerceToNumber other

/-- Each missing array coordinate contributes NA at its argument position. -/
def orderedPair (left right : Option CoercionInput) : Except CoercionError (Rat × Rat) := do
  let lhs ← number (left.getD (.error .na))
  let rhs ← number (right.getD (.error .na))
  pure (lhs, rhs)

/-- The admitted MEDIAN contribution slice after aggregate origin expansion. -/
def medianContribution (direct : Bool) (value : CoercionInput) :
    Except CoercionError (Option Rat) :=
  match value with
  | .emptyCell => .ok none
  | .missingArg => .ok (some 0)
  | .number n => .ok (some n)
  | .error code => .error (.worksheetError code)
  | other => if direct then do
      let n ← coerceToNumber other
      pure (some n)
    else .ok none

/-- Direct scalar coercion precedes array/reference errors. Values are then
collected in their original order rather than being reordered by origin. -/
def medianDirectPass : List (Bool × CoercionInput) → Except CoercionError Unit
  | [] => .ok ()
  | (direct, value) :: tail => do
      if direct then
        let _ ← medianContribution true value
        pure ()
      medianDirectPass tail

def medianValues (args : List (Bool × CoercionInput)) : Except CoercionError (List Rat) := do
  medianDirectPass args
  args.foldlM (fun values arg => do
    let value ← medianContribution arg.1 arg.2
    pure (values ++ value.toList)) []

/-- Omission selects the dedicated logarithm; explicit base values remain binary. -/
def logWithOptionalBase (ln ln10 : Float → Float)
    (divide : Float → Float → Float) (value : Float) (base : Option Float) :
    Except WorksheetErrorCode Float :=
  match base with
  | none => if value ≤ 0 then .error .num else .ok (ln10 value)
  | some supplied => ElementaryPublication.log ln divide value supplied

theorem missing_is_zero : number .missingArg = .ok 0 := by rfl
theorem first_error_precedes_later_padding :
    orderedPair (some (.error .div0)) none = .error (.worksheetError .div0) := by rfl
theorem padding_precedes_later_error :
    orderedPair none (some (.error .div0)) = .error (.worksheetError .na) := by rfl
theorem missing_precedes_later_error :
    orderedPair (some .missingArg) (some (.error .num)) = .error (.worksheetError .num) := by rfl
theorem median_empty_and_missing_differ :
    medianContribution true .emptyCell = .ok none ∧
    medianContribution true .missingArg = .ok (some 0) := by constructor <;> rfl
theorem median_direct_error_precedes_earlier_collection_error :
    medianValues [(false, .error .div0), (true, .error .num)] =
      .error (.worksheetError .num) := by rfl
theorem median_collection_error_order_after_direct_pass :
    medianValues [(false, .error .div0), (true, .number 8), (false, .error .num)] =
      .error (.worksheetError .div0) := by rfl
theorem omitted_and_explicit_zero_base_differ :
    ((logWithOptionalBase (fun _ => 1) (fun _ => 7) (fun x y => x / y) 2 none).toOption.map Float.toBits) = some 0x401c000000000000 ∧
    ((logWithOptionalBase (fun _ => 1) (fun _ => 7) (fun x y => x / y) 2 (some 0)).toOption.map Float.toBits) = none := by native_decide

end OxFunc.ElementaryPrepared
