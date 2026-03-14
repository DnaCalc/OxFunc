import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

structure ColumnCallerContext where
  row : Nat
  col : Nat
  deriving DecidableEq, Repr

structure ColumnRef where
  startRow : Nat
  startCol : Nat
  endRow : Nat
  endCol : Nat
  deriving DecidableEq, Repr

def columnMeta : FunctionMeta := {
  functionId := "FUNC.COLUMN"
  arity := { min := 0, max := 1 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.workbookState
  threadSafety := ThreadSafetyClass.hostSerialized
  argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.callerContext
  surfaceFecDependencyProfile := FecDependencyProfile.callerContext
}

def columnSeries (startCol endCol : Nat) : List Nat :=
  List.range (endCol + 1 - startCol) |>.map (fun i => startCol + i)

def evalColumn (caller : ColumnCallerContext) (arg : Option ColumnRef) : List Nat :=
  match arg with
  | none => [caller.col]
  | some ref => columnSeries ref.startCol ref.endCol

theorem evalColumn_omitted_uses_caller_context :
    evalColumn { row := 7, col := 3 } none = [3] := by
  rfl

theorem evalColumn_area_spills_horizontally :
    evalColumn { row := 1, col := 1 } (some { startRow := 2, startCol := 2, endRow := 3, endCol := 3 }) = [2, 3] := by
  rfl

theorem columnMeta_profiles :
    columnMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ columnMeta.hostInteraction = HostInteractionClass.workbookState
    ∧ columnMeta.fecDependencyProfile = FecDependencyProfile.callerContext
    ∧ columnMeta.surfaceFecDependencyProfile = FecDependencyProfile.callerContext := by
  simp [columnMeta]

end OxFunc.Functions
