import OxFunc.FunctionCore
import OxFunc.Functions.Row

namespace OxFunc.Functions

open OxFunc

def columnsMeta : FunctionMeta := {
  functionId := "FUNC.COLUMNS"
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

def evalColumns (ref : RowRef) : Nat :=
  ref.endCol - ref.startCol + 1

theorem evalColumns_single_cell :
    evalColumns { startRow := 5, startCol := 3, endRow := 5, endCol := 3 } = 1 := by
  rfl

theorem evalColumns_area_returns_count :
    evalColumns { startRow := 2, startCol := 1, endRow := 6, endCol := 4 } = 4 := by
  rfl

theorem columnsMeta_profiles :
    columnsMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ columnsMeta.hostInteraction = HostInteractionClass.workbookState
    ∧ columnsMeta.fecDependencyProfile = FecDependencyProfile.refOnly
    ∧ columnsMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [columnsMeta]

end OxFunc.Functions
