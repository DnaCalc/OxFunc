import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def ifNaMeta : FunctionMeta := {
  functionId := "FUNC.IFNA"
  arity := Arity.exact 2
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

theorem ifNaMeta_profiles :
    ifNaMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ ifNaMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [ifNaMeta]

end OxFunc.Functions
