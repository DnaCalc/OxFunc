import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def switchMeta : FunctionMeta := {
  functionId := "FUNC.SWITCH"
  arity := { min := 3, max := 255 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def isformulaMeta : FunctionMeta := {
  functionId := "FUNC.ISFORMULA"
  arity := Arity.exact 1
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.workbookState
  threadSafety := ThreadSafetyClass.hostSerialized
  argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.externalProvider
  surfaceFecDependencyProfile := FecDependencyProfile.composite
}

/--
`SWITCH` stays pure and reference-visible at the adapter boundary.
`ISFORMULA` is modeled as a host-query function over a reference-bearing operand.
The current batch note keeps formula-artifact inspection on the host side rather than
inside this Lean slice.
-/
theorem miscSwitchInfo_profiles :
    switchMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ switchMeta.hostInteraction = HostInteractionClass.none
    ∧ isformulaMeta.hostInteraction = HostInteractionClass.workbookState
    ∧ isformulaMeta.fecDependencyProfile = FecDependencyProfile.externalProvider := by
  simp [switchMeta, isformulaMeta]

end OxFunc.Functions
