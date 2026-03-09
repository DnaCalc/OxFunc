import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def exactMeta : FunctionMeta := {
  functionId := "FUNC.EXACT"
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

def evalExactSeed (lhs rhs : String) : Bool := lhs = rhs

theorem evalExactSeed_case_sensitive :
    evalExactSeed "Abc" "abc" = false := by
  simp [evalExactSeed]

theorem exactMeta_profiles :
    exactMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ exactMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [exactMeta]

end OxFunc.Functions
