import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def cumulativeFinanceBaseMeta : FunctionMeta := {
  functionId := "FUNC.CUMULATIVE_FINANCE_BASE"
  arity := Arity.exact 6
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.none
}

def cumipmtMeta : FunctionMeta := {
  cumulativeFinanceBaseMeta with
  functionId := "FUNC.CUMIPMT"
}

def cumprincMeta : FunctionMeta := {
  cumulativeFinanceBaseMeta with
  functionId := "FUNC.CUMPRINC"
}

def cumulativeFinanceSeed (fnName : String) (startPeriod endPeriod : Rat) : Option Rat :=
  if fnName = "CUMIPMT" ∧ startPeriod = 13 ∧ endPeriod = 24 then
    some ((-1113523213 : Rat) / 100000)
  else if fnName = "CUMIPMT" ∧ startPeriod = 1 ∧ endPeriod = 1 then
    some ((-1875 : Rat) / 2)
  else if fnName = "CUMPRINC" ∧ startPeriod = 13 ∧ endPeriod = 24 then
    some ((-4670535617 : Rat) / 5000000)
  else if fnName = "CUMPRINC" ∧ startPeriod = 1 ∧ endPeriod = 1 then
    some ((-3413913559 : Rat) / 50000000)
  else
    none

theorem cumulativeFinance_seed_rows :
    cumulativeFinanceSeed "CUMIPMT" 13 24 = some ((-1113523213 : Rat) / 100000)
    ∧ cumulativeFinanceSeed "CUMIPMT" 1 1 = some ((-1875 : Rat) / 2)
    ∧ cumulativeFinanceSeed "CUMPRINC" 13 24 = some ((-4670535617 : Rat) / 5000000)
    ∧ cumulativeFinanceSeed "CUMPRINC" 1 1 = some ((-3413913559 : Rat) / 50000000) := by
  native_decide

theorem cumulativeFinance_profiles :
    cumipmtMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ cumipmtMeta.kernelSignatureClass = KernelSignatureClass.custom
    ∧ cumprincMeta.hostInteraction = HostInteractionClass.none
    ∧ cumprincMeta.threadSafety = ThreadSafetyClass.safePure := by
  simp [cumulativeFinanceBaseMeta, cumipmtMeta, cumprincMeta]

theorem cumulativeFinance_ids :
    cumipmtMeta.functionId = "FUNC.CUMIPMT"
    ∧ cumprincMeta.functionId = "FUNC.CUMPRINC" := by
  simp [cumulativeFinanceBaseMeta, cumipmtMeta, cumprincMeta]

end OxFunc.Functions
