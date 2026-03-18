import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def sumproductBaseMeta : FunctionMeta := {
  functionId := "FUNC.SUMPRODUCT_BASE"
  arity := { min := 2, max := 255 }
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

def sumproductMeta : FunctionMeta := {
  sumproductBaseMeta with
  functionId := "FUNC.SUMPRODUCT"
}

def sumx2my2Meta : FunctionMeta := {
  sumproductBaseMeta with
  functionId := "FUNC.SUMX2MY2"
  arity := Arity.exact 2
}

def sumx2py2Meta : FunctionMeta := {
  sumproductBaseMeta with
  functionId := "FUNC.SUMX2PY2"
  arity := Arity.exact 2
}

def sumxmy2Meta : FunctionMeta := {
  sumproductBaseMeta with
  functionId := "FUNC.SUMXMY2"
  arity := Arity.exact 2
}

def seriessumMeta : FunctionMeta := {
  sumproductBaseMeta with
  functionId := "FUNC.SERIESSUM"
  arity := Arity.exact 4
}

theorem sumproductFamily_profiles :
    sumproductMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ sumproductMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ sumx2my2Meta.arity = Arity.exact 2
    ∧ sumx2py2Meta.arity = Arity.exact 2
    ∧ sumxmy2Meta.arity = Arity.exact 2
    ∧ seriessumMeta.arity = Arity.exact 4
    ∧ seriessumMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [sumproductBaseMeta, sumproductMeta, sumx2my2Meta, sumx2py2Meta, sumxmy2Meta, seriessumMeta]

end OxFunc.Functions
