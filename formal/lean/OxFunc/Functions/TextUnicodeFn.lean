import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def unicharMeta : FunctionMeta := {
  functionId := "FUNC.UNICHAR"
  arity := Arity.exact 1
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

def unicodeMeta : FunctionMeta := { unicharMeta with functionId := "FUNC.UNICODE" }

theorem textUnicodeMeta_profiles :
    unicharMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ unicodeMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [unicharMeta, unicodeMeta]

end OxFunc.Functions
