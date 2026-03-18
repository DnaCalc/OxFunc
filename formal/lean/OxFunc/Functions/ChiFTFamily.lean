import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def chiFTMetaBase : FunctionMeta := {
  functionId := "FUNC.CHI_F_T_BASE"
  arity := Arity.exact 2
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

def chisqDistMeta : FunctionMeta := {
  chiFTMetaBase with
  functionId := "FUNC.CHISQ.DIST"
  arity := Arity.exact 3
}

def fDistMeta : FunctionMeta := {
  chiFTMetaBase with
  functionId := "FUNC.F.DIST"
  arity := Arity.exact 4
}

def tDistMeta : FunctionMeta := {
  chiFTMetaBase with
  functionId := "FUNC.T.DIST"
  arity := Arity.exact 3
}

def chidistMeta : FunctionMeta := {
  chiFTMetaBase with
  functionId := "FUNC.CHIDIST"
}

def tInvMeta : FunctionMeta := {
  chiFTMetaBase with
  functionId := "FUNC.T.INV"
}

theorem chiFTFamily_profiles :
    chisqDistMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ fDistMeta.arity = Arity.exact 4
    ∧ tDistMeta.kernelSignatureClass = KernelSignatureClass.custom
    ∧ chidistMeta.hostInteraction = HostInteractionClass.none
    ∧ tInvMeta.threadSafety = ThreadSafetyClass.safePure := by
  simp [chiFTMetaBase, chisqDistMeta, fDistMeta, tDistMeta, chidistMeta, tInvMeta]

theorem chiFTFamily_ids :
    chisqDistMeta.functionId = "FUNC.CHISQ.DIST"
    ∧ fDistMeta.functionId = "FUNC.F.DIST"
    ∧ tDistMeta.functionId = "FUNC.T.DIST"
    ∧ chidistMeta.functionId = "FUNC.CHIDIST"
    ∧ tInvMeta.functionId = "FUNC.T.INV" := by
  simp [chisqDistMeta, fDistMeta, tDistMeta, chidistMeta, tInvMeta]

end OxFunc.Functions
