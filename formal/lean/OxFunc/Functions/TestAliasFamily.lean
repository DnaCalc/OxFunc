import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def testAliasMetaBase : FunctionMeta := {
  functionId := "FUNC.TEST_ALIAS_BASE"
  arity := Arity.exact 2
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

def chisqTestMeta : FunctionMeta := {
  testAliasMetaBase with functionId := "FUNC.CHISQ.TEST"
}

def chitestMeta : FunctionMeta := {
  testAliasMetaBase with functionId := "FUNC.CHITEST"
}

def fTestMeta : FunctionMeta := {
  testAliasMetaBase with functionId := "FUNC.F.TEST"
}

def ftestMeta : FunctionMeta := {
  testAliasMetaBase with functionId := "FUNC.FTEST"
}

def tTestMeta : FunctionMeta := {
  testAliasMetaBase with
  functionId := "FUNC.T.TEST"
  arity := Arity.exact 4
}

def ttestMeta : FunctionMeta := {
  testAliasMetaBase with
  functionId := "FUNC.TTEST"
  arity := Arity.exact 4
}

def ztestMeta : FunctionMeta := {
  testAliasMetaBase with
  functionId := "FUNC.ZTEST"
  arity := { min := 2, max := 3 }
}

theorem testAlias_meta_profiles :
    chisqTestMeta.arity = Arity.exact 2
    ∧ fTestMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ tTestMeta.arity = Arity.exact 4
    ∧ ttestMeta.arity = Arity.exact 4
    ∧ ztestMeta.arity = { min := 2, max := 3 } := by
  simp [testAliasMetaBase, chisqTestMeta, fTestMeta, tTestMeta, ttestMeta, ztestMeta]

theorem testAlias_ids :
    chisqTestMeta.functionId = "FUNC.CHISQ.TEST"
    ∧ chitestMeta.functionId = "FUNC.CHITEST"
    ∧ fTestMeta.functionId = "FUNC.F.TEST"
    ∧ ftestMeta.functionId = "FUNC.FTEST"
    ∧ tTestMeta.functionId = "FUNC.T.TEST"
    ∧ ttestMeta.functionId = "FUNC.TTEST"
    ∧ ztestMeta.functionId = "FUNC.ZTEST" := by
  simp [chisqTestMeta, chitestMeta, fTestMeta, ftestMeta, tTestMeta, ttestMeta, ztestMeta]

end OxFunc.Functions
