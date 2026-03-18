import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def switchMeta : FunctionMeta := {
  functionId := "FUNC.SWITCH"
  arity := { min := 3, max := 255 }
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

theorem switchMeta_profile :
    switchMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ switchMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [switchMeta]

end OxFunc.Functions
