import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def hyperlinkMeta : FunctionMeta := {
  functionId := "FUNC.HYPERLINK"
  arity := { min := 1, max := 2 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.environmentState
  threadSafety := ThreadSafetyClass.hostSerialized
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.textToText
  fecDependencyProfile := FecDependencyProfile.composite
  surfaceFecDependencyProfile := FecDependencyProfile.composite
}

def hyperlinkDisplay (linkLocation : String) (displayText : Option String) : String :=
  displayText.getD linkLocation

inductive RtdProjectedOutcome where
  | value (payload : String)
  | noValueYet
  | capabilityDenied
  | connectionFailed
  deriving DecidableEq, Repr

def rtdMeta : FunctionMeta := {
  functionId := "FUNC.RTD"
  arity := { min := 3, max := 255 }
  determinism := DeterminismClass.externalEventDependent
  volatility := VolatilityClass.volatileContextual
  hostInteraction := HostInteractionClass.externalProvider
  threadSafety := ThreadSafetyClass.hostSerialized
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.externalProvider
  surfaceFecDependencyProfile := FecDependencyProfile.externalProvider
}

def projectRtdOutcome : RtdProjectedOutcome → String
  | .value payload => payload
  | .noValueYet => "#N/A"
  | .capabilityDenied => "#N/A"
  | .connectionFailed => "#N/A"

theorem hyperlinkMeta_profiles :
    hyperlinkMeta.hostInteraction = HostInteractionClass.environmentState
    ∧ hyperlinkMeta.threadSafety = ThreadSafetyClass.hostSerialized
    ∧ hyperlinkMeta.surfaceFecDependencyProfile = FecDependencyProfile.composite := by
  simp [hyperlinkMeta]

theorem hyperlinkDisplay_defaults_to_link_location :
    hyperlinkDisplay "https://example.com" none = "https://example.com" := by
  rfl

theorem hyperlinkDisplay_prefers_friendly_name :
    hyperlinkDisplay "https://example.com" (some "Example") = "Example" := by
  rfl

theorem rtdMeta_profiles :
    rtdMeta.determinism = DeterminismClass.externalEventDependent
    ∧ rtdMeta.volatility = VolatilityClass.volatileContextual
    ∧ rtdMeta.fecDependencyProfile = FecDependencyProfile.externalProvider
    ∧ rtdMeta.surfaceFecDependencyProfile = FecDependencyProfile.externalProvider := by
  simp [rtdMeta]

theorem projectRtdOutcome_no_value_yet_is_na :
    projectRtdOutcome .noValueYet = "#N/A" := by
  rfl

theorem projectRtdOutcome_value_passthrough :
    projectRtdOutcome (.value "42") = "42" := by
  rfl

end OxFunc.Functions
