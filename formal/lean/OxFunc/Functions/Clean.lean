import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def cleanMeta : FunctionMeta := {
  functionId := "FUNC.CLEAN"
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

def cleanSeed : String → String
  | "A\tB" => "AB"
  | s => s

theorem cleanSeed_removes_seed_tab :
    cleanSeed "A\tB" = "AB" := by
  simp [cleanSeed]

theorem cleanMeta_profiles :
    cleanMeta.kernelSignatureClass = KernelSignatureClass.textToText
    ∧ cleanMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [cleanMeta]

end OxFunc.Functions
