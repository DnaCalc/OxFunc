import OxFunc.FunctionCore
import OxFunc.Functions.GroupedAggregation

namespace OxFunc.Functions

open OxFunc

def groupByMeta : FunctionMeta := {
  functionId := "FUNC.GROUPBY"
  arity := { min := 3, max := 255 }
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

theorem groupByMeta_profiles :
    groupByMeta.hostInteraction = HostInteractionClass.none
    ∧ groupByMeta.threadSafety = ThreadSafetyClass.safePure
    ∧ groupByMeta.fecDependencyProfile = FecDependencyProfile.none
    ∧ groupByMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [groupByMeta]

theorem groupByMeta_arity :
    groupByMeta.arity.min = 3 ∧ groupByMeta.arity.max = 255 := by
  simp [groupByMeta]

theorem groupByMeta_deterministic :
    groupByMeta.determinism = DeterminismClass.deterministic := by
  simp [groupByMeta]

theorem groupBy_default_example :
    groupByDefaultExample =
      [ [GroupCell.text "2024", GroupCell.number 30]
      , [GroupCell.text "2025", GroupCell.number 70]
      , [GroupCell.text "Total", GroupCell.number 100] ] := by
  rfl

theorem groupBy_filter_sort_example :
    groupByFilterSortExample =
      [ [GroupCell.text "A", GroupCell.number 50]
      , [GroupCell.text "Total", GroupCell.number 50] ] := by
  rfl

theorem groupBy_hierarchical_subtotal_example :
    groupByHierarchicalSubtotalExample =
      [ [GroupCell.text "East", GroupCell.text "A", GroupCell.number 40]
      , [GroupCell.text "East", GroupCell.text "B", GroupCell.number 20]
      , [GroupCell.text "East", GroupCell.blank, GroupCell.number 60]
      , [GroupCell.text "West", GroupCell.text "A", GroupCell.number 40]
      , [GroupCell.text "West", GroupCell.text "B", GroupCell.number 50]
      , [GroupCell.text "West", GroupCell.blank, GroupCell.number 90]
      , [GroupCell.text "Grand Total", GroupCell.blank, GroupCell.number 150] ] := by
  rfl

end OxFunc.Functions
