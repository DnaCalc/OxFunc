import OxFunc.FunctionCore
import OxFunc.RefResolverSeam

namespace OxFunc.Functions

open OxFunc

def cellMeta : FunctionMeta := {
  functionId := "FUNC.CELL"
  arity := { min := 1, max := 2 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.volatileContextual
  hostInteraction := HostInteractionClass.workbookState
  threadSafety := ThreadSafetyClass.hostSerialized
  argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.callerContext
  surfaceFecDependencyProfile := FecDependencyProfile.callerContext
}

def evalCellSeedAddress (ref : ReferenceToken) : String :=
  match ref.target with
  | "B3" => "$B$3"
  | other => other

theorem evalCellSeedAddress_b3 :
    evalCellSeedAddress { kind := .a1, target := "B3" } = "$B$3" := by
  simp [evalCellSeedAddress]

theorem cellMeta_profiles :
    cellMeta.arity.min = 1
    ∧ cellMeta.arity.max = 2
    ∧ cellMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ cellMeta.fecDependencyProfile = FecDependencyProfile.callerContext
    ∧ cellMeta.surfaceFecDependencyProfile = FecDependencyProfile.callerContext := by
  simp [cellMeta]

end OxFunc.Functions
