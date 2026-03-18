import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def romanMeta : FunctionMeta := {
  functionId := "FUNC.ROMAN"
  arity := { min := 1, max := 2 }
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

theorem romanMeta_profiles :
    romanMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ romanMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ romanMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [romanMeta]

theorem romanClassic_seed_499 :
    "CDXCIX".length = 6 := by
  native_decide

end OxFunc.Functions
