import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def concatMeta : FunctionMeta := {
  functionId := "FUNC.CONCAT"
  arity := { min := 1, max := 253 }
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

def concatenateMeta : FunctionMeta := {
  concatMeta with
  functionId := "FUNC.CONCATENATE"
  arity := { min := 1, max := 255 }
}

theorem concatFamily_profiles :
    concatMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ concatenateMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ concatMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ concatenateMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [concatMeta, concatenateMeta]

end OxFunc.Functions
