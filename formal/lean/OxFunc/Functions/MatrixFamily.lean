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

end OxFunc.Functions
