import OxFunc.FunctionCore
import OxFunc.RefResolverSeam

namespace OxFunc.Functions

open OxFunc

def indirectMeta : FunctionMeta := {
  functionId := "FUNC.INDIRECT"
  arity := { min := 1, max := 2 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.volatileContextual
  hostInteraction := HostInteractionClass.workbookState
  threadSafety := ThreadSafetyClass.hostSerialized
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.callerContext
  surfaceFecDependencyProfile := FecDependencyProfile.callerContext
}

def evalIndirectAdapter (text : String) (a1Style : Bool := true) : Except String ReferenceToken :=
  if text = "" then
    Except.error "invalid_ref_text"
  else if a1Style then
    Except.ok { kind := .a1, target := text }
  else
    Except.error "unsupported_r1c1_seed"

theorem evalIndirectAdapter_a1_ok :
    evalIndirectAdapter "Sheet1!A1" true =
      Except.ok { kind := .a1, target := "Sheet1!A1" } := by
  simp [evalIndirectAdapter]

theorem evalIndirectAdapter_r1c1_seed_unsupported :
    evalIndirectAdapter "R1C1" false = Except.error "unsupported_r1c1_seed" := by
  simp [evalIndirectAdapter]

theorem indirectMeta_profiles :
    indirectMeta.volatility = VolatilityClass.volatileContextual
    ∧ indirectMeta.hostInteraction = HostInteractionClass.workbookState
    ∧ indirectMeta.fecDependencyProfile = FecDependencyProfile.callerContext
    ∧ indirectMeta.surfaceFecDependencyProfile = FecDependencyProfile.callerContext := by
  simp [indirectMeta]

end OxFunc.Functions
