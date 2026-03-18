import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def complexTextMetaBase : FunctionMeta := {
  functionId := "FUNC.IM_TEXT_BASE"
  arity := Arity.exact 1
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

def complexNumberMetaBase : FunctionMeta := {
  complexTextMetaBase with
  functionId := "FUNC.IM_NUMBER_BASE"
}

def complexMeta : FunctionMeta := {
  complexTextMetaBase with
  functionId := "FUNC.COMPLEX"
  arity := { min := 2, max := 3 }
}

def imabsMeta : FunctionMeta := { complexNumberMetaBase with functionId := "FUNC.IMABS" }
def imaginaryMeta : FunctionMeta := { complexNumberMetaBase with functionId := "FUNC.IMAGINARY" }
def imargumentMeta : FunctionMeta := { complexNumberMetaBase with functionId := "FUNC.IMARGUMENT" }
def imconjugateMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMCONJUGATE" }
def imcosMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMCOS" }
def imcoshMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMCOSH" }
def imcotMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMCOT" }
def imcscMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMCSC" }
def imcschMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMCSCH" }
def imdivMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMDIV", arity := Arity.exact 2 }
def imexpMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMEXP" }
def imlnMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMLN" }
def imlog10Meta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMLOG10" }
def imlog2Meta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMLOG2" }
def impowerMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMPOWER", arity := Arity.exact 2 }
def improductMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMPRODUCT", arity := { min := 1, max := 255 } }
def imrealMeta : FunctionMeta := { complexNumberMetaBase with functionId := "FUNC.IMREAL" }
def imsecMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMSEC" }
def imsechMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMSECH" }
def imsinMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMSIN" }
def imsinhMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMSINH" }
def imsqrtMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMSQRT" }
def imsubMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMSUB", arity := Arity.exact 2 }
def imsumMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMSUM", arity := { min := 1, max := 255 } }
def imtanMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMTAN" }

theorem complexFamily_meta_profiles :
    complexMeta.arity = { min := 2, max := 3 }
    ∧ imabsMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ imaginaryMeta.kernelSignatureClass = KernelSignatureClass.custom
    ∧ imargumentMeta.fecDependencyProfile = FecDependencyProfile.none
    ∧ imconjugateMeta.threadSafety = ThreadSafetyClass.safePure
    ∧ imdivMeta.arity = Arity.exact 2
    ∧ impowerMeta.arity = Arity.exact 2
    ∧ improductMeta.arity = { min := 1, max := 255 }
    ∧ imsumMeta.arity = { min := 1, max := 255 }
    ∧ imtanMeta.hostInteraction = HostInteractionClass.none := by
  simp [
    complexTextMetaBase,
    complexNumberMetaBase,
    complexMeta,
    imabsMeta,
    imaginaryMeta,
    imargumentMeta,
    imconjugateMeta,
    imdivMeta,
    impowerMeta,
    improductMeta,
    imsumMeta,
    imtanMeta
  ]

theorem complexFamily_ids :
    complexMeta.functionId = "FUNC.COMPLEX"
    ∧ imabsMeta.functionId = "FUNC.IMABS"
    ∧ imdivMeta.functionId = "FUNC.IMDIV"
    ∧ impowerMeta.functionId = "FUNC.IMPOWER"
    ∧ imsumMeta.functionId = "FUNC.IMSUM"
    ∧ imtanMeta.functionId = "FUNC.IMTAN" := by
  simp [complexMeta, imabsMeta, imdivMeta, impowerMeta, imsumMeta, imtanMeta]

end OxFunc.Functions
