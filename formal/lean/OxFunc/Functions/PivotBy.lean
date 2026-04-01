import OxFunc.FunctionCore
import OxFunc.Functions.GroupedAggregation

namespace OxFunc.Functions

open OxFunc

def pivotByMeta : FunctionMeta := {
  functionId := "FUNC.PIVOTBY"
  arity := { min := 4, max := 255 }
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

theorem pivotByMeta_profiles :
    pivotByMeta.hostInteraction = HostInteractionClass.none
    ∧ pivotByMeta.threadSafety = ThreadSafetyClass.safePure
    ∧ pivotByMeta.fecDependencyProfile = FecDependencyProfile.none
    ∧ pivotByMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [pivotByMeta]

theorem pivotByMeta_arity :
    pivotByMeta.arity.min = 4 ∧ pivotByMeta.arity.max = 255 := by
  simp [pivotByMeta]

theorem pivotByMeta_deterministic :
    pivotByMeta.determinism = DeterminismClass.deterministic := by
  simp [pivotByMeta]

theorem pivotBy_default_example :
    pivotByDefaultExample =
      [ [GroupCell.blank, GroupCell.text "A", GroupCell.text "B", GroupCell.text "Total"]
      , [GroupCell.text "East", GroupCell.number 10, GroupCell.number 20, GroupCell.number 30]
      , [GroupCell.text "West", GroupCell.number 40, GroupCell.number 50, GroupCell.number 90]
      , [GroupCell.text "Total", GroupCell.number 50, GroupCell.number 70, GroupCell.number 120] ] := by
  rfl

theorem pivotBy_filter_zero_totals_example :
    pivotByFilterZeroTotalsExample =
      [ [GroupCell.blank, GroupCell.text "A"]
      , [GroupCell.text "East", GroupCell.number 10]
      , [GroupCell.text "West", GroupCell.number 40] ] := by
  rfl

theorem pivotBy_sorted_example :
    pivotBySortedExample =
      [ [GroupCell.blank, GroupCell.text "B", GroupCell.text "A", GroupCell.text "Total"]
      , [GroupCell.text "West", GroupCell.number 50, GroupCell.number 40, GroupCell.number 90]
      , [GroupCell.text "East", GroupCell.number 20, GroupCell.number 10, GroupCell.number 30]
      , [GroupCell.text "Total", GroupCell.number 70, GroupCell.number 50, GroupCell.number 120] ] := by
  rfl

end OxFunc.Functions
