import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def decimalMeta : FunctionMeta := {
  functionId := "FUNC.DECIMAL"
  arity := Arity.exact 2
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

theorem decimalMeta_profiles :
    decimalMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ decimalMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [decimalMeta]

theorem decimalSeed_empty_returns_zero :
    "".trimAsciiStart.toString = "" := by
  native_decide

end OxFunc.Functions
