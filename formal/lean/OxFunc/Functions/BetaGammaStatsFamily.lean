import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def betaGammaStatsBaseMeta : FunctionMeta := {
  functionId := "FUNC.BETA_GAMMA_STATS_BASE"
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

def betaDistMeta : FunctionMeta := {
  betaGammaStatsBaseMeta with
  functionId := "FUNC.BETA.DIST"
  arity := { min := 4, max := 6 }
}

def betaInvMeta : FunctionMeta := {
  betaGammaStatsBaseMeta with
  functionId := "FUNC.BETA.INV"
  arity := { min := 3, max := 5 }
}

def betadistMeta : FunctionMeta := {
  betaGammaStatsBaseMeta with
  functionId := "FUNC.BETADIST"
  arity := { min := 3, max := 5 }
}

def betainvMeta : FunctionMeta := {
  betaGammaStatsBaseMeta with
  functionId := "FUNC.BETAINV"
  arity := { min := 3, max := 5 }
}

def gammaDistMeta : FunctionMeta := {
  betaGammaStatsBaseMeta with
  functionId := "FUNC.GAMMA.DIST"
  arity := Arity.exact 4
}

def gammaInvMeta : FunctionMeta := {
  betaGammaStatsBaseMeta with
  functionId := "FUNC.GAMMA.INV"
  arity := Arity.exact 3
}

def gammadistMeta : FunctionMeta := {
  betaGammaStatsBaseMeta with
  functionId := "FUNC.GAMMADIST"
  arity := Arity.exact 4
}

def gammainvMeta : FunctionMeta := {
  betaGammaStatsBaseMeta with
  functionId := "FUNC.GAMMAINV"
  arity := Arity.exact 3
}

theorem betaGammaStats_profiles :
    betaDistMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ betaInvMeta.arity = { min := 3, max := 5 }
    ∧ betadistMeta.kernelSignatureClass = KernelSignatureClass.custom
    ∧ betainvMeta.surfaceFecDependencyProfile = FecDependencyProfile.none
    ∧ gammaDistMeta.arity = Arity.exact 4
    ∧ gammaInvMeta.arity = Arity.exact 3
    ∧ gammadistMeta.hostInteraction = HostInteractionClass.none
    ∧ gammainvMeta.threadSafety = ThreadSafetyClass.safePure := by
  simp [
    betaGammaStatsBaseMeta,
    betaDistMeta,
    betaInvMeta,
    betadistMeta,
    betainvMeta,
    gammaDistMeta,
    gammaInvMeta,
    gammadistMeta,
    gammainvMeta
  ]

theorem betaGammaStats_ids :
    betaDistMeta.functionId = "FUNC.BETA.DIST"
    ∧ betaInvMeta.functionId = "FUNC.BETA.INV"
    ∧ betadistMeta.functionId = "FUNC.BETADIST"
    ∧ betainvMeta.functionId = "FUNC.BETAINV"
    ∧ gammaDistMeta.functionId = "FUNC.GAMMA.DIST"
    ∧ gammaInvMeta.functionId = "FUNC.GAMMA.INV"
    ∧ gammadistMeta.functionId = "FUNC.GAMMADIST"
    ∧ gammainvMeta.functionId = "FUNC.GAMMAINV" := by
  simp [
    betaDistMeta,
    betaInvMeta,
    betadistMeta,
    betainvMeta,
    gammaDistMeta,
    gammaInvMeta,
    gammadistMeta,
    gammainvMeta
  ]

end OxFunc.Functions
