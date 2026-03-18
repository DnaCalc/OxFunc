import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def datePartBaseMeta : FunctionMeta := {
  functionId := "FUNC.DATE_PART_BASE"
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

def dayMeta : FunctionMeta := { datePartBaseMeta with functionId := "FUNC.DAY" }
def monthMeta : FunctionMeta := { datePartBaseMeta with functionId := "FUNC.MONTH" }
def yearMeta : FunctionMeta := { datePartBaseMeta with functionId := "FUNC.YEAR" }
def daysMeta : FunctionMeta := { datePartBaseMeta with functionId := "FUNC.DAYS", arity := Arity.exact 2 }
def hourMeta : FunctionMeta := { datePartBaseMeta with functionId := "FUNC.HOUR" }
def minuteMeta : FunctionMeta := { datePartBaseMeta with functionId := "FUNC.MINUTE" }
def secondMeta : FunctionMeta := { datePartBaseMeta with functionId := "FUNC.SECOND" }
def timeMeta : FunctionMeta := { datePartBaseMeta with functionId := "FUNC.TIME", arity := Arity.exact 3 }

theorem datePartsMeta_profiles :
    dayMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ monthMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ yearMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ daysMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ hourMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ minuteMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ secondMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ timeMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [datePartBaseMeta, dayMeta, monthMeta, yearMeta, daysMeta, hourMeta, minuteMeta, secondMeta, timeMeta]

end OxFunc.Functions
