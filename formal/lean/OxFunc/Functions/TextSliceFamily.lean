import OxFunc.FunctionCore
import OxFunc.Functions.Date
import OxFunc.TextScalarBroadcast

namespace OxFunc.Functions

open OxFunc

def lenMeta : FunctionMeta := {
  functionId := "FUNC.LEN"
  arity := Arity.exact 1
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.none
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def leftMeta : FunctionMeta := {
  functionId := "FUNC.LEFT"
  arity := { min := 1, max := 2 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.none
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def rightMeta : FunctionMeta := {
  functionId := "FUNC.RIGHT"
  arity := { min := 1, max := 2 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.none
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def midMeta : FunctionMeta := {
  functionId := "FUNC.MID"
  arity := Arity.exact 3
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.none
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def lenUtf16Units (units : List UInt16) : Nat :=
  units.length

def leftUtf16 (units : List UInt16) (count : Nat) : List UInt16 :=
  units.take count

def rightUtf16 (units : List UInt16) (count : Nat) : List UInt16 :=
  let takeCount := Nat.min count units.length
  units.drop (units.length - takeCount)

def midUtf16 (units : List UInt16) (startOneBased count : Nat) : List UInt16 :=
  if startOneBased = 0 then
    []
  else
    (units.drop (startOneBased - 1)).take count

def textHighSurrogate (unit : UInt16) : Bool :=
  decide (0xD800 ≤ unit.toNat ∧ unit.toNat ≤ 0xDBFF)

def textLowSurrogate (unit : UInt16) : Bool :=
  decide (0xDC00 ≤ unit.toNat ∧ unit.toNat ≤ 0xDFFF)

def textCharacterChunks : List UInt16 → List (List UInt16)
  | [] => []
  | a :: rest =>
    match rest with
    | b :: tail =>
      if textHighSurrogate a && textLowSurrogate b then [a,b] :: textCharacterChunks tail
      else [a] :: textCharacterChunks (b :: tail)
    | [] => [[a]]

def textForwardUnits (units : List UInt16) : List UInt16 :=
  if (units.getLast?.map textHighSurrogate).getD false then units.dropLast else units

def lenCharactersV2 (units : List UInt16) : Nat :=
  (textCharacterChunks (textForwardUnits units)).length

def leftCharactersV2 (units : List UInt16) (count : Nat) : List UInt16 :=
  if (units.getLast?.map textHighSurrogate).getD false && count > lenCharactersV2 units
  then units.take count
  else ((textCharacterChunks units).take count).flatten

def rightCharactersV2 (units : List UInt16) (count : Nat) : List UInt16 :=
  let chars := textCharacterChunks units
  (chars.drop (chars.length - count)).flatten

def midCharactersV2 (units : List UInt16) (start count : Nat) : List UInt16 :=
  ((textCharacterChunks (textForwardUnits units)).drop (start - 1) |>.take count).flatten

def textLeftRightCount (number : Rat) : Except WorksheetErrorCode Nat :=
  if number < 0 then .error .value else .ok (dateIntegerFloor number).toNat

def textMidInteger (number : Rat) (start : Bool) : Except WorksheetErrorCode Nat :=
  if number < (if start then 1 else 0) ∨ number ≥ 2147483648 then .error .value
  else .ok (Int.ediv number.num number.den).toNat

def textExplicitNumericSlot : CoercionInput → Except CoercionError Rat
  | .missingArg => .ok 0
  | .emptyCell => .ok 0
  | value => coerceToNumber value

theorem text_v2_distinct_surrogate_paths :
    lenCharactersV2 [0xD800] = 0
    ∧ leftCharactersV2 [0xD800] 1 = [0xD800]
    ∧ leftCharactersV2 [0xD800,0xDC00,0xD800] 2 = [0xD800,0xDC00]
    ∧ leftCharactersV2 [0xD800,0xDC00,0xD800] 3 = [0xD800,0xDC00,0xD800]
    ∧ leftCharactersV2 [0xD800,0xDC00,0xD800,0xDC00,0xD800] 3 = [0xD800,0xDC00,0xD800]
    ∧ rightCharactersV2 [0xD800,0xDC00,0xD800] 1 = [0xD800]
    ∧ midCharactersV2 [0xD800,0xDC00,0xD800] 2 1 = []
    ∧ midUtf16 [0xD800,0xDC00,0xD800] 2 1 = [0xDC00] := by
  native_decide

theorem text_numeric_admission_and_rounding :
    textLeftRightCount (1 - 2049/8589934592) = .ok 1
    ∧ textMidInteger (1 - 2049/8589934592) false = .ok 0
    ∧ textLeftRightCount (-1/2) = .error .value
    ∧ textMidInteger 2147483648 false = .error .value
    ∧ textExplicitNumericSlot .missingArg = .ok 0 := by
  native_decide

theorem lenUtf16_seed_surrogate_pair :
    lenUtf16Units [0xD83D, 0xDE00] = 2 := by
  native_decide

theorem leftUtf16_seed_first_code_unit :
    leftUtf16 [0xD83D, 0xDE00] 1 = [0xD83D] := by
  native_decide

theorem rightUtf16_seed_last_code_unit :
    rightUtf16 [0xD83D, 0xDE00] 1 = [0xDE00] := by
  native_decide

theorem midUtf16_seed_one_based_slice :
    midUtf16 [65, 66, 67, 68] 2 1 = [66] := by
  native_decide

theorem lenMeta_profile :
    lenMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  rfl

theorem leftMeta_surface_profile :
    leftMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  rfl

theorem rightMeta_surface_profile :
    rightMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  rfl

theorem midMeta_surface_profile :
    midMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  rfl

end OxFunc.Functions
