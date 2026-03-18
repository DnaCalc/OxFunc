import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def dollardeMeta : FunctionMeta := {
  functionId := "FUNC.DOLLARDE"
  arity := Arity.exact 2
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def dollarfrMeta : FunctionMeta := {
  functionId := "FUNC.DOLLARFR"
  arity := Arity.exact 2
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

theorem dollarFractionMeta_profiles :
    dollardeMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ dollardeMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ dollarfrMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ dollarfrMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [dollardeMeta, dollarfrMeta]

theorem dollarFractionMeta_arity_and_kernel :
    dollardeMeta.arity = Arity.exact 2
    ∧ dollarfrMeta.arity = Arity.exact 2
    ∧ dollardeMeta.kernelSignatureClass = KernelSignatureClass.custom
    ∧ dollarfrMeta.kernelSignatureClass = KernelSignatureClass.custom := by
  simp [dollardeMeta, dollarfrMeta]

end OxFunc.Functions