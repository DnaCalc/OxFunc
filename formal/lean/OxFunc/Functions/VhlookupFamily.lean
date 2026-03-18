import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def vhlookupBaseMeta : FunctionMeta := {
  functionId := "FUNC.VHLOOKUP_BASE"
  arity := { min := 3, max := 4 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.workbookState
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.lookupMatch
  fecDependencyProfile := FecDependencyProfile.refOnly
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def vlookupMeta : FunctionMeta := { vhlookupBaseMeta with functionId := "FUNC.VLOOKUP" }
def hlookupMeta : FunctionMeta := { vhlookupBaseMeta with functionId := "FUNC.HLOOKUP" }

theorem vhlookupFamily_ids_and_arity :
    vlookupMeta.functionId = "FUNC.VLOOKUP"
    ∧ hlookupMeta.functionId = "FUNC.HLOOKUP"
    ∧ vlookupMeta.arity = { min := 3, max := 4 }
    ∧ hlookupMeta.arity = { min := 3, max := 4 } := by
  simp [vhlookupBaseMeta, vlookupMeta, hlookupMeta]

theorem vhlookupFamily_profiles :
    vlookupMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ hlookupMeta.hostInteraction = HostInteractionClass.workbookState
    ∧ vlookupMeta.kernelSignatureClass = KernelSignatureClass.lookupMatch
    ∧ hlookupMeta.kernelSignatureClass = KernelSignatureClass.lookupMatch := by
  simp [vhlookupBaseMeta, vlookupMeta, hlookupMeta]

end OxFunc.Functions
