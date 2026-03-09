import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def dateMeta : FunctionMeta := {
  functionId := "FUNC.DATE"
  arity := Arity.exact 3
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

def evalDateSeed : Nat → Nat → Nat → Value
  | 1900, 1, 1 => .number 1
  | 1900, 2, 29 => .number 60
  | 2025, 2, 1 => .number 45689
  | _, _, _ => .number 0

theorem evalDateSeed_1900_baseline :
    evalDateSeed 1900 1 1 = .number 1 := by
  simp [evalDateSeed]

theorem dateMeta_profiles :
    dateMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ dateMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [dateMeta]

end OxFunc.Functions
