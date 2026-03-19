import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def regressionForecastBaseMeta : FunctionMeta := {
  functionId := "FUNC.REGRESSION_FORECAST_BASE"
  arity := { min := 1, max := 4 }
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

def growthMeta : FunctionMeta := { regressionForecastBaseMeta with functionId := "FUNC.GROWTH" }
def forecastMeta : FunctionMeta := {
  regressionForecastBaseMeta with
    functionId := "FUNC.FORECAST"
    arity := Arity.exact 3
}
def forecastLinearMeta : FunctionMeta := {
  regressionForecastBaseMeta with
    functionId := "FUNC.FORECAST.LINEAR"
    arity := Arity.exact 3
}
def trendMeta : FunctionMeta := { regressionForecastBaseMeta with functionId := "FUNC.TREND" }
def linestMeta : FunctionMeta := { regressionForecastBaseMeta with functionId := "FUNC.LINEST" }
def logestMeta : FunctionMeta := { regressionForecastBaseMeta with functionId := "FUNC.LOGEST" }

/--
W24 Batch 09 widens the admitted slice from the old single-predictor note to
the current-baseline multivariate raw-result family while keeping Lean at the
metadata/binding alignment layer rather than duplicating the numeric kernel.
-/
theorem regressionForecastFamily_meta_profiles :
    growthMeta.arity = { min := 1, max := 4 }
    ∧ forecastMeta.arity = Arity.exact 3
    ∧ forecastLinearMeta.arity = Arity.exact 3
    ∧ trendMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ linestMeta.kernelSignatureClass = KernelSignatureClass.custom
    ∧ logestMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ trendMeta.threadSafety = ThreadSafetyClass.safePure := by
  simp [regressionForecastBaseMeta, growthMeta, forecastMeta, forecastLinearMeta, trendMeta, linestMeta, logestMeta]

theorem regressionForecastFamily_ids :
    growthMeta.functionId = "FUNC.GROWTH"
    ∧ forecastMeta.functionId = "FUNC.FORECAST"
    ∧ forecastLinearMeta.functionId = "FUNC.FORECAST.LINEAR"
    ∧ trendMeta.functionId = "FUNC.TREND"
    ∧ linestMeta.functionId = "FUNC.LINEST"
    ∧ logestMeta.functionId = "FUNC.LOGEST" := by
  simp [growthMeta, forecastMeta, forecastLinearMeta, trendMeta, linestMeta, logestMeta]

end OxFunc.Functions
