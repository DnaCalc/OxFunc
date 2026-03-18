import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def isEvenMeta : FunctionMeta := {
  functionId := "FUNC.ISEVEN"
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

theorem isEvenMeta_profiles :
    isEvenMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ isEvenMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [isEvenMeta]

end OxFunc.Functions
