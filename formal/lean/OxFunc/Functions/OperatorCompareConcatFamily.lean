import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def opConcatMeta : FunctionMeta := {
  functionId := "FUNC.OP_CONCAT"
  arity := Arity.exact 2
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.textToText
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def opCompareMeta (id : String) : FunctionMeta := {
  functionId := id
  arity := Arity.exact 2
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

def opEqualMeta : FunctionMeta := opCompareMeta "FUNC.OP_EQUAL"
def opNotEqualMeta : FunctionMeta := opCompareMeta "FUNC.OP_NOT_EQUAL"
def opLessThanMeta : FunctionMeta := opCompareMeta "FUNC.OP_LESS_THAN"
def opLessEqualMeta : FunctionMeta := opCompareMeta "FUNC.OP_LESS_EQUAL"
def opGreaterThanMeta : FunctionMeta := opCompareMeta "FUNC.OP_GREATER_THAN"
def opGreaterEqualMeta : FunctionMeta := opCompareMeta "FUNC.OP_GREATER_EQUAL"

inductive CompareValue where
  | blank
  | number (n : Rat)
  | text (s : String)
  | logical (b : Bool)
  deriving DecidableEq, Repr

inductive CompareOp where
  | eq
  | ne
  | lt
  | le
  | gt
  | ge
  deriving DecidableEq, Repr

def typeRank : CompareValue → Nat
  | .blank => 0
  | .number _ => 0
  | .text _ => 1
  | .logical _ => 2

def normalizeBlankPair : CompareValue → CompareValue → CompareValue × CompareValue
  | .blank, .blank => (.blank, .blank)
  | .blank, .number n => (.number 0, .number n)
  | .number n, .blank => (.number n, .number 0)
  | .blank, .text s => (.text "", .text s)
  | .text s, .blank => (.text s, .text "")
  | .blank, .logical b => (.logical false, .logical b)
  | .logical b, .blank => (.logical b, .logical false)
  | lhs, rhs => (lhs, rhs)

def compareOrdering : CompareValue → CompareValue → Ordering
  | lhs, rhs =>
      let (lhs', rhs') := normalizeBlankPair lhs rhs
      match lhs', rhs' with
      | .blank, .blank => .eq
      | .number l, .number r =>
          if l < r then .lt else if l = r then .eq else .gt
      | .text l, .text r =>
          if l < r then .lt else if l = r then .eq else .gt
      | .logical l, .logical r =>
          if l = r then .eq else if l = false then .lt else .gt
      | l, r =>
          if typeRank l < typeRank r then .lt
          else if typeRank l = typeRank r then .eq
          else .gt

def compareValues (op : CompareOp) (lhs rhs : CompareValue) : Bool :=
  match compareOrdering lhs rhs, op with
  | .eq, .eq => true
  | .eq, .le => true
  | .eq, .ge => true
  | .eq, .ne => false
  | .eq, .lt => false
  | .eq, .gt => false
  | .lt, .eq => false
  | .lt, .ne => true
  | .lt, .lt => true
  | .lt, .le => true
  | .lt, .gt => false
  | .lt, .ge => false
  | .gt, .eq => false
  | .gt, .ne => true
  | .gt, .lt => false
  | .gt, .le => false
  | .gt, .gt => true
  | .gt, .ge => true

def opConcatKernel (lhs rhs : String) : String := lhs ++ rhs

theorem operatorCompareConcatFamily_meta_profiles :
    opConcatMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ opEqualMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ opNotEqualMeta.threadSafety = ThreadSafetyClass.safePure
    ∧ opLessThanMeta.hostInteraction = HostInteractionClass.none
    ∧ opLessEqualMeta.volatility = VolatilityClass.nonvolatile
    ∧ opGreaterThanMeta.determinism = DeterminismClass.deterministic
    ∧ opGreaterEqualMeta.kernelSignatureClass = KernelSignatureClass.custom := by
  simp [opConcatMeta, opCompareMeta, opEqualMeta, opNotEqualMeta, opLessThanMeta,
    opLessEqualMeta, opGreaterThanMeta, opGreaterEqualMeta]

theorem opConcatKernel_appends :
    opConcatKernel "a" "1" = "a1" := rfl

theorem compare_blank_equals_zero :
    compareValues .eq .blank (.number 0) = true := by
  native_decide

theorem compare_text_greater_than_number :
    compareValues .gt (.text "10") (.number 2) = true := by
  native_decide

theorem compare_false_less_than_true :
    compareValues .lt (.logical false) (.logical true) = true := by
  native_decide

end OxFunc.Functions
