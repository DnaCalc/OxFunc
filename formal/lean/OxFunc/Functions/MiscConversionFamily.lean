import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def miscConversionBaseMeta : FunctionMeta := {
  functionId := "FUNC.MISC_CONVERSION_BASE"
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

def bahttextMeta : FunctionMeta := {
  miscConversionBaseMeta with
  functionId := "FUNC.BAHTTEXT"
}

def convertMeta : FunctionMeta := {
  { miscConversionBaseMeta with functionId := "FUNC.CONVERT" } with
  arity := Arity.exact 3
}

def euroconvertMeta : FunctionMeta := {
  { miscConversionBaseMeta with functionId := "FUNC.EUROCONVERT" } with
  arity := { min := 3, max := 5 }
}

def percentofMeta : FunctionMeta := {
  { miscConversionBaseMeta with functionId := "FUNC.PERCENTOF" } with
  arity := Arity.exact 2
}

def randarraMeta : FunctionMeta := {
  functionId := "FUNC.RANDARRA"
  arity := { min := 0, max := 5 }
  determinism := DeterminismClass.pseudoRandom
  volatility := VolatilityClass.volatileFull
  hostInteraction := HostInteractionClass.applicationState
  threadSafety := ThreadSafetyClass.hostSerialized
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.randomProvider
  surfaceFecDependencyProfile := FecDependencyProfile.randomProvider
}

def miscConversionSeed (fnName : String) : Option String :=
  if fnName = "BAHTTEXT_1234_56" then some "หนึ่งพันสองร้อยสามสิบสี่บาทห้าสิบหกสตางค์"
  else if fnName = "CONVERT_LBM_KG" then some "0.45359237"
  else if fnName = "PERCENTOF_15_60" then some "0.25"
  else if fnName = "RANDARRA_SHAPE" then some "2x2"
  else none

theorem miscConversion_seed_rows :
    miscConversionSeed "BAHTTEXT_1234_56" = some "หนึ่งพันสองร้อยสามสิบสี่บาทห้าสิบหกสตางค์"
    ∧ miscConversionSeed "CONVERT_LBM_KG" = some "0.45359237"
    ∧ miscConversionSeed "PERCENTOF_15_60" = some "0.25"
    ∧ miscConversionSeed "RANDARRA_SHAPE" = some "2x2" := by
  native_decide

theorem miscConversion_profiles :
    bahttextMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ convertMeta.arity = Arity.exact 3
    ∧ euroconvertMeta.arity = { min := 3, max := 5 }
    ∧ percentofMeta.kernelSignatureClass = KernelSignatureClass.custom
    ∧ randarraMeta.determinism = DeterminismClass.pseudoRandom
    ∧ randarraMeta.surfaceFecDependencyProfile = FecDependencyProfile.randomProvider := by
  simp [
    miscConversionBaseMeta,
    bahttextMeta,
    convertMeta,
    euroconvertMeta,
    percentofMeta,
    randarraMeta
  ]

end OxFunc.Functions
