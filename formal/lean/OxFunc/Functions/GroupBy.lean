import OxFunc.FunctionCore

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

end OxFunc.Functions
