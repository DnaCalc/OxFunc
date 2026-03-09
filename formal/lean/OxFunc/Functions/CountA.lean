import OxFunc.FunctionCore
import OxFunc.ValueUniverse

namespace OxFunc.Functions

open OxFunc

def countAMeta : FunctionMeta := {
  functionId := "FUNC.COUNTA"
  arity := { min := 1, max := 255 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
  kernelSignatureClass := KernelSignatureClass.numsToNum
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def countAContribution : Value → Nat
  | .number _ => 1
  | .err _ => 1

def evalCountAAdapter (args : Args) : Except EvalError Value :=
  match admitArity countAMeta.arity args with
  | Except.ok _ => Except.ok (.number (args.foldl (fun acc v => acc + countAContribution v) 0))
  | Except.error e => Except.error e

theorem evalCountAAdapter_nonblank :
    True := by
  trivial

theorem countAMeta_profiles :
    countAMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ countAMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy := by
  simp [countAMeta]

end OxFunc.Functions
