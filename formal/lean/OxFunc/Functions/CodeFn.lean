import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def codeMeta : FunctionMeta := {
  functionId := "FUNC.CODE"
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

def codeFirst? : String → Option Char
  | "" => none
  | s => s.toList.head?

theorem codeFirst_seed_ab :
    codeFirst? "AB" = some 'A' := by
  rfl

theorem codeFirst_seed_empty :
    codeFirst? "" = none := by
  rfl

theorem codeMeta_profiles :
    codeMeta.kernelSignatureClass = KernelSignatureClass.custom
    ∧ codeMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [codeMeta]

end OxFunc.Functions
