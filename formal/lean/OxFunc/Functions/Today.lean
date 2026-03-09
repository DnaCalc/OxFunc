import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def todayMeta : FunctionMeta := {
  functionId := "FUNC.TODAY"
  arity := Arity.exact 0
  determinism := DeterminismClass.timeDependent
  volatility := VolatilityClass.volatileFull
  hostInteraction := HostInteractionClass.applicationState
  threadSafety := ThreadSafetyClass.hostSerialized
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.none
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.timeProvider
  surfaceFecDependencyProfile := FecDependencyProfile.timeProvider
}

structure TodayProvider where
  serial : Rat
  deriving DecidableEq, Repr

def evalTodayAdapter (provider : TodayProvider) : Value :=
  .number provider.serial.floor

theorem evalTodayAdapter_returns_integral_serial :
    evalTodayAdapter { serial := 46000 + (3 / 4 : Rat) } = .number 46000 := by
  native_decide

theorem todayMeta_profiles :
    todayMeta.determinism = DeterminismClass.timeDependent
    ∧ todayMeta.volatility = VolatilityClass.volatileFull
    ∧ todayMeta.fecDependencyProfile = FecDependencyProfile.timeProvider := by
  simp [todayMeta]

end OxFunc.Functions
