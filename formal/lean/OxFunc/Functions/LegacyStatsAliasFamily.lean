import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def legacyStatsAliasBase : FunctionMeta := {
  functionId := "FUNC.LEGACY_STATS_ALIAS_BASE"
  arity := Arity.exact 1
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

def covarMeta : FunctionMeta := { legacyStatsAliasBase with functionId := "FUNC.COVAR", arity := Arity.exact 2 }
def modeMeta : FunctionMeta := { legacyStatsAliasBase with functionId := "FUNC.MODE", arity := { min := 1, max := 255 } }
def percentileMeta : FunctionMeta := { legacyStatsAliasBase with functionId := "FUNC.PERCENTILE", arity := Arity.exact 2 }
def percentrankMeta : FunctionMeta := { legacyStatsAliasBase with functionId := "FUNC.PERCENTRANK", arity := { min := 2, max := 3 } }
def quartileMeta : FunctionMeta := { legacyStatsAliasBase with functionId := "FUNC.QUARTILE", arity := Arity.exact 2 }
def loginvMeta : FunctionMeta := { legacyStatsAliasBase with functionId := "FUNC.LOGINV", arity := Arity.exact 3 }

theorem legacyStatsAlias_profiles :
    covarMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ modeMeta.arity.max = 255
    ∧ percentileMeta.functionId = "FUNC.PERCENTILE"
    ∧ percentrankMeta.arity.max = 3
    ∧ quartileMeta.threadSafety = ThreadSafetyClass.safePure
    ∧ loginvMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [legacyStatsAliasBase, covarMeta, modeMeta, percentileMeta, percentrankMeta, quartileMeta, loginvMeta]

end OxFunc.Functions
