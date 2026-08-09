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

/--
W109 G4-05 executable binding for the current-reference CONVERT linear graph.
Lean records the semantic route and its store order without duplicating the
x87 floating-point engine: every arithmetic site publishes PC64 to binary64,
and the prefix delta is a distinct final site after the direct-factor core.
-/
inductive ConvertLinearPublicationSite where
  | productStore
  | quotientStore
  | prefixDeltaStore
  deriving DecidableEq, Repr

def convertLinearPublicationSchedule : List ConvertLinearPublicationSite :=
  [.productStore, .quotientStore, .prefixDeltaStore]

structure ConvertPublicationRoute where
  lengthUsesIntegerAngstromTable : Bool
  pressureUsesRoundedReciprocalTable : Bool
  barPublishesNA : Bool
  prefixDeltaAfterDirectCore : Bool
  temperatureUsesDirectPairRoutes : Bool
  deriving DecidableEq, Repr

def convertPublicationRoute : ConvertPublicationRoute := {
  lengthUsesIntegerAngstromTable := true
  pressureUsesRoundedReciprocalTable := true
  barPublishesNA := true
  prefixDeltaAfterDirectCore := true
  temperatureUsesDirectPairRoutes := true
}

inductive ConvertTemperatureRoute where
  | identity
  | kelvinToCelsius
  | celsiusToKelvin
  | kelvinToFahrenheit
  | fahrenheitToKelvin
  | celsiusToFahrenheit
  | fahrenheitToCelsius
  | unsupported
  deriving DecidableEq, Repr

def convertTemperatureRoute (fromUnit toUnit : String) : ConvertTemperatureRoute :=
  if fromUnit = toUnit ∧ (fromUnit = "K" ∨ fromUnit = "C" ∨ fromUnit = "F") then
    .identity
  else if fromUnit = "K" ∧ toUnit = "C" then .kelvinToCelsius
  else if fromUnit = "C" ∧ toUnit = "K" then .celsiusToKelvin
  else if fromUnit = "K" ∧ toUnit = "F" then .kelvinToFahrenheit
  else if fromUnit = "F" ∧ toUnit = "K" then .fahrenheitToKelvin
  else if fromUnit = "C" ∧ toUnit = "F" then .celsiusToFahrenheit
  else if fromUnit = "F" ∧ toUnit = "C" then .fahrenheitToCelsius
  else .unsupported

/-- Rational value semantics for the admitted linear substrate. Exact binary64
publication remains Rust-owned; this model pins factor/core/prefix ordering. -/
def convertLinearRat (number fromFactor toFactor prefixDelta : Rat) : Rat :=
  ((number * fromFactor) / toFactor) * prefixDelta

def euroconvertMeta : FunctionMeta := {
  { miscConversionBaseMeta with functionId := "FUNC.EUROCONVERT" } with
  arity := { min := 3, max := 5 }
}

def percentofMeta : FunctionMeta := {
  { miscConversionBaseMeta with functionId := "FUNC.PERCENTOF" } with
  arity := Arity.exact 2
}

def RANDARRAYMeta : FunctionMeta := {
  functionId := "FUNC.RANDARRAY"
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
  else if fnName = "RANDARRAY_SHAPE" then some "2x2"
  else none

theorem miscConversion_seed_rows :
    miscConversionSeed "BAHTTEXT_1234_56" = some "หนึ่งพันสองร้อยสามสิบสี่บาทห้าสิบหกสตางค์"
    ∧ miscConversionSeed "CONVERT_LBM_KG" = some "0.45359237"
    ∧ miscConversionSeed "PERCENTOF_15_60" = some "0.25"
    ∧ miscConversionSeed "RANDARRAY_SHAPE" = some "2x2" := by
  native_decide

theorem miscConversion_profiles :
    bahttextMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ convertMeta.arity = Arity.exact 3
    ∧ euroconvertMeta.arity = { min := 3, max := 5 }
    ∧ percentofMeta.kernelSignatureClass = KernelSignatureClass.custom
    ∧ RANDARRAYMeta.determinism = DeterminismClass.pseudoRandom
    ∧ RANDARRAYMeta.surfaceFecDependencyProfile = FecDependencyProfile.randomProvider := by
  simp [
    miscConversionBaseMeta,
    bahttextMeta,
    convertMeta,
    euroconvertMeta,
    percentofMeta,
    RANDARRAYMeta
  ]

theorem convert_linear_publication_schedule_is_three_stored_sites :
    convertLinearPublicationSchedule =
      [.productStore, .quotientStore, .prefixDeltaStore]
    ∧ convertPublicationRoute.lengthUsesIntegerAngstromTable = true
    ∧ convertPublicationRoute.pressureUsesRoundedReciprocalTable = true
    ∧ convertPublicationRoute.barPublishesNA = true
    ∧ convertPublicationRoute.prefixDeltaAfterDirectCore = true := by
  native_decide

theorem convert_temperature_routes_are_direct_and_pair_specific :
    convertTemperatureRoute "C" "F" = .celsiusToFahrenheit
    ∧ convertTemperatureRoute "F" "C" = .fahrenheitToCelsius
    ∧ convertTemperatureRoute "K" "K" = .identity
    ∧ convertTemperatureRoute "C" "Pa" = .unsupported := by
  native_decide

theorem convert_linear_rat_seed_rows :
    convertLinearRat 1 (45359237 / 100000 : Rat) 1 (1 / 1000 : Rat)
      = (45359237 / 100000000 : Rat)
    ∧ convertLinearRat (7 / 2 : Rat) 1 1 1000 = 3500 := by
  native_decide

end OxFunc.Functions
