import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def matrixBaseMeta : FunctionMeta := {
  functionId := "FUNC.MATRIX_BASE"
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

def mdetermMeta : FunctionMeta := {
  matrixBaseMeta with
  functionId := "FUNC.MDETERM"
}

def minverseMeta : FunctionMeta := {
  matrixBaseMeta with
  functionId := "FUNC.MINVERSE"
}

/--
W109 executable route tag for the identified current-reference MINVERSE
numeric body. The tag records the eight per-operation x87 PC64-to-PC53
publication sites and the final positive-zero normalization without
duplicating the Rust floating-point engine in Lean.
-/
structure MinversePublicationRoute where
  factorDivisionX87 : Bool
  eliminationMultiplyX87 : Bool
  eliminationSubtractX87 : Bool
  forwardMultiplyX87 : Bool
  forwardSubtractX87 : Bool
  backwardMultiplyX87 : Bool
  backwardSubtractX87 : Bool
  finalDivisionX87 : Bool
  canonicalPositiveZero : Bool
  deriving DecidableEq, Repr

def minversePublicationRoute : MinversePublicationRoute := {
  factorDivisionX87 := true
  eliminationMultiplyX87 := true
  eliminationSubtractX87 := true
  forwardMultiplyX87 := true
  forwardSubtractX87 := true
  backwardMultiplyX87 := true
  backwardSubtractX87 := true
  finalDivisionX87 := true
  canonicalPositiveZero := true
}

def munitMeta : FunctionMeta := {
  matrixBaseMeta with
  functionId := "FUNC.MUNIT"
}

def mmultMeta : FunctionMeta := {
  matrixBaseMeta with
  functionId := "FUNC.MMULT"
  arity := Arity.exact 2
}

theorem matrixFamily_profiles :
    mdetermMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ minverseMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ munitMeta.kernelSignatureClass = KernelSignatureClass.custom
    ∧ mmultMeta.arity = Arity.exact 2 := by
  simp [matrixBaseMeta, mdetermMeta, minverseMeta, munitMeta, mmultMeta]

theorem minverse_publication_route_has_all_x87_sites_and_positive_zero :
    minversePublicationRoute.factorDivisionX87 = true
    ∧ minversePublicationRoute.eliminationMultiplyX87 = true
    ∧ minversePublicationRoute.eliminationSubtractX87 = true
    ∧ minversePublicationRoute.forwardMultiplyX87 = true
    ∧ minversePublicationRoute.forwardSubtractX87 = true
    ∧ minversePublicationRoute.backwardMultiplyX87 = true
    ∧ minversePublicationRoute.backwardSubtractX87 = true
    ∧ minversePublicationRoute.finalDivisionX87 = true
    ∧ minversePublicationRoute.canonicalPositiveZero = true := by
  native_decide

end OxFunc.Functions
