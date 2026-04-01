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

def chisqDistRtMeta : FunctionMeta := {
  chiFTMetaBase with
  functionId := "FUNC.CHISQ.DIST.RT"
}

def chisqInvMeta : FunctionMeta := {
  chiFTMetaBase with
  functionId := "FUNC.CHISQ.INV"
}

def chisqInvRtMeta : FunctionMeta := {
  chiFTMetaBase with
  functionId := "FUNC.CHISQ.INV.RT"
}

def fDistMeta : FunctionMeta := {
  chiFTMetaBase with
  functionId := "FUNC.F.DIST"
  arity := Arity.exact 4
}

def fDistRtMeta : FunctionMeta := {
  chiFTMetaBase with
  functionId := "FUNC.F.DIST.RT"
  arity := Arity.exact 3
}

def fInvMeta : FunctionMeta := {
  chiFTMetaBase with
  functionId := "FUNC.F.INV"
  arity := Arity.exact 3
}

def fInvRtMeta : FunctionMeta := {
  chiFTMetaBase with
  functionId := "FUNC.F.INV.RT"
  arity := Arity.exact 3
}

def tDistMeta : FunctionMeta := {
  chiFTMetaBase with
  functionId := "FUNC.T.DIST"
  arity := Arity.exact 3
}

def tDist2tMeta : FunctionMeta := {
  chiFTMetaBase with
  functionId := "FUNC.T.DIST.2T"
  arity := Arity.exact 2
}

def tDistRtMeta : FunctionMeta := {
  chiFTMetaBase with
  functionId := "FUNC.T.DIST.RT"
  arity := Arity.exact 2
}

def chidistMeta : FunctionMeta := {
  chiFTMetaBase with
  functionId := "FUNC.CHIDIST"
}

def chiinvMeta : FunctionMeta := {
  chiFTMetaBase with
  functionId := "FUNC.CHIINV"
}

def fdistMeta : FunctionMeta := {
  chiFTMetaBase with
  functionId := "FUNC.FDIST"
  arity := Arity.exact 3
}

def finvMeta : FunctionMeta := {
  chiFTMetaBase with
  functionId := "FUNC.FINV"
  arity := Arity.exact 3
}

def tInvMeta : FunctionMeta := {
  chiFTMetaBase with
  functionId := "FUNC.T.INV"
}

def tInv2tMeta : FunctionMeta := {
  chiFTMetaBase with
  functionId := "FUNC.T.INV.2T"
}

def tdistMeta : FunctionMeta := {
  chiFTMetaBase with
  functionId := "FUNC.TDIST"
  arity := Arity.exact 3
}

def tinvMeta : FunctionMeta := {
  chiFTMetaBase with
  functionId := "FUNC.TINV"
}

theorem chiFTFamily_profiles :
    chisqDistMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ chisqDistRtMeta.arity = Arity.exact 2
    ∧ chisqInvMeta.arity = Arity.exact 2
    ∧ chisqInvRtMeta.surfaceFecDependencyProfile = FecDependencyProfile.none
    ∧ fDistMeta.arity = Arity.exact 4
    ∧ fDistRtMeta.arity = Arity.exact 3
    ∧ fInvMeta.arity = Arity.exact 3
    ∧ fInvRtMeta.arity = Arity.exact 3
    ∧ tDistMeta.kernelSignatureClass = KernelSignatureClass.custom
    ∧ tDist2tMeta.arity = Arity.exact 2
    ∧ tDistRtMeta.arity = Arity.exact 2
    ∧ chidistMeta.hostInteraction = HostInteractionClass.none
    ∧ chiinvMeta.threadSafety = ThreadSafetyClass.safePure
    ∧ fdistMeta.functionId = "FUNC.FDIST"
    ∧ finvMeta.functionId = "FUNC.FINV"
    ∧ tInvMeta.threadSafety = ThreadSafetyClass.safePure
    ∧ tInv2tMeta.functionId = "FUNC.T.INV.2T"
    ∧ tdistMeta.functionId = "FUNC.TDIST"
    ∧ tinvMeta.functionId = "FUNC.TINV" := by
  simp [
    chiFTMetaBase,
    chisqDistMeta,
    chisqDistRtMeta,
    chisqInvMeta,
    chisqInvRtMeta,
    fDistMeta,
    fDistRtMeta,
    fInvMeta,
    fInvRtMeta,
    tDistMeta,
    tDist2tMeta,
    tDistRtMeta,
    chidistMeta,
    chiinvMeta,
    fdistMeta,
    finvMeta,
    tInvMeta,
    tInv2tMeta,
    tdistMeta,
    tinvMeta
  ]

theorem chiFTFamily_ids :
    chisqDistMeta.functionId = "FUNC.CHISQ.DIST"
    ∧ chisqDistRtMeta.functionId = "FUNC.CHISQ.DIST.RT"
    ∧ chisqInvMeta.functionId = "FUNC.CHISQ.INV"
    ∧ chisqInvRtMeta.functionId = "FUNC.CHISQ.INV.RT"
    ∧ fDistMeta.functionId = "FUNC.F.DIST"
    ∧ fDistRtMeta.functionId = "FUNC.F.DIST.RT"
    ∧ fInvMeta.functionId = "FUNC.F.INV"
    ∧ fInvRtMeta.functionId = "FUNC.F.INV.RT"
    ∧ tDistMeta.functionId = "FUNC.T.DIST"
    ∧ tDist2tMeta.functionId = "FUNC.T.DIST.2T"
    ∧ tDistRtMeta.functionId = "FUNC.T.DIST.RT"
    ∧ chidistMeta.functionId = "FUNC.CHIDIST"
    ∧ chiinvMeta.functionId = "FUNC.CHIINV"
    ∧ fdistMeta.functionId = "FUNC.FDIST"
    ∧ finvMeta.functionId = "FUNC.FINV"
    ∧ tInvMeta.functionId = "FUNC.T.INV"
    ∧ tInv2tMeta.functionId = "FUNC.T.INV.2T"
    ∧ tdistMeta.functionId = "FUNC.TDIST"
    ∧ tinvMeta.functionId = "FUNC.TINV" := by
  simp [
    chisqDistMeta,
    chisqDistRtMeta,
    chisqInvMeta,
    chisqInvRtMeta,
    fDistMeta,
    fDistRtMeta,
    fInvMeta,
    fInvRtMeta,
    tDistMeta,
    tDist2tMeta,
    tDistRtMeta,
    chidistMeta,
    chiinvMeta,
    fdistMeta,
    finvMeta,
    tInvMeta,
    tInv2tMeta,
    tdistMeta,
    tinvMeta
  ]

end OxFunc.Functions
