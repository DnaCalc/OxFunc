import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def nowMeta : FunctionMeta := {
  functionId := "FUNC.NOW"
  arity := Arity.exact 0
  determinism := DeterminismClass.timeDependent
  volatility := VolatilityClass.volatileFull
  hostInteraction := HostInteractionClass.applicationState
  threadSafety := ThreadSafetyClass.hostSerialized
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.none
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.timeProvider
  surfaceFecDependencyProfile := FecDependencyProfile.timeProvider
}

structure TimeProvider where
  nowSerial : Rat
  deriving DecidableEq, Repr

def evalNowAdapter (provider : TimeProvider) (args : Args) : Except EvalError Value :=
  match admitArity nowMeta.arity args with
  | Except.ok _ => Except.ok (.number provider.nowSerial)
  | Except.error e => Except.error e

theorem evalNowAdapter_returns_provider_value (serial : Rat) :
    evalNowAdapter { nowSerial := serial } [] = Except.ok (.number serial) := by
  simp [evalNowAdapter, nowMeta, admitArity, Arity.exact, Arity.accepts]

theorem evalNowAdapter_rejects_args (x : Value) (xs : Args) :
    evalNowAdapter { nowSerial := 42 } (x :: xs) =
      Except.error (EvalError.arityMismatch 0 (List.length (x :: xs))) := by
  simp [evalNowAdapter, nowMeta, admitArity, Arity.exact, Arity.accepts]

theorem nowMeta_profiles :
    nowMeta.determinism = DeterminismClass.timeDependent
    ∧ nowMeta.volatility = VolatilityClass.volatileFull
    ∧ nowMeta.fecDependencyProfile = FecDependencyProfile.timeProvider
    ∧ nowMeta.surfaceFecDependencyProfile = FecDependencyProfile.timeProvider := by
  simp [nowMeta]

end OxFunc.Functions
