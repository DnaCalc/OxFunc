import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def randArrayMeta : FunctionMeta := {
  functionId := "FUNC.RANDARRAY"
  arity := { min := 0, max := 5 }
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

structure RandArrayDims where
  rows : Nat
  cols : Nat
  deriving DecidableEq, Repr

def randArrayDefaultDims : RandArrayDims := { rows := 1, cols := 1 }

/-- Validate that dimensions are positive. -/
def randArrayValidateDims (dims : RandArrayDims) : Bool :=
  dims.rows ≥ 1 ∧ dims.cols ≥ 1

/-- Total number of cells in the output array. -/
def randArrayCellCount (dims : RandArrayDims) : Nat :=
  dims.rows * dims.cols

theorem randArray_default_dims_are_1x1 :
    randArrayDefaultDims = { rows := 1, cols := 1 } := by rfl

theorem randArray_default_cell_count :
    randArrayCellCount randArrayDefaultDims = 1 := by rfl

theorem randArray_dims_valid_default :
    randArrayValidateDims randArrayDefaultDims = true := by rfl

theorem randArrayMeta_profiles :
    randArrayMeta.determinism = DeterminismClass.pseudoRandom
    ∧ randArrayMeta.volatility = VolatilityClass.volatileFull
    ∧ randArrayMeta.fecDependencyProfile = FecDependencyProfile.randomProvider
    ∧ randArrayMeta.surfaceFecDependencyProfile = FecDependencyProfile.randomProvider := by
  simp [randArrayMeta]

end OxFunc.Functions
