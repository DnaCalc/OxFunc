import OxFunc.FunctionCore
import OxFunc.RefResolverSeam

namespace OxFunc.Functions

open OxFunc

def opRangeRefMeta : FunctionMeta := {
  functionId := "FUNC.OP_RANGE_REF"
  arity := Arity.exact 2
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.workbookState
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.refOnly
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def opIntersectionRefMeta : FunctionMeta := { opRangeRefMeta with functionId := "FUNC.OP_INTERSECTION_REF" }
def opUnionRefMeta : FunctionMeta := { opRangeRefMeta with functionId := "FUNC.OP_UNION_REF" }
def opTrimRefLeadingMeta : FunctionMeta := { opRangeRefMeta with functionId := "FUNC.OP_TRIM_REF_LEADING", arity := Arity.exact 1 }
def opTrimRefTrailingMeta : FunctionMeta := { opRangeRefMeta with functionId := "FUNC.OP_TRIM_REF_TRAILING", arity := Arity.exact 1 }
def opTrimRefBothMeta : FunctionMeta := { opRangeRefMeta with functionId := "FUNC.OP_TRIM_REF_BOTH", arity := Arity.exact 1 }

structure ReferenceRect where
  row1 : Nat
  col1 : Nat
  row2 : Nat
  col2 : Nat
  deriving DecidableEq, Repr

def mergeRange (lhs rhs : ReferenceRect) : ReferenceRect := {
  row1 := Nat.min lhs.row1 rhs.row1
  col1 := Nat.min lhs.col1 rhs.col1
  row2 := Nat.max lhs.row2 rhs.row2
  col2 := Nat.max lhs.col2 rhs.col2
}

def intersectRange (lhs rhs : ReferenceRect) : Option ReferenceRect :=
  let row1 := Nat.max lhs.row1 rhs.row1
  let col1 := Nat.max lhs.col1 rhs.col1
  let row2 := Nat.min lhs.row2 rhs.row2
  let col2 := Nat.min lhs.col2 rhs.col2
  if row1 ≤ row2 && col1 ≤ col2 then
    some { row1 := row1, col1 := col1, row2 := row2, col2 := col2 }
  else
    none

def unionRefToken (lhs rhs : ReferenceToken) : ReferenceToken := {
  kind := .area
  target := "(" ++ lhs.target ++ "," ++ rhs.target ++ ")"
}

theorem operatorReferenceFamily_meta_profiles :
    opRangeRefMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ opIntersectionRefMeta.hostInteraction = HostInteractionClass.workbookState
    ∧ opUnionRefMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ opTrimRefLeadingMeta.arity = Arity.exact 1
    ∧ opTrimRefTrailingMeta.arity = Arity.exact 1
    ∧ opTrimRefBothMeta.arity = Arity.exact 1 := by
  simp [opRangeRefMeta, opIntersectionRefMeta, opUnionRefMeta, opTrimRefLeadingMeta,
    opTrimRefTrailingMeta, opTrimRefBothMeta]

theorem mergeRange_normalizes_bounds :
    mergeRange { row1 := 2, col1 := 2, row2 := 2, col2 := 2 }
        { row1 := 1, col1 := 1, row2 := 1, col2 := 1 } =
      { row1 := 1, col1 := 1, row2 := 2, col2 := 2 } := by
  rfl

theorem intersectRange_overlap :
    intersectRange
      { row1 := 1, col1 := 1, row2 := 3, col2 := 3 }
      { row1 := 2, col1 := 2, row2 := 4, col2 := 4 } =
      some { row1 := 2, col1 := 2, row2 := 3, col2 := 3 } := by
  native_decide

theorem intersectRange_disjoint :
    intersectRange
      { row1 := 1, col1 := 1, row2 := 2, col2 := 1 }
      { row1 := 1, col1 := 3, row2 := 2, col2 := 3 } = none := by
  native_decide

theorem unionRefToken_preserves_multi_area_shape :
    unionRefToken { kind := .area, target := "A1:A2" } { kind := .area, target := "G1:G2" } =
      { kind := .area, target := "(A1:A2,G1:G2)" } := by
  rfl

end OxFunc.Functions
