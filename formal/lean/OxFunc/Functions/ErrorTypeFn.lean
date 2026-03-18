import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def errorTypeMeta : FunctionMeta := {
  functionId := "FUNC.ERROR.TYPE"
  arity := Arity.exact 1
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

theorem errorTypeMeta_profiles :
    errorTypeMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ errorTypeMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [errorTypeMeta]

end OxFunc.Functions
