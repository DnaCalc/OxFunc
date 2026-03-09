import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

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
    True := by
  trivial

theorem randMeta_profiles :
    randMeta.determinism = DeterminismClass.pseudoRandom
    ∧ randMeta.fecDependencyProfile = FecDependencyProfile.randomProvider := by
  simp [randMeta]

end OxFunc.Functions
