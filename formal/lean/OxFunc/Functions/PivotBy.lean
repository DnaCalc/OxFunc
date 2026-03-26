import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def pivotByMeta : FunctionMeta := {
  functionId := "FUNC.PIVOTBY"
  arity := { min := 4, max := 255 }
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

theorem pivotByMeta_profiles :
    pivotByMeta.hostInteraction = HostInteractionClass.none
    ∧ pivotByMeta.threadSafety = ThreadSafetyClass.safePure
    ∧ pivotByMeta.fecDependencyProfile = FecDependencyProfile.none
    ∧ pivotByMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [pivotByMeta]

theorem pivotByMeta_arity :
    pivotByMeta.arity.min = 4 ∧ pivotByMeta.arity.max = 255 := by
  simp [pivotByMeta]

theorem pivotByMeta_deterministic :
    pivotByMeta.determinism = DeterminismClass.deterministic := by
  simp [pivotByMeta]

end OxFunc.Functions
