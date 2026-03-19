import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def informationPredicateBaseMeta : FunctionMeta := {
  functionId := "FUNC.INFORMATION_PREDICATE_BASE"
  arity := Arity.exact 1
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.none
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def isBlankMeta : FunctionMeta := { informationPredicateBaseMeta with functionId := "FUNC.ISBLANK" }
def isErrMeta : FunctionMeta := { informationPredicateBaseMeta with functionId := "FUNC.ISERR" }
def isErrorMeta : FunctionMeta := { informationPredicateBaseMeta with functionId := "FUNC.ISERROR" }
def isLogicalMeta : FunctionMeta := { informationPredicateBaseMeta with functionId := "FUNC.ISLOGICAL" }
def isNaMeta : FunctionMeta := { informationPredicateBaseMeta with functionId := "FUNC.ISNA" }
def isNonTextMeta : FunctionMeta := { informationPredicateBaseMeta with functionId := "FUNC.ISNONTEXT" }
def isTextMeta : FunctionMeta := { informationPredicateBaseMeta with functionId := "FUNC.ISTEXT" }

def isOddMeta : FunctionMeta := {
  informationPredicateBaseMeta with
    functionId := "FUNC.ISODD"
    coercionLiftProfile := CoercionLiftProfile.custom
}

def isRefMeta : FunctionMeta := {
  informationPredicateBaseMeta with
    functionId := "FUNC.ISREF"
    argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
    fecDependencyProfile := FecDependencyProfile.refOnly
    surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

theorem informationPredicateFamily_profiles :
    isBlankMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ isErrMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ isOddMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ isRefMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ isRefMeta.fecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [informationPredicateBaseMeta, isBlankMeta, isErrMeta, isOddMeta, isRefMeta]

theorem informationPredicateFamily_ids :
    isBlankMeta.functionId = "FUNC.ISBLANK"
    ∧ isErrMeta.functionId = "FUNC.ISERR"
    ∧ isErrorMeta.functionId = "FUNC.ISERROR"
    ∧ isLogicalMeta.functionId = "FUNC.ISLOGICAL"
    ∧ isNaMeta.functionId = "FUNC.ISNA"
    ∧ isNonTextMeta.functionId = "FUNC.ISNONTEXT"
    ∧ isOddMeta.functionId = "FUNC.ISODD"
    ∧ isRefMeta.functionId = "FUNC.ISREF"
    ∧ isTextMeta.functionId = "FUNC.ISTEXT" := by
  simp [isBlankMeta, isErrMeta, isErrorMeta, isLogicalMeta, isNaMeta, isNonTextMeta, isOddMeta, isRefMeta, isTextMeta]

end OxFunc.Functions
