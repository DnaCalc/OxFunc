import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def criteriaBaseMeta : FunctionMeta := {
  functionId := "FUNC.CRITERIA_BASE"
  arity := { min := 2, max := 255 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.refOnly
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def countifMeta : FunctionMeta := { criteriaBaseMeta with functionId := "FUNC.COUNTIF", arity := Arity.exact 2 }
def countifsMeta : FunctionMeta := { criteriaBaseMeta with functionId := "FUNC.COUNTIFS", arity := { min := 2, max := 254 } }
def sumifMeta : FunctionMeta := { criteriaBaseMeta with functionId := "FUNC.SUMIF", arity := { min := 2, max := 3 } }
def sumifsMeta : FunctionMeta := { criteriaBaseMeta with functionId := "FUNC.SUMIFS", arity := { min := 3, max := 255 } }
def averageifMeta : FunctionMeta := { criteriaBaseMeta with functionId := "FUNC.AVERAGEIF", arity := { min := 2, max := 3 } }
def averageifsMeta : FunctionMeta := { criteriaBaseMeta with functionId := "FUNC.AVERAGEIFS", arity := { min := 3, max := 255 } }
def maxifsMeta : FunctionMeta := { criteriaBaseMeta with functionId := "FUNC.MAXIFS", arity := { min := 3, max := 255 } }
def minifsMeta : FunctionMeta := { criteriaBaseMeta with functionId := "FUNC.MINIFS", arity := { min := 3, max := 255 } }

theorem sumifMeta_profiles :
    sumifMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ sumifMeta.arity = { min := 2, max := 3 }
    ∧ sumifMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ sumifMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [criteriaBaseMeta, sumifMeta]

theorem criteriaFamily_profiles :
    countifMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ countifsMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ sumifMeta.arity = { min := 2, max := 3 }
    ∧ sumifsMeta.kernelSignatureClass = KernelSignatureClass.custom
    ∧ averageifMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ averageifsMeta.arity = { min := 3, max := 255 }
    ∧ maxifsMeta.arity = { min := 3, max := 255 }
    ∧ minifsMeta.arity = { min := 3, max := 255 } := by
  simp [criteriaBaseMeta, countifMeta, countifsMeta, sumifMeta, sumifsMeta, averageifMeta, averageifsMeta, maxifsMeta, minifsMeta]

end OxFunc.Functions
