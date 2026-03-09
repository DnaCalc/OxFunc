import OxFunc.FunctionCore
import OxFunc.RefResolverSeam

namespace OxFunc.Functions

open OxFunc

def offsetMeta : FunctionMeta := {
  functionId := "FUNC.OFFSET"
  arity := { min := 3, max := 5 }
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

def evalOffsetSeed (ref : ReferenceToken) (suffix : String) : ReferenceToken :=
  { kind := ref.kind, target := ref.target ++ suffix }

theorem evalOffsetSeed_reference_shift :
    evalOffsetSeed { kind := .a1, target := "A1" } "#OFFSET(1,2)" =
      { kind := .a1, target := "A1#OFFSET(1,2)" } := by
  simp [evalOffsetSeed]

theorem offsetMeta_profiles :
    offsetMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ offsetMeta.fecDependencyProfile = FecDependencyProfile.callerContext
    ∧ offsetMeta.surfaceFecDependencyProfile = FecDependencyProfile.callerContext := by
  simp [offsetMeta]

end OxFunc.Functions
