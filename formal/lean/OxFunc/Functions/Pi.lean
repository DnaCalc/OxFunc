import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def piMeta : FunctionMeta := {
  functionId := "FUNC.PI"
  arity := Arity.exact 0
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  fecDependencyProfile := FecDependencyProfile.none
}

def piConst : Rat := (884279719003555 : Rat) / (281474976710656 : Rat)

def evalPiKernel : Rat := piConst

def evalPi (args : Args) : Except EvalError Value :=
  match admitArity piMeta.arity args with
  | Except.ok _ => Except.ok (Value.number evalPiKernel)
  | Except.error e => Except.error e

theorem evalPi_admitted_result :
    evalPi [] = Except.ok (Value.number piConst) := by
  simp [evalPi, evalPiKernel, piMeta, admitArity, Arity.exact, Arity.accepts]

theorem evalPi_rejects_cons (x : Value) (xs : Args) :
    evalPi (x :: xs) =
      Except.error (EvalError.arityMismatch 0 (List.length (x :: xs))) := by
  simp [evalPi, piMeta, admitArity, Arity.exact, Arity.accepts]

theorem evalPi_deterministic (args : Args) :
    evalPi args = evalPi args := rfl

end OxFunc.Functions
