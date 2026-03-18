import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def upperMeta : FunctionMeta := {
  functionId := "FUNC.UPPER"
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

theorem upperSeed_true_textified :
    "true".toUpper = "TRUE" := by
  native_decide

theorem upperMeta_profiles :
    upperMeta.kernelSignatureClass = KernelSignatureClass.textToText
    ∧ upperMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [upperMeta]

end OxFunc.Functions
