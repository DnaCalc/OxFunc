import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def dateWeekBaseMeta : FunctionMeta := {
  functionId := "FUNC.DATE_WEEK_BASE"
  arity := Arity.exact 1
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

def edateMeta : FunctionMeta := {
  dateWeekBaseMeta with
  functionId := "FUNC.EDATE"
  arity := Arity.exact 2
}

def eomonthMeta : FunctionMeta := {
  dateWeekBaseMeta with
  functionId := "FUNC.EOMONTH"
  arity := Arity.exact 2
}

def weekdayMeta : FunctionMeta := {
  dateWeekBaseMeta with
  functionId := "FUNC.WEEKDAY"
  arity := { min := 1, max := 2 }
}

def weeknumMeta : FunctionMeta := {
  dateWeekBaseMeta with
  functionId := "FUNC.WEEKNUM"
  arity := { min := 1, max := 2 }
}

def isoweeknumMeta : FunctionMeta := {
  dateWeekBaseMeta with
  functionId := "FUNC.ISOWEEKNUM"
}

theorem dateWeekMeta_profiles :
    edateMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ eomonthMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ weekdayMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ weeknumMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ isoweeknumMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [dateWeekBaseMeta, edateMeta, eomonthMeta, weekdayMeta, weeknumMeta, isoweeknumMeta]

end OxFunc.Functions
