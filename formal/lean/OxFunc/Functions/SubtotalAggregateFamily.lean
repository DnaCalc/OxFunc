import OxFunc.FunctionCore
import OxFunc.HostInfoSeam

namespace OxFunc.Functions

open OxFunc

def subtotalMeta : FunctionMeta := {
  functionId := "FUNC.SUBTOTAL"
  arity := { min := 2, max := 255 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.workbookState
  threadSafety := ThreadSafetyClass.hostSerialized
  argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.composite
  surfaceFecDependencyProfile := FecDependencyProfile.composite
}

def aggregateMeta : FunctionMeta := {
  functionId := "FUNC.AGGREGATE"
  arity := { min := 3, max := 255 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.workbookState
  threadSafety := ThreadSafetyClass.hostSerialized
  argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.composite
  surfaceFecDependencyProfile := FecDependencyProfile.composite
}

structure AggregateFilterRules where
  ignoreNestedAggregate : Bool
  ignoreManualHiddenRows : Bool
  ignoreFilteredRows : Bool
  ignoreErrors : Bool
  deriving DecidableEq, Repr

def subtotalRules (fnNum : Int) : Option AggregateFilterRules :=
  match fnNum with
  | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 =>
      some { ignoreNestedAggregate := true, ignoreManualHiddenRows := false, ignoreFilteredRows := true, ignoreErrors := false }
  | 101 | 102 | 103 | 104 | 105 | 106 | 107 | 108 | 109 | 110 | 111 =>
      some { ignoreNestedAggregate := true, ignoreManualHiddenRows := true, ignoreFilteredRows := true, ignoreErrors := false }
  | _ => none

def aggregateRules (opt : Int) : Option AggregateFilterRules :=
  match opt with
  | 0 => some { ignoreNestedAggregate := true, ignoreManualHiddenRows := false, ignoreFilteredRows := false, ignoreErrors := false }
  | 1 => some { ignoreNestedAggregate := true, ignoreManualHiddenRows := true, ignoreFilteredRows := true, ignoreErrors := false }
  | 2 => some { ignoreNestedAggregate := true, ignoreManualHiddenRows := false, ignoreFilteredRows := false, ignoreErrors := true }
  | 3 => some { ignoreNestedAggregate := true, ignoreManualHiddenRows := true, ignoreFilteredRows := true, ignoreErrors := true }
  | 4 => some { ignoreNestedAggregate := false, ignoreManualHiddenRows := false, ignoreFilteredRows := false, ignoreErrors := false }
  | 5 => some { ignoreNestedAggregate := false, ignoreManualHiddenRows := true, ignoreFilteredRows := true, ignoreErrors := false }
  | 6 => some { ignoreNestedAggregate := false, ignoreManualHiddenRows := false, ignoreFilteredRows := false, ignoreErrors := true }
  | 7 => some { ignoreNestedAggregate := false, ignoreManualHiddenRows := true, ignoreFilteredRows := true, ignoreErrors := true }
  | _ => none

theorem subtotalProfiles_are_hostSerializedComposite :
    subtotalMeta.hostInteraction = HostInteractionClass.workbookState
    ∧ subtotalMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ subtotalMeta.fecDependencyProfile = FecDependencyProfile.composite := by
  simp [subtotalMeta]

theorem aggregateProfiles_are_hostSerializedComposite :
    aggregateMeta.hostInteraction = HostInteractionClass.workbookState
    ∧ aggregateMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ aggregateMeta.fecDependencyProfile = FecDependencyProfile.composite := by
  simp [aggregateMeta]

theorem subtotal109_ignores_manual_hidden_rows :
    subtotalRules 109 =
      some {
        ignoreNestedAggregate := true
        ignoreManualHiddenRows := true
        ignoreFilteredRows := true
        ignoreErrors := false } := by
  rfl

theorem aggregateOption3_ignores_hidden_filtered_nested_and_errors :
    aggregateRules 3 =
      some {
        ignoreNestedAggregate := true
        ignoreManualHiddenRows := true
        ignoreFilteredRows := true
        ignoreErrors := true } := by
  rfl

theorem aggregateOption6_ignores_only_errors :
    aggregateRules 6 =
      some {
        ignoreNestedAggregate := false
        ignoreManualHiddenRows := false
        ignoreFilteredRows := false
        ignoreErrors := true } := by
  rfl

end OxFunc.Functions
