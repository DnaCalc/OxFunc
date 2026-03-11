import OxFunc.FunctionCore
import OxFunc.CoercionPrimitives

namespace OxFunc.Functions

open OxFunc

private instance instDecidableEqExceptSequence [DecidableEq ε] [DecidableEq α] : DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def sequenceMeta : FunctionMeta := {
  functionId := "FUNC.SEQUENCE"
  arity := { min := 1, max := 4 }
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

structure SequenceArray where
  rows : Nat
  cols : Nat
  payload : List Rat
  deriving DecidableEq, Repr

def ratToPosNat? (n : Rat) : Option Nat :=
  if n > 0 then
    if n.den = 1 then
      some n.num.natAbs
    else
      none
  else
    none

def parseDimensionDefault (arg : CoercionInput) (default : Nat) :
    Except CoercionError Nat :=
  match arg with
  | .missingArg | .emptyCell => Except.ok default
  | other =>
      match coerceToNumber other with
      | Except.ok n =>
          match ratToPosNat? n with
          | some d => Except.ok d
          | none => Except.error (CoercionError.unsupportedKind "invalid_dimension")
      | Except.error e => Except.error e

def parseScalarDefault (arg : Option CoercionInput) (default : Rat) :
    Except CoercionError Rat :=
  match arg with
  | none => Except.ok default
  | some .missingArg | some .emptyCell => Except.ok default
  | some other => coerceToNumber other

def buildSequencePayload (count : Nat) (start step : Rat) : List Rat :=
  (List.range count).map (fun idx => start + step * Rat.ofInt (Int.ofNat idx))

def evalSequenceAdapter
    (rows : CoercionInput)
    (cols : Option CoercionInput := none)
    (start : Option CoercionInput := none)
    (step : Option CoercionInput := none) : Except CoercionError SequenceArray := do
  let parsedRows ← parseDimensionDefault rows 1
  let parsedCols ←
    match cols with
    | none => Except.ok 1
    | some c => parseDimensionDefault c 1
  let parsedStart ← parseScalarDefault start 1
  let parsedStep ← parseScalarDefault step 1
  let cellCount := parsedRows * parsedCols
  pure {
    rows := parsedRows
    cols := parsedCols
    payload := buildSequencePayload cellCount parsedStart parsedStep
  }

theorem evalSequenceAdapter_rows_only_defaults_cols_to_one :
    evalSequenceAdapter (.number 3) =
      Except.ok {
        rows := 3
        cols := 1
        payload := ([1, 2, 3] : List Rat)
      } := by
  native_decide

theorem evalSequenceAdapter_missing_rows_default_to_one :
    evalSequenceAdapter .missingArg (some (.number 3)) =
      Except.ok {
        rows := 1
        cols := 3
        payload := ([1, 2, 3] : List Rat)
      } := by
  native_decide

theorem evalSequenceAdapter_missing_cols_default_to_one :
    evalSequenceAdapter (.number 2) (some .missingArg) (some (.number 10)) =
      Except.ok {
        rows := 2
        cols := 1
        payload := ([10, 11] : List Rat)
      } := by
  native_decide

theorem evalSequenceAdapter_missing_start_default_to_one :
    evalSequenceAdapter (.number 2) (some (.number 3)) (some .missingArg) (some (.number 2)) =
      Except.ok {
        rows := 2
        cols := 3
        payload := ([1, 3, 5, 7, 9, 11] : List Rat)
      } := by
  native_decide

theorem evalSequenceAdapter_missing_step_default_to_one :
    evalSequenceAdapter (.number 2) (some (.number 3)) (some (.number 10)) (some .missingArg) =
      Except.ok {
        rows := 2
        cols := 3
        payload := ([10, 11, 12, 13, 14, 15] : List Rat)
      } := by
  native_decide

theorem evalSequenceAdapter_negative_step_payload :
    evalSequenceAdapter (.number 2) (some (.number 3)) (some (.number 10)) (some (.number (-2))) =
      Except.ok {
        rows := 2
        cols := 3
        payload := ([10, 8, 6, 4, 2, 0] : List Rat)
      } := by
  native_decide

theorem evalSequenceAdapter_zero_dimension_rejected :
    evalSequenceAdapter (.number 0) =
      Except.error (CoercionError.unsupportedKind "invalid_dimension") := by
  native_decide

theorem sequenceMeta_profiles :
    sequenceMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ sequenceMeta.fecDependencyProfile = FecDependencyProfile.none
    ∧ sequenceMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [sequenceMeta]

end OxFunc.Functions
