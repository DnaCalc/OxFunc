import OxFunc.FunctionCore
import OxFunc.ValueUniverse

namespace OxFunc.Functions

open OxFunc

def hstackMeta : FunctionMeta := {
  functionId := "FUNC.HSTACK"
  arity := { min := 1, max := 255 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

inductive HstackCell where
  | number (n : Rat)
  | text (s : String)
  | logical (b : Bool)
  | error (code : WorksheetErrorCode)
  | empty
  deriving DecidableEq, Repr

inductive HstackArg where
  | scalar (cell : HstackCell)
  | array (rows : List (List HstackCell))
  deriving DecidableEq, Repr

def hstackArgRows : HstackArg → Nat
  | .scalar _ => 1
  | .array rows => rows.length

def hstackArgCols : HstackArg → Nat
  | .scalar _ => 1
  | .array [] => 0
  | .array (row :: _) => row.length

private def listGet? : List α → Nat → Option α
  | [], _ => none
  | x :: _, 0 => some x
  | _ :: xs, n + 1 => listGet? xs n

def hstackCellAt : HstackArg → Nat → Nat → Option HstackCell
  | .scalar cell, 0, 0 => some cell
  | .scalar _, _, _ => none
  | .array rows, row, col =>
      match listGet? rows row with
      | some cells => listGet? cells col
      | none => none

def hstackPadRow (arg : HstackArg) (row : Nat) : List HstackCell :=
  let cols := hstackArgCols arg
  (List.range cols).map fun col =>
    match hstackCellAt arg row col with
    | some cell => cell
    | none => .error .na

def buildHstackRow (args : List HstackArg) (row : Nat) : List HstackCell :=
  args.foldl (fun acc arg => acc ++ hstackPadRow arg row) []

def hstackRowCount (args : List HstackArg) : Nat :=
  args.foldl (fun acc arg => max acc (hstackArgRows arg)) 1

def evalHstackCore (args : List HstackArg) : List (List HstackCell) :=
  (List.range (hstackRowCount args)).map (buildHstackRow args)

theorem evalHstackCore_scalar_appends_as_payload :
    evalHstackCore [
      .array [[.number 1, .number 2]],
      .scalar (.number 3)
    ] =
      [[.number 1, .number 2, .number 3]] := by
  native_decide

theorem evalHstackCore_pads_shorter_argument_with_na :
    evalHstackCore [
      .array [[.number 1], [.number 2]],
      .array [[.number 3]]
    ] =
      [[.number 1, .number 3], [.number 2, .error .na]] := by
  native_decide

theorem evalHstackCore_preserves_full_payload_shape :
    evalHstackCore [
      .array [[.number 1, .number 2], [.number 3, .number 4]],
      .array [[.number 10], [.number 20]]
    ] =
      [[.number 1, .number 2, .number 10], [.number 3, .number 4, .number 20]] := by
  native_decide

theorem hstackMeta_profiles :
    hstackMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ hstackMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [hstackMeta]

end OxFunc.Functions
