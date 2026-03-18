import OxFunc.FunctionCore
import OxFunc.ValueUniverse

namespace OxFunc.Functions

open OxFunc

def statisticalTestsMetaBase : FunctionMeta := {
  functionId := "FUNC.STATISTICAL.TESTS.BASE"
  arity := { min := 2, max := 2 }
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

def statisticalChisqTestMeta : FunctionMeta := {
  statisticalTestsMetaBase with functionId := "FUNC.CHISQ.TEST"
}

def statisticalChitestMeta : FunctionMeta := {
  statisticalTestsMetaBase with functionId := "FUNC.CHITEST"
}

def statisticalFTestMeta : FunctionMeta := {
  statisticalTestsMetaBase with functionId := "FUNC.F.TEST"
}

def statisticalFtestMeta : FunctionMeta := {
  statisticalTestsMetaBase with functionId := "FUNC.FTEST"
}

def statisticalTTestMeta : FunctionMeta := {
  statisticalTestsMetaBase with
  functionId := "FUNC.T.TEST"
  arity := { min := 4, max := 4 }
}

def statisticalTtestMeta : FunctionMeta := {
  statisticalTestsMetaBase with
  functionId := "FUNC.TTEST"
  arity := { min := 4, max := 4 }
}

def admittedTTestType (n : Int) : Bool :=
  n = 1 ∨ n = 2 ∨ n = 3

def admittedTailCount (n : Int) : Bool :=
  n = 1 ∨ n = 2

theorem statisticalTests_meta_profiles :
    statisticalChisqTestMeta.arity = { min := 2, max := 2 }
    ∧ statisticalFTestMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ statisticalTTestMeta.arity = { min := 4, max := 4 }
    ∧ statisticalTtestMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [statisticalTestsMetaBase, statisticalChisqTestMeta, statisticalFTestMeta, statisticalTTestMeta, statisticalTtestMeta]

theorem statisticalTests_ids :
    statisticalChisqTestMeta.functionId = "FUNC.CHISQ.TEST"
    ∧ statisticalChitestMeta.functionId = "FUNC.CHITEST"
    ∧ statisticalFTestMeta.functionId = "FUNC.F.TEST"
    ∧ statisticalFtestMeta.functionId = "FUNC.FTEST"
    ∧ statisticalTTestMeta.functionId = "FUNC.T.TEST"
    ∧ statisticalTtestMeta.functionId = "FUNC.TTEST" := by
  simp [statisticalChisqTestMeta, statisticalChitestMeta, statisticalFTestMeta, statisticalFtestMeta, statisticalTTestMeta, statisticalTtestMeta]

theorem admittedTTestType_seed :
    admittedTTestType 1 = True
    ∧ admittedTTestType 2 = True
    ∧ admittedTTestType 3 = True
    ∧ admittedTTestType 4 = False := by
  simp [admittedTTestType]

theorem admittedTailCount_seed :
    admittedTailCount 1 = True
    ∧ admittedTailCount 2 = True
    ∧ admittedTailCount 3 = False := by
  simp [admittedTailCount]

end OxFunc.Functions
