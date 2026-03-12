import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

private instance instDecidableEqExceptRand [DecidableEq ε] [DecidableEq α] :
    DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def randMeta : FunctionMeta := {
  functionId := "FUNC.RAND"
  arity := Arity.exact 0
  determinism := DeterminismClass.pseudoRandom
  volatility := VolatilityClass.volatileFull
  hostInteraction := HostInteractionClass.applicationState
  threadSafety := ThreadSafetyClass.hostSerialized
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.none
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.randomProvider
  surfaceFecDependencyProfile := FecDependencyProfile.randomProvider
}

structure RandomProvider where
  unitValue : Rat
  deriving DecidableEq, Repr

def evalRandAdapter (provider : RandomProvider) : Except String Value :=
  if 0 ≤ provider.unitValue ∧ provider.unitValue < 1 then
    Except.ok (.number provider.unitValue)
  else
    Except.error "out_of_range"

theorem evalRandAdapter_in_range :
    evalRandAdapter { unitValue := 0 } = .ok (.number 0) := by
  native_decide

theorem evalRandAdapter_upper_bound_exclusive :
    evalRandAdapter { unitValue := 1 } = .error "out_of_range" := by
  native_decide

theorem evalRandAdapter_mid_range_value :
    evalRandAdapter { unitValue := 3 / 4 } = .ok (.number (3 / 4 : Rat)) := by
  native_decide

theorem randMeta_profiles :
    randMeta.determinism = DeterminismClass.pseudoRandom
    ∧ randMeta.fecDependencyProfile = FecDependencyProfile.randomProvider := by
  simp [randMeta]

end OxFunc.Functions
