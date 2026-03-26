import OxFunc.FunctionCore
import OxFunc.Functions.Row

namespace OxFunc.Functions

open OxFunc

def rowsMeta : FunctionMeta := {
  functionId := "FUNC.ROWS"
  arity := { min := 1, max := 1 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.workbookState
  threadSafety := ThreadSafetyClass.hostSerialized
  argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.refOnly
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def evalRows (ref : RowRef) : Nat :=
  ref.endRow - ref.startRow + 1

theorem evalRows_single_cell :
    evalRows { startRow := 5, startCol := 3, endRow := 5, endCol := 3 } = 1 := by
  rfl

theorem evalRows_area_returns_count :
    evalRows { startRow := 2, startCol := 1, endRow := 6, endCol := 4 } = 5 := by
  rfl

theorem rowsMeta_profiles :
    rowsMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ rowsMeta.hostInteraction = HostInteractionClass.workbookState
    ∧ rowsMeta.fecDependencyProfile = FecDependencyProfile.refOnly
    ∧ rowsMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [rowsMeta]

end OxFunc.Functions
