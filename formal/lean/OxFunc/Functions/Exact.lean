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

def evalExactCore (lhs rhs : String) : Bool := lhs = rhs

theorem evalExactSeed_case_sensitive :
    evalExactCore "Abc" "abc" = false := by
  simp [evalExactCore]

theorem evalExactSeed_numeric_textification :
    evalExactCore "1" "1" = true := by
  simp [evalExactCore]

theorem evalExactSeed_logical_textification :
    evalExactCore "TRUE" "TRUE" = true := by
  simp [evalExactCore]

theorem evalExactSeed_blank_equals_empty_string :
    evalExactCore "" "" = true := by
  simp [evalExactCore]

theorem evalExactSeed_precomposed_vs_combining_distinct :
    evalExactCore "é" ("e" ++ "\u0301") = false := by
  decide

theorem evalExactSeed_identical_emoji_equal :
    evalExactCore "😀" "😀" = true := by
  decide

theorem exactMeta_profiles :
    exactMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ exactMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [exactMeta]

end OxFunc.Functions
