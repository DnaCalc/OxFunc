import OxFunc.FunctionCore
import OxFunc.CoercionPrimitives

namespace OxFunc.Functions

open OxFunc

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

structure SequenceShape where
  rows : Nat
  cols : Nat
  deriving DecidableEq, Repr

def ratToPosNat? (n : Rat) : Option Nat :=
  if n > 0 then
    if n.den = 1 then
      some n.num.natAbs
    else
      none
  else
    none

def parseDimension (arg : CoercionInput) : Except CoercionError Nat :=
  match coerceToNumber arg with
  | Except.ok n =>
      match ratToPosNat? n with
      | some d => Except.ok d
      | none => Except.error (CoercionError.unsupportedKind "invalid_dimension")
  | Except.error e => Except.error e

def evalSequenceAdapter
    (rows : CoercionInput)
    (cols : Option CoercionInput := none)
    (start : Option CoercionInput := none)
    (step : Option CoercionInput := none) : Except CoercionError SequenceShape := do
  let parsedRows ← parseDimension rows
  let parsedCols ←
    match cols with
    | none => Except.ok 1
    | some c => parseDimension c
  -- Start and step are admitted for coercion side-effects in this seed model.
  match start with
  | none => pure ()
  | some s =>
      match coerceToNumber s with
      | Except.ok _ => pure ()
      | Except.error e => throw e
  match step with
  | none => pure ()
  | some s =>
      match coerceToNumber s with
      | Except.ok _ => pure ()
      | Except.error e => throw e
  pure { rows := parsedRows, cols := parsedCols }

theorem parseDimension_bad_text :
    parseDimension (.text "bad") =
      Except.error (CoercionError.nonNumericText "bad") := by
  simp [parseDimension, coerceToNumber, parseSimpleNumber]

theorem evalSequenceAdapter_deterministic
    (rows : CoercionInput)
    (cols start step : Option CoercionInput) :
    evalSequenceAdapter rows cols start step =
      evalSequenceAdapter rows cols start step := rfl

theorem sequenceMeta_profiles :
    sequenceMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ sequenceMeta.fecDependencyProfile = FecDependencyProfile.none
    ∧ sequenceMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [sequenceMeta]

end OxFunc.Functions
