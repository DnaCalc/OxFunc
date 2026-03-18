import OxFunc.FunctionCore

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
