import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def opAddMeta : FunctionMeta := {
  functionId := "FUNC.OP_ADD"
  arity := Arity.exact 2
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.numsToNum
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def opAddKernel (lhs rhs : Rat) : Rat := lhs + rhs

def evalOpAddAdapter (args : List CoercionInput) : Except (EvalError ⊕ CoercionError) Value :=
  match args with
  | [lhs, rhs] =>
      match coerceToNumber lhs, coerceToNumber rhs with
      | Except.ok l, Except.ok r => Except.ok (Value.number (opAddKernel l r))
      | Except.error e, _ => Except.error (Sum.inr e)
      | _, Except.error e => Except.error (Sum.inr e)
  | _ => Except.error (Sum.inl (EvalError.arityMismatch 2 args.length))

theorem opAddKernel_deterministic (a b : Rat) :
    opAddKernel a b = opAddKernel a b := rfl

theorem evalOpAddAdapter_two_numbers :
    evalOpAddAdapter [CoercionInput.number 2, CoercionInput.number 3] =
      Except.ok (Value.number (opAddKernel 2 3)) := by
  simp [evalOpAddAdapter, opAddKernel, coerceToNumber]

theorem opAddMeta_profiles :
    opAddMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ opAddMeta.kernelSignatureClass = KernelSignatureClass.numsToNum
    ∧ opAddMeta.fecDependencyProfile = FecDependencyProfile.none
    ∧ opAddMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [opAddMeta]

end OxFunc.Functions
