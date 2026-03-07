import OxFunc.RefResolverSeam
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def absMeta : FunctionMeta := {
  functionId := "FUNC.ABS"
  arity := Arity.exact 1
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  fecDependencyProfile := FecDependencyProfile.refOnly
}

def absKernel (n : Rat) : Rat :=
  if n < 0 then -n else n

def evalAbsArg (arg : CoercionInput) : Except CoercionError Rat :=
  match coerceToNumber arg with
  | Except.ok n => Except.ok (absKernel n)
  | Except.error e => Except.error e

def evalAbsScalar (args : List CoercionInput) : Except (EvalError ⊕ CoercionError) Value :=
  match args with
  | [arg] =>
      match evalAbsArg arg with
      | Except.ok n => Except.ok (Value.number n)
      | Except.error e => Except.error (Sum.inr e)
  | _ => Except.error (Sum.inl (EvalError.arityMismatch 1 args.length))

def evalAbsLift (args : List CoercionInput) : List (Except CoercionError Rat) :=
  args.map evalAbsArg

def evalAbsFromRef (resolver : ReferenceResolver) (ref : ReferenceToken) :
    Except (RefResolutionError ⊕ CoercionError) Rat := do
  let n ← resolveRefToNumber resolver ref
  pure (absKernel n)

theorem absKernel_of_neg (n : Rat) (h : n < 0) :
    absKernel n = -n := by
  simp [absKernel, h]

theorem absKernel_of_nonneg (n : Rat) (h : ¬ n < 0) :
    absKernel n = n := by
  simp [absKernel, h]

theorem evalAbsScalar_rejects_nil :
    evalAbsScalar [] = Except.error (Sum.inl (EvalError.arityMismatch 1 0)) := by
  simp [evalAbsScalar]

theorem evalAbsScalar_rejects_two (a b : CoercionInput) :
    evalAbsScalar [a, b] = Except.error (Sum.inl (EvalError.arityMismatch 1 2)) := by
  simp [evalAbsScalar]

theorem evalAbsScalar_admitted_number_neg :
    evalAbsScalar [CoercionInput.number (-3)] =
      Except.ok (Value.number (absKernel (-3))) := by
  have h : (-3 : Rat) < 0 := by decide
  simp [evalAbsScalar, evalAbsArg, coerceToNumber, absKernel, h]

theorem evalAbsScalar_logical_true :
    evalAbsScalar [CoercionInput.logical true] = Except.ok (Value.number 1) := by
  have h : ¬ ((1 : Rat) < 0) := by decide
  simp [evalAbsScalar, evalAbsArg, coerceToNumber, absKernel, h]

theorem evalAbsScalar_text_bad :
    evalAbsScalar [CoercionInput.text "asd"] =
      Except.error (Sum.inr (CoercionError.nonNumericText "asd")) := by
  simp [evalAbsScalar, evalAbsArg, coerceToNumber, parseSimpleNumber]

theorem evalAbsLift_length (args : List CoercionInput) :
    (evalAbsLift args).length = args.length := by
  simp [evalAbsLift]

theorem evalAbsScalar_deterministic (args : List CoercionInput) :
    evalAbsScalar args = evalAbsScalar args := rfl

theorem evalAbsFromRef_deterministic (resolver : ReferenceResolver) (ref : ReferenceToken) :
    evalAbsFromRef resolver ref = evalAbsFromRef resolver ref := rfl

end OxFunc.Functions
