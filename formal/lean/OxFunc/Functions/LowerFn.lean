import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def lowerMeta : FunctionMeta := {
  functionId := "FUNC.LOWER"
  arity := Arity.exact 1
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.none
  kernelSignatureClass := KernelSignatureClass.textToText
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

theorem lowerSeed_true_textified :
    "TRUE".toLower = "true" := by
  native_decide

theorem lowerMeta_profiles :
    lowerMeta.kernelSignatureClass = KernelSignatureClass.textToText
    ∧ lowerMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [lowerMeta]

end OxFunc.Functions
