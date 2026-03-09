import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def hstackMeta : FunctionMeta := {
  functionId := "FUNC.HSTACK"
  arity := { min := 1, max := 255 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def hstackShapeSeed (shapes : List (Nat × Nat)) : Nat × Nat :=
  (shapes.foldl (fun acc shape => max acc shape.1) 1,
   shapes.foldl (fun acc shape => acc + shape.2) 0)

theorem hstackShapeSeed_scalar_plus_array :
    hstackShapeSeed [(2, 2), (1, 1)] = (2, 3) := by
  simp [hstackShapeSeed]

theorem hstackMeta_profiles :
    hstackMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ hstackMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [hstackMeta]

end OxFunc.Functions
