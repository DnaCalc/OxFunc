import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def isNumberMeta : FunctionMeta := {
  functionId := "FUNC.ISNUMBER"
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

def isNumberKernel : CoercionInput → Bool
  | .number _ => true
  | _ => false

def evalIsNumberAdapter (args : List CoercionInput) : Except EvalError Value :=
  match args with
  | [arg] =>
      let b := isNumberKernel arg
      Except.ok (Value.number (if b then 1 else 0))
  | _ => Except.error (EvalError.arityMismatch 1 args.length)

theorem isNumberKernel_number_true (n : Rat) :
    isNumberKernel (.number n) = true := by
  simp [isNumberKernel]

theorem isNumberKernel_text_false (s : String) :
    isNumberKernel (.text s) = false := by
  simp [isNumberKernel]

theorem evalIsNumberAdapter_error_false_as_zero :
    evalIsNumberAdapter [.error .na] = Except.ok (Value.number 0) := by
  simp [evalIsNumberAdapter, isNumberKernel]

theorem isNumberMeta_profiles :
    isNumberMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ isNumberMeta.fecDependencyProfile = FecDependencyProfile.none
    ∧ isNumberMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [isNumberMeta]

end OxFunc.Functions
