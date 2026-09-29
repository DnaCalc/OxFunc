import OxFunc.CoercionPrimitives
import OxFunc.Functions.LogicalFold

namespace OxFunc.Functions
open OxFunc

/-- Prepared numeric parameters of the observed NORM/EXPON distribution
surfaces admit an explicit missing argument as zero. -/
def distributionNumber : CoercionInput → Except WorksheetErrorCode Rat
  | .missingArg => .ok 0
  | input => match coerceToNumber input with
    | .ok number => .ok number
    | .error (.worksheetError code) => .error code
    | .error _ => .error .value

def distributionCumulative : CoercionInput → Except WorksheetErrorCode Bool
  | .logical value => .ok value
  | .number value => .ok (value ≠ 0)
  | .text text => match parseLogicalFoldText text with
    | some value => .ok value
    | none => .error .value
  | .missingArg | .emptyCell => .ok false
  | .error code => .error code

def distributionPrepared (numericCount : Nat) (args : List CoercionInput) :
    Except WorksheetErrorCode (List Rat × Bool) := do
  if args.length != numericCount + 1 then throw .value
  let numbers ← (args.take numericCount).mapM distributionNumber
  let flag ← distributionCumulative (args[numericCount]?.getD .missingArg)
  return (numbers, flag)

/-- Observed multiargument surfaces without a cumulative parameter. -/
def distributionNumericPrepared (count : Nat) (args : List CoercionInput) :
    Except WorksheetErrorCode (List Rat) := do
  if args.length != count then throw .value
  args.mapM distributionNumber

/-- Each group preserves all argument positions after singleton-dimension
broadcast. An absent coordinate is the #N/A value at its argument position,
so an earlier present coercion error still wins. -/
def distributionPreparedLift (numericCount : Nat)
    (groups : List (List CoercionInput)) :
    List (Except WorksheetErrorCode (List Rat × Bool)) :=
  groups.map (distributionPrepared numericCount)

theorem distribution_missing_number_zero : distributionNumber .missingArg = .ok 0 := by rfl

theorem distribution_flag_text_binding :
    distributionCumulative (.text "TrUe") = .ok true ∧
    distributionCumulative (.text "FALSE") = .ok false ∧
    distributionCumulative (.text "1") = .error .value ∧
    distributionCumulative (.text " TRUE") = .error .value := by
  exact ⟨rfl, rfl, rfl, rfl⟩

theorem distribution_explicit_error_order :
    distributionPrepared 1 [.error .ref, .error .num] = .error .ref := by rfl

theorem distribution_numeric_missing_and_error_binding :
    distributionNumericPrepared 3 [.missingArg, .number 1, .number 2] = .ok [0, 1, 2] ∧
    distributionNumericPrepared 3 [.error .ref, .error .na, .number 2] = .error .ref := by
  exact ⟨rfl, rfl⟩

theorem distribution_per_cell_errors :
    distributionPreparedLift 1 [[.missingArg, .logical false],
      [.error .div0, .logical true], [.error .na, .logical false]] =
      [.ok ([0], false), .error .div0, .error .na] := by rfl

theorem distribution_padding_preserves_error_order :
    distributionPreparedLift 1 [[.error .ref, .error .na],
      [.error .na, .error .ref]] = [.error .ref, .error .na] := by rfl

end OxFunc.Functions
