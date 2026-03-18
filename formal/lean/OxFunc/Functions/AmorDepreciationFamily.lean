import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def amorDepreciationMetaBase : FunctionMeta := {
  functionId := "FUNC.AMOR_DEPRECIATION_BASE"
  arity := { min := 6, max := 7 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.none
}

def amordegrcMeta : FunctionMeta := {
  amorDepreciationMetaBase with functionId := "FUNC.AMORDEGRC"
}

def amorlincMeta : FunctionMeta := {
  amorDepreciationMetaBase with functionId := "FUNC.AMORLINC"
}

/--
W24 Batch 14 packet-evidences the admitted scalar AMOR depreciation slice for
`AMORDEGRC` and `AMORLINC`. The detailed depreciation schedule logic lives in
Rust; this Lean file keeps metadata alignment explicit.
-/
theorem amorDepreciation_meta_profiles :
    amordegrcMeta.arity = { min := 6, max := 7 }
    ∧ amorlincMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ amordegrcMeta.surfaceFecDependencyProfile = FecDependencyProfile.none := by
  simp [amorDepreciationMetaBase, amordegrcMeta, amorlincMeta]

theorem amorDepreciation_ids :
    amordegrcMeta.functionId = "FUNC.AMORDEGRC"
    ∧ amorlincMeta.functionId = "FUNC.AMORLINC" := by
  simp [amordegrcMeta, amorlincMeta]

end OxFunc.Functions
