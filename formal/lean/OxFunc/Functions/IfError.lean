import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def ifErrorMeta : FunctionMeta := {
  functionId := "FUNC.IFERROR"
  arity := Arity.exact 2
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def evalIfErrorAdapter (primary fallback : Value) : Except EvalError Value :=
  match primary with
  | .err _ => Except.ok fallback
  | _ => Except.ok primary

theorem evalIfErrorAdapter_primary_ok :
    evalIfErrorAdapter (.number 2) (.number 4) = Except.ok (.number 2) := by
  simp [evalIfErrorAdapter]

theorem evalIfErrorAdapter_error_fallback :
    evalIfErrorAdapter (.err (.arityMismatch 1 0)) (.number 4) = Except.ok (.number 4) := by
  simp [evalIfErrorAdapter]

theorem ifErrorMeta_profiles :
    ifErrorMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ ifErrorMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [ifErrorMeta]

end OxFunc.Functions
