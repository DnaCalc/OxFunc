import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def momentStatsBase : FunctionMeta := {
  functionId := "FUNC.MOMENT_STATS_BASE"
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

def kurtMeta : FunctionMeta := {
  momentStatsBase with
  functionId := "FUNC.KURT"
  arity := { min := 1, max := 255 }
  coercionLiftProfile := CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
  kernelSignatureClass := KernelSignatureClass.numsToNum
}

def skewMeta : FunctionMeta := {
  momentStatsBase with
  functionId := "FUNC.SKEW"
  arity := { min := 1, max := 255 }
  coercionLiftProfile := CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
  kernelSignatureClass := KernelSignatureClass.numsToNum
}

def skewPMeta : FunctionMeta := {
  momentStatsBase with
  functionId := "FUNC.SKEW.P"
  arity := { min := 1, max := 255 }
  coercionLiftProfile := CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
  kernelSignatureClass := KernelSignatureClass.numsToNum
}

def steyxMeta : FunctionMeta := {
  momentStatsBase with
  functionId := "FUNC.STEYX"
  arity := Arity.exact 2
}

def trimmeanMeta : FunctionMeta := {
  momentStatsBase with
  functionId := "FUNC.TRIMMEAN"
  arity := Arity.exact 2
  coercionLiftProfile := CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
}

theorem momentStats_profiles :
    kurtMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ skewMeta.arity.max = 255
    ∧ skewPMeta.functionId = "FUNC.SKEW.P"
    ∧ steyxMeta.arity = Arity.exact 2
    ∧ trimmeanMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [momentStatsBase, kurtMeta, skewMeta, skewPMeta, steyxMeta, trimmeanMeta]

end OxFunc.Functions
