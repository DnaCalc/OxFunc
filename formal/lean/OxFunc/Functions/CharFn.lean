import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def charMeta : FunctionMeta := {
  functionId := "FUNC.CHAR"
  arity := Arity.exact 1
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

def charDomainOk (n : Int) : Bool :=
  1 <= n && n <= 255

theorem charDomain_seed_trunc_65_ok :
    charDomainOk 65 = true := by
  native_decide

theorem charDomain_seed_zero_rejected :
    charDomainOk 0 = false := by
  native_decide

theorem charMeta_profiles :
    charMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ charMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [charMeta]

end OxFunc.Functions
