import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def textJoinMeta : FunctionMeta := {
  functionId := "FUNC.TEXTJOIN"
  arity := { min := 3, max := 255 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.textToText
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def evalTextJoinSeed (parts : List String) : String :=
  String.intercalate "," parts

theorem evalTextJoinSeed_basic :
    evalTextJoinSeed ["1", "2"] = "1,2" := by
  native_decide

theorem textJoinMeta_profiles :
    textJoinMeta.kernelSignatureClass = KernelSignatureClass.textToText
    ∧ textJoinMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [textJoinMeta]

end OxFunc.Functions
