import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def trimRangeMeta : FunctionMeta := {
  functionId := "FUNC.TRIMRANGE"
  arity := { min := 1, max := 4 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.workbookState
  threadSafety := ThreadSafetyClass.hostSerialized
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

inductive TrimType where
  | none
  | trailing
  | leading
  | both
  deriving DecidableEq, Repr

structure TrimRangeInput where
  rows : Nat
  cols : Nat
  blankLeadingRows : Nat
  blankTrailingRows : Nat
  deriving DecidableEq, Repr

/-- Compute remaining rows after trimming blank rows. -/
def trimRows (input : TrimRangeInput) (trimType : TrimType) : Nat :=
  match trimType with
  | .none => input.rows
  | .trailing => input.rows - input.blankTrailingRows
  | .leading => input.rows - input.blankLeadingRows
  | .both => input.rows - input.blankLeadingRows - input.blankTrailingRows

theorem trimRange_none_preserves_input :
    trimRows { rows := 10, cols := 5, blankLeadingRows := 2, blankTrailingRows := 3 } TrimType.none = 10 := by
  rfl

theorem trimRange_trailing_removes_blanks :
    trimRows { rows := 10, cols := 5, blankLeadingRows := 0, blankTrailingRows := 3 } TrimType.trailing = 7 := by
  rfl

theorem trimRange_both_removes_all_blanks :
    trimRows { rows := 10, cols := 5, blankLeadingRows := 2, blankTrailingRows := 3 } TrimType.both = 5 := by
  rfl

theorem trimRangeMeta_profiles :
    trimRangeMeta.hostInteraction = HostInteractionClass.workbookState
    ∧ trimRangeMeta.threadSafety = ThreadSafetyClass.hostSerialized
    ∧ trimRangeMeta.fecDependencyProfile = FecDependencyProfile.none
    ∧ trimRangeMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [trimRangeMeta]

end OxFunc.Functions
