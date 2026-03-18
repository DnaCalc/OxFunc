import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def normalLogBaseMeta : FunctionMeta := {
  functionId := "FUNC.NORMAL_LOG_BASE"
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

def confidenceMeta : FunctionMeta := {
  normalLogBaseMeta with
  functionId := "FUNC.CONFIDENCE"
  arity := Arity.exact 3
}

def confidenceNormMeta : FunctionMeta := {
  normalLogBaseMeta with
  functionId := "FUNC.CONFIDENCE.NORM"
  arity := Arity.exact 3
}

def lognormDistMeta : FunctionMeta := {
  normalLogBaseMeta with
  functionId := "FUNC.LOGNORM.DIST"
  arity := Arity.exact 4
}

def lognormInvMeta : FunctionMeta := {
  normalLogBaseMeta with
  functionId := "FUNC.LOGNORM.INV"
  arity := Arity.exact 3
}

def lognormdistMeta : FunctionMeta := {
  normalLogBaseMeta with
  functionId := "FUNC.LOGNORMDIST"
  arity := Arity.exact 3
}

def normDistMeta : FunctionMeta := {
  normalLogBaseMeta with
  functionId := "FUNC.NORM.DIST"
  arity := Arity.exact 4
}

def normInvMeta : FunctionMeta := {
  normalLogBaseMeta with
  functionId := "FUNC.NORM.INV"
  arity := Arity.exact 3
}

def normSDistMeta : FunctionMeta := {
  normalLogBaseMeta with
  functionId := "FUNC.NORM.S.DIST"
  arity := Arity.exact 2
}

def normSInvMeta : FunctionMeta := {
  normalLogBaseMeta with
  functionId := "FUNC.NORM.S.INV"
}

def normdistMeta : FunctionMeta := {
  normalLogBaseMeta with
  functionId := "FUNC.NORMDIST"
  arity := Arity.exact 4
}

def norminvMeta : FunctionMeta := {
  normalLogBaseMeta with
  functionId := "FUNC.NORMINV"
  arity := Arity.exact 3
}

def normsdistMeta : FunctionMeta := {
  normalLogBaseMeta with
  functionId := "FUNC.NORMSDIST"
}

def normsinvMeta : FunctionMeta := {
  normalLogBaseMeta with
  functionId := "FUNC.NORMSINV"
}

theorem normalLogMeta_profiles :
    confidenceMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ confidenceNormMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ lognormDistMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ lognormInvMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ lognormdistMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ normDistMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ normInvMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ normSDistMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ normSInvMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ normdistMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ norminvMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ normsdistMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ normsinvMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [
    normalLogBaseMeta, confidenceMeta, confidenceNormMeta, lognormDistMeta, lognormInvMeta,
    lognormdistMeta, normDistMeta, normInvMeta, normSDistMeta, normSInvMeta, normdistMeta,
    norminvMeta, normsdistMeta, normsinvMeta
  ]

end OxFunc.Functions
