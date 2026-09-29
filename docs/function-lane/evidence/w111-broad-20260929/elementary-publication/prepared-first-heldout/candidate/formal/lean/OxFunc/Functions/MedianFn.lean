import OxFunc.FunctionCore
import OxFunc.ElementaryPublication
import OxFunc.ElementaryPrepared

namespace OxFunc.Functions

open OxFunc
abbrev medianEvenNumericBinding := OxFunc.ElementaryPublication.medianEven
abbrev medianContributionBinding := OxFunc.ElementaryPrepared.medianContribution

def medianMeta : FunctionMeta := {
  functionId := "FUNC.MEDIAN"
  arity := { min := 1, max := 255 }
  determinism := .deterministic
  volatility := .nonvolatile
  hostInteraction := .none
  threadSafety := .safePure
  argPreparationProfile := .valuesOnlyPreAdapter
  coercionLiftProfile := .aggregateDirectAndRangeDualPolicy
  kernelSignatureClass := .numsToNum
  fecDependencyProfile := .none
  surfaceFecDependencyProfile := .refOnly
}

theorem medianMeta_profiles :
    medianMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ medianMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
    ∧ medianMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [medianMeta]

end OxFunc.Functions
