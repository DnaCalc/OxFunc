import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

structure RowCallerContext where
  row : Nat
  col : Nat
  deriving DecidableEq, Repr

structure RowRef where
  startRow : Nat
  startCol : Nat
  endRow : Nat
  endCol : Nat
  deriving DecidableEq, Repr

def rowMeta : FunctionMeta := {
  functionId := "FUNC.ROW"
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

def rowSeries (startRow endRow : Nat) : List Nat :=
  List.range (endRow + 1 - startRow) |>.map (fun i => startRow + i)

def evalRow (caller : RowCallerContext) (arg : Option RowRef) : List Nat :=
  match arg with
  | none => [caller.row]
  | some ref => rowSeries ref.startRow ref.endRow

theorem evalRow_omitted_uses_caller_context :
    evalRow { row := 7, col := 3 } none = [7] := by
  rfl

theorem evalRow_area_spills_vertically :
    evalRow { row := 1, col := 1 } (some { startRow := 2, startCol := 2, endRow := 3, endCol := 3 }) = [2, 3] := by
  rfl

theorem rowMeta_profiles :
    rowMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ rowMeta.hostInteraction = HostInteractionClass.workbookState
    ∧ rowMeta.fecDependencyProfile = FecDependencyProfile.callerContext
    ∧ rowMeta.surfaceFecDependencyProfile = FecDependencyProfile.callerContext := by
  simp [rowMeta]

end OxFunc.Functions
