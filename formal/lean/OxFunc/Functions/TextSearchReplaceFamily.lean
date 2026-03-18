import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def properMeta : FunctionMeta := {
  functionId := "FUNC.PROPER"
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

def substituteMeta : FunctionMeta := {
  functionId := "FUNC.SUBSTITUTE"
  arity := { min := 3, max := 4 }
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

def replaceMeta : FunctionMeta := {
  functionId := "FUNC.REPLACE"
  arity := Arity.exact 4
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

def findMeta : FunctionMeta := {
  functionId := "FUNC.FIND"
  arity := { min := 2, max := 3 }
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

def searchMeta : FunctionMeta := {
  functionId := "FUNC.SEARCH"
  arity := { min := 2, max := 3 }
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

def asciiFoldUnit (u : Nat) : Nat :=
  if 65 <= u ∧ u <= 90 then u + 32 else u

def searchWildcardUnits : List Nat := [42, 63, 126]

theorem textSearchReplace_profiles :
    properMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ substituteMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ replaceMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ findMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ searchMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [properMeta, substituteMeta, replaceMeta, findMeta, searchMeta]

theorem textSearchReplace_surface_profiles :
    properMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ substituteMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ replaceMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ findMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ searchMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [properMeta, substituteMeta, replaceMeta, findMeta, searchMeta]

theorem asciiFoldUnit_uppercaseA : asciiFoldUnit 65 = 97 := by
  simp [asciiFoldUnit]

theorem asciiFoldUnit_digitStable : asciiFoldUnit 52 = 52 := by
  simp [asciiFoldUnit]

theorem searchWildcardUnits_contains_ascii_meta :
    searchWildcardUnits = [42, 63, 126] := by
  rfl

end OxFunc.Functions
