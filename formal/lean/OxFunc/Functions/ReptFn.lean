import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def reptMeta : FunctionMeta := {
  functionId := "FUNC.REPT"
  arity := Arity.exact 2
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.none
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def reptCore (s : String) (count : Nat) : String :=
  String.join (List.replicate count s)

theorem reptSeed_ab_twice :
    reptCore "ab" 2 = "abab" := by
  native_decide

theorem reptSeed_zero :
    reptCore "ab" 0 = "" := by
  native_decide

theorem reptMeta_profiles :
    reptMeta.kernelSignatureClass = KernelSignatureClass.custom
    ∧ reptMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [reptMeta]

end OxFunc.Functions
