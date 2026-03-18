import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def countBlankMeta : FunctionMeta := {
  functionId := "FUNC.COUNTBLANK"
  arity := { min := 1, max := 255 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

theorem countBlankMeta_profiles :
    countBlankMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ countBlankMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [countBlankMeta]

end OxFunc.Functions
