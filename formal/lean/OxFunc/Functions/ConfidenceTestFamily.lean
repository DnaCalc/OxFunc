import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def confidenceTMeta : FunctionMeta := {
  functionId := "FUNC.CONFIDENCE.T"
  arity := Arity.exact 3
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.none
}

def zTestMeta : FunctionMeta := {
  functionId := "FUNC.Z.TEST"
  arity := { min := 2, max := 3 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.refOnly
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

theorem confidenceTest_profiles :
    confidenceTMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ zTestMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ zTestMeta.arity = { min := 2, max := 3 } := by
  simp [confidenceTMeta, zTestMeta]

end OxFunc.Functions
