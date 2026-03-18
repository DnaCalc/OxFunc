import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def bondCoreBase : FunctionMeta := {
  functionId := "FUNC.BOND_CORE_BASE"
  arity := { min := 1, max := 1 }
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

def accrintMeta : FunctionMeta := { bondCoreBase with functionId := "FUNC.ACCRINT", arity := { min := 6, max := 8 } }
def accrintmMeta : FunctionMeta := { bondCoreBase with functionId := "FUNC.ACCRINTM", arity := { min := 3, max := 5 } }
def durationMeta : FunctionMeta := { bondCoreBase with functionId := "FUNC.DURATION", arity := { min := 5, max := 6 } }
def mdurationMeta : FunctionMeta := { bondCoreBase with functionId := "FUNC.MDURATION", arity := { min := 5, max := 6 } }
def priceMeta : FunctionMeta := { bondCoreBase with functionId := "FUNC.PRICE", arity := { min := 6, max := 7 } }
def pricematMeta : FunctionMeta := { bondCoreBase with functionId := "FUNC.PRICEMAT", arity := { min := 5, max := 6 } }
def yieldMeta : FunctionMeta := { bondCoreBase with functionId := "FUNC.YIELD", arity := { min := 6, max := 7 } }
def yielddiscMeta : FunctionMeta := { bondCoreBase with functionId := "FUNC.YIELDDISC", arity := { min := 4, max := 5 } }
def yieldmatMeta : FunctionMeta := { bondCoreBase with functionId := "FUNC.YIELDMAT", arity := { min := 5, max := 6 } }

theorem bond_core_profiles :
    accrintMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ priceMeta.arity.max = 7
    ∧ yieldMeta.functionId = "FUNC.YIELD"
    ∧ yielddiscMeta.threadSafety = ThreadSafetyClass.safePure
    ∧ yieldmatMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [bondCoreBase, accrintMeta, priceMeta, yieldMeta, yielddiscMeta, yieldmatMeta]

end OxFunc.Functions
