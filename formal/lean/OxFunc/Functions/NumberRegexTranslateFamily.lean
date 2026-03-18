import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def numberRegexTranslateBase : FunctionMeta := {
  functionId := "FUNC.NUMBER_REGEX_TRANSLATE_BASE"
  arity := { min := 1, max := 1 }
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

def numbervalueMeta : FunctionMeta := {
  numberRegexTranslateBase with
  functionId := "FUNC.NUMBERVALUE"
  arity := { min := 1, max := 3 }
}

def regexextractMeta : FunctionMeta := {
  numberRegexTranslateBase with
  functionId := "FUNC.REGEXEXTRACT"
  arity := { min := 2, max := 4 }
}

def regexreplaceMeta : FunctionMeta := {
  numberRegexTranslateBase with
  functionId := "FUNC.REGEXREPLACE"
  arity := { min := 3, max := 5 }
}

def regextestMeta : FunctionMeta := {
  numberRegexTranslateBase with
  functionId := "FUNC.REGEXTEST"
  arity := { min := 2, max := 3 }
}

def translateMeta : FunctionMeta := {
  numberRegexTranslateBase with
  functionId := "FUNC.TRANSLATE"
  arity := { min := 1, max := 3 }
  hostInteraction := HostInteractionClass.externalProvider
  threadSafety := ThreadSafetyClass.hostSerialized
  fecDependencyProfile := FecDependencyProfile.externalProvider
  surfaceFecDependencyProfile := FecDependencyProfile.externalProvider
}

theorem number_regex_translate_profiles :
    numbervalueMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ regexextractMeta.arity.max = 4
    ∧ regexreplaceMeta.kernelSignatureClass = KernelSignatureClass.custom
    ∧ regextestMeta.functionId = "FUNC.REGEXTEST"
    ∧ translateMeta.hostInteraction = HostInteractionClass.externalProvider
    ∧ translateMeta.surfaceFecDependencyProfile = FecDependencyProfile.externalProvider := by
  simp [numberRegexTranslateBase, numbervalueMeta, regexextractMeta, regexreplaceMeta,
    regextestMeta, translateMeta]

end OxFunc.Functions
