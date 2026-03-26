import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def randbetweenMeta : FunctionMeta := {
  functionId := "FUNC.RANDBETWEEN"
  arity := { min := 2, max := 2 }
  determinism := DeterminismClass.pseudoRandom
  volatility := VolatilityClass.volatileFull
  hostInteraction := HostInteractionClass.applicationState
  threadSafety := ThreadSafetyClass.hostSerialized
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.randomProvider
  surfaceFecDependencyProfile := FecDependencyProfile.randomProvider
}

/-- Map a provider value in [0, range) to an integer in [bottom, top]. -/
def randbetweenKernel (providerValue : Nat) (bottom top : Int) : Int :=
  let range := top - bottom + 1
  if range ≤ 0 then bottom
  else bottom + (Int.ofNat providerValue) % range

theorem randbetween_equal_bounds :
    randbetweenKernel 42 5 5 = 5 := by rfl

theorem randbetween_simple_range :
    randbetweenKernel 0 1 10 = 1 := by rfl

theorem randbetween_deterministic (p : Nat) (b t : Int) :
    randbetweenKernel p b t = randbetweenKernel p b t := rfl

theorem randbetweenMeta_profiles :
    randbetweenMeta.determinism = DeterminismClass.pseudoRandom
    ∧ randbetweenMeta.volatility = VolatilityClass.volatileFull
    ∧ randbetweenMeta.fecDependencyProfile = FecDependencyProfile.randomProvider
    ∧ randbetweenMeta.surfaceFecDependencyProfile = FecDependencyProfile.randomProvider := by
  simp [randbetweenMeta]

end OxFunc.Functions
